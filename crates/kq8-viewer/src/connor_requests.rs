//! Synchronous ordinary state requests (conversation.md admission addendum).
//! Borrow only disjoint state fields so a completion callback can change the
//! owner before Mover resumes its physics passes.
use super::*;
use kq8_core::active::{Active, Request, WorldState};
use kq8_core::conversation::{self, StateRequest};

/// Recreation-format continuation only. This is deliberately narrower than
/// serializing the command enum, and is not an original save-queue claim.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedPendingState {
    request: Option<SavedRequest>,
    jump_finished: bool,
    fall_finished: bool,
    talk_exit_open: bool,
    preparation_stop: bool,
    interrupt_override: Option<bool>,
    airborne: Option<SavedAirborne>,
    #[serde(default)]
    forced_choice: Option<kq8_core::choice::ChoiceList>,
    #[serde(default)]
    forced_choice_recorded: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum SavedRequest {
    Talk, Wait { at: Option<String> }, Move, Fall,
    Special { items: String, at: String }, DieNow { at: String },
    Die { kind: kq8_core::fight::HitKind, at: String },
    Walk { auto: kq8_core::connor_auto::AutoMove, at: String },
    Swap(kq8_core::weapons::SwapRequest),
    Choice { list: kq8_core::choice::ChoiceList, at: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum AirborneMode { Jump, Fall, Land }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SavedAirborne {
    mode: AirborneMode,
    kind: u8,
    pose: SavedControllerPose,
    step_up: f32,
    step_down: f32,
    jump_glanced: bool,
}

impl SavedRequest {
    fn take(request: &Request) -> Result<Self, String> {
        Ok(match request {
            Request::ConnorTalk => Self::Talk,
            Request::ConnorWeaponSwap(request) => Self::Swap(request.clone()),
            Request::ConnorChoice { list, at } => Self::Choice { list: list.clone(), at: at.clone() },
            Request::ConnorWait { at } => Self::Wait { at: Some(at.clone()) },
            Request::ConnorTransition(StateRequest::Wait) => Self::Wait { at: None },
            Request::ConnorTransition(StateRequest::Move) => Self::Move,
            Request::ConnorTransition(StateRequest::Fall) => Self::Fall,
            Request::ConnorSpecial { items, at } => Self::Special { items: items.clone(), at: at.clone() },
            Request::ConnorDieNow { at } => Self::DieNow { at: at.clone() },
            Request::ConnorDies { kind, at } => Self::Die { kind: *kind, at: at.clone() },
            Request::ConnorAuto { auto, at } if matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. }) => Self::Walk { auto: auto.clone(), at: at.clone() },
            _ => return Err("unsupported pending Connor state payload".into()),
        })
    }
    fn request(&self) -> Request {
        match self {
            Self::Talk => Request::ConnorTalk,
            Self::Swap(request) => Request::ConnorWeaponSwap(request.clone()),
            Self::Choice { list, at } => Request::ConnorChoice { list: list.clone(), at: at.clone() },
            Self::Wait { at: Some(at) } => Request::ConnorWait { at: at.clone() },
            Self::Wait { at: None } => Request::ConnorTransition(StateRequest::Wait),
            Self::Move => Request::ConnorTransition(StateRequest::Move),
            Self::Fall => Request::ConnorTransition(StateRequest::Fall),
            Self::Special { items, at } => Request::ConnorSpecial { items: items.clone(), at: at.clone() },
            Self::DieNow { at } => Request::ConnorDieNow { at: at.clone() },
            Self::Die { kind, at } => Request::ConnorDies { kind: *kind, at: at.clone() },
            Self::Walk { auto, at } => Request::ConnorAuto { auto: auto.clone(), at: at.clone() },
        }
    }
    fn valid(&self) -> bool {
        let text = |s: &str| s.len() <= 16_384 && !s.contains('\0');
        match self {
            Self::Talk | Self::Move | Self::Fall | Self::Wait { at: None } => true,
            Self::Swap(r) => !matches!(r.mode, WeaponMode::Raw(_)) && text(&r.how) && r.movement.as_ref().is_none_or(|m| text(m)),
            Self::Choice { list, at } => list.valid() && text(at),
            Self::Wait { at: Some(at) } | Self::DieNow { at } | Self::Die { at, .. } => text(at),
            Self::Special { items, at } => text(items) && text(at),
            Self::Walk { auto, at } => text(at) && matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. })
                && auto.target.iter().all(|n| n.is_finite())
                && auto.cue.as_ref().is_none_or(|c| [&c.who, &c.c, &c.d].into_iter().all(|s| text(s)))
                && match auto.phase { kq8_core::connor_auto::AutoPhase::Turning => true,
                    kq8_core::connor_auto::AutoPhase::Walking { from, distance } => from.iter().all(|v| v.is_finite()) && distance.is_finite() },
        }
    }
}

impl SavedPendingState {
    pub fn valid(&self) -> bool {
        self.request.as_ref().is_none_or(SavedRequest::valid)
            && self.forced_choice.as_ref().is_none_or(kq8_core::choice::ChoiceList::valid)
            && self.airborne.as_ref().is_none_or(|a| self.request.is_some() && a.kind <= 2 && a.pose.valid()
                && [a.step_up, a.step_down].into_iter().all(|v| v.is_finite() && v >= 0.))
    }

