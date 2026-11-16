//! Two placed Swamp melee families in one unattended GOG-data process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_fiend_hits_and_both_families_follow_death_rules() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/swamp-fiend-bubble-live/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Swamp",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--at",
            "168074,72961,3.14159",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "move fiend2 0 166500 72961 5355 0 0 0,20",
            "--command",
            "sendEvent 0 fiend2 KQAttackEvent Connor 0 100000 0,120",
            "--command",
            "move swmpbb2a 0 167000 72961 5355 0 0 0,190",
            "--command",
            "KQMonster::goHome swmpbb2a 0,191",
            "--command",
            "sendEvent 0 swmpbb2a KQAttackEvent Connor 0 100000 0,300",
            "--updates",
            "390",
            "--frames",
            "12000",
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
    for (name, stats, xp, corpse) in [
        (
            "fiend2",
            "[220, 35, 35, 31, 7500]",
            7500,
            "a corpse Sf_death.KQS",
        ),
        (
            "swmpbb2a",
            "[120, 25, 25, 11, 10000]",
            10000,
            "no corpse (no shape for this kind)",
        ),
    ] {
        assert!(
            viewer.contains(&format!("test --command move {name}"))
                && fight.contains(&format!("{name}: fights (rule 2): stats {stats}")),
            "{name} was not staged with its installed stats; inspect {}",
            dir.display()
        );
        let death = format!("{name}: dies (rule 30): kind Normal");
        assert_eq!(
            fight.matches(&death).count(),
            1,
            "{name} did not die exactly once; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("{name}: drops: nothing (rule 31)"))
                .count(),
            1,
            "{name} has an unexpected drop; inspect {}",
            dir.display()
        );
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {xp}");
        assert_eq!(
            fight.matches(&finish).count(),
            1,
            "{name} did not award experience once; inspect {}",
            dir.display()
        );
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&finish) && line.contains(corpse)),
            "{name} has the wrong Normal corpse; inspect {}",
            dir.display()
        );
    }
    assert!(
        fight.lines().any(|line| line.contains("fiend2: Do (Attack")
            && line.contains("hits Connor")
            && line.contains("reach 2000")
            && line.contains("hand damage 35, Normal")),
        "the fiend made no authored melee hit; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by fiend2, Normal: D 35,"))
        .collect();
    assert!(
        !hits.is_empty(),
        "the fiend did not reach Connor; inspect {}",
        dir.display()
    );
    for hit in hits {
        let health = hit.rsplit_once("health ").unwrap().1;
        let (before, after) = health.split_once(" -> ").unwrap();
        assert!(
            after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
            "the fiend did not lower health: {hit}"
        );
    }
    assert!(
        fight.find("fiend2: Do (Attack").unwrap()
            < fight.find("fiend2: dies (rule 30): kind Normal").unwrap(),
        "the fiend died before its checked attack; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| line.contains("swmpbb2a: Do (Attack")
            && line.contains("hits Connor")
            && line.contains("hand damage 25, Normal")),
        "the released bubble made no authored melee hit; inspect {}",
        dir.display()
    );
    let bubble_hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by swmpbb2a, Normal: D 25,"))
        .collect();
    assert!(
        !bubble_hits.is_empty(),
        "the bubble did not reach Connor; inspect {}",
        dir.display()
    );
    for hit in bubble_hits {
        let health = hit.rsplit_once("health ").unwrap().1;
        let (before, after) = health.split_once(" -> ").unwrap();
        assert!(
            after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
            "the bubble did not lower health: {hit}"
        );
    }
    assert!(
        fight.find("swmpbb2a: Do (Attack").unwrap()
            < fight.find("swmpbb2a: dies (rule 30): kind Normal").unwrap(),
        "the bubble died before its checked attack; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("Connor: hit by world") && !fight.contains("god mode is on"),
        "world damage or god mode obscured combat; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
