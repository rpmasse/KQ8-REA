//! Four placed Dead City melee variants share one original-data combat run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_dead_city_zero_hit_key_zombie_birth_and_rat_death() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/deadcity-ordinary-variants-live/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "DeadCity",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            // Use the measured crossbow lane for four different placed types.
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "move skeleton1 0 183300 45000 864 0 0 0,20",
            "--command",
            "sendEvent 0 skeleton1 KQAttackEvent Connor 0 100000 0,95",
            "--command",
            "move Skfight1 0 182650 45000 864 0 0 0,100",
            "--command",
            "sendEvent 0 Skfight1 KQAttackEvent Connor 0 100000 0,165",
            "--command",
            "move zombie5 0 182900 45000 864 0 0 0,170",
            "--command",
            "sendEvent 0 zombie5 KQAttackEvent Connor 0 100000 0,260",
            "--command",
            "move rat5 0 183000 45000 864 0 0 0,265",
            "--command",
            "sendEvent 0 rat5 KQAttackEvent Connor 0 100000 0,290",
            "--updates",
            "360",
            "--frames",
            "8000",
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
    for (name, kind, stats) in [
        ("skeleton1", "skeleton2", "[60, 0, 0, 15, 200]"),
        ("Skfight1", "skeleton3", "[105, 15, 15, 15, 500]"),
        ("zombie5", "zombie3", "[310, 33, 33, 2, 252]"),
        ("rat5", "rat2", "[5, 5, 5, 2, 51]"),
    ] {
        assert!(viewer.contains(&format!("test --command move {name}"))
            && viewer.contains(&format!("test --command sendEvent 0 {name} KQAttackEvent Connor 0 100000 0: Some(\"0\")")),
            "{name}'s controlled route did not run; inspect {}", dir.display());
        assert!(
            fight.lines().any(|line| line
                .contains(&format!("{name}: fights (rule 2): stats {stats}"))
                && line.contains(&format!("from type {kind} "))),
            "{name}'s original combat type was not loaded; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("{name}: dies (rule 30): kind Normal"))
                .count(),
            1,
            "{name} did not die exactly once; inspect {}",
            dir.display()
        );
    }
    let zero_hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by skeleton1, Normal: D 0,"))
        .collect();
    assert!(
        !zero_hits.is_empty()
            && fight
                .lines()
                .any(|line| line.contains("skeleton1: Do (Attack")
                    && line.contains("hits Connor:")
                    && line.contains("hand damage 0, Normal")),
        "the zero-damage skeleton did not make a normal hit cue; inspect {}",
        dir.display()
    );
    for hit in zero_hits {
        let (before, after) = hit
            .rsplit_once("health ")
            .unwrap()
            .1
            .split_once(" -> ")
            .unwrap();
        assert_eq!(
            before, after,
            "the zero-damage skeleton lowered health: {hit}"
        );
    }
    for (name, damage) in [("Skfight1", 15), ("zombie5", 33)] {
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&format!("{name}: Do (Attack"))
                    && line.contains("hits Connor:")
                    && line.contains(&format!("hand damage {damage}, Normal"))),
            "{name} did not land its ordinary melee attack; inspect {}",
            dir.display()
        );
        let marker = format!("Connor: hit by {name}, Normal: D {damage},");
        let hits: Vec<_> = fight
            .lines()
            .filter(|line| line.contains(&marker))
            .collect();
        assert!(
            !hits.is_empty(),
            "{name} caused no health loss; inspect {}",
            dir.display()
        );
        for hit in hits {
            let (before, after) = hit
                .rsplit_once("health ")
                .unwrap()
                .1
                .split_once(" -> ")
                .unwrap();
            let before: i64 = before.parse().unwrap();
            let after: i64 = after.parse().unwrap();
            assert!(
                before > after && after > 0,
                "{name} did not damage living Connor: {hit}"
            );
        }
    }
    assert!(
        characters.contains("zombie5: rises (rule 30): Connor 679 away")
            && characters.contains("zombie5: joins the active list"),
        "the hidden zombie did not complete its Birth; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("rat5: Do (Attack"),
        "the zero-distance ambient rat unexpectedly attacked; inspect {}",
        dir.display()
    );
    for (name, experience, corpse) in [
        ("skeleton1", 200, "skmdead.kqs"),
        ("Skfight1", 500, "skmdead.kqs"),
        ("zombie5", 252, "zdead.kqs"),
        ("rat5", 51, "rt_dead.kqs"),
    ] {
        let finish =
            format!("{name}: its death ended (rule 32): Connor gets its experience {experience}");
        assert_eq!(
            fight.matches(&finish).count(),
            1,
            "{name} did not grant its experience once; inspect {}",
            dir.display()
        );
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&finish) && line.contains(&format!("a corpse {corpse}"))),
            "{name} has the wrong Normal corpse; inspect {}",
            dir.display()
        );
    }
    for (name, items_expected) in [
        (
            "skeleton1",
            ["INVITEM_Strength x1", "INVITEM_SilverCoins x7"].as_slice(),
        ),
        ("Skfight1", ["INVITEM_RoundKey x1"].as_slice()),
        ("zombie5", ["INVITEM_mushroom x1"].as_slice()),
    ] {
        let drops: Vec<_> = items
            .lines()
            .filter(|line| line.contains(&format!("{name}: drops (rule 31;")))
            .collect();
        assert_eq!(
            drops.len(),
            1,
            "{name} did not drop once; inspect {}",
            dir.display()
        );
        for item in items_expected {
            assert!(
                drops[0].contains(item),
                "{name} did not drop {item}; inspect {}",
                dir.display()
            );
        }
    }
    assert_eq!(fight.matches("rat5: drops: nothing (rule 31)").count(), 1);
    assert!(
        !fight.contains("god mode is on"),
        "god mode obscured combat; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
