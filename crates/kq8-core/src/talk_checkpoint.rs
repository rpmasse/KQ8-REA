//! Speech queues on a disk save. Message text and audio stay in the install.
use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedTalk {
    queues: Vec<Queue>,
    slots: BTreeMap<String, Speak>,
    waits: Vec<(f64, String)>,
    next_id: u64,
    goodbyes_ended: Vec<String>,
}

impl SavedTalk {
    pub fn valid(&self) -> bool {
        let name = |s: &str| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control);
        let clock = |t: f64| t.is_finite() && t.abs() <= 1_000_000.0;
        let tuple = |t: Tuple| t.seq != 0;
        let mut ids = std::collections::BTreeSet::new();
        self.queues.len() <= 4096 && self.slots.len() <= 4096 && self.waits.len() <= 16384
            && self.queues.iter().all(|q| name(&q.issuer) && q.lines.len() <= 16384
                && q.lines.iter().all(|e| tuple(e.tuple) && tuple(e.first))
                && q.playing.as_ref().is_none_or(|p| {
                    name(&p.speaker) && tuple(p.tuple) && tuple(p.first)
                        && p.id < self.next_id && ids.insert(p.id)
                        && clock(p.started) && clock(p.ends) && p.started <= 1e-6 && p.ends >= p.started
                        && p.mouths.len() <= 65536
                        && p.mouths.iter().map(|m| m.time).collect::<Vec<_>>().windows(2).all(|w| w[0] <= w[1])
                }))
            && self.slots.iter().all(|(key, s)| name(key) && name(&s.issuer) && tuple(s.first) && s.last >= s.first.seq)
            && self.waits.iter().all(|(t, key)| clock(*t) && name(key))
            && self.goodbyes_ended.len() <= 4096 && self.goodbyes_ended.iter().all(|n| name(n))
    }
}

impl Talk {
    pub fn saved(&self, now: f64) -> SavedTalk {
        let mut queues = self.queues.clone();
        for p in queues.iter_mut().filter_map(|q| q.playing.as_mut()) {
            p.started -= now;
            p.ends -= now;
        }
        SavedTalk { queues, slots: self.slots.clone(), waits: self.waits.iter().map(|(t, n)| (t - now, n.clone())).collect(),
            next_id: self.next_id, goodbyes_ended: self.goodbyes_ended.clone() }
    }

    pub fn restore(&mut self, saved: &SavedTalk, now: f64, captions: bool) -> Result<(), String> {
        if !saved.valid() || !now.is_finite() { return Err("invalid saved speech queues".into()); }
        let mut events = Vec::new();
        for q in &saved.queues {
            let Some(p) = &q.playing else { continue };
            let (folder, line) = self.book.line(p.tuple).ok_or_else(|| format!("saved speech {} is absent from the installed message file", p.tuple))?;
            let voice = self.voices.as_ref().and_then(|v| v(folder, p.tuple)).filter(|_| p.voiced);
            if p.voiced && voice.is_none() { return Err(format!("saved speech {} has no installed voice", p.tuple)); }
            let elapsed = (-p.started).max(0.0);
            let mouth = voice.as_ref().and_then(|v| v.mouths.iter().rev().find(|m| f64::from(m.time) * LIP_TICK <= elapsed && m.shape <= 7)).map_or(0, |m| m.shape);
            events.push(TalkEvent::Resume { id: p.id, issuer: q.issuer.clone(), speaker: p.speaker.clone(), tuple: p.tuple,
                voice: voice.map(|v| v.path), caption: captions.then(|| line.shown().text), elapsed, mouth });
        }
        self.queues = saved.queues.clone();
        for p in self.queues.iter_mut().filter_map(|q| q.playing.as_mut()) {
            p.started += now;
            p.ends += now;
        }
        self.slots = saved.slots.clone();
        self.waits = saved.waits.iter().map(|(t, n)| (t + now, n.clone())).collect();
        self.next_id = saved.next_id;
        self.goodbyes_ended = saved.goodbyes_ended.clone();
        self.events = events;
        self.note(now, format!("speech checkpoint restored: {} queues, {} playing lines, {} delayed lines; no line-start camera cue replayed",
            self.queues.len(), self.queues.iter().filter(|q| q.playing.is_some()).count(), self.waits.len()));
        Ok(())
    }
}
