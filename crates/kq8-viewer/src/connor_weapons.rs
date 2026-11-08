//! Original weapon-state entry, borrowing disjoint fields for synchronous admission.
//! Source: ranged-weapons.md rules 6–9. Internal list refreshes keep their origin.
use super::*;

impl ConnorSim {
    /// Resolve the queued out clip before Walker can advance to it. Selection
    /// clearing and the current numeric mode stay at the assignment boundary.
    pub(super) fn prepare_pending_weapon_list(&mut self, world: &kq8_core::active::WorldState) {
        if let Err(line) = resolve_pending_list(&mut self.walker, &self.movements, &mut self.pending_set,
            &mut self.weapon_target, &mut self.ready_item, self.first_person, world) {
            self.log.detail(line);
        }
    }

    /// The script preparation and lower-handler delivery precede ordinary
    /// state arbitration. A refused controller does not manufacture a swap.
    pub(crate) fn deliver_weapon_request(&mut self, active: &mut kq8_core::active::Active,
        request: &kq8_core::weapons::Request, use_modifier: bool) -> bool {
        use kq8_core::{active::Request, conversation::StateRequest, weapons::{Ask, SwapRequest}};
        if request.ask.script_mode() == Some(active.world.weapon_mode) { return false; }
        if request.ask.script_mode().is_some() && !matches!(request.ask, Ask::Required(_)) {
            kq8_core::connor_update::finish_jump(&mut self.mover, &mut self.walker);
            if request.ask.movement().is_none() { self.walker.pre_special(); }
            self.dispatch_state(active, Request::ConnorTransition(StateRequest::Wait));
        }
        let Some(change) = active.world.plan_weapon_request(request, self.may_ready(), self.hands_off) else { return false; };
        if matches!(change.to, WeaponMode::Raw(_)) {
            active.world.commit_weapon_change(change.to, use_modifier);
            self.target_mode = change.to;
            self.set_mode = change.to;
            self.walker.hand_mode = change.to.number();
            // No list selection, re-request or Swap entry in this branch.
            kq8_core::connor_inventory::weapon_mode_assigned(&mut active.world);
        } else {
            self.dispatch_state(active, Request::ConnorWeaponSwap(SwapRequest {
                mode: change.to, movement: request.ask.movement().map(str::to_owned),
                script: change.script, shift: self.shift_held, how: request.how.clone(),
            }));
        }
        true
    }
}

