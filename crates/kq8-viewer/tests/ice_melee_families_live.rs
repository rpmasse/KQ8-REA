//! Three placed Ice World melee families attack and die in one original-data run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn frosty_hammer_orc_and_axe_griffin_damage_connor() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/ice-melee-families-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--world", "IceWorld", "--windowed", "--renderer", "dx12",
            "--sound", "off", "--at", "268329,480983,0",
            // Raise Connor's health ceiling while retaining ordinary damage.
            "--command", "KQConner::setLevel 1000,1",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            // All three are installed actors moved into the measured floor lane.
            "--command", "move fsboy1 0 266900 480983 31365 0 0 0,20",
            "--command", "sendEvent 0 fsboy1 KQAttackEvent Connor 0 100000 0,95",
            "--command", "move orcpath1 0 266500 480983 31365 0 0 0,100",
            "--command", "sendEvent 0 orcpath1 KQAttackEvent Connor 0 100000 0,170",
            "--command", "move griffbax2 0 267500 480983 31365 0 0 0,175",
            "--command", "sendEvent 0 griffbax2 KQAttackEvent Connor 0 100000 0,245",
            "--updates", "310", "--frames", "10000",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    for (name, stats, damage, reach) in [
        ("fsboy1", "[1200, 75, 75, 40, 10000]", 75, 1500),
        ("orcpath1", "[1000, 80, 80, 30, 15000]", 80, 2000),
        ("griffbax2", "[1700, 100, 100, 25, 0]", 100, 1000),
    ] {
        assert!(viewer.contains(&format!("test --command move {name}")),
            "{name}'s explicit relocation must remain visible; inspect {}", dir.display());
        assert!(fight.contains(&format!("{name}: fights (rule 2): stats {stats}")),
            "{name} did not load its installed combat stats; inspect {}", dir.display());
        let attack = fight.lines().find(|line| line.contains(&format!("{name}: Do (Attack"))
            && line.contains("hits Connor") && line.contains(&format!("reach {reach}"))
            && line.contains(&format!("hand damage {damage}, Normal")));
        assert!(attack.is_some(), "{name} did not land its authored melee attack; inspect {}", dir.display());
        let marker = format!("Connor: hit by {name}, Normal: D {damage},");
        let hits: Vec<_> = fight.lines().filter(|line| line.contains(&marker)).collect();
        assert!(!hits.is_empty(), "{name} caused no Normal health loss; inspect {}", dir.display());
        for hit in hits {
            let health = hit.rsplit_once("health ").unwrap().1;
            let (before, after) = health.split_once(" -> ").unwrap();
            assert!(after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
                "{name} did not lower Connor's health: {hit}");
        }
        let entry = format!("{name}: dies (rule 30): kind Normal");
        assert_eq!(fight.matches(&entry).count(), 1,
            "{name} did not die exactly once; inspect {}", dir.display());
        assert!(fight.find(&format!("{name}: Do (Attack")).unwrap() < fight.find(&entry).unwrap(),
            "{name} died before its checked attack; inspect {}", dir.display());
    }
    for (name, experience, corpse) in [
        ("fsboy1", 10000, "no corpse (no shape for this kind)"),
        ("orcpath1", 15000, "a corpse oh_death.kqs"),
        ("griffbax2", 0, "a corpse grb_dead.kqs"),
    ] {
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {experience}");
        assert_eq!(fight.matches(&finish).count(), 1,
            "{name} did not grant its authored experience once; inspect {}", dir.display());
        assert!(fight.lines().any(|line| line.contains(&finish) && line.contains(corpse)),
            "{name} has the wrong Normal corpse; inspect {}", dir.display());
    }
    for name in ["fsboy1", "griffbax2"] {
        assert_eq!(fight.matches(&format!("{name}: drops: nothing (rule 31)")).count(), 1,
            "{name} unexpectedly dropped an item; inspect {}", dir.display());
    }
    let orc_drops: Vec<_> = items.lines()
        .filter(|line| line.contains("orcpath1: drops (rule 31;")).collect();
    assert_eq!(orc_drops.len(), 1, "hammer Orc drops did not occur once; inspect {}", dir.display());
    assert_eq!(orc_drops[0].matches("INVITEM_sacredwater x1").count(), 1);
    assert_eq!(orc_drops[0].matches("INVITEM_crystal x1").count(), 1);
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"),
        "world hazards or god mode obscured combat; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
