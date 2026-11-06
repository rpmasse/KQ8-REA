//! The original bell-stand load gets fresh stats, including a reused bell name.
use std::{fs, path::Path, process::Command};
#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_bell_load_and_recreation_use_file_health_without_old_hit_history() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/copy-stats/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part); fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        fs::read_to_string(logs.join("doors.log")).unwrap()
    };
    let read = |n| -> serde_json::Value { serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap() };
    let first = run("hang-and-save", &["--world", "Daventry", "--item", "INVITEM_LadyBell", "--frames", "1700",
        "--command", "move Connor 0 147461 211000 7675 0 0 0,5", "--click", "belstnd1,15",
        "--command", "sendEvent 0 bell KQAttackEvent test 0 1 0,25",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=HungBell:enter:release:down:down:down:enter:release:left:enter:release,40"]);
    let saved = read(1);
    let combat = &saved["kept"]["daventry"]["interactions"]["object_stats"]["bell"];
    assert_eq!(combat["stats"][0], 0, "the original one-health bell must survive one point of damage; inspect {}", root.display());
    assert_eq!(combat["last_hit"], "Normal");
    assert!(first.contains("fresh stats for bell: [1, 0, 0, 0, 0]"));
    assert!(!first.contains("bell Die Normal"));
    let after = run("restore-and-recreate", &["--menu-input", "down,enter,release,enter,release", "--frames", "1900",
        "--command", "sendEvent 0 bell KQStatsChangeRequestEvent -1 0 0 0 0,5",
        "--console", "deleteObject bell,10", "--console", "loadKQ bell.kq,12",
        "--command", "sendEvent 0 bell KQAttackEvent test 3 1 0,20",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=NewBell:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    assert!(after.contains("fresh stats for bell: [1, 0, 0, 0, 0]; no previous hit kind"));
    assert_eq!(after.matches("script bell.cs bell Die Normal").count(), 1);
    assert!(!after.contains("bell Die Ice"));
    let second = read(2);
    let fresh = &second["kept"]["daventry"]["interactions"]["object_stats"]["bell"];
    assert_eq!(fresh["stats"][0], 0);
    assert_eq!(fresh["last_hit"], "Ice");
    assert!(!second["kept"]["daventry"]["gone"].as_array().unwrap().iter().any(|name| name == "bell"));
    let final_log = run("reload-new-bell", &["--menu-input", "down,enter,release,down,enter,release", "--frames", "1900",
        "--command", "sendEvent 0 bell KQStatsChangeRequestEvent -1 0 0 0 0,5",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=KeptBell:enter:release:down:down:down:enter:release:left:enter:release,25"]);
    assert_eq!(final_log.matches("script bell.cs bell Die Ice").count(), 1);
    assert_eq!(read(3)["kept"]["daventry"]["interactions"]["object_stats"]["bell"]["stats"][0], -1);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
