//! The original in-world help panel and its game-time tutorial events.
use std::collections::{BTreeSet, VecDeque};

pub const CATALOG: [u8; 24] = [
    127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145,
    146, 149, 150, 151, 153,
];
const THREE_SECONDS: f32 = 3.;
const ONE_SECOND: f32 = 1.;

fn update_time(update: u64) -> f32 {
    (update as f64 / crate::UPDATES_PER_SECOND) as f32
}

/// Q196: Connor's source flags, last delivery time and held case IDs are
/// persistent. An event or retry deadline is separate (Q191).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct History {
    pub start_help: u32,
    pub updates_since_delivery: Option<u64>,
    pub held_cases: VecDeque<u8>,
}

pub const HEALTH_ITEM: u32 = 1;
pub const QUEST_ITEM: u32 = 2;
pub const ROPE: u32 = 4;
pub const FIRST_HIT: u32 = 8;
pub const LOW_HEALTH: u32 = 16;
pub const FIRST_KILL: u32 = 32;
pub const STANDING_JUMP: u32 = 64;

impl History {
    pub fn valid(&self) -> bool {
        let mut seen = BTreeSet::new();
        self.start_help <= 127 && self.held_cases.iter().all(|case| seen.insert(*case))
    }
    /// Old Rust slots recorded only these individual flags. Carried items
    /// cannot tell whether a tutorial was consumed.
    pub fn from_legacy(items: &BTreeSet<String>, hits: [bool; 2], kill: bool) -> Self {
        let mut history = Self::default();
        for (bit, consumed) in [
            (HEALTH_ITEM, items.contains("0x83")),
            (QUEST_ITEM, items.contains("0x84")),
            (ROPE, items.contains("0x80")),
            (FIRST_HIT, hits[0]),
            (LOW_HEALTH, hits[1]),
            (FIRST_KILL, kill),
        ] {
            if consumed {
                history.start_help |= bit;
            }
        }
        history
    }

    pub fn legacy_item_flags(&self) -> BTreeSet<String> {
        [(HEALTH_ITEM, "0x83"), (QUEST_ITEM, "0x84"), (ROPE, "0x80")]
            .into_iter()
            .filter(|(bit, _)| self.start_help & bit != 0)
            .map(|(_, name)| name.to_owned())
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Requests {
    pending: VecDeque<(f32, Notice)>,
    last_attempt: Option<f32>,
    start_help: u32,
    held: VecDeque<u8>,
    pub map_go: bool,
}

/// Death entry posts a separate event. Its later handler owns the two
/// tutorial deadlines; neither is measured directly from death entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Notice {
    Case(u8),
    FirstKill,
}

impl Requests {
    /// Returns true only for the first source to consume this bit.
    pub(crate) fn consume(&mut self, bit: u32) -> bool {
        let fresh = self.start_help & bit == 0;
        self.start_help |= bit;
        fresh
    }

    pub(crate) fn stop_help(&mut self) {
        self.start_help = 127;
    }

    pub fn history(&self, now: u64) -> History {
        self.history_at(update_time(now))
    }

    /// Keep the old Rust save's delivery-age ticks, using the current game
    /// clock to compute that age. Pending events are not saved by History.
    pub fn history_at(&self, now: f32) -> History {
        History {
            start_help: self.start_help,
            updates_since_delivery: self
                .last_attempt
                .map(|at| ((now - at).max(0.) as f64 * crate::UPDATES_PER_SECOND).round() as u64),
            held_cases: self.held.clone(),
        }
    }

    pub fn restore_history(&mut self, history: &History, now: u64) {
        self.restore_history_at(history, update_time(now));
    }

    pub fn restore_history_at(&mut self, history: &History, now: f32) {
        self.pending.clear();
        self.map_go = false;
        self.start_help = history.start_help;
        self.last_attempt = history
            .updates_since_delivery
            .map(|age| now - update_time(age));
        self.held.clone_from(&history.held_cases);
    }

    pub fn script(&mut self, now: u64, case: u8) {
        self.script_at(update_time(now), case);
    }

    pub fn script_at(&mut self, now: f32, case: u8) {
        self.schedule_at(now + THREE_SECONDS, case);
    }

