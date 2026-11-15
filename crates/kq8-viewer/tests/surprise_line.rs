//! An unseen surviving hit on an original Dead City skeleton requests its authored line.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn dead_city_skeleton_uses_its_authored_unseen_hit_line() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/surprise-line/{nonce}"));
    fs::create_dir_all(&root).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--no-ai",
            "--god",
            "--world",
            "DeadCity",
        ])
        .args([
            "--command",
            "KQConner::setDoMode 0,5",
            "--command",
            "move skeleton1 0 183300 45000 864 0 0 0,31",
            "--swing",
            "skeleton1,32",
            "--updates",
            "70",
        ])
        .output()
        .unwrap();
    fs::write(root.join("stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("stderr.log"), &output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        root.display()
    );
    assert!(
        fight.contains("skeleton1: hit by Connor")
            && fight.contains("unseen hit surprise: selected 2000 65 0 98"),
        "the surviving hit did not request the authored line; inspect {}",
        root.display()
    );
    assert!(
        talk.lines()
            .any(|line| line.contains("skeleton1Talk") && line.contains("queued 2000 65 0 98")),
        "the line did not reach the ordinary speech queue; inspect {}",
        root.display()
    );
    assert!(
        talk.lines().any(|line| line.contains("speaker skeleton1")
            && line.contains("voice data/DeadCity/English")),
        "the original spoken line did not start; inspect {}",
        root.display()
    );
    assert!(
        characters
            .lines()
            .find(|line| line.contains("skeleton1: sight (rule 20)"))
            .is_some_and(|line| line.starts_with("update 45:")),
        "the fixture took a sight sample before its update-39 hit; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
