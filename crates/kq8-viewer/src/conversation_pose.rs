use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Moving {
    state: kq8_core::connor_mover::State,
    on_ground: bool,
    use_gravity: bool,
    keep_on_ground: bool,
    fall_timer: f32,
    air_updates: u32,
    pull: [f32; 3],
    submerged: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedTalkPose {
    movement: String,
    phase: f32,
    bank: f32,
    fresh: bool,
    talk_play: Option<String>,
    wanted: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state_wanted: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reaction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reaction_playing: Option<String>,
    #[serde(default)]
    animation_input_lock: bool,
    auto: Option<kq8_core::connor_auto::AutoMove>,
    waiting: Option<SavedSwingTransition>,
    playing: Option<SavedSwingTransition>,
    hand_mode: i64,
    hands_off: bool,
    turn_by_script: bool,
    turn_press: bool,
    blocked: [bool; 2],
    moving: Moving,
}

/// Keep the existing conversation format while sharing its pose fields
/// with a Choice-owned pivot, a reaction or a rope movement.
pub type SavedControllerPose = SavedTalkPose;

impl SavedTalkPose {
    /// Shared CPU restoration used by existing owner poses and pending airborne
    /// continuation. Resolved movement names still come from the loaded assets.
    pub(super) fn restore_into(&self, walker: &mut Walker, mover: &mut Mover, hands_off: &mut bool, mode: Mode, resolve: &dyn Fn(&str) -> Result<&'static str, String>) -> Result<(), String> {
        if !self.valid() { return Err("invalid controller pose".into()); }
        let saved = self;
        let movement = resolve(&saved.movement)?;
        let talk_play = saved.talk_play.as_deref().map(&resolve).transpose()?;
        let wanted = saved.wanted.as_deref().map(&resolve).transpose()?;
        let state_wanted = saved.state_wanted.as_deref().map(&resolve).transpose()?;
        let reaction = saved.reaction.as_deref().map(&resolve).transpose()?;
        let reaction_playing = saved.reaction_playing.as_deref().map(&resolve).transpose()?;
        let transition = |t: &SavedSwingTransition| -> Result<kq8_core::connor::Transition, String> {
            Ok(kq8_core::connor::Transition { from: t.from, to: resolve(&t.to)?, to_phase: t.to_phase, via: resolve(&t.via)? })
        };
        let waiting = saved.waiting.as_ref().map(&transition).transpose()?;
        let playing = saved.playing.as_ref().map(&transition).transpose()?;
        walker.mode = mode; walker.movement = movement; walker.phase = saved.phase;
        walker.bank = saved.bank; walker.fresh = saved.fresh; walker.talk_play = talk_play;
        walker.wanted = wanted; walker.reaction = reaction; walker.auto = saved.auto.clone(); walker.waiting = waiting; walker.playing = playing;
        walker.state_wanted = state_wanted;
        walker.reaction_playing = reaction_playing; walker.animation_input_lock = saved.animation_input_lock;
        walker.hand_mode = saved.hand_mode; *hands_off = saved.hands_off;
        walker.turn_by_script = saved.turn_by_script; walker.turn_press = saved.turn_press;
        walker.blocked_forward = saved.blocked[0]; walker.blocked_back = saved.blocked[1];
        walker.submerged = saved.moving.submerged;
        mover.state = saved.moving.state; mover.on_ground = saved.moving.on_ground;
        mover.use_gravity = saved.moving.use_gravity; mover.keep_on_ground = saved.moving.keep_on_ground;
        mover.fall_timer = saved.moving.fall_timer; mover.air_updates = saved.moving.air_updates;
        mover.pull = saved.moving.pull; mover.submerged = saved.moving.submerged;
        Ok(())
    }
    pub(super) fn movement(&self) -> &str { &self.movement }
    pub(super) fn reaction(&self) -> Option<&str> { self.reaction.as_deref() }
    pub(super) fn reaction_playing(&self) -> Option<&str> { self.reaction_playing.as_deref() }
    pub(super) fn animation_input_lock(&self) -> bool { self.animation_input_lock }
    pub fn finished_weapon_away(&self) -> bool {
        self.movement.eq_ignore_ascii_case("away") && self.phase == 1.0
            && self.talk_play.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("out"))
            && self.hand_mode == kq8_core::connor::EMPTY_HANDS
    }

    pub fn valid(&self) -> bool {
        valid_swing_name(&self.movement) && self.phase.is_finite() && (0.0..=1.0).contains(&self.phase) && self.bank.is_finite()
            && [&self.talk_play, &self.wanted, &self.state_wanted, &self.reaction, &self.reaction_playing].into_iter().flatten().all(|s| valid_swing_name(s))
            && self.state_wanted.as_deref().is_none_or(|s| Some(s) == self.wanted.as_deref())
            && self.reaction_playing.as_deref().is_none_or(|s| s == self.movement)
            && [&self.waiting, &self.playing].into_iter().flatten().all(SavedSwingTransition::valid)
            && self.moving.pull.iter().all(|n| n.is_finite()) && self.moving.fall_timer.is_finite() && self.moving.fall_timer >= 0.0
            && self.auto.as_ref().is_none_or(|a| a.target.iter().all(|n| n.is_finite()) && a.turn_updates < u32::MAX / 2
                && a.cue.as_ref().is_none_or(|c| [&c.who, &c.c, &c.d].into_iter().all(|s| valid_swing_name(s)))
                && match a.phase { kq8_core::connor_auto::AutoPhase::Turning => true,
                    kq8_core::connor_auto::AutoPhase::Walking { from, distance } => from.into_iter().all(f32::is_finite) && distance.is_finite() })
    }
}

