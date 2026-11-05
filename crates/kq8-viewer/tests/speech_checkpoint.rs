//! An installed voiced line resumes its phase and completes once after relaunch.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_speech_resumes_audio_caption_and_completion_after_relaunch() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/speech-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        fs::write(dir.join("kq8-artifacts/Options.cs"), b"set Options::Captions true\n").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
        fs::write(dir.join("stdout.log"), &result.stdout).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(result.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("talk.log")).expect("speech log missing; inspect the run's viewer.log"), fs::read_to_string(logs.join("sound.log")).unwrap())
    };
    let (before, _, _) = run("save", &["--world", "Daventry", "--frames", "190",
        "--command", "move Wizard 0 277744 198295 0 0 0 0,5",
        "--command", "KQMonster::speak Connor Connor 1000 4 0 8 1 1,12",
        "--command", "move Wizard 0 287744 198295 0 0 0 0,25",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=MidSpeech:enter:release:escape,45"]);
    assert!(before.contains("Save screen -> Save(1, \"MidSpeech\")"));
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let saved = read(1);
    let queue = saved["speech"]["queues"].as_array().unwrap().iter().find(|q| q["issuer"] == "Connor").unwrap();
    let line = &queue["playing"];
    let id = line["id"].as_u64().unwrap();
    let elapsed = -line["started"].as_f64().unwrap();
    let remaining = line["ends"].as_f64().unwrap();
    assert!(elapsed > 1.0 && remaining > 1.0 && line["voiced"] == true, "the save missed active original speech; inspect {}", root.display());
    let source = &saved["speech_sources"][id.to_string()];
    let source_at: Vec<_> = source.as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
    let wizard = saved["kept"]["daventry"]["characters"].as_array().unwrap().iter().find(|a| a["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case("Wizard"))).unwrap();
    assert!((wizard["animation"]["placement"]["position"][0].as_f64().unwrap() - source_at[0]).abs() > 5000.0,
        "the test did not separate voice origin from current speaker position; inspect {}", root.display());
    let (after, talk, sound) = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "2000",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=SpeechFinished:enter:release:escape,130"]);
    assert!(after.contains("save restored in Daventry without replaying Begin"));
    let expected = format!("line #{id}: resume 1000 4 0 8 1 voice at {elapsed:.3} seconds");
    assert!(talk.contains(&expected), "voice restarted or phase changed; inspect {}", root.display());
    assert!(sound.contains(&format!("to {elapsed:.3} seconds (speech checkpoint)")), "mixer clock did not seek; inspect {}", root.display());
    assert!(sound.contains(&format!("placed at ({:.0}, {:.0}, {:.0})", source_at[0], source_at[1], source_at[2])),
        "spatial audio moved to the speaker's new location; inspect {}", root.display());
    assert!(after.contains("caption picture shown:"), "caption was not restored; inspect {}", root.display());
    let finished = format!("line #{id} 1000 4 0 8 1 finished");
    assert_eq!(talk.matches(&finished).count(), 1, "completion repeated or disappeared; inspect {}", root.display());
    assert!(!talk.contains("1000 4 0 8 1 starts:"), "the restored line restarted; inspect {}", root.display());
    assert!(read(2)["speech"]["queues"].as_array().unwrap().iter().all(|q| q["playing"].is_null()));
}
