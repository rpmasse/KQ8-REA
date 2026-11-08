//! Forced replacement table in characters.md. Deterministic production
//! controller/dispatcher checks; the interrupted actions are never completed.
use super::*;
use super::weapon_selection_tests::controller;
use kq8_core::{active::{Active, Request}, conversation::StateRequest, secondary_tiles::Kind};

fn active() -> Active {
    Active::new(Default::default(), vec![], vec![],
        ["Connor", "observer", "tile00", "tile46"].map(str::to_owned),
        [("choice.cs".into(), b"if test $2 == StartChoice\n inc Matrix::Choices\n sendEvent 0 observer KQCueEvent 900 0\nendif\n".to_vec())])
}
fn choice() -> Request {
    Request::ConnorChoice { list: kq8_core::choice::ChoiceList::parse("choice.cs,0,0,-200,1,200,0").unwrap(), at: "forced owner matrix".into() }
}
fn queued(a: &mut Active) -> Vec<kq8_core::events::Pending> {
    let mut out = vec![];
    while let Some(e) = a.world.queue.take_due(f64::MAX) { out.push(e); }
    out
}
fn rope() -> kq8_core::rope::Climb {
    kq8_core::rope::Climb { from: kq8_core::rope::End::Bottom, stage: kq8_core::rope::Stage::Hanging,
        place: [1.,2.,3.], dir: [0.,-1.,0.], cam: [None,None], top_limit: 9., bottom_limit: 1.,
        hand_before: WeaponMode::Hand, weapon_before: Some("previous weapon".into()), first_to_third: true,
        aim: [0.;3], range: (1.,9.), saved_steps: Some((500.,600.)), done: false }
}
fn projectile() -> kq8_core::fight::Bolt {
    kq8_core::fight::Bolt { name: "rock1".into(), shooter: "Connor".into(), shape: None,
        position: [1.,2.,3.], direction: [0.,-1.,0.], heading: 0., pitch: 0., speed: 100.,
        travelled: 7., max: 1000., damage: 1, kind: kq8_core::fight::HitKind::Projectile,
        mask: 0, balls: None, updates: 2, hammer: None, script: None, arc: None, ouch_sound: None, drain: None }
}

