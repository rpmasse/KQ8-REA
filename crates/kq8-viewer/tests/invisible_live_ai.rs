//! Original Daventry goblin loses Connor to Invisible and sees him after expiry.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn invisible_breaks_then_restores_a_live_goblin_chase() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/invisible-live-ai/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--god",
            "--world",
            "Daventry",
            "--at",
            "289284,195000,0",
        ])
        .args([
            "--key",
            "popup:autohelp,0",
            "--command",
            "KQMonster::inventory Connor INVITEM_Invisible 1,20",
            "--key",
            "8,45",
            "--command",
            "move goblin22 0 289284 194000 13770 0 0 0,275",
            "--updates",
            "305",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        items.contains("slot 8, INVITEM_Invisible")
            && items.contains("invisible: characters are told Connor is 100,000 below")
            && items.contains("Dur 15 s:")
            && items
                .lines()
                .any(|line| line.starts_with("update 270:") && line.contains("invisible ends")),
        "the original potion did not run for 15 seconds; inspect {}",
        dir.display()
    );
    let events: Vec<_> = characters
        .lines()
        .filter(|line| line.contains("goblin22:"))
        .collect();
    let update = |line: &str| {
        line.strip_prefix("update ")
            .and_then(|tail| tail.split_once(':'))
            .and_then(|(number, _)| number.parse::<u32>().ok())
            .unwrap()
    };
    assert!(
        events
            .iter()
            .any(|line| update(line) < 45 && line.contains("alarmed: sees Connor")),
        "the original goblin never started chasing; inspect {}",
        dir.display()
    );
    assert!(
        events
            .iter()
            .any(|line| (45..270).contains(&update(line))
                && line.contains("can't see him (the ground)"))
            && events.iter().any(|line| (45..270).contains(&update(line))
                && line.contains("gives up: has not seen Connor for 45 updates")),
        "Invisible did not break the goblin's sight and chase; inspect {}",
        dir.display()
    );
    assert!(
        events.iter().any(|line| update(line) > 275
            && line.contains("alarmed: sees Connor")
            && line.contains("within its alarm radius 4000")),
        "after expiry the goblin did not see Connor again; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
