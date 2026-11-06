//! The event queue behind `sendEvent` (M3 step 3a-2).
//!
//! Rule from `docs/behaviour/doors.md` "Commands door scripts use"
//! (traced): `sendEvent <delay> <object> <type> [data…]` sends the event
//! after `<delay>` seconds of game time; `0` means the next time the queue
//! is handled, after events already waiting for "now"; at the same time,
//! the first sent is handled first.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Serialize, Deserialize};

/// An event waiting to be handled.
#[derive(Debug, Clone, PartialEq)]
pub struct Pending {
    /// Game time (seconds) it is due.
    pub due: f64,
    /// Send order, for events due at the same time.
    pub seq: u64,
    pub target: String,
    pub kind: String,
    pub data: Vec<String>,
    /// Script line that sent it (for the log).
    pub from: String,
}

#[derive(Debug, Clone, Default)]
pub struct EventQueue {
    waiting: Vec<Pending>,
    next_seq: u64,
}

/// Version-2 disk saves retain the script queue in simulation time, along
/// with the registered targets of getEndLoop. World travel starts a new
/// queue, whereas restoring a save continues its current one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedEvents {
    waiting: Vec<SavedEvent>,
    next_seq: u64,
    listeners: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedEvent {
    /// Keep negative values too: two overdue events can have different
    /// deadlines, and must still dispatch in that order.
    seconds_remaining: f64,
    seq: u64,
    target: String,
    kind: String,
    data: Vec<String>,
    from: String,
}

impl SavedEvents {
    pub fn valid(&self) -> bool {
        let mut seqs = BTreeSet::new();
        self.next_seq < u64::MAX / 2 && self.waiting.iter().all(|event| {
            event.seq < self.next_seq && seqs.insert(event.seq)
                && event.seconds_remaining.is_finite() && !event.kind.is_empty()
        })
    }
}

impl EventQueue {
    pub(crate) fn sequence(&self) -> u64 { self.next_seq }

    pub(crate) fn immediate_since(&self, seq: u64, now: f64) -> Vec<Pending> {
        let mut notices: Vec<_> = self.waiting.iter().filter(|e| e.seq >= seq && (e.due - now).abs() <= 1e-9).cloned().collect();
        notices.sort_by_key(|e| e.seq);
        notices
    }

    pub(crate) fn restore_with_init(&mut self, now: f64, saved: SavedEvents, notices: Vec<Pending>) -> BTreeMap<String, String> {
        let listeners = self.restore(now, saved);
        let saved_end = self.next_seq;
        for notice in notices {
            // An equivalent retained notification keeps its existing deadline.
            // Newly emitted Init duplicates still keep their own multiplicity.
            let retained = self.waiting.iter().any(|e| e.seq < saved_end && e.target.eq_ignore_ascii_case(&notice.target) && e.kind == notice.kind && e.data == notice.data);
            if !retained { self.send(now, 0.0, &notice.target, &notice.kind, &notice.data, &notice.from); }
        }
        listeners
    }

    pub(crate) fn checkpoint(&self, now: f64, held_for_load: &[Pending], listeners: &BTreeMap<String, String>) -> SavedEvents {
        // A due event can be held briefly until the renderer hands over
        // a script-loaded object. On restore its object is built with the
        // world, so include that event with its original deadline/order.
        let waiting = self.waiting.iter().chain(held_for_load).map(|e| SavedEvent {
            seconds_remaining: e.due - now, seq: e.seq, target: e.target.clone(), kind: e.kind.clone(), data: e.data.clone(), from: e.from.clone(),
        }).collect();
        SavedEvents { waiting, next_seq: self.next_seq, listeners: listeners.clone() }
    }

    pub(crate) fn restore(&mut self, now: f64, saved: SavedEvents) -> BTreeMap<String, String> {
        // Replace the queue built during object Init. Appending would
        // duplicate recurring timers and already-consumed startup events.
        self.waiting = saved.waiting.into_iter().map(|e| Pending {
            due: now + e.seconds_remaining, seq: e.seq, target: e.target, kind: e.kind, data: e.data, from: e.from,
        }).collect();
        self.next_seq = saved.next_seq;
        saved.listeners
    }

    /// Queues an event `delay` seconds after `now`.
    pub fn send(&mut self, now: f64, delay: f64, target: &str, kind: &str, data: &[String], from: &str) {
        self.waiting.push(Pending {
            due: now + delay.max(0.0),
            seq: self.next_seq,
            target: target.to_string(),
            kind: kind.to_string(),
            data: data.to_vec(),
            from: from.to_string(),
        });
        self.next_seq += 1;
    }

