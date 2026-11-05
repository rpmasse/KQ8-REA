//! A character in a script's Special movement list. The named animation
//! player is also the cue cursor: the next update fires only the cues
//! crossed from its saved phase. See characters.md rule 22.
use super::{Character, Rng, State, Walk};
use crate::active::WorldState;
use crate::connor_mover::{Mover, State as MoverState};
use crate::kept_interactions::Playback;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpecialActor {
    animation: Playback,
    playing: String,
    remaining: VecDeque<String>,
    rng: Rng,
    first_update_seen: bool,
    random_pending: bool,
    turn_to: Option<[f32; 2]>,
    no_turn: bool,
    mouth: u16,
    gravity: bool,
    unborn: bool,
    mover: Option<Moving>,
    step: [f32; 3],
    fall_from: Option<f32>,
    mover_started: bool,
    script: Option<String>,
    translucency: Option<f32>,
    hidden: Option<bool>,
    state_seen: Option<u32>,
    collision_type: u32,
    collision_mask: u32,
}

// Keep the moving body and the animation at their respective saved
// positions. The installed model still supplies its collision geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Moving {
    position: [f32; 3],
    state: MoverState,
    on_ground: bool,
    use_gravity: bool,
    keep_on_ground: bool,
    air_updates: u32,
    fall_timer: f32,
    pull: [f32; 3],
    submerged: bool,
}

impl Moving {
    fn take(m: &Mover) -> Self {
        Self {
            position: m.position,
            state: m.state,
            on_ground: m.on_ground,
            use_gravity: m.use_gravity,
            keep_on_ground: m.keep_on_ground,
            air_updates: m.air_updates,
            fall_timer: m.fall_timer,
            pull: m.pull,
            submerged: m.submerged,
        }
    }

    fn restore(&self, m: &mut Mover) {
        m.position = self.position;
        m.state = self.state;
        m.on_ground = self.on_ground;
        m.use_gravity = self.use_gravity;
        m.keep_on_ground = self.keep_on_ground;
        m.air_updates = self.air_updates;
        m.fall_timer = self.fall_timer;
        m.pull = self.pull;
        m.submerged = self.submerged;
    }
}

/// The mutable part of a script-created character. Its installed type and
/// model are reconstructed before these values are applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpawnedCharacter {
    state: State,
    started: bool,
    first_update_seen: bool,
    wait_due: Option<f64>,
    random_pending: bool,
    special: VecDeque<String>,
    special_playing: Option<String>,
    rng: Rng,
    turn_to: Option<[f32; 2]>,
    mouth: u16,
    no_turn: bool,
    walk_slope: Option<f32>,
    gravity: bool,
    unborn: bool,
    mover: Option<Moving>,
    walk: Walk,
    ai: Option<crate::character_ai::Ai>,
    fight: Option<SpawnedFight>,
    resume_move_due: Option<f64>,
    falling_from: Option<f32>,
    mover_started: bool,
    collision_type: u32,
    collision_mask: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpawnedFight {
    stats: crate::stats::Stats,
    attack_due: Option<f64>,
    last_kind: Option<crate::fight::HitKind>,
    must_flee: bool,
    flinch_end: Option<f64>,
    last_ouch: Option<f64>,
    flinching: Option<String>,
    dying: Option<Option<String>>,
    combat_scaled: bool,
    shoot_offset: Option<[f32; 3]>,
    kept_drop: Option<(String, i64)>,
}

fn relative(t: f64, now: f64) -> f64 { if t == 0.0 { 0.0 } else { t - now } }
fn absolute(t: f64, now: f64) -> f64 { if t == 0.0 { 0.0 } else { t + now } }

impl SpawnedCharacter {
    pub(crate) fn take(c: &Character, now: f64) -> Self {
        let mut walk = c.walk.clone();
        walk.steer_off_until = relative(walk.steer_off_until, now);
        walk.next_check = relative(walk.next_check, now);
        if let Some((_, t)) = walk.second_from.as_mut() { *t = relative(*t, now); }
        let mut ai = c.ai.clone();
        if let Some(ai) = ai.as_mut() {
            ai.idle_due = relative(ai.idle_due, now);
            ai.react_at = relative(ai.react_at, now);
            ai.chase_until = relative(ai.chase_until, now);
            ai.rise_at = ai.rise_at.map(|t| relative(t, now));
        }
        let fight = c.fight.as_ref().map(|f| SpawnedFight {
            stats: f.stats,
            attack_due: f.attack_time.is_finite().then(|| relative(f.attack_time, now)),
            last_kind: f.last_kind,
            must_flee: f.must_flee,
            flinch_end: f.flinch_end.is_finite().then(|| relative(f.flinch_end, now)),
            last_ouch: f.last_ouch.is_finite().then(|| relative(f.last_ouch, now)),
            flinching: f.flinching.clone(), dying: f.dying.clone(),
            combat_scaled: f.combat_scaled, shoot_offset: f.shoot_offset,
            kept_drop: f.kept_drop.clone(),
        });
        Self {
            state: c.state, started: c.started, first_update_seen: c.first_update.is_some(),
            wait_due: c.wait_at.map(|t| relative(t, now)), random_pending: c.random_pending,
            special: c.special.clone(), special_playing: c.special_playing.clone(), rng: c.rng.clone(),
            turn_to: c.turn_to, mouth: c.mouth, no_turn: c.no_turn, walk_slope: c.walk_slope, gravity: c.gravity,
            unborn: c.unborn, mover: c.mover.as_deref().map(Moving::take), walk, ai, fight,
            resume_move_due: c.resume_move_at.is_finite().then(|| relative(c.resume_move_at, now)), falling_from: c.falling_from,
            mover_started: c.mover_started, collision_type: c.collision_type,
            collision_mask: c.collision_mask,
        }
    }

