//! Temple 3's placed bow, hammer and zombie families fight in one original-data run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn temple_three_bows_hammer_and_zombie_damage_connor() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/temple-three-families-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--world", "temple3", "--windowed", "--renderer", "dx12",
            "--sound", "off", "--at", "127000,100000,0", "--lift", "3562",
            // The large health ceiling lets all four actors inflict ordinary damage.
            "--command", "KQConner::setLevel 1000,1",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            // Both zombie placements activate but remain at their original points.
            // Bring one inside its listed 2000-unit melee reach after the bow shots.
            "--command", "move zombie1 0 126000 100000 3562 0 0 0,60",
            "--command", "sendEvent 0 skelhamr1 KQAttackEvent Connor 0 100000 0,100",
            "--command", "sendEvent 0 skelblue1 KQAttackEvent Connor 0 100000 0,100",
            "--command", "sendEvent 0 skelblue2 KQAttackEvent Connor 0 100000 0,100",
            "--command", "sendEvent 0 zombie1 KQAttackEvent Connor 0 100000 0,180",
            // The second placement uses the same type but a 1.2 s attack delay.
            "--command", "move zombie2 0 126000 100000 3562 0 0 0,216",
            "--command", "sendEvent 0 zombie2 KQAttackEvent Connor 0 100000 0,285",
            "--updates", "350", "--frames", "10000",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(viewer.contains("test --command move zombie1")
        && viewer.contains("test --command move zombie2"),
        "the zombies' explicit relocations must remain visible; inspect {}", dir.display());
    for (name, stats) in [
        ("zombie1", "[2020, 260, 260, 40, 15003]"),
        ("zombie2", "[2020, 260, 260, 40, 15003]"),
        ("skelhamr1", "[2800, 450, 450, 70, 60000]"),
        ("skelblue1", "[2060, 215, 215, 45, 20200]"),
        ("skelblue2", "[2060, 215, 215, 45, 20200]"),
    ] {
        assert!(fight.contains(&format!("{name}: fights (rule 2): stats {stats}")),
            "{name} did not load its installed combat stats; inspect {}", dir.display());
        assert!(fight.contains(&format!("{name}: Do (Attack")),
            "{name} did not complete its authored attack; inspect {}", dir.display());
    }
    assert!(characters.contains("zombie1: status: Move, active, at (126000, 100000, 3562)"),
        "the placed zombie did not enter the measured melee lane; inspect {}", dir.display());
    assert!(characters.contains("zombie2: status: Move, active, at (126000, 100000, 3562)"),
        "the second placed zombie did not enter the measured melee lane; inspect {}", dir.display());
    assert!(fight.contains("zombie1: fights (rule 2): stats [2020, 260, 260, 40, 15003]")
        && fight.contains("zombie2: fights (rule 2): stats [2020, 260, 260, 40, 15003]"),
        "the two zombie placements lost their common combat stats; inspect {}", dir.display());
    assert!(characters.lines().any(|line| line.contains("zombie1: 'Attack' ended: next attack due")
        && line.contains("attackDelay 1 s"))
        && characters.lines().any(|line| line.contains("zombie2: 'Attack' ended: next attack due")
            && line.contains("attackDelay 1.200")),
        "the two placements did not use their distinct authored attack delays; inspect {}", dir.display());
    for name in ["skelblue1", "skelblue2"] {
        assert!(fight.lines().any(|line| line.contains(&format!("{name}: Do (Attack"))
            && line.contains("Prmonarw.kq") && line.contains("Projectile")),
            "{name} did not fire its authored skeletal arrow; inspect {}", dir.display());
        assert!(fight.lines().any(|line| line.contains(&format!("{name}: bolt"))
            && line.contains("touches Connor") && line.contains("a hit of 215, Projectile")),
            "{name}'s arrow did not contact Connor; inspect {}", dir.display());
    }
    for (name, kind) in [
        ("skelhamr1", "Normal"), ("zombie1", "Normal"), ("zombie2", "Normal"),
        ("skelblue1", "Projectile"), ("skelblue2", "Projectile"),
    ] {
        let marker = format!("Connor: hit by {name}, {kind}:");
        let hits: Vec<_> = fight.lines().filter(|line| line.contains(&marker)).collect();
        assert!(!hits.is_empty(), "{name} caused no {kind} damage; inspect {}", dir.display());
        for hit in hits {
            let health = hit.rsplit_once("health ").unwrap().1;
            let (before, after) = health.split_once(" -> ").unwrap();
            assert!(after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
                "{name} did not lower Connor's health: {hit}");
        }
    }
    for (name, experience, item, corpse) in [
        ("skelhamr1", 60000, "INVITEM_elixeroflife", "sk_dead.kqs"),
        ("skelblue1", 20200, "INVITEM_sacredwater", "skbdead.kqs"),
        ("skelblue2", 20200, "INVITEM_sacredwater", "skbdead.kqs"),
        ("zombie1", 15003, "INVITEM_sacredwater", "zdead.kqs"),
        ("zombie2", 15003, "INVITEM_sacredwater", "zdead.kqs"),
    ] {
        assert_eq!(fight.matches(&format!("{name}: dies (rule 30): kind Normal")).count(), 1,
            "{name} did not die once; inspect {}", dir.display());
        let drops: Vec<_> = items.lines()
            .filter(|line| line.contains(&format!("{name}: drops (rule 31;"))).collect();
        assert_eq!(drops.len(), 1, "{name} did not drop once; inspect {}", dir.display());
        assert_eq!(drops[0].matches(&format!("{item} x1")).count(), 1,
            "{name} has the wrong authored drop; inspect {}", dir.display());
        if name.starts_with("skelblue") {
            assert!(!drops[0].contains("INVITEM_SkelBow"),
                "the held bow is not an authored drop; inspect {}", dir.display());
        }
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {experience}");
        assert_eq!(fight.matches(&finish).count(), 1,
            "{name} did not grant experience once; inspect {}", dir.display());
        assert!(fight.lines().any(|line| line.contains(&finish)
            && line.contains(&format!("a corpse {corpse}"))),
            "{name} has the wrong Normal corpse; inspect {}", dir.display());
    }
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"),
        "world hazards or god mode obscured combat; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
