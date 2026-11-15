//! An original Dead City skeleton's unkeyed alarm selects and plays a built-in cry.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn dead_city_skeleton_plays_a_built_in_alarm_cry() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/built-in-alarm-cry/{nonce}"));
    fs::create_dir_all(&root).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root)
        .arg("--game")
        .arg(&game)
        .args(["--windowed", "--sound", "off", "--world", "DeadCity", "--at", "134186,92993,0", "--god"])
        .args([
            "--command", "move skeleton4 0 134700 92993 864 0 0 0,5",
            "--command", "move skeleton5 0 134900 92993 864 0 0 0,5",
            "--command", "move skeleton6 0 135100 92993 864 0 0 0,5",
            "--updates", "100",
        ])
        .output()
        .unwrap();
    fs::write(root.join("stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("stderr.log"), &output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", root.display());
    assert!(characters.contains("skeleton5: built-in alarm cry (Q213): skeleton5Talk: queued 2000 65 0 89 5…5"),
        "the built-in line was not selected; inspect {}", root.display());
    assert!(characters.contains("skeleton4: alarm speech: shared deadline 4.00 has not passed")
        && characters.contains("skeleton6: alarm speech: shared deadline 4.00 has not passed"),
        "teammates did not share the speech deadline; inspect {}", root.display());
    assert!(talk.contains("skeleton5Talk: line #0 2000 65 0 89 5 starts: speaker skeleton5")
        && talk.contains("voice data/DeadCity/English/A1JK1T00.2H5"),
        "the original voice did not start; inspect {}", root.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
