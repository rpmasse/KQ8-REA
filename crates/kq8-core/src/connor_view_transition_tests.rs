//! Q200: playing-name F2 refresh and ordinary Move completion, original data.
use super::*;
use kq8_data::{GameDir, VolArchive, characters::AnimList, shape::Shape};

fn original_loops(game: &GameDir, file: &str) -> Vec<(&'static str, MovementLoop)> {
    let list = AnimList::parse(&std::fs::read(game.find(&format!("GAME/Kq/{file}")).unwrap()).unwrap()).unwrap();
    let archive = VolArchive::open(&game.volume_path("Daventry", "resource").unwrap()).unwrap();
    list.movements.iter()
        .filter(|(name, _)| ["stop", "stoprun", "run", "walk", "stopwalkstop"].contains(&name.to_ascii_lowercase().as_str()))
        .map(|(name, movement)| {
            let bytes = if let Some(entry) = archive.find(&movement.shape) {
                archive.unpack(entry).unwrap().data
            } else {
                std::fs::read(game.find(&format!("GAME/Common/{}", movement.shape)).unwrap()).unwrap()
            };
            let shape = Shape::parse(&bytes).unwrap();
            let sequence = shape.loops.iter().find(|l| shape.names.get(l.name as usize)
                .is_some_and(|name| name.eq_ignore_ascii_case(&movement.sequence))).unwrap_or(&shape.loops[0]);
            let first = sequence.first_sequence as usize;
            let count = sequence.sequence_count as usize;
            let frame_steps: Vec<_> = shape.sequences[first..first + count].iter().map(|s| s.transform.p).collect();
            let mut step_sum = [0.; 3];
            for step in &frame_steps { for i in 0..3 { step_sum[i] += step[i]; } }
            (intern(&name.to_ascii_lowercase()), MovementLoop {
                duration: sequence.duration, frames: count, frame_steps, step_sum,
                first_in_file: first == 0, speed: movement.speed, start: movement.start,
                repeat: movement.repeat, cut_short: movement.unknown != 0,
                transitions: movement.transitions.iter().map(|t| Transition {
                    from: t.from, to: intern(&t.to.to_ascii_lowercase()), to_phase: t.to_phase,
                    via: intern(&t.via.to_ascii_lowercase()),
                }).collect(), cues: movement.cues.clone(),
            })
        }).collect()
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; grouped original-data F2 transition completion"]
fn original_f2_transition_completion_reselects_held_input() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let old = original_loops(&game, "Conner.anm");
    let current = original_loops(&game, "Connerfp.anm");
    let old_loop = |name: &str| old.iter().find(|(n, _)| *n == name).map(|(_, l)| l);
    let current_loop = |name: &str| current.iter().find(|(n, _)| *n == name).map(|(_, l)| l);
    let loops = |name: &str| match name.strip_prefix("conner.anm/") {
        Some(name) => old_loop(name), None => current_loop(name),
    };
    let transition = old_loop("stop").unwrap().transitions.iter().find(|t| t.to == "run").copied().unwrap();
    assert_eq!(transition.via, "stoprun");
    assert!((transition.to_phase - 0.866667).abs() < 1e-6);
    let replay = current_loop("stoprun").unwrap();
    assert_eq!(replay.frames, 3);
    assert!((replay.start - 0.666667).abs() < 1e-6);
    assert!(!replay.repeat && !replay.cut_short);
    let held = Intent { forward: 1, turn: 0, run: true };
    let mut walker = Walker::new(0.);
    walker.movement = "stoprun";
    walker.choice = "run";
    walker.phase = 0.4;
    walker.playing = Some(transition);
    walker.wanted = Some("back");
    let refresh = walker.rerequest_view("conner.anm/stoprun", "stoprun", &loops);
    assert_eq!((walker.movement, walker.phase, walker.wanted), ("conner.anm/stoprun", 0.4, Some("stoprun")));
    assert_eq!(walker.playing.unwrap().to, "run");
    assert_eq!(walker.playing.unwrap().to_phase, transition.to_phase);
    assert_eq!(walker.playing.unwrap().via, "conner.anm/stoprun");
    println!("Q200 refresh: {}", refresh.join("; "));
    let mut ended = false;
    for _ in 0..30 {
        let step = walker.update(held, &loops);
        assert!(!step.missing);
        if step.ended == Some("conner.anm/stoprun") {
            println!("Q200 old completion: {}", step.notes.join("; "));
            assert!(step.notes.iter().any(|n| n.contains("run from phase 0.866667")));
            assert_eq!((walker.movement, walker.phase, walker.wanted, walker.playing), ("stoprun", 0., Some("run"), None));
            ended = true;
            break;
        }
    }
    assert!(ended, "original transition must finish");
    let first = walker.update(held, &loops);
    assert_eq!(first.played, "stoprun");
    assert!((first.played_from - replay.start_phase()).abs() < 1e-6);
    let mut replay_ends = usize::from(first.ended == Some("stoprun"));
    for _ in 0..8 {
        let step = walker.update(held, &loops);
        replay_ends += usize::from(step.ended == Some("stoprun"));
        assert!(!step.missing);
    }
    assert_eq!(replay_ends, 1);
    assert_eq!((walker.mode, walker.movement, walker.choice, walker.wanted), (Mode::Free, "run", "run", None));

    // While a transition is merely waiting, F2 requests the source name
    // and clears the old stop destination; it does not invent a run request.
    let mut waiting = Walker::new(0.);
    waiting.movement = "walk";
    waiting.choice = "stop";
    waiting.phase = 0.5;
    waiting.wanted = Some("stop");
    waiting.waiting = old_loop("walk").unwrap().transitions.iter().find(|t| t.to == "stop").copied();
    assert!(waiting.waiting.is_some());
    waiting.rerequest_view("conner.anm/walk", "walk", &loops);
    assert_eq!((waiting.movement, waiting.phase, waiting.wanted, waiting.waiting, waiting.playing), ("walk", 0., None, None, None));

    // The new follow-up belongs to Move; a Talk one-shot and a controller's
    // owned motion retain their existing completion behavior.
    for controller in [false, true] {
        let mut other = Walker::new(0.);
        other.mode = if controller { Mode::Free } else { Mode::Talk };
        other.controller_owned_input = controller;
        other.choice = "run";
        other.movement = "stoprun";
        other.phase = 0.9;
        let step = other.update(held, &loops);
        assert_eq!(step.ended, Some("stoprun"));
        assert_eq!((other.movement, other.phase, other.wanted), ("stoprun", 1., None));
    }
    println!("Q200 PASS: original ANM/KQS; pending request overwritten; old transition retained; run destination phase; new replay authored start; held-run recovery; waiting-source rebuild; state boundaries");
}