#[test]
fn forced_choice_owner_exit_and_reentry_matrix() {
    for owner in ["Tile", "Tile2", "Tile3", "Scales", "Rope", "Rock", "Hammer", "Swap", "Swing", "Shot"] {
        let mut a = active();
        let mut sim = controller();
        match owner {
            "Tile" => { sim.enter_tile(&|_| None, "fixture"); sim.tile_calls.clear(); }
            "Tile2" => { sim.enter_secondary_tile(Kind::Swamp, &a.world, "fixture"); }
            "Tile3" => { sim.enter_secondary_tile(Kind::Barren, &a.world, "fixture"); sim.secondary_calls.clear(); sim.choice_music_available = true; }
            "Scales" => sim.enter_scales(),
            "Rope" => { sim.rope = Some(rope()); sim.walker.mode = Mode::Rope;
                sim.rope_model = Some(("throw", 0.2, 0.3)); sim.rope_ended.push("climb"); a.world.connor_roping = true; }
            "Rock" => { assert!(sim.start_rock_throw("new rock".into(), [0.;3], Some("previous projectile".into()))); }
            "Hammer" => { sim.throw = Some(Throw { attack: "throw", wait: "stop", catch: "out", aim: [0.;3], back: false, from_update: 0 }); sim.walker.throw_play("throw"); }
            "Swap" => { sim.readying = true; sim.pending_set = Some((Some("bow.anm".into()), WeaponMode::Ranged, 1));
                sim.weapon_selection.pending_ordinary = true;
                sim.walker.start_special(vec![kq8_core::connor::SpecialStep::Play("away", "away".into()), kq8_core::connor::SpecialStep::Play("bow.anm/out", "out".into())]); }
            "Swing" => { sim.swing = Some(Swing { movement:"throw", target:Some("observer".into()), big:true, waiting:true, point:[1.,2.] });
                sim.walker.start_special(vec![kq8_core::connor::SpecialStep::Play("throw","throw".into())]); }
            "Shot" => { sim.shot = Some(Shot { movement:"throw", aim:[1.,2.,3.], waiting:true, started:false });
                sim.walker.start_special(vec![kq8_core::connor::SpecialStep::Play("throw","throw".into())]); }
            _ => unreachable!(),
        }
        // Actor settings deliberately disagree with normal completion's
        // restoration. Forced entry must leave them alone.
        sim.walker.lean_allowed = false;
        sim.mover.position = [100.,200.,300.]; sim.mover.use_gravity = false;
        sim.mover.keep_on_ground = false; sim.mover.step_up = 7.; sim.mover.step_down = 9.;
        sim.hands_off = true; sim.first_person = true;
        sim.intent.forward = 1; sim.turn_keys.turn = 1;
        a.world.hand = Some("held selection".into());
        a.world.rope_first_person_lock = true; a.world.connor_projectile = Some("current projectile".into());
        a.world.bolts.push(projectile());
        let old_bolts = a.world.bolts.clone();
        if matches!(owner, "Rock"|"Hammer") { sim.throw_ended.push("out"); sim.throw_cues.push(("throw", 0.5)); }
        // Already issued tile effects are separate from controller lifetime.
        sim.secondary_calls.push(SecondaryTileCall::Sound("T2.aud".into()));
        a.world.connor_state = Some(kq8_core::character::state_number::MOVE);
        a.world.state_changes();
        assert_eq!(a.console_command("KQMonster::stateChangeNotify observer Connor", "matrix"), Some("0".into()));
        let request = choice();
        sim.dispatch_state(&mut a, request.clone()).unwrap();
        assert!(sim.choosing(), "{owner}");
        assert!(sim.tile.is_none() && sim.secondary_tile.is_none() && sim.scales.is_none()
            && sim.rope.is_none() && sim.rock.is_none() && sim.throw.is_none(), "{owner}");
        assert!(!sim.readying && sim.pending_set.is_none());
        assert!(sim.swing.is_none() && sim.shot.is_none());
        assert_eq!(sim.pending_state, Some(request));
        assert_eq!(sim.walker.movement, "stop");
        assert_eq!((sim.intent.forward, sim.turn_keys.turn), (0,0));
        assert_eq!(sim.mover.position, [100.,200.,300.]);
        assert_eq!((sim.mover.use_gravity,sim.mover.keep_on_ground,sim.mover.step_up,sim.mover.step_down), (false,false,7.,9.), "{owner}");
        assert!(sim.hands_off && sim.first_person && a.world.rope_first_person_lock, "{owner}");
        assert_eq!(a.world.hand.as_deref(), Some("held selection"));
        assert_eq!(a.world.connor_projectile.as_deref(), Some("current projectile"));
        assert_eq!(a.world.bolts, old_bolts);
        assert_eq!(sim.secondary_calls, [SecondaryTileCall::Sound("T2.aud".into())]);
        assert_eq!(sim.walker.lean_allowed, matches!(owner,"Tile"|"Tile2"|"Tile3"|"Scales"));
        if owner == "Rope" { assert_eq!(sim.rope_model, Some(("throw",0.2,0.3))); assert!(!a.world.connor_roping && sim.rope_ended.is_empty()); }
        if matches!(owner,"Rock"|"Hammer") {
            assert!(sim.throw_ended.is_empty() && sim.throw_cues.is_empty());
            assert!(sim.throw_caught("late catch").contains("not throwing"));
            assert!(sim.rock_step(&mut a).is_none());
        }
        assert_eq!(a.scripts.vars.get("Matrix::Choices"), Some("1"));
        let reset = a.world.requests.iter().position(|r| matches!(r,Request::ResetCameraInput {..})).expect("Choice camera input reset");
        if owner == "Tile" {
            let release = a.world.requests.iter().position(|r| matches!(r,Request::CameraHold {on:false,..})).unwrap();
            assert!(release < reset);
        }
        let events = queued(&mut a);
        let music: Vec<_> = events.iter().filter(|e| kq8_core::events::canonical_kind(&e.kind) == "KQMusicPopEvent").collect();
        let cues: Vec<_> = events.iter().filter(|e| e.kind == "KQCueEvent" && e.data.first().is_some_and(|d|d=="2402")).collect();
        if owner == "Tile3" {
            assert_eq!(music.len(),1); assert!((music[0].due-0.2).abs()<1e-6);
            assert_eq!(cues.iter().map(|e|e.target.as_str()).collect::<Vec<_>>(), ["tile00","tile46"]);
        } else { assert!(music.is_empty() && cues.is_empty()); }
        let start = events.iter().find(|e|e.data.first().is_some_and(|d|d=="900")).expect("StartChoice's event");
        let state = events.iter().find(|e|kq8_core::events::canonical_kind(&e.kind)=="KQStateChangeNotifyEvent").expect("Choice state notification");
        assert!(start.seq < state.seq, "StateChange is queued after StartChoice");
    }

    // Solved state and service existence are independent exit conditions.
    for (solved,music) in [(false,false),(true,false),(true,true)] {
        let mut a = active(); let mut sim = controller(); a.world.solved_tile3 = solved;
        sim.enter_secondary_tile(Kind::Barren,&a.world,"exit conditions"); sim.secondary_calls.clear();
        sim.choice_music_available = music;
        sim.dispatch_state(&mut a,choice()).unwrap();
        let events = queued(&mut a);
        assert_eq!(events.iter().filter(|e|kq8_core::events::canonical_kind(&e.kind)=="KQMusicPopEvent").count(),usize::from(music));
        assert_eq!(events.iter().filter(|e|e.kind=="KQCueEvent" && e.data.first().is_some_and(|d|d=="2402")).count(),if solved {0} else {2});
    }

    // An admitted Rock must return through the common request while Throw
    // owns Connor, so the remembered Choice gets a fresh entry and Wait is
    // retained. No normal completion is synthesized for a forced interruption.
    let mut a = active(); let mut sim = controller();
    sim.dispatch_state(&mut a,choice()).unwrap();
    assert!(sim.start_rock_throw("rock".into(),[0.;3],None));
    sim.throw_ended.push("away"); assert_eq!(sim.rock_step(&mut a),Some(Some("rock".into())));
    sim.throw_ended.push("throw"); assert_eq!(sim.rock_step(&mut a),Some(None));
    assert!(sim.choosing()); assert_eq!(a.scripts.vars.get("Matrix::Choices"),Some("2"));
    assert_eq!(sim.pending_state,Some(Request::ConnorTransition(StateRequest::Wait)));
}
