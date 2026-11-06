//! The original in-world help panel and its 15 Hz tutorial events.
use std::collections::{BTreeSet, VecDeque};

pub const CATALOG: [u8; 24] = [
    127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145,
    146, 149, 150, 151, 153,
];
const THREE_SECONDS: u64 = 45;
const ONE_SECOND: u64 = 15;

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
    pending: VecDeque<(u64, u8)>,
    last_attempt: Option<i128>,
    start_help: u32,
    held: VecDeque<u8>,
    pub map_go: bool,
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
        History {
            start_help: self.start_help,
            updates_since_delivery: self
                .last_attempt
                .map(|at| (i128::from(now) - at).clamp(0, i128::from(u64::MAX)) as u64),
            held_cases: self.held.clone(),
        }
    }

    pub fn restore_history(&mut self, history: &History, now: u64) {
        self.pending.clear();
        self.map_go = false;
        self.start_help = history.start_help;
        self.last_attempt = history
            .updates_since_delivery
            .map(|age| i128::from(now) - i128::from(age));
        self.held.clone_from(&history.held_cases);
    }

    pub fn script(&mut self, now: u64, case: u8) {
        self.schedule(now + THREE_SECONDS, case);
    }

    /// Native item notices and delayed script requests reach the same
    /// Connor delivery gate, but have different first eligible updates.
    pub(crate) fn schedule(&mut self, first_update: u64, case: u8) {
        if !self.pending.iter().any(|(_, pending)| *pending == case) {
            self.pending.push_back((first_update, case));
        }
    }

    /// The disabled preference suppresses tutorials, not F1 or the Go button.
    pub fn due(&mut self, now: u64, enabled: bool, busy: bool, occupied: bool) -> Option<u8> {
        let n = self.pending.iter().position(|(due, _)| *due <= now)?;
        let (_, case) = self.pending.remove(n)?;
        if busy
            || self
                .last_attempt
                .is_some_and(|at| i128::from(now) - at < i128::from(THREE_SECONDS))
        {
            if !self.held.contains(&case) {
                self.held.push_back(case);
            }
            self.pending.push_back((now + ONE_SECOND, case));
            return None;
        }
        // Let the earliest held tutorial reach the UI first. Retain the
        // order in which distinct requests first encountered a busy Connor.
        if self
            .held
            .iter()
            .take_while(|&&older| older != case)
            .any(|older| self.pending.iter().any(|(_, pending)| pending == older))
        {
            self.pending.push_back((now + THREE_SECONDS, case));
            return None;
        }
        self.held.retain(|&held| held != case);
        self.last_attempt = Some(i128::from(now));
        if case < 170 && !enabled {
            return None;
        }
        if occupied {
            self.pending.push_back((now + THREE_SECONDS, case));
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
