//! Shared simulation queue; behaviour/saved-world-events.md rules 1–15.
//! Rust token payloads are not codecs for unstudied original resource pointers.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PERSISTENT_TYPES: [&str; 29] = [
    "KQSimEvent",
    "KQCueEvent",
    "KQLookupEvent",
    "KQAnimCueEvent",
    "KQAnimPlayEvent",
    "KQAnimTransitionEvent",
    "KQMoveEvent",
    "KQMoveNotifyEvent",
    "KQStateChangeEvent",
    "KQStateChangeNotifyEvent",
    "KQConnerDoEvent",
    "KQStatsChangeRequestEvent",
    "KQAttackEvent",
    "KQCollisionEvent",
    "KQSetGravEvent",
    "KQInventoryNotifyEvent",
    "KQInventoryChangeEvent",
    "KQInventoryItemReceiveEvent",
    "KQStartConversationEvent",
    "KQTalkPlaySequenceEvent",
    "KQTalkMessageSetupEvent",
    "KQTalkMessageCompleteEvent",
    "KQMusicPlayEvent",
    "KQMusicPushEvent",
    "KQMusicPopEvent",
    "KQDoorEvent",
    "KQPuzzleEvent",
    "KQRevealEvent",
    "KQTogglePersistAcceptEvent",
];
pub const STARTUP_SERVICE_SECONDS: f32 = 0.0667;
const MAX_EVENTS: usize = 100_000;
const MAX_TEXT: usize = 4096;

