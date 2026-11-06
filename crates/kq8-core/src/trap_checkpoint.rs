//! Mutable generic trap state; original geometry and scripts load by name.
use super::Trap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct State {
    name: String,
    inside: Vec<String>,
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
            inside: t.inside.clone(), shrunk: t.shrunk, gone: t.gone, mask: t.settings.mask, spirit: t.settings.spirit }).collect() }
    }

    pub fn valid(&self) -> bool {
        let name = |n: &str| !n.is_empty() && n.len() <= 1024 && !n.chars().any(char::is_control);
        let mut traps = BTreeSet::new();
        self.states.len() <= 4096 && self.states.iter().all(|t| {
            let mut occupants = BTreeSet::new();
            name(&t.name) && traps.insert(t.name.to_ascii_lowercase()) && t.inside.len() <= 4096
                && t.spirit.is_none_or(|s| s.valid())
                && t.inside.iter().all(|n| name(n) && occupants.insert(n.to_ascii_lowercase()))
        })
    }

    pub fn restore(&self, traps: &mut [Trap], known: &BTreeSet<String>) -> Result<(), String> {
        self.apply(traps, |name| known.contains(&name.to_ascii_lowercase()))
    }

    /// Region objects read their fields before Init. Occupant references can
    /// name actors recreated later in the load. Resolve those before Begin.
    pub fn restore_for_arrival(&self, traps: &mut [Trap]) -> Result<(), String> {
        self.apply(traps, |_| true)
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
                trap.shrunk = t.shrunk; trap.gone = t.gone; trap.settings.mask = t.mask;
                if let Some(spirit) = t.spirit { trap.settings.spirit = Some(spirit); }
            }
        }
        Ok(())
    }
}
