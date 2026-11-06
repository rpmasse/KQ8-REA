//! Resume the active rope controller from the saved update, including
//! the separate hook-throw model. The camera has its own checkpoint.
use super::*;
use kq8_core::rope::{Anim, Climb, End, Stage};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedClimb {
    state: Climb,
    pose: SavedControllerPose,
    steps: [f32; 2],
    direction: i8,
    was_climbing: bool,
    model: Option<(String, f32, f32)>,
    ended: Vec<String>,
}

impl SavedClimb {
    pub fn valid(&self) -> bool {
        let state = &self.state;
        let stages_match = match state.stage {
            Stage::Draw | Stage::Btoss | Stage::RopeModel | Stage::Bon => state.from == End::Bottom,
            Stage::Drawdown | Stage::Ttoss | Stage::Ton => state.from == End::Top,
            _ => true,
        };
        let phase = |p: f32| (0.0..=1.0).contains(&p);
        let finite_steps = |up: f32, down: f32| up.is_finite() && down.is_finite() && up >= 0.0 && down >= 0.0;
        !state.done && stages_match && self.pose.valid()
            && (state.stage != Stage::Hanging || self.pose.movement().eq_ignore_ascii_case("conrope.anm/climb"))
            && state.place.iter().chain(&state.dir).chain(&state.aim)
                .chain(state.cam.iter().flatten().flatten()).all(|n| n.is_finite())
            && [state.range.0, state.range.1, state.top_limit, state.bottom_limit].into_iter().all(f32::is_finite)
            && state.weapon_before.as_deref().is_none_or(valid_swing_name)
            && state.saved_steps.is_none_or(|(up, down)| finite_steps(up, down))
            && finite_steps(self.steps[0], self.steps[1])
            && (-1..=1).contains(&self.direction)
            && self.model.as_ref().is_none_or(|(name, before, now)| state.stage == Stage::RopeModel
                && name.eq_ignore_ascii_case("conrope.anm/rope") && phase(*before) && phase(*now) && before <= now)
            && self.ended.iter().all(|name| valid_swing_name(name))
    }

    pub fn on_rope(&self) -> bool {
        matches!(self.state.stage, Stage::Bon | Stage::Hanging)
    }
}

impl ConnorSim {
    pub fn saved_climb(&self) -> Result<Option<SavedClimb>, String> {
        let Some(state) = &self.rope else {
            if self.walker.mode == Mode::Rope { return Err("missing active rope controller".into()); }
            return Ok(None);
        };
        if self.walker.mode != Mode::Rope || self.walker.rope_hang != (state.stage == Stage::Hanging) {
            return Err("rope movement and controller disagree".into());
        }
        let saved = SavedClimb {
            state: state.clone(), pose: self.capture_controller_pose(),
            steps: [self.mover.step_up, self.mover.step_down], direction: self.walker.climb_dir,
            was_climbing: self.rope_was_climbing,
            model: self.rope_model.map(|(name, before, now)| (name.to_owned(), before, now)),
            ended: self.rope_ended.iter().map(|name| (*name).to_owned()).collect(),
        };
        if !saved.valid() { return Err("invalid active rope climb".into()); }
        Ok(Some(saved))
    }

    pub fn restore_climb(&mut self, saved: &SavedClimb) -> Result<(), String> {
        if !saved.valid() { return Err("invalid saved rope climb".into()); }
        let model = if let Some((name, before, now)) = &saved.model {
            let movement = self.rope_movement(Anim::Rope("rope")).filter(|movement| *movement == name)
                .ok_or("saved rope throw model is absent from the installed movement list")?;
            Some((movement, *before, *now))
        } else { None };
        let ended = saved.ended.iter().map(|name| self.movements.iter().find(|m| m.name == name)
            .map(|m| m.name).ok_or_else(|| format!("missing saved rope movement '{name}'")))
            .collect::<Result<Vec<_>, _>>()?;
        self.restore_controller_pose(&saved.pose, Mode::Rope, "rope")?;
        self.mover.step_up = saved.steps[0];
        self.mover.step_down = saved.steps[1];
        self.walker.climb_dir = saved.direction;
        self.walker.rope_hang = saved.state.stage == Stage::Hanging;
        self.rope_was_climbing = saved.was_climbing;
        self.rope_model = model;
        self.rope_ended = ended;
        self.rope = Some(saved.state.clone());
        Ok(())
    }
}
