//! Original-data Dragon projectile contacts: zero-mask heads and body.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn fire_arrow_contacts_dragon_head_and_body_without_health_loss() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/dragon-projectile-contact/{nonce}"));

    for (label, setup, aim, target, health) in [
        ("head", vec![], "draglhed,60", "draglhed", 600),
        ("body-start-inside", vec![
            "--command", "move dragbody 0 182734 488910 26000 0 0 0,5",
            "--command", "move draglhed 0 200000 490000 26000 0 0 0,5",
            "--command", "move dragrhed 0 200000 495000 26000 0 0 0,5",
        ], "dragbody,60", "dragbody", 300),
    ] {
        let run = root.join(label);
        fs::create_dir_all(&run).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&run).arg("--game").arg(&game)
            .args(["--windowed", "--sound", "off", "--no-ai", "--god",
                "--world", "IceWorld", "--at", "182334,490176,0"])
            .args(&setup)
            .args(["--command", "KQMonster::inventory Connor INVITEM_FlameBow 1,20",
                "--command", "KQConner::setDoMode 1,35", "--aim", aim,
                "--updates", "120"])
            .output().unwrap();
        fs::write(run.join("stderr.log"), &output.stderr).unwrap();
        let logs = run.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", run.display());
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert_eq!(fight.matches("shoots bolt1 (Prfarrow.kq").count(), 1, "inspect {}", run.display());
        assert_eq!(fight.matches(&format!("touches {target}")).count(), 1, "inspect {}", run.display());
        assert_eq!(fight.matches(&format!("{target}: hit by Connor, Fire:")).count(), 1, "inspect {}", run.display());
        assert!(fight.contains(&format!("health {health} -> {health}")), "inspect {}", run.display());
        assert!(!fight.contains(if target == "dragbody" { "touches draglhed" } else { "touches dragbody" }),
            "arrow passed through the first actor; inspect {}", run.display());
        if target == "draglhed" {
            assert!(fight.contains("its type 0x4042 has the immune bit 0x4000: a hit of 0")
                && fight.contains("draglhed: hit by Connor, Fire: D 0"),
                "head immunity missing; inspect {}", run.display());
        } else {
            assert!(fight.contains("bolt1 update 1: touches dragbody")
                && fight.contains("0 travelled; removed there")
                && fight.contains("dragbody: hit by Connor, Fire: D 151")
                && fight.contains("P 100: no loss"),
                "body start-inside contact or protection missing; inspect {}", run.display());
        }
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