    pub(crate) fn restore(&self, c: &mut Character, now: f64, updates: u64) {
        c.state = self.state;
        c.started = self.started;
        c.first_update = self.first_update_seen.then_some(updates);
        c.wait_at = self.wait_due.map(|t| absolute(t, now));
        c.random_pending = self.random_pending;
        c.special.clone_from(&self.special);
        c.special_playing.clone_from(&self.special_playing);
        c.rng.clone_from(&self.rng);
        c.turn_to = self.turn_to;
        c.mouth = self.mouth;
        c.no_turn = self.no_turn;
        c.walk_slope = self.walk_slope;
        c.gravity = self.gravity;
        c.unborn = self.unborn;
        if let (Some(saved), Some(m)) = (&self.mover, c.mover.as_deref_mut()) { saved.restore(m); }
        c.walk = self.walk.clone();
        c.walk.steer_off_until = absolute(c.walk.steer_off_until, now);
        c.walk.next_check = absolute(c.walk.next_check, now);
        if let Some((_, t)) = c.walk.second_from.as_mut() { *t = absolute(*t, now); }
        c.ai.clone_from(&self.ai);
        if let Some(ai) = c.ai.as_mut() {
            ai.idle_due = absolute(ai.idle_due, now);
            ai.react_at = absolute(ai.react_at, now);
            ai.chase_until = absolute(ai.chase_until, now);
            ai.rise_at = ai.rise_at.map(|t| absolute(t, now));
        }
        if let (Some(saved), Some(f)) = (&self.fight, c.fight.as_mut()) {
            f.stats = saved.stats;
            f.attack_time = saved.attack_due.map_or(crate::fight::NEVER, |t| absolute(t, now));
            f.last_kind = saved.last_kind;
            f.must_flee = saved.must_flee;
            f.flinch_end = saved.flinch_end.map_or(f64::NEG_INFINITY, |t| absolute(t, now));
            f.last_ouch = saved.last_ouch.map_or(f64::NEG_INFINITY, |t| absolute(t, now));
            f.flinching.clone_from(&saved.flinching);
            f.dying.clone_from(&saved.dying);
            f.combat_scaled = saved.combat_scaled;
            f.shoot_offset = saved.shoot_offset;
            f.kept_drop.clone_from(&saved.kept_drop);
        }
        c.resume_move_at = self.resume_move_due.map_or(f64::NEG_INFINITY, |t| absolute(t, now));
        c.falling_from = self.falling_from;
        c.mover_started = self.mover_started;
        c.collision_type = self.collision_type;
        c.collision_mask = self.collision_mask;
    }

    pub(crate) fn valid(&self) -> bool {
        self.mouth < 8 && self.wait_due.is_none_or(f64::is_finite)
            && self.walk_slope.is_none_or(f32::is_finite)
            && self.resume_move_due.is_none_or(f64::is_finite)
            && self.fight.as_ref().is_none_or(|f| [f.attack_due, f.flinch_end, f.last_ouch].into_iter().flatten().all(f64::is_finite))
            && self.mover.as_ref().is_none_or(|m| m.position.iter().all(|x| x.is_finite()) && m.fall_timer.is_finite())
    }
}

impl SpecialActor {
    pub(crate) fn valid(&self) -> bool {
        !self.playing.is_empty()
            && self.animation.valid()
            && self.mouth < 8
            && self.mover.as_ref().is_none_or(|m| {
                m.air_updates < u32::MAX / 2
                    && !matches!(m.state, MoverState::Fall { stuck, .. } if stuck >= u32::MAX / 2)
            })
    }
    pub(crate) fn capture_world(w: &WorldState) -> BTreeMap<String, Self> {
        w.anims
            .iter()
            .filter(|a| !a.gone)
            .filter_map(|a| {
                let c = a
                    .character
                    .as_deref()
                    .filter(|c| c.state == State::Special)?;
                let playing = c.special_playing.clone()?;
                let name = a.player.name.to_ascii_lowercase();
                let saved = Self {
                    animation: Playback::take(&a.player),
                    playing,
                    remaining: c.special.clone(),
                    rng: c.rng.clone(),
                    first_update_seen: c.first_update.is_some(),
                    random_pending: c.random_pending,
                    turn_to: c.turn_to,
                    no_turn: c.no_turn,
                    mouth: c.mouth,
                    gravity: c.gravity,
                    unborn: c.unborn,
                    mover: c.mover.as_deref().map(Moving::take),
                    step: c.walk.step,
                    fall_from: c.falling_from,
                    mover_started: c.mover_started,
                    script: a.script.clone(),
                    translucency: a.translucency,
                    hidden: w.plain_hidden.get(&name).copied(),
                    state_seen: w.state_seen.get(&name).copied(),
                    collision_type: c.collision_type,
                    collision_mask: c.collision_mask,
                };
                Some((name, saved))
            })
            .collect()
    }