    fn restore_into(&self, pending: &mut Option<Request>, walker: &mut Walker, mover: &mut Mover, hands_off: &mut bool, resolve: &dyn Fn(&str) -> Result<&'static str, String>) -> Result<(), String> {
        if !self.valid() { return Err("invalid pending Connor continuation".into()); }
        if let Some(a) = &self.airborne {
            let mode = match a.mode { AirborneMode::Jump => Mode::Jump, AirborneMode::Fall => Mode::Fall, AirborneMode::Land => Mode::Land };
            a.pose.restore_into(walker, mover, hands_off, mode, resolve)?;
        }
        *pending = self.request.as_ref().map(SavedRequest::request);
        walker.jump_finished = self.jump_finished;
        walker.fall_finished = self.fall_finished;
        walker.talk_exit_open = self.talk_exit_open;
        walker.pre_special_stop = self.preparation_stop;
        walker.interrupt_override = self.interrupt_override;
        if let Some(a) = &self.airborne {
            walker.kind = match a.kind { 0 => kq8_core::connor::JumpKind::Stand, 1 => kq8_core::connor::JumpKind::Run, _ => kq8_core::connor::JumpKind::Back };
            mover.step_up = a.step_up; mover.step_down = a.step_down; mover.jump_glanced = a.jump_glanced;
        }
        Ok(())
    }
}

pub(super) fn kind(r: &Request) -> Option<StateRequest> {
    Some(match r {
        Request::ConnorConversationStart | Request::ConnorTalk => StateRequest::Talk,
        Request::ConnorWeaponSwap(_) => StateRequest::Swap,
        Request::ConnorChoice { .. } => StateRequest::Choice,
        Request::ConnorTransition(next) => *next,
        Request::ConnorWait { .. } => StateRequest::Wait,
        Request::ConnorSpecial { .. } => StateRequest::Special,
        Request::ConnorDieNow { .. } | Request::ConnorDies { .. } => StateRequest::Die,
        Request::ConnorAuto { auto, .. } if matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. }) => StateRequest::Wait,
        _ => return None,
    })
}

/// Choice's explicit discard boundary precedes both its made flag and the
/// DoChoice callback; entry to Choice is not an unconditional discard.
pub(super) fn commit_choice(pending: &mut Option<Request>, forced: &mut Option<kq8_core::choice::ChoiceList>, made: &mut bool, calls: &mut Vec<Vec<String>>, number: String) {
    *pending = None;
    *forced = None;
    *made = true;
    calls.push(vec!["Connor".into(), "DoChoice".into(), number]);
}

pub(super) struct Dispatch<'a> {
    pending: &'a mut Option<Request>,
    hands_off: &'a mut bool,
    swap: &'a mut bool,
    held: bool,
    movements: &'a [DrawnMovement],
    weapon_set: &'a mut Option<String>,
    weapon_target: &'a mut Option<String>,
    first_person: &'a mut bool,
    pending_set: &'a mut Option<(Option<String>, WeaponMode, usize)>,
    target_mode: &'a mut WeaponMode,
    set_mode: &'a mut WeaponMode,
    ready_item: &'a mut Option<String>,
    selection: &'a mut WeaponSelection,
    held_walk: &'a mut Option<kq8_core::connor_auto::AutoMove>,
    choice: &'a mut Option<kq8_core::choice::Choice>,
    forced_choice: &'a mut Option<kq8_core::choice::ChoiceList>,
    swing: &'a mut Option<Swing>,
    shot: &'a mut Option<Shot>,
    turn_keys: &'a mut kq8_core::turn_keys::TurnKeys,
    intent: &'a mut Intent,
    held_keys: &'a mut Vec<KeyCode>,
    log: &'a mut Log,
    updates: u64,
}

impl ConnorSim {
    pub(crate) fn saved_pending_state(&self) -> Result<SavedPendingState, String> {
        let request = self.pending_state.as_ref().map(SavedRequest::take).transpose()?;
        // Special, weapon changes, Talk and attacks already have owner poses.
        // Only the previously omitted airborne owner needs an extra pose.
        let airborne = if request.is_some() {
            match self.walker.mode {
                Mode::Jump | Mode::Fall | Mode::Land => Some(SavedAirborne {
                    mode: match self.walker.mode { Mode::Jump => AirborneMode::Jump, Mode::Fall => AirborneMode::Fall, _ => AirborneMode::Land },
                    kind: match self.walker.kind { kq8_core::connor::JumpKind::Stand => 0, kq8_core::connor::JumpKind::Run => 1, kq8_core::connor::JumpKind::Back => 2 },
                    pose: self.capture_controller_pose(), step_up: self.mover.step_up, step_down: self.mover.step_down, jump_glanced: self.mover.jump_glanced,
                }),
                _ => None,
            }
        } else { None };
        Ok(SavedPendingState { request, airborne, forced_choice: self.forced_choice.clone(), forced_choice_recorded: true, jump_finished: self.walker.jump_finished,
            fall_finished: self.walker.fall_finished, talk_exit_open: self.walker.talk_exit_open,
            preparation_stop: self.walker.pre_special_stop, interrupt_override: self.walker.interrupt_override })
    }

