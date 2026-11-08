//! Original request/ownership boundaries (ranged-weapons.md 6–9,
//! conversation.md common admission, characters.md Choice). Real item data,
//! production controller/dispatcher and deterministic timed clips; no renderer.
use super::*;
use super::weapon_selection_tests::{active, controller, pending_save, restore, tick};
use kq8_core::{active::{Active, Request}, connor::{JumpKind, SpecialStep},
    conversation::StateRequest, weapons::{Ask, Request as WeaponRequest, SwapRequest}};
use kq8_data::{GameDir, items::ItemTypes};

fn types() -> ItemTypes {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    ItemTypes::parse(&std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap()).0
}

fn actor() -> ConnorSim {
    let mut sim = controller();
    for bank in ["", "fists.anm", "bow.anm"] {
        for action in ["custom", "away2", "jump"] {
            let name: &'static str = Box::leak(if bank.is_empty() { action.to_owned() }
                else { format!("{bank}/{action}") }.into_boxed_str());
            sim.movements.push(DrawnMovement { name, qualified: name, first_person: false,
                movement_loop: MovementLoop { duration: 0.2, frames: 2, repeat: false,
                    cut_short: true, speed: 1., ..Default::default() }, frames: 2, anim: 0, parts: vec![] });
        }
    }
    sim
}

fn deliver(sim: &mut ConnorSim, active: &mut Active, ask: Ask) -> bool {
    sim.deliver_weapon_request(active, &WeaponRequest { ask, how: "shared admission matrix".into() }, false)
}

fn swap(mode: WeaponMode, movement: Option<&str>) -> Request {
    Request::ConnorWeaponSwap(SwapRequest { mode, movement: movement.map(str::to_owned),
        script: true, shift: true, how: "retained full Swap payload".into() })
}

fn finish(sim: &mut ConnorSim, active: &mut Active) {
    for _ in 0..40 {
        if !sim.readying { return; }
        tick(sim, active);
    }
    panic!("swap still owns {:?} {} at {}", sim.walker.mode, sim.walker.movement, sim.walker.phase);
}

fn with_scripts(types: &ItemTypes, scripts: &[(&str, &str)]) -> (Active, String) {
    let (base, held) = active(types);
    let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()],
        scripts.iter().map(|(name, text)| (name.to_string(), text.as_bytes().to_vec())));
    a.world.items = base.world.items;
    a.world.connor_fight = base.world.connor_fight;
    a.world.hand = base.world.hand;
    (a, held)
}

