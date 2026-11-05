//! The moving-shot delay and scheduled run return survive a new process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn moving_crossbow_cooldown_resumes_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/moving-shot-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
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
        "--world", "DeadCity", "--item", "INVITEM_SmCrossbow",
        "--command", "KQConner::setDoMode 1,25",
        "--hold", "1,0,1,45,100", "--aim-point", "182000,55000,1800,50",
        "--frames", "110",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Moving:enter:release:escape,52",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Moving\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let timing = &first["shot_timing"];
    assert_eq!(timing["run"], false);
    assert!(timing["next_shot_in"].as_f64().is_some_and(|n| (n - 0.366667).abs() < 0.001));
    assert!(timing["run_back_in"].as_f64().is_some_and(|n| (n - 0.866667).abs() < 0.001));
    assert_eq!(first["bolts"]["bolts_shot"], 1);

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--hold", "1,0,0,0,65",
        "--aim-point", "182000,55000,1800,2",
        "--aim-point", "182000,55000,1800,8",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterMoving:enter:release:escape,30",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterMoving\")"), "inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("moving: the last shot was less than 0.5 s ago").count(), 1,
        "the shot during the saved cooldown was accepted; inspect {}", root.display());
    assert_eq!(fight.matches("shoots bolt2 (Prscbolt.kq").count(), 1,
        "the later moving shot did not fire once; inspect {}", root.display());
    assert!(fight.contains("update 23: Connor: 1 s after the last moving shot: the run is switched on again"),
        "the run did not return after the second shot; inspect {}", root.display());
    let second = read(2);
    assert_eq!(second["bolts"]["bolts_shot"], 2);
    assert_eq!(second["shot_timing"]["run"], true);
    assert!(second["shot_timing"]["run_back_in"].is_null());
}
