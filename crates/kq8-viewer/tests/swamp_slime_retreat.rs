//! Unattended original-data route for a ranged Swamp slime's attack and retreat.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_slime_shoots_then_keeps_distance_and_respects_its_leash() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/swamp-slime-retreat/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args(["--world", "Swamp", "--at", "168074,72961,0",
            "--god", "--sound", "off", "--windowed", "--updates", "120"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(fight.lines().any(|line| line.contains("slime1: fights (rule 2)")
        && line.contains("likelyToFlee 0.75; tooClose 0.4 farEnough 3")
        && line.contains("ranged with INVITEM_SlmSlime")), "original slime data: inspect {}", dir.display());
    let shot = fight.lines().find(|line| line.contains("slime1: Do")
        && line.contains("shoots bolt1 (Prslime.kq"))
        .unwrap_or_else(|| panic!("original ranged shot: inspect {}", dir.display()));
    assert!(fight.lines().any(|line| line.contains("slime1: bolt1")
        && line.contains("touches Connor")), "projectile impact: inspect {}", dir.display());
    let start = characters.lines().find(|line| line.contains("slime1: keeping distance starts: chance")
        && line.contains("tooClose 2000"))
        .unwrap_or_else(|| panic!("retreat entry: inspect {}", dir.display()));
    let stop = characters.lines().find(|line| line.contains("slime1: keeping distance ends:")
        && line.contains("beyond 15000"))
        .unwrap_or_else(|| panic!("retreat stop: inspect {}", dir.display()));
    let give_up = characters.lines().find(|line| line.contains("slime1: gives up:")
        && line.contains("beyond its chase radius 6000"))
        .unwrap_or_else(|| panic!("chase leash: inspect {}", dir.display()));
    let update = |line: &str| -> u64 { line.strip_prefix("update ").unwrap()
        .split(':').next().unwrap().parse().unwrap() };
    assert!(update(shot) < update(start) && update(start) < update(stop)
        && update(stop) < update(give_up), "wrong shot/retreat/leash order: inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
