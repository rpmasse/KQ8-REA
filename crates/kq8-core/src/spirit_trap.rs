//! Spirit-trap state and integer drain, from traps.md Q188.
//! Event delivery and reaction selection belong to the ordinary active world.
use serde::{Deserialize, Serialize};

pub const CLASS: &str = "KQSpiriTrap";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpiritTrap {
    pub damage: f32,
    pub spirit_state: i32,
    pub trap_status: i32,
}

impl Default for SpiritTrap {
    fn default() -> Self {
        Self {
            damage: 0.2,
            spirit_state: 2,
            trap_status: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drain {
    pub before: i64,
    pub whole_loss: i64,
    pub floor: i64,
    pub after: i64,
    pub difference: i64,
}

impl SpiritTrap {
    pub fn valid(&self) -> bool {
        self.damage.is_finite()
    }

    /// Actual Connor identity, rather than a name test; generic occupancy
    /// has already added the visitor before this handler runs.
    pub fn enter(&mut self, is_connor: bool, occupants: usize) -> Option<u8> {
        if !is_connor || occupants != 1 {
            return None;
        }
        self.trap_status = 1;
        match self.spirit_state {
            1 => Some(1),
            2 => Some(2),
            _ => None,
        }
    }

    pub fn leave(&mut self, occupants: usize, shutting_down: bool) {
        if occupants == 0 && !shutting_down {
            self.trap_status = 0;
        }
    }

    /// Cue 1 is repeatable, even when already in state 1. Cue 2 is not a
    /// state-2 assignment or clear. Reading/restoring state calls none of these.
    pub fn cue(&mut self, value: i32) -> Option<u8> {
        match value {
            0 => {
                self.spirit_state = 0;
                None
            }
            1 => {
                self.spirit_state = 1;
                (self.trap_status != 0).then_some(1)
            }
            _ => None,
        }
    }

    /// Calculate at handler time, before the separately queued health and
    /// hit events. The stored 1% constants intentionally floor to zero at
    /// levels 1, 3 and 10. No protective minimum-one change is applied.
    pub fn drain(&self, health: i64, level: i64, fractions: u8) -> Option<Drain> {
        if !self.valid() || level < 1 || !matches!(fractions, 1 | 2) {
            return None;
        }
        let maximum = 10.0 * level as f64;
        let amount = f64::from(self.damage) * f64::from(fractions) * maximum;
        if !amount.is_finite() || amount < i64::MIN as f64 || amount >= i64::MAX as f64 {
            return None;
        }
        let whole_loss = amount as i64;
        let remaining = health.checked_sub(whole_loss)?;
        let threshold = f64::from(0.01_f32) * maximum;
        let floor = (f64::from(0.01_f32) * 10.0 * level as f64) as i64;
        let after = if (remaining as f64) < threshold {
            floor
        } else {
            remaining
        };
        Some(Drain {
            before: health,
            whole_loss,
            floor,
            after,
            difference: after.checked_sub(health)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_original_health_table_and_low_integer_floor_are_preserved() {
        let trap = SpiritTrap::default();
        for (level, health, fractions, loss, after) in [
            (1, 10, 1, 2, 8),
            (3, 30, 1, 6, 24),
            (1, 10, 2, 4, 6),
            (3, 30, 2, 12, 18),
            (1, 1, 1, 2, 0),
            (3, 1, 1, 6, 0),
            (10, 1, 1, 20, 0),
            (11, 1, 1, 22, 1),
        ] {
            let result = trap.drain(health, level, fractions).unwrap();
            assert_eq!(
                (result.whole_loss, result.after, result.difference),
                (loss, after, after - health)
            );
        }
        assert_eq!(trap.drain(1, 100, 1).unwrap().floor, 9);
        // The hit is a later ordinary event, not part of this calculation.
        assert_eq!(trap.drain(10, 1, 1).unwrap().after, 8);
        let mut custom = trap;
        custom.damage = 0.25;
        assert_eq!(
            custom.drain(30, 3, 1).unwrap().whole_loss,
            7,
            "cut 7.5 toward zero"
        );
        custom.damage = f32::NAN;
        assert!(!custom.valid() && custom.drain(10, 1, 1).is_none());
    }

    #[test]
    fn state_two_entry_repeat_cues_and_the_occupancy_flag_are_not_conflated() {
        let mut trap = SpiritTrap::default();
        assert_eq!(trap.enter(false, 1), None);
        assert_eq!(trap.enter(true, 2), None);
        assert_eq!(trap.trap_status, 0);
        trap.leave(0, false);
        assert_eq!(trap.enter(true, 1), Some(2));
        trap.leave(1, false);
        assert_eq!(
            trap.trap_status, 1,
            "Connor may leave with another visitor remaining"
        );
        for _ in 0..3 {
            assert_eq!(trap.cue(1), Some(1));
        }
        assert_eq!(trap.cue(2), None);
        assert_eq!(trap.spirit_state, 1);
        assert_eq!(trap.cue(0), None);
        assert_eq!(trap.spirit_state, 0);
        trap.leave(0, true);
        assert_eq!(trap.trap_status, 1, "shutdown suppresses clear");
        trap.leave(0, false);
        assert_eq!(trap.trap_status, 0);
        assert_eq!(trap.cue(1), None);
        assert_eq!(trap.spirit_state, 1);
        assert_eq!(trap.enter(true, 1), Some(1));
        trap.spirit_state = -7;
        assert_eq!(trap.enter(true, 1), None);
        assert_eq!(trap.trap_status, 1);
        trap.trap_status = -3;
        assert_eq!(
            trap.cue(1),
            Some(1),
            "any nonzero saved status admits the Cue drain"
        );
    }
}

impl crate::active::WorldState {
    pub(crate) fn spirit_touch(&mut self, index: usize, args: &[String], is_connor: bool) {
        let trap = &mut self.traps[index];
        let count = trap.inside.len();
        let Some(spirit) = &mut trap.settings.spirit else {
            return;
        };
        let before = *spirit;
        let entering = args.first().is_some_and(|s| s == "On")
            || (args.first().is_some_and(|s| s == "Cue")
                && args.get(1).is_some_and(|s| s == crate::trap::CUE_ENTER));
        let fractions = if entering {
            spirit.enter(is_connor, count)
        } else {
            spirit.leave(count, false);
            None
        };
        let after = *spirit;
        let name = trap.settings.name.clone();
        self.note(format!("spirit trap {name}: {}: actual Connor {is_connor}, occupants {count}, spiritState {}, trapStatus {} -> {}",args.join(" "),after.spirit_state,before.trap_status,after.trap_status));
        if let Some(fractions) = fractions {
            self.spirit_drain(index, fractions);
        }
    }

    pub(crate) fn spirit_cue(&mut self, index: usize, cue: i32, from: &str) {
        let trap = &mut self.traps[index];
        let Some(spirit) = &mut trap.settings.spirit else {
            return;
        };
        let before = *spirit;
        let fractions = spirit.cue(cue);
        let after = *spirit;
        let name = trap.settings.name.clone();
        let line = format!("spirit trap {name}: Cue {cue} from {from}: spiritState {} -> {}, trapStatus {}, occupants {}; {}",before.spirit_state,after.spirit_state,after.trap_status,trap.inside.len(),if fractions.is_some() {"one queued drain pair"} else {"no health event"});
        self.note(line);
        if let Some(fractions) = fractions {
            self.spirit_drain(index, fractions);
        }
    }

    fn spirit_drain(&mut self, index: usize, fractions: u8) {
        let trap = &self.traps[index];
        let (name, spirit) = (trap.settings.name.clone(), trap.settings.spirit.unwrap());
        let Some(stats) = self.connor_stats else {
            self.note(format!(
                "spirit trap {name}: Connor's stats unknown; drain not queued"
            ));
            return;
        };
        let Some(drain) = spirit.drain(stats.stats.0[0], stats.level, fractions) else {
            self.note(format!(
                "spirit trap {name}: invalid drain arithmetic; refused"
            ));
            return;
        };
        let now = self.now();
        let from = format!("spirit trap {name}");
        self.queue.send(
            now,
            0.0,
            crate::talk::CONNOR,
            "KQStatsChangeRequestEvent",
            &[drain.difference.to_string()],
            &from,
        );
        self.queue.send(
            now,
            0.0,
            crate::talk::CONNOR,
            "KQAttackEvent",
            &[name.clone(), "0".into(), "1".into(), "1".into()],
            &from,
        );
        let line = format!("spirit trap {name}: queues direct stage: health {}, level {}, damage {} x {fractions}, whole loss {}, floor {}, calculated target {}, stored difference {}; then separate Normal strength 1 forced 1 hit; health unchanged at handler time",drain.before,stats.level,spirit.damage,drain.whole_loss,drain.floor,drain.after,drain.difference);
        self.note(line);
    }
}

#[cfg(test)]
mod active_checks {
    use super::*;
    use crate::{
        active::{Active, Request},
        character::state_number,
        collide::BallStack,
        stats::{ConnorStats, Stats},
        trap::{Trap, TrapSettings, Visitor},
    };

    fn world(level: i64, health: i64, protection: i64, occupied: bool) -> Active {
        let mut active = Active::new(
            Default::default(),
            vec![],
            vec![],
            ["Connor".to_string()],
            [],
        );
        let mut trap = Trap::new(TrapSettings {
            name: "spirtp1".into(),
            position: [0.0; 3],
            min: [-250.0, -400.0, 0.0],
            max: [250.0, 400.0, 3000.0],
            script: None,
            lookup_call: false,
            mask: 4,
            trap_type: 0,
            inner_percent: None,
            spirit: Some(SpiritTrap::default()),
        });
        if occupied {
            trap.inside.push("Connor".into());
            trap.settings.spirit.as_mut().unwrap().trap_status = 1;
        }
        active.add_traps([trap]);
        active.set_connor_stats(ConnorStats {
            stats: Stats([health, 10, 0, protection, 0]),
            level,
        });
        active.world.connor_state = Some(state_number::WAIT);
        active
    }
    fn cue(active: &mut Active, value: &str) {
        let now = active.world.now();
        active.world.queue.send(
            now,
            0.0,
            "spirtp1",
            "KQCueEvent",
            &[value.into(), "0".into()],
            "test",
        );
    }
    fn health(active: &Active) -> i64 {
        active.world.connor_stats.unwrap().stats.0[0]
    }

    #[test]
    fn spirit_entry_queues_two_distinct_stages_and_protection_or_rope_only_affects_the_hit() {
        let balls = BallStack::from_box([-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0]);
        for (protection, roping, expected) in [(0, false, 5), (100, false, 6), (0, true, 6)] {
            let mut active = world(1, 10, protection, false);
            active.world.connor_roping = roping;
            let visitor = Visitor {
                name: "Connor",
                collision_type: 4,
                from: [0.0; 3],
                position: [0.0; 3],
                min: [-200.0, -200.0, 0.0],
                max: [200.0, 200.0, 1300.0],
                shape: crate::trap::VisitorShape::Balls(&balls),
            };
            assert_eq!(active.connor_moved(&visitor), ["spirtp1 On Connor"]);
            assert_eq!(
                health(&active),
                10,
                "the handler queues, it does not change health"
            );
            assert_eq!(active.world.queue.len(), 2);
            active.update();
            assert_eq!(health(&active), expected);
            assert_eq!(
                active.world.traps[0].settings.spirit.unwrap().spirit_state,
                2
            );
            let hurt = active
                .world
                .requests
                .iter()
                .filter(|r| matches!(r, Request::ConnorHurt { forced: true, .. }))
                .count();
            assert_eq!(hurt, usize::from(!roping));
            assert_eq!(
                active
                    .world
                    .trap_log
                    .iter()
                    .filter(|s| s.contains("queues direct stage:"))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn repeated_spirit_cues_preserve_the_no_op_and_can_kill_after_the_zero_floor() {
        for (level, initial, values) in [(1, 10, vec![7, 4, 1, 0]), (3, 30, vec![23, 16, 9, 2, 0])]
        {
            let mut active = world(level, initial, 0, true);
            cue(&mut active, "2");
            active.update();
            assert_eq!(health(&active), initial);
            assert_eq!(
                active.world.traps[0].settings.spirit.unwrap().spirit_state,
                2
            );
            for expected in values {
                cue(&mut active, "1");
                active.update();
                assert_eq!(health(&active), expected);
            }
            assert!(active.world.connor_dying);
            assert_eq!(
                active
                    .world
                    .requests
                    .iter()
                    .filter(|r| matches!(r, Request::ConnorDies { .. }))
                    .count(),
                1
            );
            assert_eq!(active.world.connor_state, Some(state_number::DIE));
        }
        let mut active = world(1, 10, 0, true);
        cue(&mut active, "0");
        active.update();
        assert_eq!(health(&active), 10);
        assert_eq!(
            active.world.traps[0].settings.spirit.unwrap().spirit_state,
            0
        );
        active.world.connor_dying = true;
        active.world.connor_state = Some(state_number::DIE);
        cue(&mut active, "1");
        active.update();
        assert_eq!(health(&active), 8, "direct stats still apply when dying");
        assert!(active.world.requests.is_empty());
    }

    #[test]
    fn simultaneous_spirit_cues_keep_their_calculated_deltas_and_fifo_descendants() {
        let mut active = world(1, 1, 0, true);
        cue(&mut active, "1");
        cue(&mut active, "1");
        active.update();
        assert_eq!(
            health(&active),
            -1,
            "0, lethal hit/entry zero, later direct -1, ignored hit"
        );
        assert!(active.world.connor_dying);
        assert_eq!(
            active
                .world
                .requests
                .iter()
                .filter(|r| matches!(r, Request::ConnorDies { .. }))
                .count(),
            1
        );
        // The viewer can mirror its pre-presentation Wait state before
        // servicing the requested death. That must not zero health twice.
        active.world.connor_state = Some(state_number::WAIT);
        active.world.enter_connor_death();
        assert_eq!(
            health(&active),
            -1,
            "presentation must not re-enter and erase the later direct delta"
        );
        let lines = active.world.trap_log.join("\n");
        assert_eq!(lines.matches("stored difference -1;").count(), 2);
    }
}