    pub(crate) fn restore_pending_state(&mut self, saved: &SavedPendingState) -> Result<(), String> {
        let movements = &self.movements; let bank = self.weapon_set.as_deref();
        let resolve = |name: &str| find(movements, bank, name)
            .map(|m| if name.contains('/') { m.qualified } else { kq8_core::connor::plain_movement_name(m.name) })
            .ok_or_else(|| format!("saved pending movement '{name}' is absent"));
        saved.restore_into(&mut self.pending_state, &mut self.walker, &mut self.mover, &mut self.hands_off, &resolve)?;
        self.forced_choice = if saved.forced_choice_recorded { saved.forced_choice.clone() } else { self.choice.as_ref().filter(|c| !c.made).map(|c| c.list.clone()) };
        self.redraw_from_walker();
        Ok(())
    }

    pub(crate) fn retry_pending_state(&mut self, active: &mut Active) {
        let (mover, walker, mut dispatcher) = self.request_parts();
        dispatcher.request(mover, walker, active, Request::ConnorTransition(StateRequest::RetryPending));
        self.redraw_from_walker();
    }
    pub(crate) fn ordinary_state_request(&self, request: &Request) -> bool { kind(request).is_some() }
    pub(super) fn request_parts(&mut self) -> (&mut Mover, &mut Walker, Dispatch<'_>) {
        let held = self.tile_holds() || self.scales.is_some() || self.secondary_holds()
            || self.roping() || self.rock.is_some() || self.throw.is_some()
            ;
        let Self { controls, mover, walker, pending_state, hands_off, readying, movements,
            weapon_set, weapon_target, first_person, pending_set, target_mode,
            set_mode, ready_item, weapon_selection, held_walk, choice, forced_choice, swing, shot, turn_keys, intent, log, updates, .. } = self;
        (mover, walker, Dispatch { pending: pending_state, hands_off, swap: readying,
            held, movements, weapon_set, weapon_target, first_person, pending_set,
            target_mode, set_mode, ready_item, selection: weapon_selection, held_walk, choice, forced_choice, swing, shot, turn_keys,
            intent, held_keys: &mut controls.held, log, updates: *updates })
    }

    /// `None` means this is not an ordinary state request, so the existing
    /// command dispatcher still owns its payload and side effects.
    pub(crate) fn dispatch_state(&mut self, active: &mut Active, incoming: Request) -> Option<String> {
        kind(&incoming)?;
        // A stale start-message is rejected before the common state request;
        // it must not tear down an owner merely because Choice is remembered.
        let has_request = !matches!(incoming, Request::ConnorConversationStart)
            || active.world.conversation.as_ref().is_some_and(|c| !c.entered());
        if has_request && self.choice_replaces_owner(&incoming) { self.discard_owner_for_choice(active); }
        let (mover, walker, mut dispatcher) = self.request_parts();
        let line = dispatcher.request(mover, walker, active, incoming);
        self.redraw_from_walker();
        Some(line)
    }

    pub(crate) fn finish_talk_request(&mut self, active: &mut Active) {
        let Some(why) = active.world.connor_leave.take() else { return; };
        let Some(next) = conversation::exit_request(&mut self.walker) else { return; };
        let line = self.dispatch_state(active, Request::ConnorTransition(next)).unwrap();
        self.log.detail(format!("Talk end ({why}): {line}"));
    }

    pub(super) fn tick_with_state_requests(&mut self, active: Option<&mut Active>, intent: Intent, jump: bool, world: &dyn MoverWorld) -> kq8_core::connor_update::Tick {
        if let Some(active) = active.as_deref() { self.prepare_pending_weapon_list(&active.world); }
        self.walker.restore_swap_owner(self.readying);
        let (mover, walker, dispatcher) = self.request_parts();
        let movements = dispatcher.movements;
        // The loops returned borrow immutable loaded movements, not this cell.
        // Each short bank lookup finishes before the synchronous callback runs.
        let dispatcher = std::cell::RefCell::new(dispatcher);
        let loops = |name: &str| {
            let bank = dispatcher.borrow().weapon_set.clone();
            find(movements, bank.as_deref(), name).map(|m| &m.movement_loop)
        };
        match active {
            Some(active) => kq8_core::connor_update::tick_with_requests(mover, walker, intent, jump, &loops, world, &mut |mover, walker, next| {
                dispatcher.borrow_mut().request(mover, walker, active, Request::ConnorTransition(next));
            }),
            None => kq8_core::connor_update::tick(mover, walker, intent, jump, &loops, world),
        }
    }
}

