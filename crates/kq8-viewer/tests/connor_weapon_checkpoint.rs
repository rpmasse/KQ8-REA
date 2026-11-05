//! A GOG Small Crossbow swap keeps its current animation and set-switch point.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn crossbow_swap_and_ready_stance_resume_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-weapon-checkpoint/{nonce}"));
    for (case, at, movement, phase, pending) in [
        ("away", 28, Some("away"), 0.5, true),
        ("out", 35, Some("conscbow.anm/out"), 0.4, false),
        ("ready", 45, None, 0.0, false),
    ] {
        let base = root.join(case);
        let slots = base.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = base.join(part);
            fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
            let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            viewer
        };
        let key = format!("menu:down:down:enter:release:enter:release:escape:type=Swap:enter:release:escape,{at}");
        let before = run("save", &[
            "--world", "DeadCity", "--item", "INVITEM_SmCrossbow",
            "--command", "KQConner::setDoMode 1,25", "--frames", "75", "--key", &key,
        ]);
        assert!(before.contains("Save screen -> Save(1, \"Swap\")"), "inspect {}", base.display());
        let read = |n| -> serde_json::Value {
            serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
        };
        let first = read(1);
        assert_eq!(first["journey"]["weapon_ready"], "INVITEM_SmCrossbow");
        assert_eq!(first["weapon_state"]["weapon_target"], "conscbow.anm");
        assert_eq!(first["weapon_state"]["readying"], movement.is_some());
        assert_eq!(first["weapon_state"]["pending_set"].is_null(), !pending);
        if let Some(movement) = movement {
            assert_eq!(first["special"]["movement"], movement);
            assert!((first["special"]["phase"].as_f64().unwrap() - phase).abs() < 0.001,
                "wrong saved phase; inspect {}", base.display());
        } else {
            assert!(first["special"].is_null());
        }

        let after = run("restore", &[
            "--menu-input", "down,enter,release,enter,release", "--frames", "700",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterSwap:enter:release:escape,70",
        ]);
        assert!(after.contains("save restored in DeadCity without replaying Begin")
            && after.contains("Save screen -> Save(2, \"AfterSwap\")"), "inspect {}", base.display());
        let connor = fs::read_to_string(base.join("restore/kq8-artifacts/logs/connor.log")).unwrap();
        let fight = fs::read_to_string(base.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
        assert!(!fight.contains("swap (none -> ranged)"), "the swap restarted; inspect {}", base.display());
        if let Some(movement) = movement {
            let fields: Vec<&str> = connor.lines().find(|line| line.starts_with("1 ")).unwrap().split_whitespace().collect();
            let mode = fields.iter().position(|field| *field == "special").unwrap();
            assert_eq!(fields[mode + 1], movement);
            let resumed: f64 = fields[mode + 3].parse().unwrap();
            let increment = if case == "away" { 1.0 / 6.0 } else { 0.1 };
            assert!((resumed - (phase + increment)).abs() < 0.002,
                "the swap animation restarted; inspect {}", base.display());
        }
        assert_eq!(connor.matches("weapon: the set switches now").count(), usize::from(pending),
            "the movement set switched at the wrong point; inspect {}", base.display());
        let second = read(2);
        assert_eq!(second["weapon_state"]["weapon_set"], "conscbow.anm");
        assert_eq!(second["weapon_state"]["set_mode"], "Ranged");
        assert_eq!(second["weapon_state"]["readying"], false);
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn small_axe_swap_resumes_before_and_after_the_set_switch() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-axe-checkpoint/{nonce}"));
    for (case, at, pending, expected_list) in [
        ("away", 28, true, "away"),
        ("out", 35, false, "conaxe.anm/out"),
    ] {
        let base = root.join(case);
        let slots = base.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = base.join(part);
            fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
            let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            viewer
        };
        let key = format!("menu:down:down:enter:release:enter:release:escape:type=Axe:enter:release:escape,{at}");
        let before = run("save", &[
            "--world", "DeadCity", "--item", "INVITEM_SmallAx",
            "--command", "KQConner::setDoMode 0,25", "--frames", "75", "--key", &key,
        ]);
        assert!(before.contains("Save screen -> Save(1, \"Axe\")"), "inspect {}", base.display());
        let read = |n| -> serde_json::Value {
            serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
        };
        let first = read(1);
        assert_eq!(first["journey"]["weapon_ready"], "INVITEM_SmallAx");
        assert_eq!(first["weapon_state"]["weapon_target"], "conAxe.anm");
        assert_eq!(first["weapon_state"]["pending_set"].is_null(), !pending);
        assert_eq!(first["special"]["movement"], expected_list);
        let phase = first["special"]["phase"].as_f64().unwrap();
        assert!((0.0..1.0).contains(&phase), "wrong saved phase; inspect {}", base.display());

        let after = run("restore", &[
            "--menu-input", "down,enter,release,enter,release", "--frames", "700",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterAxe:enter:release:escape,70",
        ]);
        assert!(after.contains("save restored in DeadCity without replaying Begin")
            && after.contains("Save screen -> Save(2, \"AfterAxe\")"), "inspect {}", base.display());
        let connor = fs::read_to_string(base.join("restore/kq8-artifacts/logs/connor.log")).unwrap();
        let fields: Vec<&str> = connor.lines().find(|line| line.starts_with("1 ")).unwrap().split_whitespace().collect();
        let mode = fields.iter().position(|field| *field == "special").unwrap();
        assert_eq!(fields[mode + 1], expected_list);
        let resumed: f64 = fields[mode + 3].parse().unwrap();
        assert!(resumed > phase, "the axe animation restarted; inspect {}", base.display());
        assert_eq!(connor.matches("weapon: the set switches now").count(), usize::from(pending),
            "the set switched at the wrong point; inspect {}", base.display());
        let second = read(2);
        assert_eq!(second["weapon_state"]["weapon_set"], "conAxe.anm");
        assert_eq!(second["weapon_state"]["set_mode"], "Hand");
        assert_eq!(second["weapon_state"]["readying"], false);
    }
}
