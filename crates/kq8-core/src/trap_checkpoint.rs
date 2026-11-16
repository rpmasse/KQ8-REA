//! Mutable generic trap state; original geometry and scripts load by name.
use super::Trap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct State {
    name: String,
    inside: Vec<String>,
    /// Rust continuation safeguard for a save made before all contacts attach.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pending_inside: Vec<String>,
    shrunk: bool,
    gone: bool,
    mask: u32,
    #[serde(default)]
    spirit: Option<crate::spirit_trap::SpiritTrap>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedTraps {
    states: Vec<State>,
}

impl SavedTraps {
    pub fn capture<'a>(traps: impl IntoIterator<Item = &'a Trap>) -> Self {
        Self { states: traps.into_iter().map(|t| State { name: t.settings.name.to_ascii_lowercase(),
            inside: t.inside.clone(), pending_inside: t.pending_occupants.clone(),
            shrunk: t.shrunk, gone: t.gone, mask: t.settings.mask, spirit: t.settings.spirit }).collect() }
    }

    pub fn valid(&self) -> bool {
        let name = |n: &str| !n.is_empty() && n.len() <= 1024 && !n.chars().any(char::is_control);
        let mut traps = BTreeSet::new();
        self.states.len() <= 4096 && self.states.iter().all(|t| {
            let mut occupants = BTreeSet::new();
            name(&t.name) && traps.insert(t.name.to_ascii_lowercase()) && t.inside.len() + t.pending_inside.len() <= 4096
                && t.spirit.is_none_or(|s| s.valid())
                && t.inside.iter().chain(&t.pending_inside).all(|n| name(n) && occupants.insert(n.to_ascii_lowercase()))
        })
    }

    pub fn restore(&self, traps: &mut [Trap], known: &BTreeSet<String>) -> Result<(), String> {
        self.apply(traps, |name| known.contains(&name.to_ascii_lowercase()))
    }

    /// Complete a disk restore after saved items have had their initial
    /// collider contact. Other occupant classes still use the bulk loader.
    /// A saved item without actual contact remains pending, never live.
    pub fn restore_after_item_registration(&self, traps: &mut [Trap], known: &BTreeSet<String>,
        item_names: &BTreeSet<String>) -> Result<(), String> {
        let contacted: Vec<_> = traps.iter().map(|trap| {
            (trap.settings.name.to_ascii_lowercase(), trap.inside.iter()
                .filter(|name| item_names.contains(&name.to_ascii_lowercase())).cloned().collect::<Vec<_>>())
        }).collect();
        self.restore(traps, known)?;
        for trap in traps {
            let Some(state) = self.states.iter().find(|s| s.name.eq_ignore_ascii_case(&trap.settings.name)) else { continue };
            let live_items = &contacted.iter().find(|(name, _)| name.eq_ignore_ascii_case(&trap.settings.name))
                .expect("every restored trap was captured before bulk restore").1;
            let legacy = std::mem::take(&mut trap.inside);
            let mut inside = Vec::new();
            for saved in state.inside.iter().chain(&state.pending_inside) {
                let source = if item_names.contains(&saved.to_ascii_lowercase()) { live_items } else { &legacy };
                if let Some(found) = source.iter().find(|name| name.eq_ignore_ascii_case(saved)) {
                    if !inside.iter().any(|name: &String| name.eq_ignore_ascii_case(found)) { inside.push(found.clone()); }
                }
            }
            for live in live_items {
                if !inside.iter().any(|name| name.eq_ignore_ascii_case(live)) { inside.push(live.clone()); }
            }
            trap.inside = inside;
            trap.pending_occupants = state.inside.iter().chain(&state.pending_inside)
                .filter(|name| {
                    // Non-item saved `inside` names are handled by the
                    // legacy restore. Only unresolved items stay pending.
                    (item_names.contains(&name.to_ascii_lowercase()) || state.pending_inside.iter()
                        .any(|pending| pending.eq_ignore_ascii_case(name)))
                        && !trap.inside.iter().any(|live| live.eq_ignore_ascii_case(name))
                }).cloned().collect();
        }
        Ok(())
    }

    /// Legacy bulk-loader compatibility only: names are copied to membership.
    /// This does not establish the original contact-before-Init behavior.
    pub fn restore_for_arrival(&self, traps: &mut [Trap]) -> Result<(), String> {
        self.apply(traps, |_| true)
    }

    /// Apply one object's saved fields before active registration and Init.
    /// A later eligible contact, not this call, establishes live occupancy.
    /// Returns false when this snapshot has no entry for the new trap.
    pub fn restore_before_registration(&self, trap: &mut Trap) -> Result<bool, String> {
        if !self.valid() { return Err("invalid saved trap state".into()); }
        let Some(state) = self.states.iter().find(|s| s.name.eq_ignore_ascii_case(&trap.settings.name)) else {
            return Ok(false);
        };
        if state.spirit.is_some() && trap.settings.spirit.is_none() {
            return Err(format!("saved spirit trap '{}' has another restored class", state.name));
        }
        trap.prepare_saved_occupants(state.inside.iter().chain(&state.pending_inside).cloned().collect())?;
        trap.shrunk = state.shrunk;
        trap.gone = state.gone;
        trap.settings.mask = state.mask;
        if let Some(spirit) = state.spirit { trap.settings.spirit = Some(spirit); }
        Ok(true)
    }

    fn apply(&self, traps: &mut [Trap], keep_occupant: impl Fn(&str) -> bool) -> Result<(), String> {
        if !self.valid() { return Err("invalid saved trap state".into()); }
        for t in &self.states {
            if !t.gone && !traps.iter().any(|v| v.settings.name.eq_ignore_ascii_case(&t.name)) {
                return Err(format!("saved trap '{}' is absent from the restored world", t.name));
            }
            if t.spirit.is_some() && traps.iter().any(|v| v.settings.name.eq_ignore_ascii_case(&t.name) && v.settings.spirit.is_none()) {
                return Err(format!("saved spirit trap '{}' has another restored class",t.name));
            }
        }
        for t in &self.states {
            if let Some(trap) = traps.iter_mut().find(|v| v.settings.name.eq_ignore_ascii_case(&t.name)) {
                trap.inside = t.inside.iter().filter(|n| keep_occupant(n)).cloned().collect();
                trap.pending_occupants = t.pending_inside.clone();
                trap.shrunk = t.shrunk; trap.gone = t.gone; trap.settings.mask = t.mask;
                if let Some(spirit) = t.spirit { trap.settings.spirit = Some(spirit); }
            }
        }
        Ok(())
    }
}
