//! Original Gnome batmanta wakes, attacks, and injures Connor through AI.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn gnome_batmanta_wakes_and_lands_a_normal_attack() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/gnome-batmanta-live-combat/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--world",
            "Gnome",
            "--at",
            "100819,59000,0",
            // Put Connor on the encounter floor. The original closed door
            // otherwise blocks this bat's line of sight from the test spot.
            "--command",
            "move Connor 0 100819 58700 17575 0 0 0,4",
            "--command",
            "deleteObject door01a,5",
            "--updates",
            "65",
            "--frames",
            "500",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("deleteObject door01a: Some(\"0\")"),
        "inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("batmanta1: alarmed: seen in visible PreBirth")
            && characters.contains("batmanta1: status: Move, active"),
        "the bat did not wake through its normal AI; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("batmanta1: attack (rule 7, moving)"),
        "the bat did not attack through its normal AI; inspect {}",
        dir.display()
    );
    let hit = fight
        .lines()
        .find(|line| line.contains("Connor: hit by batmanta1, Normal: D 50,"))
        .unwrap_or_else(|| panic!("no batmanta hit; inspect {}", dir.display()));
    let health = hit.rsplit_once("health ").unwrap().1;
    let (before, after) = health.split_once(" -> ").unwrap();
    let before: i32 = before.parse().unwrap();
    let after: i32 = after
        .split(|c: char| !c.is_ascii_digit() && c != '-')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        before > after,
        "batmanta did not lower Connor's health: {hit}"
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
