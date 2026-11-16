//! Three distinct Temple 1 enemy families attack and die in one original-data run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn temple_one_water_demon_beast_and_shadow_monster() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/temple-one-families-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--world", "temple1", "--windowed", "--renderer", "dx12",
            "--sound", "off", "--at", "98000,55204,0", "--lift", "5282",
            // Raise the health ceiling after conStart.cs; ordinary damage remains enabled.
            "--command", "KQConner::setLevel 100,20",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,21",
            "--command", "sendEvent 0 watdemon1 KQAttackEvent Connor 0 100000 0,45",
            // The placed beast starts far away. Keep it outside Connor's rise-clearance
            // radius but inside its 1500-unit alarm and attack distances.
            "--command", "move vicbeast2 0 96650 55204 5282 0 0 0,55",
            "--command", "sendEvent 0 vicbeast2 KQAttackEvent Connor 0 100000 0,160",
            // hench01 is a placed Shadow Monster parked off-map by the world script.
            "--command", "move hench01 0 96650 55204 5282 0 0 0,212",
            "--command", "sendEvent 0 hench01 KQAttackEvent Connor 0 100000 0,280",
            "--updates", "340", "--frames", "8000",
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
    assert!(viewer.contains("test --command move vicbeast2")
        && viewer.contains("test --command move hench01"),
        "the explicit actor relocations must remain visible in this test; inspect {}", dir.display());
    assert!(fight.contains("watdemon1: fights (rule 2): stats [2300, 635, 635, 30, 120000]")
        && fight.contains("vicbeast2: fights (rule 2): stats [3000, 1000, 1000, 25, 30000]")
        && fight.contains("hench01: fights (rule 2): stats [1050, 135, 135, 30, 15000]"),
        "the installed Temple 1 combat types were not loaded; inspect {}", dir.display());
    assert!(characters.contains("vicbeast2: rises (rule 30): Connor 1350 away")
        && characters.contains("hench01: status: Wait, active"),
        "the relocated actors did not activate as expected; inspect {}", dir.display());
    for (name, kind) in [("watdemon1", "Normal"), ("vicbeast2", "Normal"), ("hench01", "Fire")] {
        assert!(fight.contains(&format!("{name}: Do (Attack")),
            "{name} did not complete an authored attack; inspect {}", dir.display());
        let marker = format!("Connor: hit by {name}, {kind}:");
        let hits: Vec<_> = fight.lines().filter(|line| line.contains(&marker)).collect();
        assert!(!hits.is_empty(), "{name} made no {kind} health loss; inspect {}", dir.display());
        for hit in hits {
            let health = hit.rsplit_once("health ").unwrap().1;
            let (before, after) = health.split_once(" -> ").unwrap();
            assert!(after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
                "{name} did not lower Connor's health: {hit}");
        }
    }
    assert!(fight.lines().any(|line| line.contains("hench01: Do (Attack")
        && line.contains("its part 'frbll' in frame")
        && line.contains("Prfire.kq") && line.contains("Fire")),
        "Shadow Monster did not use its authored fireball; inspect {}", dir.display());
    assert!(fight.lines().any(|line| line.contains("hench01: bolt")
        && line.contains("touches Connor") && line.contains("a hit of 135, Fire")),
        "the fireball did not contact Connor; inspect {}", dir.display());
    for (name, experience, corpse) in [
        ("watdemon1", 120000, "a corpse wd_death.KQS"),
        ("vicbeast2", 30000, "no corpse (no shape for this kind)"),
        ("hench01", 15000, "a corpse shm_ded2.KQS"),
    ] {
        assert_eq!(fight.matches(&format!("{name}: dies (rule 30): kind Normal")).count(), 1,
            "{name} did not die exactly once; inspect {}", dir.display());
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {experience}");
        assert_eq!(fight.matches(&finish).count(), 1,
            "{name} did not grant experience once; inspect {}", dir.display());
        assert!(fight.lines().any(|line| line.contains(&finish) && line.contains(corpse)),
            "{name} has the wrong Normal corpse; inspect {}", dir.display());
    }
    for name in ["watdemon1", "vicbeast2"] {
        assert_eq!(fight.matches(&format!("{name}: drops: nothing (rule 31)")).count(), 1,
            "{name} unexpectedly dropped an item; inspect {}", dir.display());
    }
    let shadow_drops: Vec<_> = items.lines()
        .filter(|line| line.contains("hench01: drops (rule 31;")).collect();
    assert_eq!(shadow_drops.len(), 1, "Shadow Monster drops did not occur once; inspect {}", dir.display());
    assert_eq!(shadow_drops[0].matches("INVITEM_crystal x1").count(), 3,
        "Shadow Monster did not drop its three authored crystals; inspect {}", dir.display());
    assert!(!shadow_drops[0].contains("INVITEM_WitchBolt"),
        "the held WitchBolt is not an authored drop; inspect {}", dir.display());
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"),
        "world hazards or god mode obscured combat; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
