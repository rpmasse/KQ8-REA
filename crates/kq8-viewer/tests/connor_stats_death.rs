//! Stats-event death follows Connor's remembered weapon kind after loading.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn connor_stats_death_plays_normal_and_saved_fire_sequences_before_game_over() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-stats-death/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("connor.log")).unwrap(), fs::read_to_string(logs.join("doors.log")).unwrap())
    };
    let normal = run("normal", &["--world", "DeadCity", "--frames", "900",
        "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent -100000 0 0 0 0,10"]);
    let check = |logs: &(String, String, String), kind: &str, sequence: &str| {
        let marker = format!("dies ({kind}): plays {sequence}");
        assert_eq!(logs.1.matches(&marker).count(), 1, "missing/repeated death sequence; inspect {}", root.display());
        assert!(!logs.1.contains("not in his list: skipped"));
        assert!(logs.2.contains(&format!("dying state, last hit {kind}")));
        // A fresh-process load first shows the title menu. Count the
        // death menu only after that saved world has been restored.
        let menu_log = logs.0.split_once("save restored in DeadCity without replaying Begin").map_or(logs.0.as_str(), |(_, tail)| tail);
        assert_eq!(menu_log.matches("menu: shown Choice { game: false, row: 0").count(), 1,
            "the completed death must open the main menu once; inspect {}", root.display());
        assert!(logs.1.contains(" finished: Die state"), "the death movement did not finish; inspect {}", root.display());
    };
    check(&normal, "Normal", "[\"die\"]");
    run("save-fire-history", &["--world", "DeadCity", "--frames", "1500",
        "--command", "KQConner::setLevel 8,1",
        "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 80 0 0 0 0,2",
        "--command", "sendEvent 0 Connor KQAttackEvent trap 2 1 1,15",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=FireHistory:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    assert_eq!(saved["journey"]["last_hit_kind"], "Fire");
    assert!(saved["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    let fire = run("restore-and-die", &["--menu-input", "down,enter,release,enter,release", "--frames", "1100",
        "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent -100000 0 0 0 0,5"]);
    assert!(fire.0.contains("save restored in DeadCity without replaying Begin"));
    check(&fire, "Fire", "[\"dropweapon\", \"firedie\"]");
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
