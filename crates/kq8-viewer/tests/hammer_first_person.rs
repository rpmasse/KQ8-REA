//! A first-person Azriel Hammer throw keeps Connor's heading while it hits and returns.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn first_person_hammer_throw_does_not_turn_connor() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/hammer-first-person/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "DeadCity",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--no-ai",
            "--god",
            "--command",
            "give AzrielHammer 1,5",
            "--command",
            "move skeleton1 0 184500 43091 864 0 0 0,5",
            "--command",
            "KQConner::setDoMode 1,25",
            "--key",
            "physical:f2,60",
            "--aim",
            "skeleton1,80",
            "--updates",
            "145",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(viewer.contains("CameraFirstPerson: first person true"));

    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        fight.contains("first-person aim: no turn (rule 2)"),
        "the throw started an auto-turn; inspect {}",
        dir.display()
    );
    assert_eq!(fight.matches("shoots hammer1 (Prhammer.kq").count(), 1);
    assert_eq!(
        fight
            .matches("skeleton1: hit by Connor, Projectile:")
            .count(),
        1
    );
    assert_eq!(fight.matches("hammer1: turns back (its touch)").count(), 1);
    assert!(fight.contains("hammer1: the way back ends"));

    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    let heading_at = |update: u32| -> f32 {
        connor
            .lines()
            .find(|line| line.starts_with(&format!("{update} ")))
            .unwrap_or_else(|| panic!("missing Connor update {update}; inspect {}", dir.display()))
            .split_whitespace()
            .nth(27)
            .unwrap()
            .parse()
            .unwrap()
    };
    let before = heading_at(80);
    for update in [81, 85, 90] {
        assert!(
            (heading_at(update) - before).abs() < 0.001,
            "Connor turned during a first-person Hammer throw at update {update}; inspect {}",
            dir.display()
        );
    }
    assert_eq!(
        connor
            .matches("throw: 'conhmrfp.anm/catch' ended: handsOff off")
            .count(),
        1
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
