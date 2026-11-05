//! A disk slot resumes an original monster's unfinished death.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_skeleton_finishes_dying_after_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/death-checkpoint-tests/{nonce}"));
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
        "--swing", "skeleton1,20", "--swing", "skeleton1,50", "--swing", "skeleton1,80",
        "--frames", "300",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=DyingSkeleton:enter:release:escape,90",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"DyingSkeleton\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let dying = first["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
        .find(|actor| actor["name"].as_str().is_some_and(|name| name.eq_ignore_ascii_case("skeleton1"))).unwrap();
    assert!(!dying["character"]["fight"]["dying"].is_null(), "the first slot missed the death movement; inspect {}", root.display());
    assert!(!first["kept"]["deadcity"]["gone"].as_array().unwrap().iter().any(|n| n == "skeleton1"));
    let xp_before = first["journey"]["stats"]["stats"][4].as_i64().unwrap();

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=DeathFinished:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    let second = read(2);
    assert!(second["kept"]["deadcity"]["gone"].as_array().unwrap().iter().any(|n| n == "skeleton1"));
    assert!(!second["kept"]["deadcity"]["characters"].as_array().unwrap().iter().any(|a| a["name"] == "skeleton1"));
    let xp_after = second["journey"]["stats"]["stats"][4].as_i64().unwrap();
    assert_eq!(xp_after - xp_before, 200, "death must award this original skeleton's experience exactly once; inspect {}", root.display());
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn script_created_rat_finishes_dying_after_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/death-checkpoint-tests/{nonce}"));
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
        "--command", "KQMonster::add testRat rat,8",
        "--command", "move testRat 0 183300 45000 864 0 0 0,10",
        "--swing", "testRat,20", "--swing", "testRat,50",
        "--frames", "300",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=DyingRat:enter:release:escape,60",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"DyingRat\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let rat = first["kept"]["deadcity"]["spawned_monsters"].as_array().unwrap().iter()
        .find(|a| a["command"][0] == "testRat").unwrap();
    assert!(!rat["character"]["fight"]["dying"].is_null(), "the first slot missed the rat's death; inspect {}", root.display());
    let reward = rat["character"]["fight"]["stats"][4].as_i64().unwrap();
    let xp_before = first["journey"]["stats"]["stats"][4].as_i64().unwrap();

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=RatFinished:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    let second = read(2);
    assert!(second["kept"]["deadcity"]["gone"].as_array().unwrap().iter().any(|n| n == "testrat"));
    assert!(!second["kept"]["deadcity"]["spawned_monsters"].as_array().unwrap().iter().any(|a| a["command"][0] == "testRat"));
    let xp_after = second["journey"]["stats"]["stats"][4].as_i64().unwrap();
    assert_eq!(xp_after - xp_before, reward, "rat reward must be awarded once; inspect {}", root.display());
}
