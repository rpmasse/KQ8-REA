//! An unfinished combat death keeps its pose and immunity to later hits after disk load.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; isolated fatal-hit setup"]
fn combat_deaths_resume_once_and_ignore_further_hits_after_loading() {
    check_deaths(&[
        ("fire-final", 2, 25, "firedie", None), ("fire-drop", 2, 13, "dropweapon", None),
        ("ice-final", 3, 25, "icedie", None), ("ice-drop", 3, 13, "dropweapon", None),
        ("normal", 0, 13, "die", None), ("projectile", 1, 13, "die", None),
    ]);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; fatal hits during equipped attacks"]
fn death_replaces_waiting_melee_and_ranged_actions_before_a_disk_save() {
    check_deaths(&[
        ("axe-attack", 0, 43, "die", Some(("INVITEM_SmallAx", "--swing", "skeleton1,38", "melee"))),
        ("crossbow-attack", 0, 43, "die", Some(("INVITEM_SmCrossbow", "--aim", "skeleton1,38", "ranged"))),
    ]);
}

type DeathCase = (&'static str, u8, u64, &'static str, Option<(&'static str, &'static str, &'static str, &'static str)>);

fn check_deaths(cases: &[DeathCase]) {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-death-checkpoint/{nonce}"));
    for &(case, kind, save_at, movement, attack) in cases {
        let case_root = root.join(case);
        let slots = case_root.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = case_root.join(part); fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), output.stderr).unwrap();
            let logs = dir.join("kq8-artifacts/logs");
            let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            (viewer, fs::read_to_string(logs.join("connor.log")).unwrap(), fs::read_to_string(logs.join("fight.log")).unwrap())
        };
        let hit_at = if attack.is_some() { 40 } else { 10 };
        let hit = format!("sendEvent 0 Connor KQAttackEvent trap {kind} 100000 1,{hit_at}");
        let key = format!("menu:down:down:enter:release:enter:release:escape:type=Dying:enter:release:down:down:down:enter:release:left:enter:release,{save_at}");
        let mut args = vec!["--world", "DeadCity", "--frames", "600", "--command", &hit, "--key", &key];
        if let Some((item, flag, action, kind)) = attack {
            let ready = if kind == "melee" { "KQConner::setDoMode 0,5" } else { "KQConner::setDoMode 1,5" };
            args.extend(["--item", item, "--command", ready, "--command", "move skeleton1 0 183300 45000 864 0 0 0,5", flag, action]);
        }
        let before = run("save", &args);
        assert_eq!(before.1.matches("dies (").count(), 1, "inspect {}", case_root.display());
        if let Some((_, _, _, kind)) = attack {
            assert!(before.1.contains(&format!("dying: cancels unfinished {kind} attack")), "the pre-death action must exist; inspect {}", case_root.display());
        }
        assert!(slots.join("slot-1.json").exists(), "death could not be saved; inspect {}", case_root.display());
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
        let saved_movement = saved["special"]["movement"].as_str().unwrap();
        assert!(saved_movement.eq_ignore_ascii_case(movement) || saved_movement.to_ascii_lowercase().ends_with(&format!("/{movement}")), "inspect {}", case_root.display());
        assert_eq!(saved["journey"]["stats"]["stats"][0], 0, "original dying entry zeros health once (Q194)");
        assert_eq!(saved["connor_dying"], true);
        assert!(saved["swing"].is_null() && saved["shot"].is_null(), "the old attack must not own the saved pose");
        let phase = saved["special"]["phase"].as_f64().unwrap();
        assert!(phase > 0. && phase < 1., "the slot must be inside the requested movement");
        if case == "fire-drop" {
            // An older version-2 slot has the same death list but no flag.
            let mut legacy = saved.clone(); legacy.as_object_mut().unwrap().remove("connor_dying");
            fs::write(slots.join("slot-1.json"), serde_json::to_vec(&legacy).unwrap()).unwrap();
        }
        let after = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1100",
            "--command", "sendEvent 0 Connor KQAttackEvent later 0 1000 1,2"]);
        assert!(after.0.contains("save restored in DeadCity without replaying Begin"));
        assert_eq!(after.1.matches("dies (").count(), 0, "a hit restarted saved death; inspect {}", case_root.display());
        assert_eq!(after.2.matches("hit by later: ignored, he is dying (rule 29)").count(), 1,
            "the restored death must ignore further damage; inspect {}", case_root.display());
        let first: Vec<_> = after.1.lines().find(|line| line.starts_with("1 ")).unwrap().split_whitespace().collect();
        let mode = first.iter().position(|field| *field == "special").unwrap();
        assert_eq!(first[mode + 1], saved_movement);
        let next_phase: f64 = first[mode + 3].parse().unwrap();
        assert!(next_phase > phase && next_phase <= phase + 0.201, "the saved pose must advance, not restart; inspect {}", case_root.display());
        assert!(!after.1.contains("cue 'playsound c_dropit.aud'"), "the earlier drop cue must not replay");
        assert_eq!(after.1.matches("special: KillConnor: his Die state").count(), 1, "inspect {}", case_root.display());
        let (_, tail) = after.0.split_once("save restored in DeadCity without replaying Begin").unwrap();
        assert_eq!(tail.matches("menu: shown Choice { game: false, row: 0").count(), 1, "inspect {}", case_root.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