/// None: ordinary runtime dispatch. Some(None): unresolved saved receiver.
/// Some(Some(token)): a particular recreation object lifetime, not a native ID.
pub type Receiver = Option<Option<u64>>;
#[derive(Debug, Clone, PartialEq)]
pub struct Pending {
    /// A widened single-precision deadline.
    pub due: f64,
    pub seq: u64,
    pub target: String,
    pub kind: String,
    pub data: Vec<String>,
    pub from: String,
    pub receiver: Receiver,
}
#[derive(Debug, Clone, Default)]
pub struct EventQueue {
    waiting: Vec<Pending>,
    next_seq: u64,
    receivers: BTreeMap<String, u64>,
    next_receiver: u64,
}
/// New envelopes store absolute times as six-decimal text. The old relative
/// envelope remains readable, but cannot recover its omitted historical clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedEvents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    clock: Option<String>,
    waiting: Vec<SavedEvent>,
    next_seq: u64,
    listeners: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedEvent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deadline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    seconds_remaining: Option<f64>,
    seq: u64,
    target: String,
    kind: String,
    data: Vec<String>,
    from: String,
}
fn text_ok(s: &str) -> bool {
    s.len() <= MAX_TEXT && !s.contains('\0')
}
fn stored_time(t: f64) -> String {
    format!("{:.6}", t as f32)
}
fn read_time(s: &str) -> Option<f32> {
    if s.len() > 64 {
        None
    } else {
        s.parse::<f32>().ok().filter(|v| v.is_finite())
    }
}
pub fn canonical_kind(kind: &str) -> &str {
    match kind {
        "KQMusic::push" => "KQMusicPushEvent",
        "KQMusic::pop" => "KQMusicPopEvent",
        "StateChange" => "KQStateChangeNotifyEvent",
        "TalkComplete" => "KQTalkMessageCompleteEvent",
        "__kq8_trap_lookup" => "KQLookupEvent",
        _ => kind,
    }
}
pub fn persistent(kind: &str) -> bool {
    PERSISTENT_TYPES.contains(&canonical_kind(kind))
}
impl SavedEvents {
    pub fn clock_seconds(&self) -> Option<f32> {
        self.clock.as_deref().and_then(read_time)
    }
    pub fn legacy_relative(&self) -> bool {
        self.clock.is_none()
    }
    pub(crate) fn listeners(&self) -> &BTreeMap<String, String> {
        &self.listeners
    }
    pub fn valid(&self) -> bool {
        let mut seqs = BTreeSet::new();
        self.waiting.len() <= MAX_EVENTS
            && self.next_seq < u64::MAX / 2
            && self
                .clock
                .as_deref()
                .is_none_or(|s| read_time(s).is_some_and(|n| n >= 0.))
            && self.listeners.len() <= MAX_EVENTS
            && self
                .listeners
                .iter()
                .all(|(a, b)| !a.is_empty() && !b.is_empty() && text_ok(a) && text_ok(b))
            && self.waiting.iter().all(|e| {
                e.seq < self.next_seq
                    && seqs.insert(e.seq)
                    && !e.target.is_empty()
                    && text_ok(&e.target)
                    && !e.kind.is_empty()
                    && text_ok(&e.kind)
                    && e.data.len() <= 256
                    && e.data.iter().all(|s| text_ok(s))
                    && text_ok(&e.from)
                    && match (&self.clock, &e.deadline, e.seconds_remaining) {
                        (Some(_), Some(t), None) => read_time(t).is_some(),
                        (None, None, Some(t)) => t.is_finite() && (t as f32).is_finite(),
                        _ => false,
                    }
            })
    }
}
impl EventQueue {
    pub(crate) fn register_receiver(&mut self, name: &str) -> u64 {
        let key = name.to_ascii_lowercase();
        if let Some(id) = self.receivers.get(&key) {
            return *id;
        }
        let id = self.next_receiver;
        self.next_receiver += 1;
        self.receivers.insert(key, id);
        id
    }
    pub(crate) fn receiver(&self, name: &str) -> Option<u64> {
        self.receivers.get(&name.to_ascii_lowercase()).copied()
    }
    pub(crate) fn remove_receiver(&mut self, name: &str) {
        self.waiting
            .retain(|e| !e.target.eq_ignore_ascii_case(name));
        self.receivers.remove(&name.to_ascii_lowercase());
    }
    pub(crate) fn deliverable(&self, e: &Pending) -> bool {
        match e.receiver {
            None => true,
            Some(None) => false,
            Some(Some(id)) => self.receiver(&e.target) == Some(id),
        }
    }
    pub(crate) fn shift_clock(&mut self, old: f32, new: f32) {
        let delta = new - old;
        for e in &mut self.waiting {
            e.due = if e.due as f32 == old {
                new as f64
            } else {
                ((e.due as f32) + delta) as f64
            };
        }
    }
    pub(crate) fn checkpoint(
        &self,
        now: f64,
        held: &[Pending],
        listeners: &BTreeMap<String, String>,
    ) -> SavedEvents {
        let mut ordered: Vec<_> = self
            .waiting
            .iter()
            .chain(held)
            .filter(|e| persistent(&e.kind) && self.deliverable(e))
            .collect();
        ordered.sort_by(|a, b| a.due.total_cmp(&b.due).then(a.seq.cmp(&b.seq)));
        let waiting = ordered
            .into_iter()
            .enumerate()
            .map(|(seq, e)| SavedEvent {
                deadline: Some(stored_time(e.due)),
                seconds_remaining: None,
                seq: seq as u64,
                target: e.target.clone(),
                kind: canonical_kind(&e.kind).into(),
                data: e.data.clone(),
                from: e.from.clone(),
            })
            .collect::<Vec<_>>();
        SavedEvents {
            clock: Some(stored_time(now)),
            next_seq: waiting.len() as u64,
            waiting,
            listeners: listeners.clone(),
        }
    }
    /// Merge after logical object loading; equal-time Init entries stay first.
    pub(crate) fn merge(&mut self, now: f64, saved: SavedEvents) -> Vec<String> {
        let mut notes = Vec::new();
        let legacy = saved.legacy_relative();
        if legacy {
            notes.push("legacy relative events: original world clock was not saved; remaining delays use the current clock".into());
        }
        let mut waiting = saved.waiting;
        if legacy {
            waiting.sort_by(|a, b| {
                a.seconds_remaining
                    .unwrap()
                    .total_cmp(&b.seconds_remaining.unwrap())
                    .then(a.seq.cmp(&b.seq))
            });
        }
        for e in waiting {
            if !persistent(&e.kind) {
                notes.push(format!(
                    "saved event {} to {} omitted: outside supported persistent family",
                    e.kind, e.target
                ));
                continue;
            }
            let due = if legacy {
                (now as f32 + e.seconds_remaining.unwrap() as f32) as f64
            } else {
                read_time(e.deadline.as_deref().unwrap()).unwrap() as f64
            };
            let receiver = Some(self.receiver(&e.target));
            if receiver == Some(None) {
                notes.push(format!("saved event {} to {} has no registered receiver; discarded when due, never rebound",e.kind,e.target));
            }
            self.waiting.push(Pending {
                due,
                seq: self.next_seq,
                target: e.target,
                kind: canonical_kind(&e.kind).into(),
                data: e.data,
                from: e.from,
                receiver,
            });
            self.next_seq += 1;
        }
        notes
    }
    fn send_at(&mut self, due: f64, target: &str, kind: &str, data: &[String], from: &str) {
        if !due.is_finite() {
            return;
        }
        self.waiting.push(Pending {
            due,
            seq: self.next_seq,
            target: target.into(),
            kind: kind.into(),
            data: data.to_vec(),
            from: from.into(),
            receiver: self.receiver(target).map(Some),
        });
        self.next_seq += 1;
    }
    pub fn send(
        &mut self,
        now: f64,
        delay: f64,
        target: &str,
        kind: &str,
        data: &[String],
        from: &str,
    ) {
        self.send_at(
            (now as f32 + (delay as f32).max(0.)) as f64,
            target,
            kind,
            data,
            from,
        );
    }
    /// Authored sendEvent crosses its own six-decimal event text boundary.
    pub(crate) fn send_script(
        &mut self,
        now: f64,
        delay: f64,
        target: &str,
        kind: &str,
        data: &[String],
        from: &str,
    ) {
        if delay == 0. {
            self.send_at(now as f32 as f64, target, kind, data, from);
            return;
        }
        let due = (now as f32 + (delay as f32).max(0.)) as f64;
        if let Some(due) = read_time(&stored_time(due)) {
            self.send_at(due as f64, target, kind, data, from);
        }
    }
    pub fn take_due(&mut self, now: f64) -> Option<Pending> {
        let i = self
            .waiting
            .iter()
            .enumerate()
            .filter(|(_, e)| e.due <= now as f32 as f64)
            .min_by(|(_, a), (_, b)| a.due.total_cmp(&b.due).then(a.seq.cmp(&b.seq)))
            .map(|(i, _)| i)?;
        Some(self.waiting.remove(i))
    }
    pub fn len(&self) -> usize {
        self.waiting.len()
    }
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_world_queue_family_precision_order_and_lifetime_matrix() {
        let mut old = EventQueue::default();
        old.register_receiver("live");
        for (i, kind) in PERSISTENT_TYPES.iter().enumerate() {
            old.send(
                103.,
                (29 - i) as f64,
                "live",
                kind,
                &["payload".into()],
                "family",
            );
        }
        for kind in [
            "SimTimeEvent",
            "SimTimerEvent",
            "KQTalkMouthPositionEvent",
            "unknown",
        ] {
            old.send(103., 0., "live", kind, &[], "transient");
        }
        let saved = old.checkpoint(103., &[], &BTreeMap::new());
        assert!(saved.valid());
        assert_eq!(saved.waiting.len(), 29);
        assert_eq!(saved.waiting[0].kind, "KQTogglePersistAcceptEvent");
        assert_eq!(saved.clock.as_deref(), Some("103.000000"));
        assert!(saved.waiting.iter().all(|e| e.data == ["payload"]));
        for (alias, kind) in [
            ("KQMusic::push", "KQMusicPushEvent"),
            ("KQMusic::pop", "KQMusicPopEvent"),
            ("StateChange", "KQStateChangeNotifyEvent"),
            ("TalkComplete", "KQTalkMessageCompleteEvent"),
            ("__kq8_trap_lookup", "KQLookupEvent"),
        ] {
            assert_eq!(canonical_kind(alias), kind);
            assert!(persistent(alias));
        }
        let mut old = EventQueue::default();
        old.send(
            0.,
            0.0000004,
            "same",
            "KQCueEvent",
            &["later".into()],
            "later sent first",
        );
        old.send(
            0.,
            0.0000003,
            "same",
            "KQCueEvent",
            &["earlier".into()],
            "earlier sent last",
        );
        let saved = old.checkpoint(0., &[], &BTreeMap::new());
        assert_eq!(saved.waiting[0].deadline, saved.waiting[1].deadline);
        let saved: SavedEvents =
            serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
        let mut q = EventQueue::default();
        q.register_receiver("same");
        q.send(
            0.,
            0.,
            "same",
            "KQCueEvent",
            &["earlier".into()],
            "Init duplicate",
        );
        q.merge(0., saved);
        let events: Vec<_> = std::iter::from_fn(|| q.take_due(0.)).collect();
        assert_eq!(
            events
                .iter()
                .map(|e| e.data[0].as_str())
                .collect::<Vec<_>>(),
            ["earlier", "earlier", "later"]
        );
        let fractional = 0.123456789f32 as f64;
        q.send_script(fractional, 0., "same", "KQCueEvent", &[], "exactzero");
        assert_eq!(q.take_due(1.).unwrap().due, fractional);
        q.send_script(
            fractional,
            0.0000001,
            "same",
            "KQCueEvent",
            &[],
            "textboundary",
        );
        assert_eq!(q.take_due(1.).unwrap().due, 0.123457f32 as f64);

        let mut q = EventQueue::default();
        q.send(4., 0., "live", "KQCueEvent", &[], "Init now");
        q.send(4., 2., "live", "KQCueEvent", &[], "Init later");
        q.shift_clock(4., 103.);
        assert_eq!(q.take_due(103.).unwrap().due, 103.);
        assert!(q.take_due(104.).is_none());
        assert_eq!(q.take_due(105.).unwrap().due, 105.);
        let mut source = EventQueue::default();
        source.send(103., 4., "live", "KQCueEvent", &[], "saved future");
        source.send(103., 4., "missing", "KQCueEvent", &[], "missing");
        let saved = source.checkpoint(103., &[], &BTreeMap::new());
        q.register_receiver("live");
        q.merge(103., saved);
        q.register_receiver("missing");
        let good = q.take_due(107.).unwrap();
        assert!(q.deliverable(&good));
        let absent = q.take_due(107.).unwrap();
        assert!(!q.deliverable(&absent));
        q.send(107., 1., "live", "KQCueEvent", &[], "remove");
        let saved = q.checkpoint(107., &[], &BTreeMap::new());
        q.merge(107., saved);
        q.remove_receiver("live");
        q.register_receiver("live");
        assert!(q.is_empty());
        assert!(!q.deliverable(&good));
        let legacy:SavedEvents=serde_json::from_value(serde_json::json!({"waiting":[{"seconds_remaining":4.0,"seq":0,"target":"live","kind":"KQCueEvent","data":[],"from":"old"}],"next_seq":1,"listeners":{}})).unwrap();
        assert!(legacy.valid());
        assert!(legacy.legacy_relative());
        assert_eq!(legacy.clock_seconds(), None);
        assert!(q.merge(2., legacy)[0].contains("clock was not saved"));
        assert_eq!(q.take_due(6.).unwrap().due, 6.);
        let mut malformed =
            serde_json::to_value(source.checkpoint(103., &[], &BTreeMap::new())).unwrap();
        malformed["clock"] = "NaN".into();
        assert!(!serde_json::from_value::<SavedEvents>(malformed)
            .unwrap()
            .valid());
    }
}