    pub(crate) fn apply(&self, w: &mut WorldState, name: &str) {
        let Some(a) = w
            .anims
            .iter_mut()
            .find(|a| !a.gone && a.player.name.eq_ignore_ascii_case(name))
        else {
            return;
        };
        let Some(c) = a.character.as_deref_mut() else {
            return;
        };
        self.animation.restore(&mut a.player);
        a.script.clone_from(&self.script);
        a.translucency = self.translucency;
        c.state = State::Special;
        c.special.clone_from(&self.remaining);
        c.special_playing = Some(self.playing.clone());
        c.rng.clone_from(&self.rng);
        c.started = true;
        c.first_update = self.first_update_seen.then_some(w.updates);
        c.wait_at = None;
        c.random_pending = self.random_pending;
        c.turn_to = self.turn_to;
        c.no_turn = self.no_turn;
        c.mouth = self.mouth;
        c.gravity = self.gravity;
        c.unborn = self.unborn;
        c.walk.step = self.step;
        c.falling_from = self.fall_from;
        c.mover_started = self.mover_started;
        c.collision_type = self.collision_type;
        c.collision_mask = self.collision_mask;
        if let (Some(saved), Some(m)) = (&self.mover, c.mover.as_deref_mut()) {
            saved.restore(m);
        }
        if let Some(click) = w
            .clickables
            .iter_mut()
            .find(|click| click.name.eq_ignore_ascii_case(name))
        {
            click.placement = a.player.placement();
            click.collision_type = c.collision_type;
            click.script.clone_from(&a.script);
        }
        if let Some(hidden) = self.hidden {
            w.plain_hidden.insert(name.to_owned(), hidden);
        } else {
            w.plain_hidden.remove(name);
        }
        if let Some(seen) = self.state_seen {
            w.state_seen.insert(name.to_owned(), seen);
        } else {
            w.state_seen.remove(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active::{Active, AnimObject};
    use crate::character::Character;
    use crate::collide::Placement;
    use crate::door::Cue;
    use crate::kept_interactions::KeptInteractions;
    use crate::named_anim::{tests::movement, NamedAnimPlayer};

    fn actor() -> Active {
        let mut active = Active::new(Default::default(), vec![], vec![], [], []);
        let movements = vec![
            movement("stop", 0., 1., true, true, vec![]),
            movement(
                "leave",
                0.,
                1.,
                false,
                false,
                vec![
                    Cue {
                        phase: 0.1,
                        action: "playsound early".into(),
                    },
                    Cue {
                        phase: 0.7,
                        action: "playsound late".into(),
                    },
                ],
            ),
            movement("settle", 0., 1., false, false, vec![]),
        ];
        active.add_anims([AnimObject {
            player: NamedAnimPlayer::new("Bird", movements, 0, Placement::at([0.; 3])),
            character: Some(Box::new(Character::new(
                false, 0x10, 0., false, false, true, 13,
            ))),
            script: None,
            gone: false,
            translucency: None,
        }]);
        active.world.characters_in_range.insert("bird".into());
        active
    }

    #[test]
    fn special_list_restores_clock_cues_queued_movement_and_random_stop() {
        let mut original = actor();
        let a = &mut original.world.anims[0];
        a.player.base = Placement::at([64., 32., 16.]);
        a.character.as_mut().unwrap().set_state(
            &mut a.player,
            "Special",
            &["leave,settle".into()],
            0.,
        );
        for _ in 0..7 {
            original.update();
        }
        assert!(original
            .take_log()
            .iter()
            .any(|l| l.contains("cue 'playsound early' fires")));
        let kept = KeptInteractions::take(&original.world);
        assert!(kept.valid_special_actors());
        let mut restored = actor();
        kept.restore_special_actors(&mut restored.world);
        let mut saw_settle = false;
        for _ in 0..37 {
            restored.update();
            original.update();
            let (a, b) = (&original.world.anims[0], &restored.world.anims[0]);
            assert_eq!(
                (a.player.phase, a.player.base, a.player.random_speed),
                (b.player.phase, b.player.base, b.player.random_speed)
            );
            assert_eq!(a.player.movement_name(), b.player.movement_name());
            saw_settle |= b.player.movement_name() == "settle";
        }
        let logs = restored.take_log();
        assert_eq!(
            logs.iter()
                .filter(|l| l.contains("cue 'playsound late' fires"))
                .count(),
            1
        );
        assert!(logs
            .iter()
            .all(|l| !l.contains("cue 'playsound early' fires")));
        assert!(saw_settle);
        assert_eq!(
            restored.world.anims[0].character.as_ref().unwrap().state,
            State::Wait
        );
    }
}