    /// The next event due by `now`: earliest time first, then send order.
    pub fn take_due(&mut self, now: f64) -> Option<Pending> {
        // A small allowance so 0.07 s after one update is not missed by
        // rounding when it falls exactly on a later update's time.
        let i = self
            .waiting
            .iter()
            .enumerate()
            .filter(|(_, e)| e.due <= now + 1e-9)
            .min_by(|(_, a), (_, b)| a.due.total_cmp(&b.due).then(a.seq.cmp(&b.seq)))
            .map(|(i, _)| i)?;
        Some(self.waiting.remove(i))
    }

    pub fn len(&self) -> usize {
        self.waiting.len()
    }

    pub(crate) fn has_target_kind(&self, target: &str, kind: &str) -> bool {
        self.waiting.iter().any(|e| e.target.eq_ignore_ascii_case(target) && e.kind == kind)
    }

    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_keeps_immediate_init_notices_after_saved_events_without_resetting_timers() {
        let mut old = EventQueue::default();
        old.send(10., 0., "already-due", "KQCueEvent", &[], "saved");
        old.send(10., 2., "clock", "KQCueEvent", &["7".into()], "timer loop");
        let saved = old.checkpoint(10., &[], &BTreeMap::new());
        let mut q = EventQueue::default();
        q.send(0., 0., "not-from-init", "KQCueEvent", &[], "earlier input");
        let mark = q.sequence();
        q.send(0., 0., "Clock", "KQCueEvent", &["7".into()], "Init line");
        q.send(0., 0., "fuse", "KQCueEvent", &["10".into(), "0".into(), "on".into()], "Init");
        q.send(0., 0., "fuse", "KQCueEvent", &["10".into(), "0".into(), "on".into()], "another Init send");
        q.send(0., 9., "future-init", "KQCueEvent", &[], "Init timer");
        let notices = q.immediate_since(mark, 0.);
        assert_eq!(notices.len(), 3);
        q.restore_with_init(0., saved, notices);
        assert_eq!(q.take_due(0.).unwrap().target, "already-due");
        assert_eq!(q.take_due(0.).unwrap().target, "fuse");
        assert_eq!(q.take_due(0.).unwrap().target, "fuse");
        assert!(q.take_due(1.999).is_none());
        let timer = q.take_due(2.).unwrap();
        assert_eq!((timer.target.as_str(), timer.from.as_str()), ("clock", "timer loop"));
        assert!(q.take_due(100.).is_none());
    }

    #[test]
    fn time_order_then_send_order() {
        let mut q = EventQueue::default();
        q.send(1.0, 0.5, "late", "KQDoorEvent", &[], "");
        q.send(1.0, 0.0, "first", "KQDoorEvent", &[], "");
        q.send(1.0, 0.0, "second", "KQDoorEvent", &[], "");
        q.send(1.0, 0.07, "soon", "KQCueEvent", &["1".into(), "0".into()], "");
        let names = |q: &mut EventQueue, now| std::iter::from_fn(|| q.take_due(now)).map(|e| e.target).collect::<Vec<_>>();
        assert_eq!(names(&mut q, 1.0), ["first", "second"]);
        // 0.07 s later: not after one update (1/15 s), but after two.
        assert!(names(&mut q, 1.0 + 1.0 / 15.0).is_empty());
        assert_eq!(names(&mut q, 1.0 + 2.0 / 15.0), ["soon"]);
        assert_eq!(names(&mut q, 2.0), ["late"]);
        assert!(q.is_empty());
    }

    #[test]
    fn checkpoint_rebases_deadlines_and_replaces_init_preserving_ties() {
        let mut q = EventQueue::default();
        q.send(5.0, 0.0, "held", "KQCueEvent", &["1".into()], "object load");
        let held = q.take_due(5.0).unwrap();
        q.send(4.0, 0.0, "older", "KQCueEvent", &[], "test");
        q.send(5.0, 2.0, "first", "KQCueEvent", &[], "test");
        q.send(5.0, 2.0, "second", "KQCueEvent", &[], "test");
        let listeners = BTreeMap::from([("bridge".to_string(), "trap".to_string())]);
        let saved = q.checkpoint(6.0, &[held], &listeners);
        assert!(saved.valid());
        let mut rebuilt = EventQueue::default();
        rebuilt.send(0.0, 1.0, "duplicate-init", "KQCueEvent", &[], "Init");
        assert_eq!(rebuilt.restore(0.0, saved), listeners);
        rebuilt.send(0.0, 1.0, "new", "KQCueEvent", &[], "after load");
        assert_eq!(rebuilt.take_due(0.0).unwrap().target, "older");
        let held = rebuilt.take_due(0.0).unwrap();
        assert_eq!((held.target.as_str(), held.data[0].as_str(), held.from.as_str()), ("held", "1", "object load"));
        assert!(rebuilt.take_due(0.99).is_none());
        let names: Vec<_> = std::iter::from_fn(|| rebuilt.take_due(1.0)).map(|e| e.target).collect();
        assert_eq!(names, ["first", "second", "new"]);
    }
}
