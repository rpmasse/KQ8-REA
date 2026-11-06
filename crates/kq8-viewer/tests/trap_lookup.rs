//! Dead City's original plate Lookup callbacks after startup and disk restore.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn deadcity_plate_lookups_restore_sylph_from_saved_plate_flags_once() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/trap-lookup/{nonce}"));
    fn sylph(slot: &serde_json::Value) -> &serde_json::Value {
        &slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
            .find(|c| c["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case("Sylph"))).unwrap()["animation"]["placement"]["position"]
    }
    fn flag<'a>(slot: &'a serde_json::Value, key: &str) -> &'a str {
        slot["kept"]["deadcity"]["vars"].as_array().unwrap().iter().find(|row| row[0] == key).unwrap()[1].as_str().unwrap()
    }
    for (case, setting, stale, expected) in [("up", "up", "0 0 0", [70662.0, 105677.0]),
        ("down", "down", "70662 105677 1584", [0.0, 0.0])] {
        eprintln!("checking original plate Lookup: {case}");
        let base = root.join(case);
        let slots = base.join("slots");
        let run = |name: &str, args: &[&str]| {
            let dir = base.join(name);
            fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
            fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
            let logs = dir.join("kq8-artifacts/logs");
            let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            (viewer, fs::read_to_string(logs.join("traps.log")).unwrap())
        };
        let read = |n| -> serde_json::Value { serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json")))
            .unwrap_or_else(|e| panic!("{e}; inspect {}", base.display()))).unwrap() };
        let flags: Vec<_> = (12..=15).map(|i| format!("set DeadCity::Sylph::B{i} {setting},20")).collect();
        let place = format!("move Sylph 0 {stale} 0 0 0,25");
        let stale_flag = format!("set DeadCity::Sylph::isIn {},26", if case == "up" { "false" } else { "true" });
        let quit = "down:down:down:enter:release:left:enter:release";
        let save = format!("menu:down:down:enter:release:enter:release:escape:type=Plates:enter:release:{quit},35");
        let (_, startup) = run("save", &["--world", "DeadCity", "--frames", "1500", "--console", &flags[0],
            "--console", &flags[1], "--console", &flags[2], "--console", &flags[3], "--command", &place,
            "--console", &stale_flag, "--key", &save]);
        let first = read(1);
        for n in 12..=15 {
            assert_eq!(flag(&first, &format!("DeadCity::Sylph::B{n}")), setting);
            assert_eq!(startup.matches(&format!("script trptrp{n}.cs trptrp{n} Lookup")).count(), 1);
        }
        let finish = format!("menu:down:down:down:enter:release:enter:release:escape:type=LookedUp:enter:release:{quit},12");
        let (viewer, restored) = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1700", "--key", &finish]);
        assert!(viewer.contains("save restored in DeadCity without replaying Begin"));
        for n in 12..=15 {
            assert_eq!(restored.matches(&format!("script trptrp{n}.cs trptrp{n} Lookup")).count(), 1, "callback repeated or missing; inspect {}", base.display());
            for object in [format!("trptrp{n}"), format!("block{n}"), format!("trip{n}")] {
                assert!(restored.contains(&format!("makeActive {object} 1: always near")));
            }
        }
        let last = read(2);
        assert_eq!(flag(&last, "DeadCity::Sylph::isIn"), if case == "up" { "true" } else { "false" });
        for axis in 0..2 { assert!((sylph(&last)[axis].as_f64().unwrap() - expected[axis]).abs() < 1.0, "lookup cue did not place Sylph; inspect {}", base.display()); }
        assert_ne!(sylph(&first), sylph(&last), "the check did not change the stale placement");
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