impl ConnorSim {
    pub fn saved_talk_pose(&self) -> Option<SavedTalkPose> {
        if self.walker.mode != Mode::Talk { return None; }
        Some(self.capture_controller_pose())
    }

    pub(super) fn capture_controller_pose(&self) -> SavedControllerPose {
        let w = &self.walker;
        SavedTalkPose { movement: w.movement.into(), phase: w.phase, bank: w.bank, fresh: w.fresh,
            talk_play: w.talk_play.map(str::to_owned), wanted: w.wanted.map(str::to_owned), reaction: w.reaction.map(str::to_owned), auto: w.auto.clone(),
            state_wanted: w.state_wanted.map(str::to_owned),
            reaction_playing: w.reaction_playing.map(str::to_owned), animation_input_lock: w.animation_input_lock,
            waiting: w.waiting.map(SavedSwingTransition::take), playing: w.playing.map(SavedSwingTransition::take),
            hand_mode: w.hand_mode, hands_off: self.hands_off, turn_by_script: w.turn_by_script, turn_press: w.turn_press,
            blocked: [w.blocked_forward, w.blocked_back], moving: Moving { state: self.mover.state, on_ground: self.mover.on_ground,
                use_gravity: self.mover.use_gravity, keep_on_ground: self.mover.keep_on_ground, fall_timer: self.mover.fall_timer,
                air_updates: self.mover.air_updates, pull: self.mover.pull, submerged: self.mover.submerged } }
    }

    pub fn restore_talk_pose(&mut self, saved: &SavedTalkPose) -> Result<(), String> {
        self.restore_controller_pose(saved, Mode::Talk, "conversation")
    }

    pub(super) fn restore_controller_pose(&mut self, saved: &SavedControllerPose, mode: Mode, label: &str) -> Result<(), String> {
        if !saved.valid() { return Err(format!("invalid {label} pose")); }
        let movements = &self.movements;
        let bank = self.weapon_set.as_deref();
        let resolve = |name: &str| find(movements, bank, name)
            .map(|m| if name.contains('/') { m.qualified } else { kq8_core::connor::plain_movement_name(m.name) })
            .ok_or_else(|| format!("saved {label} movement '{name}' is absent"));
        saved.restore_into(&mut self.walker, &mut self.mover, &mut self.hands_off, mode, &resolve)?;
        self.redraw_from_walker();
        self.log.detail(format!("{label} pose restored: {} phase {:.6}, talk_play {:?}, auto {:?}", self.walker.movement, saved.phase, self.walker.talk_play, saved.auto));
        Ok(())
    }
}