    /// Compatibility adapter for older update-count fixtures. Production
    /// pickup and script sources use the observed game clock directly.
    #[cfg(test)]
    pub(crate) fn schedule(&mut self, first_update: u64, case: u8) {
        self.schedule_at(update_time(first_update), case);
    }

    pub(crate) fn schedule_at(&mut self, deadline: f32, case: u8) {
        if !self.pending.iter().any(|(_, pending)| *pending == Notice::Case(case)) {
            self.pending.push_back((deadline, Notice::Case(case)));
        }
    }

    /// pickup-tutorials.md rule 8. Arrival-state rejection belongs to the
    /// ordinary hit handler; these checks run after accepted damage handling.
    pub(crate) fn accepted_hit(&mut self, now: f32, empty_hands: bool, character: bool,
        projectile: bool, health: i64, level: i64) -> [bool; 2]
    {
        let first = self.start_help & (FIRST_HIT | FIRST_KILL) == 0
            && empty_hands && (character || projectile) && self.consume(FIRST_HIT);
        if first { self.schedule_at(now, 140); }
        let low = i128::from(health) * 2 < 10 * i128::from(level) && self.consume(LOW_HEALTH);
        if low { self.schedule_at(now + 0.2, 130); }
        [first, low]
    }

    /// Shared ordinary Die entry only. The historical `checken2` spelling
    /// is deliberate; `chicken2` is not one of the exclusions (rule 9).
    pub(crate) fn character_death(&mut self, now: f32, connor_present: bool,
        name: &str, monster_type: &str, die_now: bool) -> bool
    {
        if !connor_present || name.eq_ignore_ascii_case("Connor") || die_now
            || crate::fight::NO_KILL_TIP.iter().any(|t| t.eq_ignore_ascii_case(monster_type))
            || !self.consume(FIRST_KILL) { return false; }
        self.pending.push_back((now + crate::fight::KILL_TIP_DELAY as f32, Notice::FirstKill));
        true
    }

    /// Called only by the Jump controller's actual movement-end boundary.
    pub fn standing_jump_completed(&mut self, now: f32, movement: &str) -> bool {
        if !movement.eq_ignore_ascii_case("jump") || !self.consume(STANDING_JUMP) { return false; }
        self.schedule_at(now + THREE_SECONDS, 135);
        true
    }

    /// Service source events independently of popup availability and Connor
    /// ownership. Demo mode is sampled here, not when the death occurred.
    pub fn service_at(&mut self, now: f32, demo: bool) {
        while let Some(n) = self.pending.iter().position(|(due, notice)| *due <= now && *notice == Notice::FirstKill) {
            self.pending.remove(n);
            if !demo {
                self.schedule_at(now + 6., 137);
                self.schedule_at(now + 9., 149);
            }
        }
    }

    /// The disabled preference suppresses tutorials, not F1 or the Go button.
    pub fn due(&mut self, now: u64, enabled: bool, busy: bool, occupied: bool) -> Option<u8> {
        self.due_at(update_time(now), enabled, busy, occupied)
    }

