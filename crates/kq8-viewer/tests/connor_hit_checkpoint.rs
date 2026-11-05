//! Connor's reaction pose and hit sound limits survive a fresh process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn connor_hit_timing_resumes_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-hit-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--command", "KQMonster::setReactPercent Connor 0,1",
        "--command", "KQConner::setLevel 8,1",
        "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 80 0 0 0 0,2",
        "--command", "sendEvent 0 Connor KQAttackEvent trap 0 2 1,15",
        "--frames", "70",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Hit:enter:release:escape,20",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Hit\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    assert_eq!(first["special"]["movement"], "react");
    let phase = first["special"]["phase"].as_f64().unwrap();
    assert!((phase - 1.0 / 3.0).abs() < 0.001, "saved hit pose changed; inspect {}", root.display());
    let hit = &first["hit_timing"];
    assert!(hit["flinch_in"].as_f64().is_some_and(|t| (2.0..=2.8).contains(&t)));
    assert!(hit["ouch_in"].as_f64().is_some_and(|t| (0.0..=1.0).contains(&t)));

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--command", "KQMonster::setReactPercent Connor 0,1",
        "--command", "sendEvent 0 Connor KQAttackEvent trap 0 2 1,2",
        "--command", "sendEvent 0 Connor KQAttackEvent trap 0 2 1,20",
        "--command", "sendEvent 0 Connor KQAttackEvent trap 0 2 1,60",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterHit:enter:release:escape,70",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterHit\")"), "inspect {}", root.display());
    let connor = fs::read_to_string(root.join("restore/kq8-artifacts/logs/connor.log")).unwrap();
    let fields: Vec<&str> = connor.lines().find(|line| line.starts_with("1 ")).unwrap().split_whitespace().collect();
    let mode = fields.iter().position(|field| *field == "special").unwrap();
    assert_eq!(fields[mode + 1], "react");
    let resumed_phase: f64 = fields[mode + 3].parse().unwrap();
    assert!((resumed_phase - (phase + 1.0 / 12.0)).abs() < 0.002,
        "Connor restarted react rather than continuing; inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert!(fight.contains("no flinch ('react' can't be cut short now (mode special)); no ouch sound")
        && fight.contains("no flinch (his last flinch ended")
        && fight.contains("his ouch sound conrow3.aud")
        && fight.contains("flinches: 'react'"), "hit limits did not resume; inspect {}", root.display());
    let second = read(2);
    assert_eq!(second["journey"]["stats"]["stats"][0], 76);
}