pub(super) fn resolve_pending_list(walker: &mut Walker, movements: &[DrawnMovement],
    pending: &mut Option<(Option<String>, WeaponMode, usize)>, target: &mut Option<String>,
    ready_item: &mut Option<String>, first_person: bool, world: &kq8_core::active::WorldState) -> Result<(), String> {
    let Some((cached, mode, at)) = pending.as_mut() else { return Ok(()) };
    if matches!(mode, WeaponMode::Raw(_)) { return Ok(()) }
    let bank = world.equipped_weapon_bank(first_person, *mode);
    if *mode != WeaponMode::None && bank.is_none() { return Err("pending weapon assignment: equipped item has no movement mapping".into()); }
    if let Some(file) = &bank {
        if !movements.iter().any(|movement| movement.qualified.split_once('/').is_some_and(|(loaded, _)| loaded.eq_ignore_ascii_case(file))) {
            return Err(format!("pending weapon assignment waits for bank {file}"));
        }
    }
    if let Some(kq8_core::connor::SpecialStep::Play(queued, _)) = walker.special.get_mut(*at) {
        let action = kq8_core::connor::plain_movement_name(queued);
        let next = find(movements, bank.as_deref(), action)
            .ok_or_else(|| format!("pending weapon assignment: {bank:?} has no {action}"))?.name;
        if walker.special_at == *at && walker.movement == *queued { walker.movement = next; }
        *queued = next;
    }
    *cached = bank.clone();
    *target = bank;
    *ready_item = world.weapon_item(*mode);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn enter(
    walker: &mut Walker, movements: &[DrawnMovement],
    weapon_target: &mut Option<String>, target_mode: &mut WeaponMode,
    ready_item: &mut Option<String>, weapon_selection: &mut WeaponSelection,
    pending_set: &mut Option<(Option<String>, WeaponMode, usize)>,
    weapon_set: &mut Option<String>, set_mode: &mut WeaponMode,
    held_walk: &mut Option<kq8_core::connor_auto::AutoMove>, readying: &mut bool,
    log: &mut Log, updates: &u64, list: Option<String>, mode: WeaponMode,
    item: Option<String>, at_once: bool, shift: bool, script: bool,
    ordinary: bool, movement: Option<&str>,
) -> String {

        use kq8_core::connor::SpecialStep;
        let old = (*weapon_set).clone();
        let old_mode = (*set_mode);
        let old_item = (*ready_item).clone();
        if !ordinary && old == list && old_mode == mode && old_item == item {
            return "no change".into();
        }
        (*weapon_target) = list.clone();
        (*target_mode) = mode;
        (*ready_item) = item.clone();
        (*weapon_selection).pending_ordinary = false;
        if at_once {
            (*pending_set) = None;
            assign(walker, weapon_set, set_mode, weapon_selection, list.clone(), mode, ordinary);
            return format!("picked up while another was ready: {list:?} at once, nothing plays (the user's check)");
        }
        let named = |file: &str, w: &str| {
            let wanted = kq8_core::connor::special_movement_name(file, w);
            (*movements).iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
        };
        let own = |w: &str| (*movements).iter().find(|m| m.name.eq_ignore_ascii_case(w)).map(|m| m.name);
        // A movement of a list (`None`: his own).
        let of = |l: &Option<String>, w: &str| match l {
            Some(f) => named(f, w),
            None => own(w),
        };
        let fists = |i: &Option<String>| i.as_deref().is_some_and(|k| k.eq_ignore_ascii_case(kq8_core::weapons::FISTS));
        let mut steps: Vec<(&'static str, String)> = Vec::new();
        let switch_at;
        let case;
        if let Some(movement) = movement {
            case = "optional movement in the assigned new list";
            assign(walker, weapon_set, set_mode, weapon_selection, list.clone(), mode, ordinary);
            if let Some(m) = of(&list, movement) { steps.push((m, movement.into())); }
            if movement.eq_ignore_ascii_case("away") || movement.eq_ignore_ascii_case("away2") {
                switch_at = steps.len();
                if let Some(m) = of(&list, "out") { steps.push((m, "out".into())); }
            } else if movement.eq_ignore_ascii_case("undraw") {
                switch_at = steps.len();
            } else { switch_at = 0; }
        } else if old_mode == WeaponMode::None && mode == WeaponMode::Hand && fists(&item) {
            case = "1: none to the Fists, the list first, then draw";
            switch_at = 0;
            if let Some(m) = of(&list, "draw") {
                steps.push((m, "draw".into()));
            }
        } else if old_mode == WeaponMode::Hand && fists(&old_item) && mode == WeaponMode::None {
            case = "2: the Fists to none, undraw, then the list";
            if let Some(m) = of(&old, "undraw") {
                steps.push((m, "undraw".into()));
            }
            switch_at = steps.len();
        } else {
            let big = old_mode == WeaponMode::Hand && shift;
            case = if big { "3: from a hand weapon with Shift: away2, the list, out" } else { "4: away, the list, out" };
            let away = if big { of(&old, "away2").map(|m| (m, "away2")).or_else(|| of(&old, "away").map(|m| (m, "away"))) } else { of(&old, "away").map(|m| (m, "away")) };
            if let Some((m, w)) = away {
                steps.push((m, format!("{w} ({})", old.as_deref().unwrap_or("his own"))));
            }
            switch_at = steps.len();
            if let Some(m) = of(&list, "out") {
                steps.push((m, format!("out ({})", list.as_deref().unwrap_or("his own"))));
            }
        }
        let set = match &list {
            Some(l) => format!("his movements come from {l} where it has them"),
            None => "his own movements again (Conner.anm)".into(),
        };
        let played: Vec<&str> = steps.iter().map(|(m, _)| *m).collect();
        let plays = ordinary || walker.mode == Mode::Free || (script && matches!(walker.mode, Mode::Special | Mode::Push | Mode::Talk));
        // Q141 (the user's check in the original, 2026-10-21): a script's
        // request while a scripted walk runs (`ax.cs`: `setDoMode 0`, then
        // `walkTo` in the same event): he stops, the swap plays, then the
        // walk starts (held until the swap is over, `weapon_step`).
        let scripted_walk = walker.auto.as_ref().is_some_and(|a| matches!(a.kind, kq8_core::connor_auto::AutoKind::Walk { .. }));
        let held = if script && scripted_walk && !steps.is_empty() {
            (*held_walk) = walker.auto.take();
            "; the scripted walk waits until the swap is over (Q141, the user's check)"
        } else {
            ""
        };
        if !plays || steps.is_empty() {
            (*pending_set) = None;
            if movement.is_none() { assign(walker, weapon_set, set_mode, weapon_selection, list, mode, ordinary); }
            return format!("rule 6 case {case}; {set} at once; mode {}: {played:?} not played", walker.mode.name());
        }
        if switch_at == 0 {
            (*pending_set) = None;
            if movement.is_none() { assign(walker, weapon_set, set_mode, weapon_selection, list, mode, ordinary); }
        } else {
            (*pending_set) = Some((list, mode, switch_at));
            (*weapon_selection).pending_ordinary = ordinary;
        }
        walker.auto = None;
        (*readying) = true;
        let notes = walker.start_swap_special(steps.iter().map(|(m, w)| SpecialStep::Play(m, w.clone())).collect());
        let line = format!("rule 6 case {case}: plays {played:?}; {set} from item {switch_at} ({}){held}", notes.join("; "));
        (*log).detail(format!("connor update {}: weapon: {line}", (*updates)));
        line
    }

fn assign(walker: &mut Walker, bank: &mut Option<String>, set_mode: &mut WeaponMode,
    selection: &mut WeaponSelection, list: Option<String>, mode: WeaponMode, ordinary: bool) {
    walker.hand_mode = mode.number();
    *bank = list;
    *set_mode = mode;
    selection.pending_ordinary = false;
    selection.assigned_ordinary |= ordinary;
}
