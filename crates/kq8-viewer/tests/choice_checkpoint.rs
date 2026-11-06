//! The original roof choice must still dispatch its selected branch after a fresh launch.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; isolated roof-choice setup"]
fn original_roof_choice_keeps_both_directions_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/choice-checkpoint/{nonce}"));
    for (name, forward, number) in [("away", -1, "1"),("spriggan", 1, "0")] {
        let case = root.join(name); let slots = case.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = case.join(part); fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound","off","--no-ai","--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), output.stderr).unwrap();
            let logs = dir.join("kq8-artifacts/logs");
            let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            (viewer,fs::read_to_string(logs.join("doors.log")).unwrap(),fs::read_to_string(logs.join("connor.log")).unwrap())
        };
        let before = run("save", &["--world","Daventry","--frames","1500",
            "--command","move Connor 0 228700 183200 12350 0 0 -1.57,5",
            "--key","menu:down:down:enter:release:enter:release:escape:type=RoofChoice:enter:release:down:down:down:enter:release:left:enter:release,35"]);
        assert_eq!(before.1.matches("script rooftrap.cs Connor StartChoice").count(), 1, "inspect {}", case.display());
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
        assert_eq!(saved["choice"]["state"]["made"], false);
        assert_eq!(saved["choice"]["state"]["circle"].as_array().unwrap().len(), 2);
        assert!(saved["choice"]["calls"].as_array().unwrap().is_empty());
        // Input is sampled per rendered frame; span several fixed ticks.
        // The made flag must still deliver only one callback while held.
        let hold = format!("{forward},0,0,10,16");
        let after = run("restore", &["--menu-input","down,enter,release,enter,release","--frames","800","--hold",&hold]);
        assert!(after.2.contains("Choice restored: rooftrap.cs, made false, cached 2 points, 0 pending call(s)"));
        let (before_dispatch, _) = after.1.split_once(&format!("script rooftrap.cs Connor DoChoice {number}")).expect("the saved choice must dispatch");
        assert_eq!(before_dispatch.matches("script rooftrap.cs Connor StartChoice").count(), 0, "load must not repeat entry");
        let before_pick = after.2.split_once(" key: chooses ").unwrap().0;
        assert!(!before_pick.contains("Choice points measured from"), "load must retain the previously measured circle");
        assert_eq!(after.1.matches(&format!("script rooftrap.cs Connor DoChoice {number}")).count(), 1, "inspect {}", case.display());
        assert!(after.2.contains("leaves the Choice state"), "the selected script must change Connor's state");
        if number == "1" { assert!(after.1.contains("rooftrap.cs line 32:") && after.1.contains("walkTo")); }
        else { assert!(after.1.to_ascii_lowercase().contains("script sprgnbow.cs sprgnbow cue 10")); }
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; scripted Choice turn"]
fn scripted_turn_in_choice_keeps_its_pose_after_loading() {
    check_actions(&[("turn",40,None)]);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; supplied pending-call checkpoint"]
fn supplied_pending_choice_call_is_delivered_once_after_loading() {
    check_actions(&[("queued",35,None)]);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; chosen state with deferred NPC cue"]
fn chosen_roof_state_keeps_its_queued_spriggan_cue() {
    check_actions(&[("chosen",35,None)]);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; nested Choice weapon swaps"]
fn choice_weapon_swaps_return_to_one_fresh_entry_after_loading() {
    check_actions(&[("away",41,Some("away")),("out",48,Some("conaxe.anm/out"))]);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; scripted one-shot in Choice"]
fn choice_movement_end_repeats_entry_without_remeasuring_after_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/choice-movement-end/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part); fs::create_dir_all(&dir).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound","off","--no-ai","--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), result.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(result.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (fs::read_to_string(logs.join("doors.log")).unwrap(),fs::read_to_string(logs.join("connor.log")).unwrap())
    };
    let read = |slot| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{slot}.json"))).unwrap()).unwrap()
    };
    let quit = "down:down:down:enter:release:left:enter:release";
    let save = format!("menu:down:down:enter:release:enter:release:escape:type=Gesture:enter:release:{quit},40");
    let before = run("save", &["--world","Daventry","--frames","1500",
        "--command","move Connor 0 228700 183200 12350 0 0 -1.57,5",
        "--command","sendEvent 0 Connor KQAnimPlayEvent gesturea,35","--key",&save]);
    let first = read(1);
    assert_eq!(before.0.matches("script rooftrap.cs Connor StartChoice").count(), 1);
    assert_eq!(first["choice"]["pose"]["movement"], "gesturea");
    let phase = first["choice"]["pose"]["phase"].as_f64().unwrap();
    assert!(phase > 0. && phase < 1., "the slot must precede animation end; inspect {}", root.display());
    let save = format!("menu:down:down:down:enter:release:enter:release:escape:type=FinishedGesture:enter:release:{quit},100");
    let after = run("finish-and-save", &["--menu-input","down,enter,release,enter,release","--frames","1800","--key",&save]);
    assert_eq!(after.0.matches("script rooftrap.cs Connor StartChoice").count(), 1, "one-shot end must notify Choice exactly once; inspect {}", root.display());
    assert!(!after.1.contains("Choice points measured"), "movement end must keep the cache");
    let second = read(2);
    assert_eq!(second["choice"]["state"], first["choice"]["state"]);
    assert!(second["choice"]["calls"].as_array().unwrap().is_empty());
    let final_run = run("choose", &["--menu-input","down,enter,release,down,enter,release","--frames","800","--hold","-1,0,0,10,16"]);
    assert_eq!(final_run.0.matches("script rooftrap.cs Connor StartChoice").count(), 0);
    assert_eq!(final_run.0.matches("script rooftrap.cs Connor DoChoice 1").count(), 1);
    assert!(!final_run.1.contains("Choice points measured"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

fn check_actions(cases: &[(&str,u64,Option<&str>)]) {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/choice-actions/{nonce}"));
    for &(name, at, special) in cases {
        let case = root.join(name); let slots = case.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = case.join(part); fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound","off","--no-ai","--god"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), output.stderr).unwrap();
            let logs = dir.join("kq8-artifacts/logs");
            let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            (viewer,fs::read_to_string(logs.join("doors.log")).unwrap(),fs::read_to_string(logs.join("connor.log")).unwrap())
        };
        let key = format!("menu:down:down:enter:release:enter:release:escape:type=ChoiceAction:enter:release:down:down:down:enter:release:left:enter:release,{at}");
        let mut args = vec!["--world","Daventry","--frames","1500","--command","move Connor 0 228700 183200 12350 0 0 -1.57,5","--key",&key];
        if name == "turn" { args.extend(["--command","pointAndCue 230000 183200 12350 rooftrap 0 0,35"]); }
        if name == "chosen" { args.extend(["--hold","1,0,0,34,40"]); }
        if special.is_some() { args.extend(["--command","KQMonster::inventory Connor INVITEM_SmallAx 1,32","--command","KQConner::setDoMode 0,38"]); }
        let before = run("save", &args);
        assert_eq!(before.1.matches("script rooftrap.cs Connor StartChoice").count(), 1);
        let mut saved: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
        assert_eq!(saved["choice"]["state"]["made"], name == "chosen", "inspect {}", case.display());
        assert!(saved["choice"]["calls"].as_array().unwrap().is_empty());
        if name == "chosen" {
            // The last tick's textual script log flushes at the next world
            // service. The saved cue itself proves this branch has run.
            assert_eq!(saved["script_events"]["waiting"].as_array().unwrap().iter().filter(|event|
                event["target"].as_str().is_some_and(|n| n.eq_ignore_ascii_case("sprgnbow"))
                && event["kind"] == "KQCueEvent" && event["data"][0] == "10"
                && event["from"] == "rooftrap.cs line 27").count(), 1, "the original NPC cue must still be pending; inspect {}", case.display());
        }
        if name == "queued" {
            // The normal UI cannot stop between the chained controller and
            // callback systems. Supply that internal checkpoint explicitly;
            // this checks restoration, not ordinary menu capture of it.
            saved["choice"]["state"]["made"] = true.into();
            saved["choice"]["calls"] = serde_json::json!([["Connor","DoChoice","1"]]);
            fs::write(slots.join("slot-1.json"),serde_json::to_vec(&saved).unwrap()).unwrap();
        }
        let pose = if let Some(movement) = special {
            assert!(saved["choice"]["pose"].is_null());
            assert_eq!(saved["special"]["movement"], movement);
            assert_eq!(saved["weapon_state"]["readying"], true);
            &saved["special"]
        } else { &saved["choice"]["pose"] };
        if name == "turn" {
            assert!(!pose["auto"].is_null());
            assert!(matches!(pose["movement"].as_str(),Some("left"|"right")));
        }
        let phase = pose["phase"].as_f64().unwrap();
        let mut after_args = vec!["--menu-input","down,enter,release,enter,release","--frames","800"];
        if name == "turn" { after_args.extend(["--hold","1,0,0,30,36"]); }
        if special.is_some() { after_args.extend(["--hold","-1,0,0,30,36"]); }
        let after = run("restore", &after_args);
        if name == "chosen" {
            assert_eq!(after.1.matches("script rooftrap.cs Connor DoChoice").count(), 0);
            assert_eq!(after.1.to_ascii_lowercase().matches("script sprgnbow.cs sprgnbow cue 10").count(), 1);
            assert!(after.2.contains("Choice restored: rooftrap.cs, made true"));
        } else {
            assert_eq!(after.1.matches("script rooftrap.cs Connor DoChoice 1").count(), 1, "inspect {}", case.display());
            assert_eq!(after.1.matches("script rooftrap.cs Connor StartChoice").count(), usize::from(special.is_some()), "entry must only run after a swap; inspect {}", case.display());
        }
        if name == "turn" || special.is_some() {
            let fields: Vec<_> = after.2.lines().find(|line| line.starts_with("1 ")).unwrap().split_whitespace().collect();
            let mode = fields.iter().position(|field| *field == if special.is_some() { "special" } else { "free" }).unwrap();
            assert_eq!(fields[mode + 1], pose["movement"].as_str().unwrap());
            let first: f64 = fields[mode + 3].parse().unwrap();
            assert!(first > phase && first < phase + 0.25, "pose restarted instead of advancing; inspect {}", case.display());
        }
        if special.is_some() {
            assert_eq!(after.2.matches("Choice points measured from (228239, 183179)").count(), 1);
            assert_eq!(after.2.matches("weapon: the swap is over: back in a fresh Choice").count(), 1);
        }
        assert!(after.2.contains("leaves the Choice state"));
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
