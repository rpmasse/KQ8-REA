//! An original Dead City fighter keeps its combat state across two processes.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_skeleton_survives_save_reload_after_a_hit() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/placed-character-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("fight.log")).unwrap_or_default())
    };
    let (before, fight) = run("save", &[
        "--world", "DeadCity", "--item", "INVITEM_Dagger",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--swing", "skeleton1,20", "--swing", "skeleton1,50", "--frames", "300",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Skeleton:enter:release:escape,90",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Skeleton\")"), "inspect {}", root.display());
    assert!(fight.contains("skeleton1") && fight.contains("hit"), "Connor did not hit skeleton1; inspect {}", root.display());
    let read = |slot| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{slot}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let placed = |slot: &serde_json::Value| slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
        .find(|actor| actor["name"].as_str().is_some_and(|name| name.eq_ignore_ascii_case("skeleton1"))).unwrap().clone();
    let injured = placed(&first);
    let health = injured["character"]["fight"]["stats"][0].as_i64().unwrap();
    assert!((1..60).contains(&health), "expected an injured living skeleton, got {health}; inspect {}", root.display());
    assert_eq!(injured["animation"]["placement"]["position"], serde_json::json!([183300.0, 45000.0, 864.0]));
    let (after, _) = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=SkeletonAgain:enter:release:escape,24"]);
    assert!(after.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    let restored = placed(&read(2));
    assert_eq!(restored["character"]["fight"]["stats"], injured["character"]["fight"]["stats"]);
    // The skeleton may turn toward Connor during the 24 updates before the
    // second save; its position and injured health must still persist.
    assert_eq!(restored["animation"]["placement"]["position"], injured["animation"]["placement"]["position"]);
}
