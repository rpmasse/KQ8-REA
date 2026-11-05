//! The original in-world help panel and its 15 Hz tutorial events.
use std::collections::VecDeque;

pub const CATALOG: [u8; 24] = [
    127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145,
    146, 149, 150, 151, 153,
];
const THREE_SECONDS: u64 = 45;
const ONE_SECOND: u64 = 15;

#[derive(Debug, Clone, Default)]
pub struct Requests {
    pending: VecDeque<(u64, u8)>,
    last_attempt: Option<u64>,
    held: VecDeque<u8>,
    pub map_go: bool,
}

impl Requests {
    pub fn script(&mut self, now: u64, case: u8) {
        if !self.pending.iter().any(|(_, pending)| *pending == case) {
            self.pending.push_back((now + THREE_SECONDS, case));
        }
    }

    /// The disabled preference suppresses tutorials, not F1 or the Go button.
    pub fn due(&mut self, now: u64, enabled: bool, busy: bool, occupied: bool) -> Option<u8> {
        let n = self.pending.iter().position(|(due, _)| *due <= now)?;
        let (_, case) = self.pending.remove(n)?;
        if busy
            || self
                .last_attempt
                .is_some_and(|at| now.saturating_sub(at) < THREE_SECONDS)
        {
            if !self.held.contains(&case) {
                self.held.push_back(case);
            }
            self.pending.push_back((now + ONE_SECOND, case));
            return None;
        }
        // Let the earliest held tutorial reach the UI first. Retain the
        // order in which distinct requests first encountered a busy Connor.
        if self.held.front().is_some_and(|first| *first != case) {
            self.pending.push_back((now + THREE_SECONDS, case));
            return None;
        }
        if self.held.front() == Some(&case) {
            self.held.pop_front();
        }
        self.last_attempt = Some(now);
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
