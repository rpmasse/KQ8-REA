//! Barren chest script arms its hidden Henchman, who then fights by ordinary AI.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn barren_chest_reveals_henchman_for_live_attack_and_death() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/barren-henchman-live/{nonce}"));
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
            "28400,36000,3.14159",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            // The installed chest script reveals the hidden actor and begins drip,taunt.
            "--command",
            "sendEvent 0 Chest1 KQCueEvent 20 50,20",
            "--command",
            "sendEvent 0 Henchman KQAttackEvent Connor 0 100000 0,360",
            "--updates",
            "500",
            "--frames",
            "15000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("test --command sendEvent 0 Chest1 KQCueEvent 20 50")
            && fight.contains("Henchman: fights (rule 2): stats [2050, 125, 125, 6, 50005]")
            && characters.contains("Henchman: Special list used up")
            && characters.contains("Henchman: status: Wait, active"),
        "the installed chest branch did not arm ordinary Henchman AI; inspect {}",
        dir.display()
    );
    assert!(
        fight
            .lines()
            .any(|line| line.contains("Henchman: Do (Attack")
                && line.contains("hits Connor")
                && line.contains("reach 2500")
                && line.contains("hand damage 125, Normal")),
        "Henchman made no authored melee hit; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by Henchman, Normal: D 125,"))
        .collect();
    assert!(
        !hits.is_empty(),
        "Henchman did not reach Connor; inspect {}",
        dir.display()
    );
    for hit in hits {
        let health = hit.rsplit_once("health ").unwrap().1;
        let (before, after) = health.split_once(" -> ").unwrap();
        assert!(
            after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
            "Henchman did not lower health: {hit}"
        );
    }
    let death = "Henchman: dies (rule 30): kind Normal";
    assert_eq!(fight.matches(death).count(), 1);
    assert!(fight.find("Henchman: Do (Attack").unwrap() < fight.find(death).unwrap());
    let drops: Vec<_> = items
        .lines()
        .filter(|line| line.contains("Henchman: drops (rule 31;"))
        .collect();
    assert_eq!(
        drops.len(),
        1,
        "Henchman drops missing or repeated; inspect {}",
        dir.display()
    );
    for item in [
        "INVITEM_strength x1",
        "INVITEM_invulnerable x1",
        "INVITEM_sacredwater x1",
    ] {
        assert_eq!(
            drops[0].matches(item).count(),
            1,
            "missing or repeated {item}; inspect {}",
            dir.display()
        );
    }
    let finish = "Henchman: its death ended (rule 32): Connor gets its experience 50005";
    assert_eq!(fight.matches(finish).count(), 1);
    assert!(
        fight
            .lines()
            .any(|line| line.contains(finish) && line.contains("a corpse hm_bded.kqs")),
        "Henchman left the wrong Normal corpse; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("Connor: hit by world") && !fight.contains("god mode is on"),
        "world damage or god mode obscured combat; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