fn ordinary_hand(sim: &mut ConnorSim, a: &mut Active) {
    a.world.commit_weapon_change(WeaponMode::Hand, false);
    sim.ready_set(Some("fists.anm".into()), WeaponMode::Hand,
        Some(kq8_core::weapons::FISTS.into()), true, false, false);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original inventory and production state/animation boundary matrix"]
fn weapon_swap_shared_admission_and_script_preparation_matrix() {
    let types = types();
    let (mut a, held) = active(&types);
    let mut sim = actor();
    sim.walker.start_jump(JumpKind::Stand);
    sim.walker.phase = 0.25;
    sim.mover.use_gravity = false;
    let before = (sim.walker.mode, sim.walker.phase, sim.walker.jump_finished,
        sim.walker.pre_special_stop, sim.mover.use_gravity);
    for ask in [Ask::Script(WeaponMode::None), Ask::ScriptMovement { mode: WeaponMode::None, movement: "custom".into() }] {
        assert!(!deliver(&mut sim, &mut a, ask));
        assert_eq!((sim.walker.mode, sim.walker.phase, sim.walker.jump_finished,
            sim.walker.pre_special_stop, sim.mover.use_gravity), before, "same-mode script performs no preparation");
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    }
    assert!(deliver(&mut sim, &mut a, Ask::Script(WeaponMode::Hand)));
    assert!(sim.mover.use_gravity && sim.readying);
    assert_eq!((sim.walker.mode, sim.walker.movement), (Mode::Special, "fists.anm/draw"));
    assert!(a.world.hand.is_none());

    // Both script forms try Wait. Only the form without optional movement
    // prepares Stop; an unfinished multi-entry Special still owns Connor.
    for optional in [false, true] {
        let (mut a, held) = active(&types);
        let mut sim = actor();
        sim.walker.start_special(vec![SpecialStep::Play("throw", "throw".into()), SpecialStep::Play("custom", "custom".into())]);
        let ask = if optional { Ask::ScriptMovement { mode: WeaponMode::Hand, movement: "custom".into() } }
            else { Ask::Script(WeaponMode::Hand) };
        assert!(!deliver(&mut sim, &mut a, ask));
        assert_eq!(sim.walker.mode, Mode::Special);
        assert_eq!(sim.walker.pre_special_stop, !optional);
        assert_eq!(sim.pending_state, Some(Request::ConnorTransition(StateRequest::Wait)));
        assert_eq!((a.world.weapon_mode, sim.set_mode), (WeaponMode::None, WeaponMode::None));
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    }

    // Preparatory Wait may admit an older strong Talk. The subsequent lower
    // request reaches Talk, so no Swap is manufactured or applied afterward.
    let (mut a, _) = active(&types);
    let mut sim = actor();
    a.world.monsters.push(kq8_core::active::Monster { name: "Bob".into(), position: [10., 20., 0.],
        script: None, goodbye: None, welcome_back: None });
    assert_eq!(a.test_command("KQConner::conversation Bob 1000 60 0 0 1 1 0", "matrix").as_deref(), Some("0"));
    a.world.requests.clear();
    assert!(a.conversation_prepare(&mut sim.walker));
    sim.pending_state = Some(Request::ConnorTalk);
    assert!(!deliver(&mut sim, &mut a, Ask::Script(WeaponMode::Hand)));
    assert_eq!(sim.walker.mode, Mode::Talk);
    assert!(!sim.readying);
    assert_eq!(sim.pending_state, Some(Request::ConnorTransition(StateRequest::Wait)));

    // Every optional name enters in the new list immediately. Names retain
    // their built-in completion meanings, including case-insensitive away2.
    for (movement, expected_out) in [("custom", false), ("AwAy2", true), ("draw", false), ("undraw", false)] {
        let (mut a, held) = active(&types);
        let mut sim = actor();
        assert!(deliver(&mut sim, &mut a, Ask::ScriptMovement { mode: WeaponMode::Ranged, movement: movement.into() }));
        assert_eq!(sim.set_mode, WeaponMode::Ranged);
        assert_eq!(sim.weapon_set.as_deref(), Some("bow.anm"));
        assert!(kq8_core::connor::plain_movement_name(sim.walker.movement).eq_ignore_ascii_case(movement));
        assert!(a.world.hand.is_none());
        a.world.hand = Some(held.clone());
        let mut saw_out = false;
        for _ in 0..40 {
            if !sim.readying { break; }
            tick(&mut sim, &mut a);
            saw_out |= sim.walker.movement == "bow.anm/out";
        }
        assert!(!sim.readying);
        assert_eq!(saw_out, expected_out);
        assert_eq!(a.world.hand.is_none(), expected_out || movement == "undraw", "only a later assignment clears the reselection");
    }

    // Swap's single final clip remains closed while it is playing. A pending
    // Special can own the completion call only after actual timed ticks end it.
    let (mut a, _) = active(&types);
    let mut sim = actor();
    deliver(&mut sim, &mut a, Ask::ScriptMovement { mode: WeaponMode::Hand, movement: "custom".into() });
    tick(&mut sim, &mut a);
    assert!(sim.readying && sim.walker.phase < 1.);
    let next = Request::ConnorSpecial { items: "throw".into(), at: "after real last clip".into() };
    sim.dispatch_state(&mut a, next.clone()).unwrap();
    assert!(sim.readying);
    assert_eq!(sim.pending_state, Some(next));
    assert_eq!(sim.walker.movement, "fists.anm/custom");
    finish(&mut sim, &mut a);
    assert_eq!((sim.walker.mode, sim.walker.movement), (Mode::Special, "throw"));
    assert_eq!(sim.pending_state, Some(Request::ConnorTransition(StateRequest::Wait)));

    // Raw assignment preserves the active bank, never enters Swap, and obeys
    // the temporary modifier's remembered-ranged exception. Normal requests
    // afterward use that retained current list for their away movement.
    for modifier in [false, true] {
        let (mut a, held) = active(&types);
        let mut sim = actor();
        a.world.commit_weapon_change(WeaponMode::Ranged, false);
        sim.ready_set(Some("bow.anm".into()), WeaponMode::Ranged, a.world.weapon_ready.clone(), true, false, false);
        a.world.last_away_ranged = false;
        let request = WeaponRequest { ask: Ask::ScriptMovement { mode: WeaponMode::Raw(37), movement: "custom".into() }, how: "raw mode".into() };
        assert!(sim.deliver_weapon_request(&mut a, &request, modifier));
        assert_eq!((sim.set_mode, a.world.weapon_mode, sim.weapon_set.as_deref()), (WeaponMode::Raw(37), WeaponMode::Raw(37), Some("bow.anm")));
        assert!(!sim.readying && a.world.hand.is_none());
        assert_eq!(a.world.last_away_ranged, !modifier);
        a.world.hand = Some(held);
        assert!(!sim.deliver_weapon_request(&mut a, &request, modifier));
        assert!(a.world.hand.is_some());
        assert!(deliver(&mut sim, &mut a, Ask::Script(WeaponMode::Hand)));
        assert_eq!(sim.walker.movement, "bow.anm/away");
        finish(&mut sim, &mut a);
        assert_eq!((sim.set_mode, sim.weapon_set.as_deref()), (WeaponMode::Hand, Some("fists.anm")));
    }

    // Choice is a remembered forced owner, with target-aware admission. The
    // first request and return's Wait remain pending until DoChoice clears it.
    let (mut a, _) = with_scripts(&types, &[("choice.cs", "if test $2 == StartChoice\n inc Matrix::Choices\nendif\n")]);
    let mut sim = actor();
    let choice = Request::ConnorChoice { list: kq8_core::choice::ChoiceList::parse("choice.cs,0,0,-200,1,200,0").unwrap(), at: "matrix Choice".into() };
    sim.dispatch_state(&mut a, choice.clone()).unwrap();
    assert_eq!(sim.pending_state, Some(choice));
    assert_eq!(a.scripts.vars.get("Matrix::Choices"), Some("1"));
    assert!(deliver(&mut sim, &mut a, Ask::Required(WeaponMode::Hand)));
    finish(&mut sim, &mut a);
    assert!(sim.choosing() && sim.forced_choice.is_some());
    assert_eq!(a.scripts.vars.get("Matrix::Choices"), Some("2"));
    assert_eq!(sim.pending_state, Some(Request::ConnorTransition(StateRequest::Wait)));
    sim.choice_step();
    sim.intent.forward = 1;
    sim.choice_step();
    assert!(sim.choice.as_ref().unwrap().made);
    assert!(sim.pending_state.is_none() && sim.forced_choice.is_none());
    assert_eq!(sim.choice_calls.iter().filter(|args| args.get(1).is_some_and(|name| name == "DoChoice")).count(), 1);

    // Required-mode correction is internal to the same Swap; it does not
    // arbitrate pending Talk until its corrected out actually completes.
    let (mut a, held) = active(&types);
    let mut sim = actor();
    a.world.required_weapon_mode = (0, None);
    deliver(&mut sim, &mut a, Ask::ScriptMovement { mode: WeaponMode::Ranged, movement: "custom".into() });
    sim.pending_state = Some(Request::ConnorTalk);
    a.world.hand = Some(held);
    let mut correction_seen = false;
    for _ in 0..40 {
        tick(&mut sim, &mut a);
        if sim.walker.movement == "bow.anm/away" {
            correction_seen = true;
            assert!(sim.readying);
            assert_eq!(sim.pending_state, Some(Request::ConnorTalk));
            assert_eq!(sim.set_mode, WeaponMode::Ranged);
            assert!(a.world.hand.is_some());
            break;
        }
    }
    assert!(correction_seen);
    finish(&mut sim, &mut a);
    assert_eq!(sim.set_mode, WeaponMode::Hand);
    assert!(sim.pending_state.is_none() && a.world.hand.is_none());
    for movement in ["draw", "undraw"] {
        let (mut a, _) = active(&types);
        let mut sim = actor();
        a.world.required_weapon_mode = (0, None);
        deliver(&mut sim, &mut a, Ask::ScriptMovement { mode: WeaponMode::Ranged, movement: movement.into() });
        finish(&mut sim, &mut a);
        assert_eq!(sim.set_mode, WeaponMode::Ranged, "{movement} skips the required-mode check");
    }
    let (mut a, _) = with_scripts(&types, &[("wrong.cs", "if test $2 == WrongWeapon\n inc Matrix::Wrong\n KQConner::setRequiredWeaponMode -1\nendif\n")]);
    let mut sim = actor();
    a.world.required_weapon_mode = (0, Some("wrong.cs".into()));
    deliver(&mut sim, &mut a, Ask::ScriptMovement { mode: WeaponMode::Ranged, movement: "custom".into() });
    finish(&mut sim, &mut a);
    assert_eq!(a.scripts.vars.get("Matrix::Wrong"), Some("1"));
    assert_eq!((sim.set_mode, a.world.weapon_mode, sim.weapon_set.as_deref()), (WeaponMode::Raw(-1), WeaponMode::Raw(-1), Some("bow.anm")));
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; full pending Swap and raw-mode recreation checkpoint matrix"]
fn weapon_swap_pending_payload_checkpoint_matrix() {
    let types = types();
    let (mut a, held) = active(&types);
    let mut sim = actor();
    let payload = swap(WeaponMode::Ranged, Some("AwAy2"));
    sim.pending_state = Some(Request::ConnorSpecial { items: "throw,custom".into(), at: "older strong state".into() });
    sim.dispatch_state(&mut a, payload.clone()).unwrap();
    assert_eq!(sim.walker.mode, Mode::Special);
    assert_eq!(sim.pending_state, Some(payload.clone()));
    assert_eq!((a.world.weapon_mode, sim.set_mode, sim.weapon_set.clone()), (WeaponMode::None, WeaponMode::None, None));
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    assert_eq!(sim.weapon_selection, WeaponSelection::default());
    let (weapon, special) = pending_save(&sim);
    let pending = sim.saved_pending_state().unwrap();
    let pending: SavedPendingState = serde_json::from_slice(&serde_json::to_vec(&pending).unwrap()).unwrap();
    let mut resumed = actor();
    restore(&mut resumed, &weapon, &special);
    resumed.restore_pending_state(&pending).unwrap();
    assert_eq!(resumed.pending_state, Some(payload));
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    // An actual ordinary Special completion admits the retained Swap and
    // retains its new Wait proposal without testing it against that new owner.
    for _ in 0..40 {
        tick(&mut resumed, &mut a);
        if resumed.readying { break; }
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    }
    assert!(resumed.readying);
    assert_eq!(resumed.walker.movement, "bow.anm/away2");
    assert_eq!(resumed.set_mode, WeaponMode::Ranged);
    assert_eq!(resumed.pending_state, Some(Request::ConnorTransition(StateRequest::Wait)));
    assert!(a.world.hand.is_none());
    finish(&mut resumed, &mut a);

    // Raw mode has no replacement bank; serialize the actual retained bank
    // and owner pose instead of deriving a list from the raw numeric value.
    let (mut a, held) = active(&types);
    let mut sim = actor();
    ordinary_hand(&mut sim, &mut a);
    sim.walker.movement = "fists.anm/custom";
    sim.walker.phase = 0.4;
    assert!(deliver(&mut sim, &mut a, Ask::Required(WeaponMode::Raw(91))));
    let weapon = sim.saved_weapon_state(None).unwrap();
    let pose = sim.capture_controller_pose();
    let pending = sim.saved_pending_state().unwrap();
    let bytes = serde_json::to_vec(&(a.world.weapon_mode, weapon, pose, pending)).unwrap();
    let (mode, weapon, pose, pending): (WeaponMode, SavedWeaponState, SavedControllerPose, SavedPendingState) = serde_json::from_slice(&bytes).unwrap();
    a.world.hand = Some(held.clone());
    let mut resumed = actor();
    resumed.restore_weapon_state(&weapon, None).unwrap();
    resumed.restore_controller_pose(&pose, Mode::Free, "raw mode checkpoint").unwrap();
    resumed.restore_pending_state(&pending).unwrap();
    assert_eq!(mode, WeaponMode::Raw(91));
    assert_eq!((resumed.set_mode, resumed.weapon_set.as_deref(), resumed.walker.movement, resumed.walker.phase),
        (WeaponMode::Raw(91), Some("fists.anm"), "fists.anm/custom", 0.4));
    assert!(!resumed.readying && resumed.pending_state.is_none());
    assert!(!resumed.apply_weapon_assignment(&mut a.world));
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
}
