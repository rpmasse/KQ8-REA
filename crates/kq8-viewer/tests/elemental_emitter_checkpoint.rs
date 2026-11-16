//! Original-data Fire and Ice trails resume with their flying bolts after a Rust save.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn saved_fire_and_ice_bolts_restart_one_attached_effect() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/elemental-emitter-checkpoint/{nonce}"));
    for (item, bolt_file, effect) in [
        ("FlameBow", "Prfarrow.kq", "firebits.kq"),
        ("IceCrossbow", "PRicycle.kq", "icebits.kq"),
    ] {
        let case = root.join(item);
        let slots = case.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = case.join(part);
            fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots)
                .arg("--game").arg(&game)
                .args(["--windowed", "--sound", "off", "--no-ai", "--god"])
                .args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
            let logs = dir.join("kq8-artifacts/logs");
            let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            (viewer, fs::read_to_string(logs.join("fight.log")).unwrap(),
                fs::read_to_string(logs.join("effects.log")).unwrap())
        };
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let (before, fight_before, effect_before) = run("save", &[
            "--world", "DeadCity", "--command", &give,
            "--command", "KQConner::setDoMode 1,35",
            "--aim-point", "182000,55000,1800,60",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=Flight:enter:release:escape,70",
            "--updates", "85", "--frames", "150",
        ]);
        assert!(before.contains("Save screen -> Save(1, \"Flight\")")
            && fight_before.contains(&format!("shoots bolt1 ({bolt_file}"))
            && effect_before.contains(&format!("started {} from {effect}", effect.trim_end_matches(".kq"))),
            "elemental launch or save missing; inspect {}", case.display());
        let slot: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
        let bolts = slot["bolts"]["bolts"].as_array().unwrap();
        assert_eq!(bolts.len(), 1, "bolt did not survive to save: inspect {}", case.display());
        assert_eq!(bolts[0]["name"], "bolt1");
        assert_eq!(bolts[0]["emitter"], effect);
        let saved_updates = bolts[0]["updates"].as_u64().unwrap();
        assert!(saved_updates > 0, "save preceded flight: inspect {}", case.display());
        if item == "IceCrossbow" {
            assert_eq!(slot["shot"]["walker_mode"], "Free", "inspect {}", case.display());
            assert_eq!(slot["shot"]["started"], true, "inspect {}", case.display());
        }

        let (after, fight_after, effects_after) = run("restore", &[
            "--menu-input", "down,enter,release,enter,release",
            "--updates", "95", "--frames", "600",
        ]);
        assert!(after.contains("save restored in DeadCity without replaying Begin"),
            "slot did not restore: inspect {}", case.display());
        let effect_name = effect.trim_end_matches(".kq");
        assert_eq!(effects_after.matches(&format!("started {effect_name} from {effect}")).count(), 1,
            "restored bolt started zero or duplicate effects: inspect {}", case.display());
        assert!(fight_after.contains(&format!("bolt1 update {}: at (", saved_updates + 1))
            && fight_after.contains("more than its range")
            && effects_after.contains("follows bolt bolt1:")
            && effects_after.contains("bolt bolt1 ended: dying")
            && effects_after.contains("all particles expired: removed"),
            "restored flight or effect tail failed: inspect {}", case.display());
        assert!(!fight_after.contains("shoots bolt"), "attack replayed after load: inspect {}", case.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
