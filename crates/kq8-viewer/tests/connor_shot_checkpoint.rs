//! A GOG crossbow shot saved at each side of its release cue must resume once.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn crossbow_turn_and_attack_resume_from_disk_without_duplicate_bolts() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-shot-tests/{nonce}"));
    for (name, trigger, waiting, before_do) in [
        ("turn", 53, true, true),
        ("attack-before-do", 54, false, true),
        ("attack-after-do", 57, false, false),
    ] {
        let case = root.join(name);
        let slots = case.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = case.join(part);
            fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
            let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            viewer
        };
        let save_key = format!("menu:down:down:enter:release:enter:release:escape:type=Crossbow:enter:release:escape,{trigger}");
        let before = run("save", &[
            "--world", "DeadCity", "--item", "INVITEM_SmCrossbow",
            "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
            "--command", "KQConner::setDoMode 1,25",
            "--aim", "skeleton1,50", "--frames", "120", "--key", &save_key,
        ]);
        assert!(before.contains("Save screen -> Save(1, \"Crossbow\")"), "inspect {}", case.display());
        let read = |n| -> serde_json::Value {
            serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
        };
        let first = read(1);
        let shot = &first["shot"];
        assert_eq!(shot["movement"], "conscbow.anm/attack", "inspect {}", case.display());
        assert_eq!(shot["waiting"], waiting, "inspect {}", case.display());
        assert_eq!(shot["started"], !waiting, "inspect {}", case.display());
        let phase = shot["phase"].as_f64().unwrap();
        if !waiting {
            assert_eq!(phase > 0.25, !before_do, "shot saved on the wrong side of Do; inspect {}", case.display());
        }
        let health = |slot: &serde_json::Value| -> i64 {
            slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
                .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap()
        };
        let first_health = health(&first);
        assert_eq!(first_health == 60, before_do, "the saved health is on the wrong side of the hit; inspect {}", case.display());

        let after = run("restore", &[
            "--menu-input", "down,enter,release,enter,release", "--frames", "700",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterShot:enter:release:escape,25",
        ]);
        assert!(after.contains("save restored in DeadCity without replaying Begin")
            && after.contains("Save screen -> Save(2, \"AfterShot\")"), "inspect {}", case.display());
        let fight = fs::read_to_string(case.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
        let releases = fight.matches("Connor: do (conscbow.anm/attack").count();
        assert_eq!(releases, usize::from(before_do), "the release cue fired the wrong number of times; inspect {}", case.display());
        let restored_health = health(&read(2));
        if before_do {
            assert!((1..60).contains(&restored_health), "the restored bolt missed the skeleton; inspect {}", case.display());
        } else {
            assert_eq!(restored_health, first_health, "a passed Do cue hit twice; inspect {}", case.display());
        }
    }
}
