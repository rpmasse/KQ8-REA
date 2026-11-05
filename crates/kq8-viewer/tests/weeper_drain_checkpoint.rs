//! A post-impact Weeper drain must keep its pulse and beam state across processes.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn dead_city_weeper_drain_resumes_remaining_pulses_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/weeper-drain-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--at", "170920,149369,0",
        "--command", "KQConner::setLevel 8,1",
        "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 80 0 0 0 0,2",
        "--frames", "130",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Drain:enter:release:escape,49",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Drain\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let drain = &first["drains"][0];
    assert_eq!(drain["bolt"], "bolt1");
    assert_eq!(drain["shooter"], "weeper14");
    assert_eq!(drain["target"], "Connor");
    assert_eq!(drain["ticks"], 17);
    assert_eq!(drain["pulses"], 4);
    assert_eq!(drain["links"].as_array().unwrap().len(), 8);
    assert_eq!(first["journey"]["stats"]["stats"][0], 76);
    assert_eq!(first["special"]["movement"], "magicReact");
    let saved_phase = first["special"]["phase"].as_f64().unwrap();
    assert!((saved_phase - 0.3777778).abs() < 0.001, "the save lost Connor's current pose; inspect {}", root.display());

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterDrain:enter:release:escape,40",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterDrain\")"), "inspect {}", root.display());
    let logs = root.join("restore/kq8-artifacts/logs");
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    for pulse in 5..=12 {
        assert!(fight.contains(&format!("bolt1 drain pulse {pulse} on Connor:")),
            "pulse {pulse} missing; inspect {}", logs.display());
    }
    assert!(!fight.contains("bolt1 drain pulse 4 on Connor:"), "a saved pulse repeated; inspect {}", logs.display());
    assert_eq!(fight.matches("bolt1 ends drain after 12 pulses").count(), 1);
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert_eq!(doors.matches("copy WeepLink.weeper14.").count(), 7);
    assert_eq!(doors.matches("copy WeepEnd.weeper14 ").count(), 1);
    let sound = fs::read_to_string(logs.join("sound.log")).unwrap();
    assert!(sound.contains("LIFESUCK.AUD") && sound.contains("Weeper drain ended"),
        "the restored loop did not start and stop; inspect {}", logs.display());
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    let first_movement = connor.lines().find(|line| line.starts_with("1 ")).unwrap();
    let fields: Vec<&str> = first_movement.split_whitespace().collect();
    let mode = fields.iter().position(|field| *field == "special").unwrap();
    assert_eq!(fields[mode + 1], "magicReact");
    let first_phase: f64 = fields[mode + 3].parse().unwrap();
    assert!((first_phase - (saved_phase + 1.0 / 45.0)).abs() < 0.002,
        "Connor restarted magicReact rather than continuing; inspect {}", logs.display());
    for line in connor.lines().filter(|line| line.contains("cue 'playsound magicrct.aud'")) {
        let update: u64 = line.strip_prefix("connor update ").unwrap().split(':').next().unwrap().parse().unwrap();
        assert!(update > 28, "the first drain replayed an earlier animation cue; inspect {}", logs.display());
    }
    let second = read(2);
    assert_eq!(second["journey"]["stats"]["stats"][0], 68);
    assert!(second["drains"].as_array().unwrap().is_empty());

    // A slot from before the Special checkpoint still uses the older
    // animation restart path while preserving the drain's remaining pulses.
    let mut old = first;
    old.as_object_mut().unwrap().remove("special");
    fs::write(slots.join("slot-1.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    let legacy = run("legacy", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
    ]);
    assert!(legacy.contains("save restored in DeadCity without replaying Begin"), "inspect {}", root.display());
    let legacy_logs = root.join("legacy/kq8-artifacts/logs");
    let old_connor = fs::read_to_string(legacy_logs.join("connor.log")).unwrap();
    assert!(old_connor.contains("connor update 6: cue 'playsound magicrct.aud'"),
        "the old-slot restart path was lost; inspect {}", legacy_logs.display());
    let old_fight = fs::read_to_string(legacy_logs.join("fight.log")).unwrap();
    assert_eq!(old_fight.matches("bolt1 ends drain after 12 pulses").count(), 1);
}
