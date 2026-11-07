//! Shared core restoration phases, not the ordered viewer loader itself.
//! Source: behaviour/saved-world-events.md rules 1–15.
use super::*;
use crate::events::{SavedEvents, STARTUP_SERVICE_SECONDS};

impl WorldState {
    /// Existing numerical fixtures advance elapsed time independently of the
    /// diagnostic counter. This is not restoration's deadline-shifting command.
    #[cfg(test)]
    pub(crate) fn advance_clock_for_test(&mut self, time: f32) {
        assert!(time.is_finite() && time >= self.simulation_time);
        self.simulation_time = time;
    }
    /// Call only when the logical game object has registered, not on cataloging.
    pub fn register_event_receiver(&mut self, name: &str) {
        self.queue.register_receiver(name);
    }

    /// Cancel work for this object lifetime before removing/reusing its name.
    pub fn remove_event_receiver(&mut self, name: &str) {
        let key = name.to_ascii_lowercase();
        self.queue.remove_receiver(name);
        self.held_for_load
            .retain(|e| !e.target.eq_ignore_ascii_case(name));
        self.pending_end_loop_listeners.remove(&key);
        self.end_loop_listeners
            .retain(|from, to| from != &key && !to.eq_ignore_ascii_case(name));
        self.end_loop_listener_ids
            .retain(|from, _| self.end_loop_listeners.contains_key(from));
    }

    /// Clock reassignment shifts already-live work. Ordinary advancement does not.
    pub fn set_current_time(&mut self, time: f32) -> Result<(), String> {
        if !time.is_finite() || time < 0. {
            return Err("invalid simulation time".into());
        }
        let old = self.simulation_time;
        self.queue.shift_clock(old, time);
        let delta = time - old;
        for e in &mut self.held_for_load {
            e.due = if e.due as f32 == old {
                time as f64
            } else {
                (e.due as f32 + delta) as f64
            };
        }
        self.simulation_time = time;
        Ok(())
    }

    pub fn saved_events(&self) -> SavedEvents {
        let mut listeners = self.end_loop_listeners.clone();
        // A pending restored name is still a property even before it resolves.
        listeners.extend(self.pending_end_loop_listeners.clone());
        // Held deliveries are game messages waiting for an already-requested
        // logical load, not arbitrary engine continuation records.
        let held: Vec<_> = self
            .held_for_load
            .iter()
            .filter(|e| self.loading.contains(&e.target.to_ascii_lowercase()))
            .cloned()
            .collect();
        self.queue.checkpoint(self.now(), &held, &listeners)
    }

    /// Before restored object registration/Init. Does not service any event.
    pub fn prepare_saved_world(&mut self, saved: &SavedEvents) -> Result<(), String> {
        if !saved.valid() {
            return Err("invalid saved world events".into());
        }
        if let Some(clock) = saved.clock_seconds() {
            self.set_current_time(clock)?;
        } else {
            self.note("legacy saved world: historical simulation clock unavailable; keeping current clock");
        }
        self.pending_end_loop_listeners = saved
            .listeners()
            .iter()
            .map(|(a, b)| (a.to_ascii_lowercase(), b.clone()))
            .collect();
        for name in self.pending_end_loop_listeners.keys() {
            self.end_loop_listeners.remove(name);
            self.end_loop_listener_ids.remove(name);
        }
        Ok(())
    }

    /// Call immediately after this restored animated object's ordinary Init.
    /// Do not deduplicate a saved lookup against this newly posted one.
    pub fn finish_saved_object_init(&mut self, name: &str) {
        if self
            .pending_end_loop_listeners
            .contains_key(&name.to_ascii_lowercase())
            && self.queue.receiver(name).is_some()
        {
            self.queue.send(
                self.now(),
                0.,
                name,
                "KQLookupEvent",
                &[],
                "saved listener after Init",
            );
        }
    }

    pub(super) fn resolve_saved_listener(&mut self, name: &str) {
        let key = name.to_ascii_lowercase();
        let Some(to) = self.pending_end_loop_listeners.remove(&key) else {
            return;
        };
        self.end_loop_listeners.remove(&key);
        self.end_loop_listener_ids.remove(&key);
        if let Some(id) = self.queue.receiver(&to).filter(|_| !self.is_gone(&to)) {
            self.end_loop_listener_ids.insert(key.clone(), id);
            self.end_loop_listeners.insert(key, to.clone());
            self.note(format!("saved listener {name}: resolved {to} once"));
        } else {
            self.note(format!(
                "saved listener {name}: {to} absent; pending name consumed, no listener"
            ));
        }
    }

    /// After restored objects, before the remaining authored world Load code.
    pub fn merge_saved_events(&mut self, saved: SavedEvents) -> Result<(), String> {
        if !saved.valid() {
            return Err("invalid saved world events".into());
        }
        for note in self.queue.merge(self.now(), saved) {
            self.note(note);
        }
        Ok(())
    }

    /// Compatibility for callers not yet migrated to ordered loader phases.
    /// Cannot retroactively make saved fields precede already-run object Init.
    pub fn restore_events(&mut self, saved: SavedEvents) {
        if let Err(e) = self.prepare_saved_world(&saved) {
            self.note(e);
            return;
        }
        let pending: Vec<_> = self.pending_end_loop_listeners.keys().cloned().collect();
        for name in pending {
            self.finish_saved_object_init(&name);
        }
        if let Err(e) = self.merge_saved_events(saved) {
            self.note(e);
        }
    }
}

impl Active {
    /// The ordered loader has completed this trap's inline Init. Unlike the
    /// bulk helper, this posts only that trap's ordinary lookup request.
    pub fn finish_loaded_trap_init(&mut self, index: usize) {
        if index < self.world.traps.len() {
            self.world.queue_trap_lookup(index, "trap Init");
        }
    }

    /// After all inline startup/Load statements; before unpause/first-frame Begin.
    /// Only queued work runs: no animation, controller, potion or diagnostic tick.
    pub fn service_startup_queue(&mut self) -> Result<usize, String> {
        let horizon = self.world.simulation_time + STARTUP_SERVICE_SECONDS;
        if !horizon.is_finite() {
            return Err("startup queue horizon overflow".into());
        }
        let mut handled = 0;
        self.finish_item_events();
        self.character_placements();
        while let Some(e) = self.world.queue.take_due(horizon as f64) {
            self.world.simulation_time = self.world.simulation_time.max(e.due as f32);
            self.handle(e);
            self.finish_item_events();
            self.character_placements();
            handled += 1;
            if handled >= 100_000 {
                return Err(
                    "startup queue exceeded 100000 deliveries; startup not completed".into(),
                );
            }
        }
        self.world.simulation_time = horizon;
        Ok(handled)
    }
}

#[cfg(test)]
#[path = "saved_world_tests.rs"]
mod tests;
