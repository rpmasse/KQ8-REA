//! The original Dead City tower actor leaves visible PreBirth after its alarm.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_tower_skeleton_rises_from_visible_prebirth() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/tower-prebirth-rise/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--windowed", "--sound", "off", "--world", "DeadCity", "--god",
            "--command", "move Connor 0 185481 122000 9814 0 0 3.14,100",
            "--command", "sendEvent 0 Skeltowr KQCueEvent 201,101",
            "--updates", "200", "--frames", "1450",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("Skeltowr: tower Cue 201: enabled its real alarm radius")
        && characters.contains("Skeltowr: start-up")
        && characters.contains("visible PreBirth loop until it rises")
        && characters.contains("Skeltowr: rises (rule 30): Connor")
        && characters.contains("'birth' starts at once")
        && characters.contains("Skeltowr: 'birth' ended: 'walk' starts")
        && characters.contains("Skeltowr: status: Move, active"),
        "the original tower actor did not rise and begin following Connor; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
