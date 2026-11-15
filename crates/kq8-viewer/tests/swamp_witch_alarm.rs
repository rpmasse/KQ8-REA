//! Unattended GOG-data route for the Swamp Witch's authored alarm dialogue.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_witch_alarm_plays_its_two_authored_lines_once() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/swamp-witch-alarm/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args(["--world", "Swamp", "--at", "159699,19000,0", "--god", "--sound", "off", "--windowed", "--updates", "80"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert_eq!(characters.matches("Swmpwich: alarm line (rule 23):").count(), 1, "inspect {}", dir.display());
    assert_eq!(talk.matches("SwmpwichTalk: queued 3000 4 0 24 1…2").count(), 1, "inspect {}", dir.display());
    let witch = talk.find("SwmpwichTalk: line #0 3000 4 0 24 1 starts: speaker Swmpwich")
        .unwrap_or_else(|| panic!("witch line: inspect {}", dir.display()));
    let reply = talk.find("SwmpwichTalk: line #1 3000 4 0 24 2 starts: speaker Connor")
        .unwrap_or_else(|| panic!("Connor reply: inspect {}", dir.display()));
    assert!(witch < reply, "wrong speech order: inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
