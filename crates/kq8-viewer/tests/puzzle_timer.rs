//! Resume the original Dead City laser controller after its first cue.
//! The file is normally loaded by laser2.cs. Its own Init, callbacks,
//! delays and target doors all come from the installed game data.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_laser_sequence_resumes_at_its_saved_deadline() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/timer-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (
            viewer,
            fs::read_to_string(logs.join("doors.log"))
                .unwrap()
                .to_ascii_lowercase(),
        )
    };

    let (save_log, _) = run(
        "save",
        &[
            "--world",
            "DeadCity",
            "--frames",
            "380",
            "--console",
            "loadKQ laserctr.kq,3",
            "--key",
            "menu:down:down:enter:release:enter:release:escape:type=Laser:enter:release:escape,56",
        ],
    );
    assert!(
        save_log.contains("Save screen -> Save(1, \"Laser\")"),
        "inspect {}",
        root.display()
    );
    let snapshot: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    assert_eq!(snapshot["version"], 2);
    let waiting = snapshot["script_events"]["waiting"].as_array().unwrap();
    let laser: Vec<_> = waiting
        .iter()
        .filter(|event| {
            event["target"]
                .as_str()
                .is_some_and(|t| t.eq_ignore_ascii_case("laserctr"))
        })
        .collect();
    assert_eq!(laser.len(), 1, "only the next cue should remain: {laser:?}");
    assert_eq!(laser[0]["kind"], "KQCueEvent");
    assert_eq!(laser[0]["data"][0], "2");
    let remaining = laser[0]["seconds_remaining"].as_f64().unwrap();
    assert!(remaining > 0.0 && remaining < 1.0);
    let (load_log, doors) = run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--frames",
            "680",
        ],
    );
    assert!(load_log.contains("save restored in DeadCity without replaying Begin"));
    let cue_ticks = |cue: u8| -> Vec<u64> {
        let call = format!(": script laserctr.cs laserctr cue {cue} 0");
        doors
            .lines()
            .filter_map(|s| {
                let (update, _) = s.split_once(&call)?;
                update.strip_prefix("update ")?.parse().ok()
            })
            .collect()
    };
    // Active advances its clock exactly once per 1/15 second update.
    // Keep the queue due comparison's tolerance on the boundary.
    let first_tick = ((remaining - 1e-9) * kq8_core::UPDATES_PER_SECOND).ceil() as u64;
    assert!(
        cue_ticks(1).is_empty(),
        "the rebuilt object's Init must not post its old cue twice"
    );
    assert_eq!(cue_ticks(2), [first_tick], "inspect {}", root.display());
    assert_eq!(cue_ticks(3), [first_tick + 15]);
    assert_eq!(cue_ticks(4), [first_tick + 60]);
    assert_eq!(cue_ticks(5), [first_tick + 75]);
    for door in ["ltddoor2", "rtddoor2"] {
        assert!(
            doors.contains(&format!("kqdoorevent 1 to {door}")),
            "missing {door} unlock"
        );
    }
}