impl Dispatch<'_> {
    fn admits(&self, walker: &Walker, request: &Request) -> bool {
        if self.held { return false; }
        let Some(next) = kind(request) else { return false; };
        if self.choice.as_ref().is_some_and(|c| !c.made) && walker.mode == Mode::Free && !*self.swap {
            return next == StateRequest::Swap;
        }
        conversation::admits(walker, *self.swap, next)
    }

    fn drain_assignment(&mut self, active: &mut Active) {
        if std::mem::take(&mut self.selection.assigned_ordinary) {
            active.world.commit_weapon_change(*self.set_mode, false);
            kq8_core::connor_inventory::weapon_mode_assigned(&mut active.world);
        }
    }

    fn force_choice(&mut self, mover: &mut Mover, walker: &mut Walker, active: &mut Active, incoming: Request) -> String {
        let list = self.forced_choice.as_ref().expect("remembered Choice").clone();
        kq8_core::connor_update::finish_jump(mover, walker);
        walker.auto = None;
        walker.stand_now();
        *self.intent = Intent::default();
        self.held_keys.clear();
        self.turn_keys.turn = 0;
        *self.swap = false;
        walker.restore_swap_owner(false);
        *self.swing = None;
        *self.shot = None;
        *self.pending_set = None;
        self.selection.pending_ordinary = false;
        *self.held_walk = None;
        *self.weapon_target = self.weapon_set.clone();
        *self.target_mode = *self.set_mode;
        *self.ready_item = active.world.weapon_ready.clone();
        *self.choice = Some(kq8_core::choice::Choice::new(list.clone()));
        *self.pending = None;
        active.world.requests.push(Request::ResetCameraInput { at: "Choice entry".into() });
        active.world.connor_state = Some(kq8_core::character::state_number::CHOICE);
        // The script runs synchronously. Its host operations are drained by
        // trap_step before the next Choice point measurement, as on initial entry.
        active.run_script(&list.script, &["Connor".into(), "StartChoice".into()]);
        active.world.state_changes();
        *self.pending = Some(incoming);
        "forced Choice entered; incoming request retained untested; StartChoice called".into()
    }

    pub(super) fn request(&mut self, mover: &mut Mover, walker: &mut Walker, active: &mut Active, mut incoming: Request) -> String {
        if matches!(incoming, Request::ConnorTransition(StateRequest::RetryPending)) {
            let Some(old) = self.pending.take() else { return "completion: no pending state".into(); };
            if !self.admits(walker, &old) {
                *self.pending = Some(old);
                return "completion: pending state still refused".into();
            }
            return self.apply(mover, walker, active, old, false);
        }
        if *self.swap && walker.mode == Mode::Special && walker.special.is_empty()
            && matches!(incoming, Request::ConnorTransition(StateRequest::Wait)) {
            self.complete_assignment(walker, active);
            if self.correct_required_mode(walker, active) { return "Swap required-mode correction; same owner, pending request untouched".into(); }
        }
        let start_message = matches!(incoming, Request::ConnorConversationStart);
        if start_message {
            if !active.conversation_prepare(walker) { return "conversation start: no unentered conversation".into(); }
            incoming = Request::ConnorTalk;
        }
        if let Request::ConnorChoice { list, .. } = &incoming {
            *self.forced_choice = Some(list.clone());
        }
        if self.forced_choice.is_some() && (self.choice.is_none() || walker.mode != Mode::Free || *self.swap || self.held) {
            return self.force_choice(mover, walker, active, incoming);
        }
        let strong = |r: &Request| kind(r).is_some_and(StateRequest::strong);
        let selected = if let Some(old) = self.pending.take() {
            if !(strong(&incoming) && !strong(&old)) && self.admits(walker, &old) {
                *self.pending = Some(incoming.clone());
                Some((old, false))
            } else { None }
        } else { None };
        let selected = selected.or_else(|| {
            // The original exception runs only after the older candidate failed.
            let _ = kq8_core::connor_update::state_request(mover, walker, *self.hands_off);
            if self.admits(walker, &incoming) { Some((incoming.clone(), true)) }
            else { *self.pending = Some(incoming.clone()); None }
        });
        let Some((r, incoming_won)) = selected else { return "state request retained in the one pending slot".into(); };
        let immediate = start_message && incoming_won && matches!(r, Request::ConnorTalk);
        let line = self.apply(mover, walker, active, r, immediate);
        if start_message && !*self.hands_off {
            active.world.gui_orders.push(kq8_core::active::GuiOrder::Conversation { start: true, at: "conversation start-message state request accepted".into() });
        }
        self.log.detail(format!("connor update {}: {line}", self.updates));
        line
    }

    fn complete_assignment(&mut self, walker: &mut Walker, active: &mut Active) {
        if let Err(line) = weapon_entry::resolve_pending_list(walker, self.movements, self.pending_set,
            self.weapon_target, self.ready_item, *self.first_person, &active.world) { self.log.detail(line); }
        if let Some((bank, mode, _)) = self.pending_set.take() {
            *self.weapon_set = bank;
            *self.set_mode = mode;
            walker.hand_mode = mode.number();
            self.selection.assigned_ordinary |= std::mem::take(&mut self.selection.pending_ordinary);
        }
        self.drain_assignment(active);
    }

    fn correct_required_mode(&mut self, walker: &mut Walker, active: &mut Active) -> bool {
        let ended = kq8_core::connor::plain_movement_name(walker.movement);
        if ended.eq_ignore_ascii_case("draw") || ended.eq_ignore_ascii_case("undraw") { return false; }
        let (required, script) = active.world.required_weapon_mode.clone();
        if required == -1 || required == self.set_mode.number() || *self.set_mode == WeaponMode::None { return false; }
        if let Some(script) = script { active.run_script(&script, &["Connor".into(), "WrongWeapon".into()]); }
        let mode = WeaponMode::from_raw(active.world.required_weapon_mode.0);
        if mode == WeaponMode::Ranged && active.world.ranged_slot().is_none() { return false; }
        if matches!(mode, WeaponMode::Raw(_)) {
            active.world.commit_weapon_change(mode, false);
            *self.target_mode = mode; *self.set_mode = mode; walker.hand_mode = mode.number();
            kq8_core::connor_inventory::weapon_mode_assigned(&mut active.world);
            return false;
        }
        // Correction remains inside Swap, always away then out, with no new
        // common request and no early pending retry or fist-entry shortcuts.
        *self.target_mode = mode;
        *self.ready_item = active.world.weapon_item(mode);
        let bank = active.world.connor_fight.as_ref().and_then(|f| f.bank(*self.first_person, mode, self.ready_item.as_deref())).map(str::to_owned);
        *self.weapon_target = bank.clone();
        let away = find(self.movements, self.weapon_set.as_deref(), "away").map(|m| m.name);
        let out = find(self.movements, bank.as_deref(), "out").map(|m| m.name);
        let (Some(away), Some(out)) = (away, out) else {
            self.log.detail("required mode correction: missing movement; original runtime result unverified");
            return false;
        };
        *self.pending_set = Some((bank, mode, 1));
        self.selection.pending_ordinary = true;
        walker.start_swap_special(vec![kq8_core::connor::SpecialStep::Play(away, "away".into()), kq8_core::connor::SpecialStep::Play(out, "out".into())]);
        true
    }
    fn apply(&mut self, mover: &mut Mover, walker: &mut Walker, active: &mut Active, r: Request, immediate: bool) -> String {
        let leaving_swap = std::mem::take(self.swap);
        if leaving_swap { walker.restore_swap_owner(false); }
        if leaving_swap && !matches!(r, Request::ConnorTransition(StateRequest::Wait) | Request::ConnorWait { .. }) {
            *self.held_walk = None;
        }
        match r {
            Request::ConnorWeaponSwap(request) => {
                let item = active.world.weapon_item(request.mode);
                let list = active.world.connor_fight.as_ref().and_then(|f| f.bank(*self.first_person, request.mode, item.as_deref())).map(str::to_owned);
                let line = weapon_entry::enter(walker, self.movements, self.weapon_target, self.target_mode,
                    self.ready_item, self.selection, self.pending_set, self.weapon_set, self.set_mode,
                    self.held_walk, self.swap, self.log, &self.updates, list, request.mode,
                    item, false, request.shift && !*self.first_person,
                    request.script, true, request.movement.as_deref());
                self.drain_assignment(active);
                line
            }
            Request::ConnorChoice { list, at } => {
                let retained = self.pending.take();
                *self.forced_choice = Some(list.clone());
                let line = self.force_choice(mover, walker, active, Request::ConnorChoice { list, at });
                *self.pending = retained;
                line
            }
            Request::ConnorTalk => {
                let bank = self.weapon_set.as_deref();
                let movements = self.movements;
                let loops = |n: &str| find(movements, bank, n).map(|m| &m.movement_loop);
                active.conversation_admitted(walker, mover.position, &loops);
                format!("Talk admitted; immediate start-message admission={immediate}")
            }
            Request::ConnorTransition(StateRequest::Fall) => { walker.set_mode(Mode::Fall); "Fall admitted".into() }
            Request::ConnorTransition(StateRequest::Move) => { walker.accepted_move(); "Move admitted".into() }
            Request::ConnorTransition(StateRequest::Wait) | Request::ConnorWait { .. } => {
                let movements = self.movements;
                let loops = |n: &str| find(movements, self.weapon_set.as_deref(), n).map(|m| &m.movement_loop);
                walker.accepted_wait(&loops);
                if leaving_swap {
                    if let Some(auto) = self.held_walk.take() { walker.start_auto(auto); }
                }
                "Wait admitted; ordinary Stop request".into()
            }
            Request::ConnorSpecial { items, at } => {
                let resolve = |list: Option<&str>, word: &str| {
                    let wanted = list.map_or_else(|| word.to_string(), |l| kq8_core::connor::special_movement_name(l, word));
                    self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
                };
                let steps = kq8_core::connor::special_steps(&items, &resolve);
                format!("{at}: special {items}: {}", walker.start_special(steps).join("; "))
            }
            Request::ConnorAuto { auto, at } => {
                *self.hands_off = true;
                format!("{at}: {}; handsOff on", walker.start_auto(auto))
            }
            Request::ConnorDieNow { at } => {
                let movements = self.movements;
                let loops = |n: &str| find(movements, self.weapon_set.as_deref(), n).map(|m| &m.movement_loop);
                active.enter_scripted_death(walker, mover, &loops);
                if matches!(active.world.scripted_death, kq8_core::connor::ScriptedDeath::HoldUntil(_)) && !*self.hands_off {
                    *self.hands_off = true;
                    active.world.requests.push(Request::HandsOff { on: true, at: at.clone() });
                }
                format!("{at}: Die Now admitted")
            }
            Request::ConnorDies { kind, at } => {
                let kind = active.world.connor_last_hit.unwrap_or(kind);
                active.world.first_person = false;
                self.perspective(walker, &active.world);
                active.world.enter_connor_death();
                *self.swing = None;
                *self.shot = None;
                use kq8_core::fight::HitKind;
                let movement = |n: &str| find(self.movements, self.weapon_set.as_deref(), n).map(|m| m.name);
                let final_name = match kind {
                    HitKind::Normal => "die", HitKind::Projectile if movement("projdie").is_some() => "projdie",
                    HitKind::Projectile => "die", HitKind::Fire => "firedie", HitKind::Ice => "icedie",
                };
                let first = if matches!(kind, HitKind::Fire | HitKind::Ice) { "dropweapon" } else { final_name };
                walker.death = movement(final_name);
                walker.mode = Mode::CombatDie;
                if let Some(name) = movement(first) {
                    let loops = |n: &str| find(self.movements, self.weapon_set.as_deref(), n).map(|m| &m.movement_loop);
                    walker.request_die_movement(name, &loops);
                }
                self.turn_keys.turn = 0;
                self.intent.turn = 0;
                let line = format!("{at}: dies ({}): first {first}, final {final_name}; Die owns the animation", kind.name());
                active.world.fight_note("Connor", &line);
                line
            }
            _ => unreachable!("only classified ordinary state payloads enter this dispatcher"),
        }
    }

    fn perspective(&mut self, walker: &mut Walker, world: &WorldState) {
        if *self.first_person == world.first_person { return; }
        if world.connor_fight.is_none() { return; }
        let next = world.equipped_weapon_bank(!*self.first_person, *self.set_mode);
        if next.is_none() && (world.first_person || *self.set_mode != WeaponMode::None) { return; }
        let action = kq8_core::connor::plain_movement_name(walker.movement);
        if let Some(bank) = &next {
            let wanted = kq8_core::connor::special_movement_name(bank, action);
            if !self.movements.iter().any(|m| m.qualified.eq_ignore_ascii_case(&wanted)) { return; }
        }
        let Some(old) = find(self.movements, self.weapon_set.as_deref(), walker.movement).map(|m| m.qualified) else { return; };
        let target = world.equipped_weapon_bank(!*self.first_person, *self.target_mode);
        if let Some((file, mode, at)) = self.pending_set {
            let mapped = world.equipped_weapon_bank(!*self.first_person, *mode);
            if let Some(kq8_core::connor::SpecialStep::Play(movement, _)) = walker.special.get_mut(*at) {
                let plain = kq8_core::connor::plain_movement_name(movement);
                let Some(m) = find(self.movements, mapped.as_deref(), plain) else { return; };
                *movement = m.name;
            }
            *file = mapped;
        }
        *self.first_person = world.first_person;
        *self.weapon_set = next;
        *self.weapon_target = target;
        *self.ready_item = world.weapon_item(*self.target_mode);
        let loops = |n: &str| find(self.movements, self.weapon_set.as_deref(), n).map(|m| &m.movement_loop);
        walker.animation_input_lock = false;
        walker.rerequest_view(old, action, &loops);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_dispatch<R>(pending: &mut Option<Request>, hands_off: &mut bool, swap: bool, work: impl FnOnce(&mut Dispatch<'_>) -> R) -> R {
        let movements = vec![DrawnMovement { name: "stop", qualified: "stop", first_person: false,
            movement_loop: MovementLoop { duration: 1., frames: 2, repeat: true, cut_short: true, speed: 1., ..Default::default() }, frames: 2, anim: 0, parts: vec![] }];
        let mut weapon_set = None; let mut weapon_target = None; let mut first_person = false;
        let mut pending_set = None; let mut swing = None; let mut shot = None;
        let mut turn_keys = kq8_core::turn_keys::TurnKeys::default(); let mut intent = Intent::default();
        let mut log = Log::create("conversation-admission-matrix").unwrap();
        let mut swap = swap; let mut target_mode = WeaponMode::None; let mut set_mode = WeaponMode::None;
        let mut ready_item = None; let mut selection = WeaponSelection::default(); let mut held_walk = None;
        let mut choice = None; let mut forced_choice = None; let mut held_keys = Vec::new();
        work(&mut Dispatch { pending, hands_off, swap: &mut swap, held: false, movements: &movements,
            weapon_set: &mut weapon_set, weapon_target: &mut weapon_target, first_person: &mut first_person,
            pending_set: &mut pending_set, target_mode: &mut target_mode, set_mode: &mut set_mode,
            ready_item: &mut ready_item, selection: &mut selection, held_walk: &mut held_walk,
            choice: &mut choice, forced_choice: &mut forced_choice, held_keys: &mut held_keys,
            swing: &mut swing, shot: &mut shot, turn_keys: &mut turn_keys, intent: &mut intent, log: &mut log, updates: 0 })
    }

    fn conversation() -> Active {
        let mut active = Active::new(kq8_data::world_script::Vars::default(), vec![], vec![], ["Connor".to_string(), "Bob".to_string()], Vec::<(String, Vec<u8>)>::new());
        active.world.monsters.push(kq8_core::active::Monster { name: "Bob".into(), position: [10., 20., 30.], script: None, goodbye: None, welcome_back: None });
        assert_eq!(active.test_command("KQConner::conversation Bob 1000 60 0 0 1 1 0", "matrix").as_deref(), Some("0"));
        active.world.requests.clear(); // The fixture explicitly delivers this start below.
        active
    }

    #[test]
    fn conversation_admission_grouped_production_dispatch_and_payloads() {
        let mut pending = None; let mut hands = false;
        let mut mover = Mover::new([0.; 3], true); let mut walker = Walker::new(0.);
        let mut active = conversation();
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorConversationStart); });
        assert_eq!(walker.mode, Mode::Talk);
        assert!(active.world.gui_orders.iter().any(|r| matches!(r, kq8_core::active::GuiOrder::Conversation { start: true, .. })));

        // Refusal performs preparation once, and the later completion can
        // enter Talk without replaying the immediate-only panel operation.
        active = conversation(); walker = Walker::new(0.); walker.start_jump(kq8_core::connor::JumpKind::Stand);
        walker.phase = 0.25;
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorConversationStart); });
        assert_eq!(pending, Some(Request::ConnorTalk));
        assert_eq!((walker.mode, walker.movement, walker.phase), (Mode::Jump, "jump", 0.25));
        active.world.monsters[0].position = [999.; 3];
        walker.jump_finished = true;
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Fall)); });
        assert_eq!(walker.mode, Mode::Talk);
        assert_eq!(active.world.conversation.as_ref().unwrap().partner_at, [10., 20., 30.]);
        assert!(!active.world.gui_orders.iter().any(|r| matches!(r, kq8_core::active::GuiOrder::Conversation { start: true, .. })));
        assert_eq!(pending, Some(Request::ConnorTransition(StateRequest::Fall)));
        walker.talk_exit_open = true;
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorWait { at: "normal Talk exit".into() }); });
        assert!(pending.is_none()); assert_eq!(walker.mode, Mode::Free);

        // A refused incoming payload replaces Talk; a later accepted old
        // payload executes now and retains the new proposal without testing it.
        active = conversation(); walker.start_jump(kq8_core::connor::JumpKind::Stand);
        pending = Some(Request::ConnorTalk);
        let special = Request::ConnorSpecial { items: "stop".into(), at: "original script payload".into() };
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, special.clone()); });
        assert_eq!(pending, Some(special));
        walker.jump_finished = true;
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Fall)); });
        assert_eq!(walker.mode, Mode::Special);
        assert_eq!(walker.movement, "stop");
        assert_eq!(pending, Some(Request::ConnorTransition(StateRequest::Fall)));
        assert!(!active.world.conversation.as_ref().unwrap().entered(), "discarded Talk never returns");

        // Same real dispatcher: a swap completion refuses old Talk, applies
        // incoming Wait synchronously and removes the obsolete request.
        pending = Some(Request::ConnorTalk); walker.phase = 1.; walker.special.clear(); walker.special_at = 1;
        with_dispatch(&mut pending, &mut hands, true, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Wait)); });
        assert!(pending.is_none()); assert_eq!(walker.mode, Mode::Free);

        // Ordinary Talk exit admits retained Wait before an incoming Move.
        walker.enter_talk(); walker.talk_exit_open = true;
        pending = Some(Request::ConnorWait { at: "retained Wait".into() });
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Move)); });
        assert_eq!(pending, Some(Request::ConnorTransition(StateRequest::Move)));
        assert_eq!(walker.mode, Mode::Free);

        // Strong Talk discards a non-strong prior request without admitting it.
        active = conversation(); pending = Some(Request::ConnorTransition(StateRequest::Fall));
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorConversationStart); });
        assert_eq!(walker.mode, Mode::Talk); assert!(pending.is_none());

        // An older strong request may win the successful start-message call.
        // Its panel action still runs, while Talk is only retained.
        active = conversation(); walker = Walker::new(0.);
        pending = Some(Request::ConnorSpecial { items: "stop".into(), at: "earlier strong state".into() });
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorConversationStart); });
        assert_eq!(walker.mode, Mode::Special); assert_eq!(pending, Some(Request::ConnorTalk));
        assert!(!active.world.conversation.as_ref().unwrap().entered());
        assert!(active.world.gui_orders.iter().any(|r| matches!(r, kq8_core::active::GuiOrder::Conversation { start: true, .. })));

        // Catch opens Throw's gate but does not enter Wait before arbitration.
        walker.mode = Mode::Throw; pending = Some(Request::ConnorTalk);
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Wait)); });
        assert_eq!(walker.mode, Mode::Free); assert!(pending.is_none());
        walker.mode = Mode::Throw;
        pending = Some(Request::ConnorSpecial { items: "stop".into(), at: "catch pending Special".into() });
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Wait)); });
        assert_eq!(walker.mode, Mode::Special);
        assert_eq!(pending, Some(Request::ConnorTransition(StateRequest::Wait)));

        // Pending-only retry does not take the handsOff Jump exception.
        walker.start_jump(kq8_core::connor::JumpKind::Stand); hands = true; pending = Some(Request::ConnorTalk);
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::RetryPending)); });
        assert!(!walker.jump_finished); assert_eq!(pending, Some(Request::ConnorTalk));

        let mut made = false; let mut calls = Vec::new();
        commit_choice(&mut pending, &mut None, &mut made, &mut calls, "42".into());
        assert!(pending.is_none() && made);
        assert_eq!(calls, [vec!["Connor".to_string(), "DoChoice".into(), "42".into()]]);

        // A full pending Die Now executes inside the real Special completion
        // callback. It cancels remaining physics without forging Mover::Dead.
        struct Flat;
        impl kq8_core::connor_mover::GroundLookup for Flat {
            fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> { Some((0., [0., 0., 1.])) }
        }
        active = conversation(); hands = false;
        mover = Mover::new([0., 0., 5000.], false);
        walker = Walker::new(0.);
        walker.start_special(vec![kq8_core::connor::SpecialStep::Play("stop", "stop".into())]); walker.phase = 1.;
        pending = Some(Request::ConnorDieNow { at: "pending at Special end".into() });
        let tick = with_dispatch(&mut pending, &mut hands, false, |d| {
            let movements = d.movements;
            let loops = |n: &str| find(movements, None, n).map(|m| &m.movement_loop);
            kq8_core::connor_update::tick_with_requests(&mut mover, &mut walker, Intent::default(), false, &loops, &Flat,
                &mut |m, w, next| { d.request(m, w, &mut active, Request::ConnorTransition(next)); })
        });
        assert_eq!(walker.mode, Mode::ScriptedDie);
        assert_eq!((mover.position, mover.state, mover.use_gravity), ([0., 0., 5000.], State::Normal, false));
        assert!(tick.report.hits.is_empty()); assert!(hands);
        assert_eq!(pending, Some(Request::ConnorTransition(StateRequest::Wait)));
        assert!(matches!(active.world.scripted_death, kq8_core::connor::ScriptedDeath::HoldUntil(45)));
    }

    #[test]
    fn conversation_admission_grouped_pending_snapshot_continuation() {
        let variants = [
            Request::ConnorTalk, Request::ConnorWait { at: "script".into() },
            Request::ConnorTransition(StateRequest::Wait), Request::ConnorTransition(StateRequest::Move), Request::ConnorTransition(StateRequest::Fall),
            Request::ConnorSpecial { items: "stop".into(), at: "saved list".into() },
            Request::ConnorDieNow { at: "saved Die Now".into() },
            Request::ConnorDies { kind: kq8_core::fight::HitKind::Ice, at: "saved hit".into() },
            Request::ConnorAuto { auto: kq8_core::connor_auto::AutoMove::new(kq8_core::connor_auto::AutoKind::Walk { run: false }, [100., 200.], None), at: "saved walk".into() },
        ];
        for request in variants {
            let saved = SavedPendingState { forced_choice: None, forced_choice_recorded: true, request: Some(SavedRequest::take(&request).unwrap()), jump_finished: false, fall_finished: false,
                talk_exit_open: false, preparation_stop: true, interrupt_override: Some(true), airborne: None };
            let copy: SavedPendingState = serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
            let mut slot = None; let mut walker = Walker::new(0.); let mut mover = Mover::new([0.; 3], true); let mut hands = false;
            copy.restore_into(&mut slot, &mut walker, &mut mover, &mut hands, &|_| Err("not needed".into())).unwrap();
            assert_eq!(slot, Some(request)); assert!(walker.pre_special_stop);
        }

        // Actual shared pose restoration retains the Jump/Fall gate and physics
        // before the saved full request is considered at its next boundary.
        let pose: SavedControllerPose = serde_json::from_value(serde_json::json!({
            "movement":"jump", "phase":0.5, "bank":0., "fresh":false, "talk_play":null,
            "wanted":null, "state_wanted":null, "reaction":null, "reaction_playing":null,
            "animation_input_lock":false, "auto":null, "waiting":null, "playing":null,
            "hand_mode":0, "hands_off":false, "turn_by_script":false, "turn_press":false,
            "blocked":[false,false], "moving":{"state":"Jump", "on_ground":false,
                "use_gravity":false, "keep_on_ground":true, "fall_timer":0., "air_updates":0,
                "pull":[0.,0.,-1.], "submerged":false}
        })).unwrap();
        let saved = SavedPendingState { forced_choice: None, forced_choice_recorded: true, request: Some(SavedRequest::Talk), jump_finished: false, fall_finished: false,
            talk_exit_open: false, preparation_stop: true, interrupt_override: Some(true), airborne: Some(SavedAirborne {
                mode: AirborneMode::Jump, kind: 0, pose, step_up: 0., step_down: 0., jump_glanced: true }) };
        let saved: SavedPendingState = serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        let mut pending = None; let mut walker = Walker::new(0.); let mut mover = Mover::new([0.,0.,5000.], true); let mut hands = true;
        saved.restore_into(&mut pending, &mut walker, &mut mover, &mut hands, &|n| if n == "jump" { Ok("jump") } else { Err(n.into()) }).unwrap();
        assert_eq!((walker.mode, walker.phase, mover.state, mover.on_ground, mover.use_gravity, mover.step_up, mover.step_down, hands),
            (Mode::Jump, 0.5, State::Jump, false, false, 0., 0., false));
        assert_eq!(pending, Some(Request::ConnorTalk)); assert!(mover.jump_glanced);
        assert!(!conversation::admits(&walker, false, StateRequest::Talk));
        let mut active = conversation();
        walker.jump_finished = true;
        with_dispatch(&mut pending, &mut hands, false, |d| { d.request(&mut mover, &mut walker, &mut active, Request::ConnorTransition(StateRequest::Fall)); });
        assert_eq!(walker.mode, Mode::Talk); assert_eq!(pending, Some(Request::ConnorTransition(StateRequest::Fall)));
        assert!(!active.world.gui_orders.iter().any(|r| matches!(r, kq8_core::active::GuiOrder::Conversation { start: true, .. })));

        let mut invalid = saved.clone(); invalid.airborne.as_mut().unwrap().kind = 3;
        assert!(!invalid.valid());
        assert!(SavedRequest::take(&Request::HandsOff { on: true, at: "not a pending state".into() }).is_err());
    }
}
