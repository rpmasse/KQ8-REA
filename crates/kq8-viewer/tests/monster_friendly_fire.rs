//! A bow spriggan's ordinary bolt can hit a goblin on its way to Connor.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_bow_spriggan_bolts_injure_another_monster_in_flight() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/monster-friendly-fire/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args([
            "--windowed", "--sound", "off", "--god", "--world", "Daventry",
            "--at", "289284,188000,0", "--key", "popup:autohelp,0",
            "--command", "move goblin22 0 289284 190500 13770 0 0 0,5",
            "--command", "move sprgnbow 0 289284 193500 13770 0 0 0,5",
            "--updates", "125", "--frames", "600",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("sprgnbow: status: Wait, active")
        && fight.contains("sprgnbow: attack (rule 7, standing)")
        && fight.contains("sprgnbow: Do (Attack at phase"),
        "the original bow spriggan did not shoot through normal AI; inspect {}", dir.display());
    let touches: Vec<_> = fight.lines().filter(|line| line.contains("sprgnbow: bolt")
        && line.contains("touches goblin22")).collect();
    assert_eq!(touches.len(), 3, "the three bolts did not strike the goblin once each; inspect {}", dir.display());
    assert!(touches.iter().all(|line| line.contains("update 2: touches goblin22")
        && !line.contains("0 travelled") && line.contains("Projectile")),
        "the bolts did not collide in flight; inspect {}", dir.display());
    let hits: Vec<_> = fight.lines().filter(|line| line.contains("goblin22: hit by sprgnbow, Projectile:")).collect();
    assert_eq!(hits.len(), touches.len(), "projectile damage did not follow each touch; inspect {}", dir.display());
    assert!(hits[0].contains("health 15 -> 13")
        && hits[1].contains("health 13 -> 11")
        && hits[2].contains("health 11 -> 9")
        && !fight.contains("Connor: hit by sprgnbow"),
        "the intercept did not protect Connor or reduce the goblin's health; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