    pub fn due_at(&mut self, now: f32, enabled: bool, busy: bool, occupied: bool) -> Option<u8> {
        let n = self.pending.iter().position(|(due, notice)| *due <= now && matches!(notice, Notice::Case(_)))?;
        let (_, Notice::Case(case)) = self.pending.remove(n)? else { unreachable!() };
        if busy
            || self
                .last_attempt
                .is_some_and(|at| now < at + THREE_SECONDS)
        {
            if !self.held.contains(&case) {
                self.held.push_back(case);
            }
            self.pending.push_back((now + ONE_SECOND, Notice::Case(case)));
            return None;
        }
        // Let the earliest held tutorial reach the UI first. Retain the
        // order in which distinct requests first encountered a busy Connor.
        if self
            .held
            .iter()
            .take_while(|&&older| older != case)
            .any(|older| self.pending.iter().any(|(_, pending)| *pending == Notice::Case(*older)))
        {
            self.pending.push_back((now + THREE_SECONDS, Notice::Case(case)));
            return None;
        }
        self.held.retain(|&held| held != case);
        self.last_attempt = Some(now);
        if case < 170 && !enabled {
            return None;
        }
        if occupied {
            self.pending.push_back((now + THREE_SECONDS, Notice::Case(case)));
            return None;
        }
        Some(case)
    }
}

pub fn expand_bindings(
    line: &str,
    keys: &crate::keyboard_menu::Bindings,
    mut label: impl FnMut(u8) -> Option<String>,
) -> String {
    let mut text = line.to_owned();
    for (row, v) in keys.0.iter().copied().enumerate() {
        let token = format!("^{}", char::from(b'0' + row as u8));
        if !text.contains(&token) {
            continue;
        }
        let key = crate::keyboard_menu::localized_key(v, &mut label);
        text = text.replace(&token, &key);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_pickup_tutorial_gate_stage_clock_and_pause_matrix() {
        // Character/Projectile, empty hands and both first-source bits are
        // independent of the post-hit health test and of delivery gates.
        for empty in [false, true] {
            for character in [false, true] {
                for projectile in [false, true] {
                    for used in [0, FIRST_HIT, FIRST_KILL, FIRST_HIT | FIRST_KILL] {
                        for health in [5, 4] {
                            let mut q = Requests::default();
                            q.restore_history_at(&History { start_help: used, ..Default::default() }, 20.);
                            let expected = empty && (character || projectile) && used == 0;
                            assert_eq!(q.accepted_hit(20., empty, character, projectile, health, 1), [expected, health == 4]);
                            assert_eq!(q.pending.iter().any(|&(at, n)| at == 20. && n == Notice::Case(140)), expected);
                            assert_eq!(q.pending.iter().any(|&(at, n)| at == 20.2 && n == Notice::Case(130)), health == 4);
                            assert_eq!(q.accepted_hit(21., empty, character, projectile, health, 1), [false, false]);
                        }
                    }
                }
            }
        }
        for (name, kind, present, immediate, eligible) in [
            ("gob", "goblin2", true, false, true),
            ("gob", "goblin2", false, false, false),
            ("Connor", "Connor", true, false, false),
            ("gob", "goblin2", true, true, false),
            ("gob", "BaT", true, false, false),
            ("gob", "CHICKEN", true, false, false),
            ("gob", "Checken2", true, false, false),
            ("gob", "chicken2", true, false, true),
        ] {
            let mut q = Requests::default();
            assert_eq!(q.character_death(20., present, name, kind, immediate), eligible);
            assert_eq!(q.history_at(20.).start_help & FIRST_KILL != 0, eligible);
            assert!(!q.character_death(20.1, present, name, kind, immediate));
            if eligible {
                assert_eq!(q.pending, [(21.5, Notice::FirstKill)]);
                q.stop_help(); // consumption prevents new sources, not delivery
                q.service_at(21.499, false);
                assert_eq!(q.pending, [(21.5, Notice::FirstKill)]);
                q.service_at(22., false); // handled late: +6/+9 from NOW
                assert_eq!(q.pending, [(28., Notice::Case(137)), (31., Notice::Case(149))]);
                q.service_at(30., false);
                assert_eq!(q.due_at(27.999, true, false, false), None);
                assert_eq!(q.due_at(28., true, false, false), Some(137));
                assert_eq!(q.due_at(30.999, true, false, false), None);
                assert_eq!(q.due_at(31., true, false, false), Some(149));
            }
        }
        let mut demo = Requests::default();
        assert!(demo.character_death(0., true, "gob", "goblin2", false));
        demo.service_at(1.5, true);
        demo.service_at(100., false);
        assert!(demo.pending.is_empty());
        assert_eq!(demo.history_at(100.).start_help, FIRST_KILL);

        for movement in ["runJump", "backJump", "jump", "JuMp"] {
            let mut q = Requests::default();
            let standing = movement.eq_ignore_ascii_case("jump");
            assert_eq!(q.standing_jump_completed(40., movement), standing);
            assert_eq!(q.history_at(40.).start_help == STANDING_JUMP, standing);
            assert_eq!(q.due_at(42.999, true, false, false), None);
            assert_eq!(q.due_at(43., true, false, false), standing.then_some(135));
            assert!(!q.standing_jump_completed(44., movement));
        }

        // Public script/delivery APIs: no frame or wall clock is consumed
        // while Main leaves the observed game time unchanged.
        let mut paused = Requests::default();
        paused.script_at(100., 127);
        for _ in 0..100 { assert_eq!(paused.due_at(102.5, true, false, false), None); }
        assert_eq!(paused.due_at(103., true, true, false), None); // busy: +1
        for _ in 0..100 { assert_eq!(paused.due_at(103.5, true, false, false), None); }
        assert_eq!(paused.due_at(104., true, false, true), None); // occupied: +3
        for _ in 0..100 { assert_eq!(paused.due_at(106.5, true, false, false), None); }
        assert_eq!(paused.due_at(107., true, false, false), Some(127));
        paused.script_at(107., 130);
        assert_eq!(paused.due_at(110., false, false, false), None); // delivery consumes suppressed tip
        assert_eq!(paused.history_at(110.).updates_since_delivery, Some(0));
        assert_eq!(paused.due_at(200., true, false, false), None);
    }

    #[test]
    fn history_keeps_delivery_age_and_held_order_across_a_new_clock() {
        let mut old = Requests::default();
        old.schedule(13, 132);
        assert_eq!(old.due(13, true, false, false), Some(132));
        old.schedule(20, 131);
        assert_eq!(old.due(20, true, true, false), None);
        old.stop_help();
        let history = old.history(21);
        assert_eq!(history.updates_since_delivery, Some(8));
        assert_eq!(history.held_cases, [131]);
        let mut resumed = Requests::default();
        resumed.restore_history(&history, 0);
        assert_eq!(resumed.history(0), history);
        let mut invalid = history;
        invalid.held_cases.push_back(131);
        assert!(!invalid.valid());
    }

    #[test]
    fn legacy_flags_recover_consumed_cases_independently_of_carried_items() {
        let recorded = BTreeSet::from(["0x80".to_string(), "0x84".to_string()]);
        let history = History::from_legacy(&recorded, [true, false], true);
        assert_eq!(
            history.start_help,
            ROPE | QUEST_ITEM | FIRST_HIT | FIRST_KILL
        );
        assert_eq!(history.legacy_item_flags(), recorded);
        assert!(history.valid());
    }
    #[test]
    fn an_earlier_held_tip_gets_through_before_a_later_one() {
        let mut q = Requests::default();
        q.script(0, 144);
        q.script(1, 127);
        assert_eq!(q.due(45, true, true, false), None);
        // 127 is now eligible but 144 is due to retry first.
        assert_eq!(q.due(46, true, false, false), None);
        assert_eq!(q.due(60, true, false, false), Some(144));
        assert_eq!(q.due(91, true, false, false), None);
        assert_eq!(q.due(106, true, false, false), Some(127));
    }

    #[test]
    fn map_and_special_cursor_tokens_keep_their_brackets() {
        let mut keys = crate::keyboard_menu::Bindings::default();
        keys.0[8] = 9;
        keys.0[10] = b'K';
        assert_eq!(
            expand_bindings("Map [^8], cursor [^:].", &keys, |n| (n == 93)
                .then(|| "Tab".into())),
            "Map [Tab], cursor [K]."
        );
    }

    #[test]
    fn delivery_checks_preferences_and_spacing_at_the_ui() {
        let mut q = Requests::default();
        q.script(5, 130);
        q.script(5, 172);
        assert_eq!(q.due(50, false, false, false), None);
        assert_eq!(q.due(50, false, false, false), None);
        assert_eq!(q.due(65, false, false, false), None);
        assert_eq!(q.due(80, false, false, false), None);
        assert_eq!(q.due(95, false, false, false), Some(172));
    }

    #[test]
    fn scripts_wait_for_connor_and_leave_occupied_panels_alone() {
        let mut q = Requests::default();
        q.script(5, 127);
        assert_eq!(q.due(49, true, false, false), None);
        assert_eq!(q.due(50, true, true, false), None);
        assert_eq!(q.due(64, true, false, false), None);
        assert_eq!(q.due(65, true, false, true), None);
        assert_eq!(q.due(109, true, false, false), None);
        assert_eq!(q.due(110, true, false, false), Some(127));
        q.script(140, 153);
        assert_eq!(q.due(185, false, false, false), None);
        assert_eq!(q.due(200, true, false, false), None);
    }
}
