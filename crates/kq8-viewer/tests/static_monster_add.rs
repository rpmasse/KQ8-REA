//! A type used by Daventry's placed monsters can also be added at runtime.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn placed_goblin_type_can_create_a_second_live_actor() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/static-monster-add/{nonce}"));
    fs::create_dir_all(&dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--windowed", "--sound", "off", "--no-ai", "--world", "Daventry",
            "--at", "289284,190000,0", "--key", "popup:autohelp,0",
            "--command", "KQMonster::add testGoblin goblin2 @ @ @ @ @ 289284 192527 -1 0 @ @ @ @ @ @ @ @ @,10",
            "--updates", "45", "--frames", "3000",
        ])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(doors.contains("KQMonster::add testGoblin type goblin2")
        && doors.contains("live character, AI and fight created"),
        "the placed monster's type did not make a second live actor; inspect {}", dir.display());
    assert!(doors.contains("copy testGoblin of goblin2.kq: drawn, 10 movement(s), 10 part entities"),
        "the second actor's original model was not drawn; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
