//! A held Rock hits an original fighter and becomes recoverable at contact.
use std::{env, fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn rock_hits_original_skeleton_and_recovers_at_contact() {
    let game = fs::canonicalize(env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/rock-tests/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    // The Daventry wall/floor landing and fresh-process continuation remain
    // covered by rock_flight_checkpoint. This one launch adds enemy contact.
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--world",
            "DeadCity",
            "--no-ai",
            "--item",
            "Rock",
            "--key",
            "left:IDCTL_INVBAR_ROCKS,10",
            "--key",
            "left:IDCTL_INVBAR_ROCKS,14",
            "--command",
            "move skeleton1 0 183300 45000 864 0 0 0,5",
            // Rock's pointer line clips at drawn terrain. Aim through the
            // skeleton's ground point; aiming through its chest overshoots.
            "--aim-point",
            "183300,45000,864,24",
            "--updates",
            "120",
            "--frames",
            "900",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("god mode is on"),
        "god mode was used; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("Rock do (throw)") && fight.contains("fixed aim [183300.0, 45000.0, 864.0]"),
        "the authored Rock throw did not aim at the ground point; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("skeleton1: hit by Connor, Normal:").count(),
        1,
        "the Rock did not deliver one Normal hit; inspect {}",
        dir.display()
    );
    assert!(
        fight
            .lines()
            .any(|line| line.contains("rock1 update ") && line.contains("touches skeleton1")),
        "the Rock did not touch the fighter; inspect {}",
        dir.display()
    );
    let hit = fight
        .lines()
        .find(|line| line.contains("skeleton1: hit by Connor, Normal:"))
        .unwrap();
    let (before, after) = hit
        .split_once("; health ")
        .and_then(|(_, pair)| pair.split(';').next())
        .and_then(|pair| pair.split_once(" -> "))
        .map(|(before, after)| {
            (
                before.parse::<i64>().unwrap(),
                after.parse::<i64>().unwrap(),
            )
        })
        .unwrap();
    assert!(
        after < before && after > 0,
        "the original skeleton did not lose health and survive ({before} -> {after}); inspect {}",
        dir.display()
    );
    assert!(
        items.contains("rockHit.cs line 22: dropItem: fresh landing")
            && items.contains("old identity TestItem retained"),
        "the Rock was not recovered at contact; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
