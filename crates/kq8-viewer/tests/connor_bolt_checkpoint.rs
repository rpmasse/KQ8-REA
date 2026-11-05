//! A crossbow bolt saved in the air must continue its flight in a new process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn crossbow_bolt_in_flight_hits_once_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-bolt-tests/{nonce}"));
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
        "--command", "move skeleton1 0 184500 43091 864 0 0 0,5",
        "--command", "KQConner::setDoMode 1,25",
        "--aim-point", "184000,44000,1800,50", "--frames", "105",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Flight:enter:release:escape,61",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Flight\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let bolts = first["bolts"]["bolts"].as_array().unwrap();
    assert_eq!(bolts.len(), 1, "the bolt must still be in the air at save; inspect {}", root.display());
    assert_eq!(bolts[0]["name"], "bolt1");
    assert_eq!(bolts[0]["shooter"], "Connor");
    assert_eq!(bolts[0]["updates"], 2);
    assert!(bolts[0]["travelled"].as_f64().is_some_and(|n| (n - 800.0).abs() < 1.0));
    let health = |slot: &serde_json::Value| -> i64 {
        slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
            .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap()
    };
    assert_eq!(health(&first), 60);

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterFlight:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterFlight\")"), "inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert!(fight.contains("bolt1 update 3: at (") && fight.contains("bolt1 update 4: touches skeleton1"),
        "the bolt did not continue from its saved position; inspect {}", root.display());
    assert_eq!(fight.matches("skeleton1: hit by Connor, Projectile:").count(), 1);
    assert!(!fight.contains("shoots bolt"), "the attack cue fired again; inspect {}", root.display());
    let second = read(2);
    assert_eq!(health(&second), 47);
    assert_eq!(second["bolts"]["bolts"].as_array().unwrap().len(), 0);
    assert_eq!(second["bolts"]["bolts_shot"], 1);
}
