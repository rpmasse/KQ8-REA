//! A persistent original door keeps its damaged health and hit-kind history.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn tomb_door_health_and_fire_kind_survive_reload_before_its_original_die_script() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/object-stats/{nonce}"));
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
    let before = run("save", &["--world", "Daventry", "--frames", "1600",
        "--command", "give SmallAx 1,5",
        "--command", "sendEvent 0 tombdor2 KQAttackEvent test 2 1 0,20",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=DamagedDoor:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    let first = read(1);
    let combat = &first["kept"]["daventry"]["interactions"]["object_stats"]["tombdor2"];
    assert_eq!(combat["stats"][0], 0, "one damage must leave the original one-health door at zero; inspect {}", root.display());
    assert_eq!(combat["last_hit"], "Fire");
    assert!(!before.contains("tombdor2 Die Fire"), "zero must not run Die yet");
    let after = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1800",
        "--command", "sendEvent 0 tombdor2 KQStatsChangeRequestEvent -1 0 0 0 0,5",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=BrokenDoor:enter:release:down:down:down:enter:release:left:enter:release,80"]);
    assert_eq!(after.matches("script tombdor2.cs tombdor2 Die Fire").count(), 1, "the saved Fire kind must reach the original script; inspect {}", root.display());
    let last = read(2);
    let interactions = &last["kept"]["daventry"]["interactions"];
    assert_eq!(interactions["object_stats"]["tombdor2"]["stats"][0], -1);
    assert_eq!(interactions["doors"]["tombdor2"]["state"], "Opened", "the original Die script must open the door; inspect {}", root.display());
    assert_eq!(interactions["doors"]["tombdor2"]["collision_type"].as_u64().unwrap() & 32, 0);
    assert_eq!(interactions["object_stats"]["tombdor2"]["last_hit"], "Fire");
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
