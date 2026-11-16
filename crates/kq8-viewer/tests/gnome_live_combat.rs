//! Two original Gnome enemy families attack and injure Connor through AI.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn gnome_bat_and_rock_attacks_lower_connors_health() {
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

    // Use the same known encounter floor for the installed ranged rockguy.
    // Both original door leaves are removed so its spawned projectile can
    // leave the rock rather than immediately hitting a door collider.
    let rock_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/gnome-rockguy-live-combat/{nonce}"
    ));
    fs::create_dir_all(&rock_dir).unwrap();
    let rock_output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&rock_dir)
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
            "--command",
            "move Connor 0 100819 58700 17575 0 0 0,4",
            "--command",
            "move Rock2 0 100819 60398 17575 0 0 0,4",
            "--command",
            "deleteObject batmanta1,4",
            "--command",
            "deleteObject door01a door01b,5",
            "--updates",
            "150",
            "--frames",
            "900",
        ])
        .output()
        .unwrap();
    fs::write(rock_dir.join("stdout.log"), &rock_output.stdout).unwrap();
    fs::write(rock_dir.join("stderr.log"), &rock_output.stderr).unwrap();
    let rock_logs = rock_dir.join("kq8-artifacts/logs");
    let rock_viewer = fs::read_to_string(rock_logs.join("viewer.log")).unwrap();
    let rock_characters = fs::read_to_string(rock_logs.join("characters.log")).unwrap();
    let rock_fight = fs::read_to_string(rock_logs.join("fight.log")).unwrap();
    assert!(
        rock_output.status.success() && rock_viewer.contains("RESULT: PASS"),
        "inspect {}",
        rock_dir.display()
    );
    assert!(
        rock_characters.contains("Rock2: alarmed: seen in visible PreBirth")
            && rock_fight.contains("Rock2: attack (rule 7, standing)"),
        "the ranged rock did not attack through AI; inspect {}",
        rock_dir.display()
    );
    assert!(
        rock_fight.contains("shoots bolt1 (PRdmnrck.kq")
            && rock_fight.contains("Rock2: bolt1 update 1: touches Connor"),
        "the original Demon Rocks projectile did not reach Connor; inspect {}",
        rock_dir.display()
    );
    let rock_hit = rock_fight
        .lines()
        .find(|line| line.contains("Connor: hit by Rock2, Projectile: D 50,"))
        .unwrap_or_else(|| panic!("no ranged damage; inspect {}", rock_dir.display()));
    let (before, after) = rock_hit
        .rsplit_once("health ")
        .unwrap()
        .1
        .split_once(" -> ")
        .unwrap();
    let before: i32 = before.parse().unwrap();
    let after: i32 = after.parse().unwrap();
    assert!(
        before > after,
        "rock projectile did not lower Connor's health: {rock_hit}"
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
