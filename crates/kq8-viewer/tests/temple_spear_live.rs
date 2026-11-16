//! Temple 2's placed spear carrier and melee guard fight on the authored upper floor.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_temple_spear_and_melee_guard_damage_connor() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/temple-spear-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--world", "temple2", "--windowed", "--renderer", "dx12",
            "--sound", "off", "--at", "109101,109105,0", "--lift", "6050",
            "--updates", "95", "--frames", "3600",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("spearchucker1: status: Wait, active")
        && characters.contains("Hallhench2: status: Wait, active"),
        "the installed enemies did not activate; inspect {}", dir.display());
    assert!(fight.contains("spearchucker1: fights (rule 2): stats [1500, 300, 375, 40, 25000]")
        && fight.contains("from type hencspr2")
        && fight.contains("Hallhench2: fights (rule 2): stats [1500, 300, 300, 40, 25000]")
        && fight.contains("from type hencspr1"),
        "the original combat types were not loaded; inspect {}", dir.display());
    let spear_do: Vec<_> = fight.lines().filter(|line|
        line.contains("spearchucker1: Do (Attack") && line.contains("shoots bolt")
            && line.contains("PRspear.kq") && line.contains("Projectile")
    ).collect();
    assert_eq!(spear_do.len(), 2, "expected two authored spear releases; inspect {}", dir.display());
    assert!(spear_do[0].contains("its part 'spear' in frame")
        && spear_do[1].contains("kept from an earlier shot"),
        "the spear's frame part and retained offset were not used; inspect {}", dir.display());
    assert_eq!(fight.lines().filter(|line|
        line.contains("spearchucker1: bolt") && line.contains("touches Connor")
            && line.contains("a hit of 375, Projectile")
    ).count(), 2, "spear contact did not deliver its authored hit; inspect {}", dir.display());
    let check_loss = |source: &str, kind: &str, count: usize| {
        let marker = format!("Connor: hit by {source}, {kind}:");
        let hits: Vec<_> = fight.lines().filter(|line| line.contains(&marker)).collect();
        assert_eq!(hits.len(), count, "wrong number of {source} hits; inspect {}", dir.display());
        for hit in hits {
            let health = hit.rsplit_once("health ").unwrap().1;
            let (before, after) = health.split_once(" -> ").unwrap();
            assert!(after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
                "{source} did not lower Connor's health: {hit}");
        }
    };
    check_loss("spearchucker1", "Projectile", 2);
    assert!(fight.contains("Hallhench2: Do (Attack"), "melee guard did not swing; inspect {}", dir.display());
    check_loss("Hallhench2", "Normal", 1);
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"),
        "world hazards or god mode obscured combat; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
