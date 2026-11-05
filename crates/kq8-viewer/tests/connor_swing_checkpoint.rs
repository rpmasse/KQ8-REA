//! A melee cue after a disk save must still hit exactly once on reload.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn axe_swing_before_do_cue_resumes_in_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-swing-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--item", "INVITEM_SmallAx",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--swing", "skeleton1,20", "--swing", "skeleton1,50", "--frames", "300",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=MidSwing:enter:release:escape,54",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"MidSwing\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let skeleton = |slot: &serde_json::Value| -> i64 {
        slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
            .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap()
    };
    assert_eq!(skeleton(&first), 60, "the original skeleton must still be unhurt at the save");
    assert!(first["swing"]["phase"].as_f64().is_some_and(|p| p < 0.5), "the saved swing must be before Do; inspect {}", root.display());

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterSwing:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    let second = read(2);
    let health = skeleton(&second);
    assert!((1..60).contains(&health), "the resumed swing did not injure skeleton1; inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("Connor: Do (conaxe.anm/attack): hits skeleton1").count(), 1,
        "the resumed Do cue must hit once; inspect {}", root.display());
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn waiting_axe_swing_resumes_its_turn_and_then_hits() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-swing-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--item", "INVITEM_SmallAx",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--swing", "skeleton1,20",
        // Turn Connor 90 degrees away without changing his position.
        "--command", "move Connor 3 0 0 0 0 0 1.57,40",
        "--swing", "skeleton1,50", "--frames", "300",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=WaitingSwing:enter:release:escape,51",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"WaitingSwing\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    assert_eq!(first["swing"]["waiting"], true, "the slot should contain a waiting turn; inspect {}", root.display());
    let first_health = first["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
        .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap();
    assert_eq!(first_health, 60);

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterTurn:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    let second = read(2);
    let health = second["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
        .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap();
    assert!((1..60).contains(&health), "the waiting swing did not hit after its restored turn; inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("Connor: Do (conaxe.anm/attack): hits skeleton1").count(), 1,
        "the restored waiting swing must hit once; inspect {}", root.display());
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn axe_swing_after_do_cue_does_not_hit_twice() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-swing-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--item", "INVITEM_SmallAx",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--swing", "skeleton1,20", "--swing", "skeleton1,50", "--frames", "300",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=AfterDo:enter:release:escape,58",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"AfterDo\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let health = |slot: &serde_json::Value| -> i64 {
        slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
            .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap()
    };
    let injured = health(&first);
    assert!((1..60).contains(&injured) && first["swing"]["phase"].as_f64().is_some_and(|p| p > 0.5),
        "the slot must be after Do but before the attack ends; inspect {}", root.display());

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=NoSecondHit:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    assert_eq!(health(&read(2)), injured, "a passed Do cue must not hit again after load; inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("Connor: Do (conaxe.anm/attack): hits skeleton1").count(), 0);
}
