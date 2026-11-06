//! Original falling boards: named copies, retained playback and one wreck per completion.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_named_board_effects_resume_once_and_their_wrecks_survive_another_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/ordinary-once-checkpoint/{nonce}"));
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
    let copies = |save: &serde_json::Value| save["kept"]["daventry"]["object_copies"].as_array().unwrap().clone();
    // Original effects have no health line. These positive stats are test
    // fixtures so a nonfatal hit can establish history before natural completion.
    let first = run("save-effects", &["--world", "Daventry", "--frames", "1700",
        "--console", "loadKQ boardpx.kq OnceProbe 140000 205000 8000,5",
        "--console", "loadKQ boardpx.kq TwinProbe 141000 206000 8000,5",
        "--console", "pauseAnim TwinProbe,6",
        "--command", "sendEvent 0 OnceProbe KQStatsChangeRequestEvent 100 0 0 0 0,6",
        "--command", "sendEvent 0 TwinProbe KQStatsChangeRequestEvent 100 0 0 0 0,6",
        "--command", "sendEvent 0 OnceProbe KQAttackEvent test 2 1 0,7",
        "--command", "sendEvent 0 TwinProbe KQAttackEvent test 3 1 0,7",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=BoardEffects:enter:release:down:down:down:enter:release:left:enter:release,12"]);
    let saved = read(1);
    let instances = copies(&saved);
    assert_eq!(instances.len(), 2, "inspect {}", root.display());
    assert_eq!(instances[0]["name"], "OnceProbe"); assert_eq!(instances[1]["name"], "TwinProbe");
    assert!(instances.iter().all(|copy| copy["file"] == "boardpx.kq"));
    let animations = &saved["kept"]["daventry"]["interactions"]["animations"];
    let phase = animations["onceprobe"]["animation"]["phase"].as_f64().unwrap();
    assert!(phase > 0.0 && phase < 1.0);
    assert_eq!(animations["twinprobe"]["animation"]["paused"], true);
    assert_eq!(saved["kept"]["daventry"]["interactions"]["object_stats"]["onceprobe"]["stats"][0], 99);
    assert_eq!(saved["kept"]["daventry"]["interactions"]["object_stats"]["onceprobe"]["last_hit"], "Fire");
    assert_eq!(saved["kept"]["daventry"]["interactions"]["object_stats"]["twinprobe"]["last_hit"], "Ice");
    assert!(!first.contains("boardpx.cs OnceProbe Die") && !first.contains("boardpx.cs TwinProbe Die"));
    let second = run("resume-effects", &["--menu-input", "down,enter,release,enter,release", "--frames", "1900",
        "--console", "pauseAnim TwinProbe,5",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=BoardWrecks:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    assert_eq!(second.matches("script boardpx.cs OnceProbe Die Fire").count(), 1, "inspect {}", root.display());
    assert_eq!(second.matches("script boardpx.cs TwinProbe Die Ice").count(), 1, "inspect {}", root.display());
    let completion_update = |name: &str| -> u64 {
        second.lines().find(|line| line.contains(&format!("script boardpx.cs {name} Die"))).unwrap()
            .strip_prefix("update ").unwrap().split(':').next().unwrap().parse().unwrap()
    };
    // drbordie has 16 frames over one second. Update 0 is the first
    // advance after restore; the paused twin first advances at update 5.
    let remaining = |phase: f64| ((0.9999 - phase) * 15.0).ceil() as u64;
    assert_eq!(completion_update("OnceProbe"), remaining(phase) - 1);
    let twin_phase = animations["twinprobe"]["animation"]["phase"].as_f64().unwrap();
    assert_eq!(completion_update("TwinProbe"), 5 + remaining(twin_phase) - 1);
    assert!(!second.contains("boardpx.cs boxsmx Die"), "no additional default-name instance");
    let second_save = read(2);
    let wrecks = copies(&second_save);
    assert_eq!(wrecks.len(), 2);
    for (wreck, effect) in wrecks.iter().zip(&instances) {
        assert_eq!(wreck["file"], "drborded.kq");
        assert!(wreck["name"].as_str().unwrap().starts_with("NoName"));
        assert_eq!(wreck["placement"], effect["placement"]);
    }
    let third = run("reload-wrecks", &["--menu-input", "down,enter,release,down,enter,release", "--frames", "1900",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=KeptWrecks:enter:release:down:down:down:enter:release:left:enter:release,25"]);
    assert!(!third.contains("boardpx.cs OnceProbe Die") && !third.contains("boardpx.cs TwinProbe Die"));
    let third_save = read(3);
    assert_eq!(copies(&third_save), wrecks);
    assert_eq!(third_save["journey"]["next_no_name"], second_save["journey"]["next_no_name"], "reconstructing named copies must not allocate more NoName identities");
    let gone = third_save["kept"]["daventry"]["gone"].as_array().unwrap();
    assert!(gone.iter().any(|name| name == "onceprobe") && gone.iter().any(|name| name == "twinprobe"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
