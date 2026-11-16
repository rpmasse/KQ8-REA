//! A placed Barren Pyrod's Lava projectile damages living Connor.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_pyrod_firebolt_and_death() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/pyrod-live-fire/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Barren",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--at",
            "116065,50000,0",
            // The same lane previously inflicted fatal lava damage before
            // Pyrod fired. Keep ordinary damage enabled and raise only the
            // health ceiling so a terrain hit cannot hide Pyrod's own hit.
            "--command",
            "KQConner::setLevel 10000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 100000 0 0 0 0,2",
            "--command",
            "sendEvent 0 pyrod1 KQAttackEvent Connor 0 100000 0,100",
            "--updates",
            "160",
            "--frames",
            "8000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("test --command KQConner::setLevel 10000: Some(\"0\")"),
        "the survival setup was not applied; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("pyrod1: fights (rule 2): stats [220, 65, 65, 41, 8000]")
            && fight.contains("ranged with INVITEM_Lava (shoots Prpyrolv.kq)"),
        "the placed Pyrod did not load its authored Lava weapon; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| line.contains("pyrod1: Do (Attack")
            && line.contains("no part 'arrow' in frame")
            && line.contains("shootOffsets fallback")
            && line.contains("Prpyrolv.kq")
            && line.contains("Fire")),
        "Pyrod did not release its authored Fire projectile; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| line.contains("pyrod1: bolt")
            && line.contains("touches Connor")
            && line.contains("a hit of 65, Fire")),
        "Pyrod's projectile did not contact Connor; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by pyrod1, Fire: D 65,"))
        .collect();
    assert!(
        !hits.is_empty(),
        "Pyrod caused no Fire health loss; inspect {}",
        dir.display()
    );
    for hit in hits {
        let health = hit.rsplit_once("health ").unwrap().1;
        let (before, after) = health.split_once(" -> ").unwrap();
        let before: i64 = before.parse().unwrap();
        let after: i64 = after.parse().unwrap();
        assert!(
            before > after && after > 0,
            "Connor was not alive after Pyrod's hit: {hit}"
        );
    }
    assert_eq!(
        fight.matches("pyrod1: dies (rule 30): kind Normal").count(),
        1
    );
    assert_eq!(fight.matches("pyrod1: drops: nothing (rule 31)").count(), 1);
    let finish = "pyrod1: its death ended (rule 32): Connor gets its experience 8000";
    assert_eq!(fight.matches(finish).count(), 1);
    assert!(
        fight.lines().any(
            |line| line.contains(finish) && line.contains("no corpse (no shape for this kind)")
        ),
        "Pyrod has the wrong Normal corpse; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("god mode is on"),
        "god mode obscured the fight; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
