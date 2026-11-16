//! Unattended GOG-data fight with the Swamp slime's ordinary AI and damage.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn slime_shoots_connor_then_dies_from_an_ordinary_flame_bow_shot() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/swamp-slime-live-combat/{nonce}"));
    for run_number in 1..=2 {
        let dir = root.join(run_number.to_string());
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game)
            .args([
                "--world", "Swamp", "--at", "168074,72961,3.14159",
                "--combat-level", "1", "--windowed", "--sound", "off",
                "--command", "KQMonster::inventory Connor INVITEM_FlameBow 1,3",
                "--command", "KQConner::setDoMode 1,8",
                "--aim-when-ready", "slime1,20", "--updates", "100",
            ]).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        assert_eq!(fight.matches("slime1: bolt1 update 1: touches Connor").count(), 1,
            "slime's original projectile missed; inspect {}", dir.display());
        assert_eq!(fight.matches("Connor: hit by slime1, Projectile:").count(), 1,
            "incoming damage missing; inspect {}", dir.display());
        assert_eq!(fight.matches("slime1: hit by Connor, Fire:").count(), 1,
            "Connor's ordinary shot missed; inspect {}", dir.display());
        assert!(fight.contains("slime1: dies (rule 30): kind Fire, 'firedie' starts")
            && fight.contains("slime1: its death ended (rule 32): Connor gets its experience 5000"),
            "death or reward missing; inspect {}", dir.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
