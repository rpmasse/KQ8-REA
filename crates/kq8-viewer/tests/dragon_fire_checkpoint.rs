//! Both original Dragon fire effects follow restored head attacks.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn dragon_head_fire_attacks_restart_their_effects_after_a_rust_save() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/dragon-fire-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots)
            .arg("--game").arg(&game)
            .args(["--windowed", "--sound", "off", "--god"])
            .args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("fight.log")).unwrap(),
            fs::read_to_string(logs.join("effects.log")).unwrap())
    };
    let (before, _fight_before, effects_before) = run("save", &[
        "--world", "IceWorld", "--at", "182334,490176,30902",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=DragonFire:enter:release:escape,30",
        "--updates", "38", "--frames", "500",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"DragonFire\")")
        && !before.contains("Save failed"), "save failed: inspect {}", root.display());
    for (effect, file) in [
        ("draglfire", "draglfir.kq"),
        ("dragrfire", "dragrfir.kq"),
    ] {
        assert_eq!(effects_before.matches(&format!("[{effect}]: started {effect} from {file}")).count(), 1,
            "original effect did not start before save: inspect {}", root.display());
    }
    let slot: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let actors = slot["kept"]["iceworld"]["characters"].as_array().unwrap();
    for head in ["draglhed", "dragrhed"] {
        let actor = actors.iter().find(|a| a["name"] == head).unwrap();
        assert_eq!(actor["animation"]["movement"], "attack2", "inspect {}", root.display());
        assert!(actor["animation"]["phase"].as_f64().is_some_and(|p| p < 0.5),
            "the saved fire cue had already passed: inspect {}", root.display());
    }

    let (restored, fight, effects) = run("restore", &[
        "--menu-input", "down,enter,release,enter,release",
        "--updates", "22", "--frames", "500",
    ]);
    assert!(restored.contains("save restored in IceWorld without replaying Begin"),
        "slot did not restore: inspect {}", root.display());
    for (head, effect, file) in [
        ("draglhed", "draglfire", "draglfir.kq"),
        ("dragrhed", "dragrfire", "dragrfir.kq"),
    ] {
        assert_eq!(effects.matches(&format!("[{effect}]: started {effect} from {file}")).count(), 1,
            "effect did not start exactly once after load: inspect {}", root.display());
        let positions: Vec<_> = effects.lines()
            .filter(|line| line.contains(&format!("[{effect}] follows {head} part Some(\"fire\")")))
            .filter_map(|line| line.split_once("loc ").map(|(_, loc)| loc)).collect();
        assert!(positions.len() >= 2 && positions[0] != positions[1],
            "effect did not follow the head's animated part: inspect {}", root.display());
        assert_eq!(fight.matches(&format!("{head}: Dragon head Do (attack2")).count(), 1,
            "pending fire cue did not resume once: inspect {}", root.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
