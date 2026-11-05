//! Original Wizard approach, cut camera, dialogue and HUD across disk reloads.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_conversation_resumes_approach_and_talking_then_returns_control() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/conversation-checkpoint/{nonce}"));
    for (case, location, save_at, expected_stage) in [
        ("approach", "193000 200000 6700", "14", "StopWalk"),
        ("talking", "192171 198091 6700", "45", "Talking"),
    ] {
        let base = root.join(case);
        let slots = base.join("slots");
        let run = |name: &str, args: &[&str]| {
            let dir = base.join(name);
            fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
            fs::write(dir.join("kq8-artifacts/Options.cs"), b"set Options::Captions true\n").unwrap();
            let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
            fs::write(dir.join("stdout.log"), &result.stdout).unwrap();
            let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
            assert!(result.status.success() && log.contains("RESULT: PASS"), "inspect {}", dir.display());
            log
        };
        let quit = "down:down:down:enter:release:left:enter:release";
        let move_connor = format!("move Connor 0 {location} 0 0 0,5");
        let save = format!("menu:down:down:enter:release:enter:release:escape:type=C1:enter:release:{quit},{save_at}");
        run("save", &["--world", "Daventry", "--frames", "500",
            "--command", "set Daventry::Wizard::Clicked true,3", "--command", &move_connor,
            "--command", "setLowerGui 1,8", "--command", "KQConner::conversation Wizard 1000 4 0 8 1 1 0,12",
            "--key", &save]);
        let read = |n| -> serde_json::Value {
            serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap_or_else(|e| panic!("slot {n}: {e}; inspect {}", base.display()))).unwrap()
        };
        let first = read(1);
        assert_eq!(first["conversation"]["active"]["stage"], expected_stage, "inspect {}", base.display());
        assert!(!first["talk_pose"].is_null());
        assert_eq!(first["panels"]["shown_before_talk"], true);
        let final_save = format!("menu:down:down:down:enter:release:enter:release:escape:type=C3:enter:release:{quit},260");
        let log = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "3000",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=C2:enter:release:escape,3",
            "--hold", "1,0,0,220,240", "--key", &final_save]);
        assert!(log.contains("save restored in Daventry without replaying Begin"), "inspect {}", base.display());
        let second = read(2);
        if expected_stage == "Talking" {
            assert_eq!(second["conversation"]["active"]["stage"], expected_stage);
        } else {
            assert!(["StopWalk", "WalkStopRight", "Talking"].iter().any(|s| second["conversation"]["active"]["stage"] == *s));
        }
        assert_eq!(second["panels"]["shown_before_talk"], true);
        if second["conversation"]["active"]["stage"] == "StopWalk" {
            assert!(second["talk_pose"]["phase"].as_f64().unwrap() > first["talk_pose"]["phase"].as_f64().unwrap(), "approach phase restarted; inspect {}", base.display());
            assert!(second["conversation"]["active"]["approach_updates"].as_u64().unwrap() > first["conversation"]["active"]["approach_updates"].as_u64().unwrap());
        }
        let camera_log = fs::read_to_string(base.join("restore/kq8-artifacts/logs/camera.log")).unwrap();
        let p = first["camera"]["cam"]["position"].as_array().unwrap();
        assert!(camera_log.contains(&format!("camera checkpoint restored at ({:.3}, {:.3}, {:.3})",
            p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32, p[2].as_f64().unwrap() as f32)), "camera not restored; inspect {}", base.display());
        if expected_stage == "Talking" {
            assert_eq!(first["camera"]["following"], false);
            assert_eq!(second["camera"]["cam"]["position"], first["camera"]["cam"]["position"]);
            let queue = first["speech"]["queues"].as_array().unwrap().iter().find(|q| q["conv"] == true && q["issuer"] == "Wizard").unwrap();
            assert_eq!(queue["playing"]["camera"], 4);
        }
        let conversation = fs::read_to_string(base.join("restore/kq8-artifacts/logs/conversation.log")).unwrap();
        assert_eq!(conversation.matches("Wizard: TalkExit:").count(), 1, "exit repeated or lost; inspect {}", base.display());
        assert!(!conversation.contains("Connor enters his talk state:"), "entry replayed; inspect {}", base.display());
        assert_eq!(conversation.matches("stopwalk ended at").count(), usize::from(expected_stage != "Talking"), "approach completion repeated or lost; inspect {}", base.display());
        assert_eq!(conversation.matches("camera 4:").count(), usize::from(expected_stage != "Talking"), "camera cue repeated or lost; inspect {}", base.display());
        let third = read(3);
        assert!(third["conversation"]["active"].is_null() && third["conversation"]["leave"].is_null() && third["talk_pose"].is_null());
        assert_eq!(third["camera"]["following"], true);
        assert_eq!(third["panels"]["slides"][1]["shown"], true);
        let moved = third["position"].as_array().unwrap().iter().zip(second["position"].as_array().unwrap())
            .map(|(a, b)| (a.as_f64().unwrap() - b.as_f64().unwrap()).abs()).sum::<f64>();
        assert!(moved > 100.0, "movement control did not return; inspect {}", base.display());
    }
}
