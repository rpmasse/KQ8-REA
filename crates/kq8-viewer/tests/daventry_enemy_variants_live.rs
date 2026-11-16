//! Three placed Daventry combat variants in one original-data run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn daventry_zombie_melee_and_second_bow_spriggan_fight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/daventry-enemy-variants/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Daventry",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--at",
            "289284,188000,0",
            "--key",
            "popup:autohelp,0",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "move Connor 0 287855 189554 13770 0 0 0,20",
            "--command",
            "deleteObject goblin22,21",
            "--command",
            "move zograv2 0 287855 190454 13770 0 0 0,25",
            "--command",
            "KQMonster::setHome zograv2 287855 190454 13770,25",
            "--command",
            "sendEvent 0 zograv2 KQAttackEvent Connor 0 100000 0,130",
            "--command",
            "move sprigan1 0 287855 190454 13770 0 0 0,175",
            "--command",
            "KQMonster::setHome sprigan1 287855 190454 13770,175",
            "--command",
            "sendEvent 0 sprigan1 KQAttackEvent Connor 0 100000 0,260",
            "--command",
            "move sprigan 0 289284 193500 13770 0 0 0,320",
            "--command",
            "sendEvent 0 sprigan KQAttackEvent Connor 0 100000 0,465",
            "--updates",
            "590",
            "--frames",
            "18000",
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
        "viewer failed; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("sprigan: fights (rule 2): stats [130, 8, 13, 15, 12]")
            && fight
                .lines()
                .any(|line| line.contains("sprigan: Do (Attack")
                    && line.contains("shoots bolt")
                    && line.contains("Prscbolt.kq"))
            && fight
                .lines()
                .any(|line| line.contains("sprigan: bolt") && line.contains("touches Connor")),
        "second bow Spriggan did not land its authored projectile; inspect {}",
        dir.display()
    );
    for (name, kind, damage) in [
        ("zograv2", "Normal", 3),
        ("sprigan1", "Normal", 7),
        ("sprigan", "Projectile", 13),
    ] {
        let hits: Vec<_> = fight
            .lines()
            .filter(|line| line.contains(&format!("Connor: hit by {name}, {kind}: D {damage},")))
            .collect();
        assert!(
            !hits.is_empty(),
            "no live hit from {name}; inspect {}",
            dir.display()
        );
        for hit in hits {
            let (before, after) = hit
                .rsplit_once("health ")
                .unwrap()
                .1
                .split_once(" -> ")
                .unwrap();
            assert!(
                after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
                "{name} did not lower Connor's health: {hit}"
            );
        }
    }
    for (name, stats) in [
        ("zograv2", "[10, 3, 3, 2, 2]"),
        ("sprigan", "[130, 8, 13, 15, 12]"),
        ("sprigan1", "[25, 7, 7, 5, 7]"),
    ] {
        assert!(
            viewer.contains(&format!("test --command move {name}"))
                && fight.contains(&format!("{name}: fights (rule 2): stats {stats}")),
            "installed {name} placement did not load; inspect {}",
            dir.display()
        );
        let death = format!("{name}: dies (rule 30): kind Normal");
        assert_eq!(
            fight.matches(&death).count(),
            1,
            "Normal death count for {name}; inspect {}",
            dir.display()
        );
    }
    for (name, xp, corpse) in [
        ("zograv2", 2, "zdead.kqs"),
        ("sprigan1", 7, "spdeath.kqs"),
        ("sprigan", 12, "spbdeath.kqs"),
    ] {
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {xp}");
        assert_eq!(fight.matches(&finish).count(), 1);
        assert!(fight
            .lines()
            .any(|line| line.contains(&finish) && line.contains(&format!("a corpse {corpse}"))));
    }
    assert_eq!(
        fight.matches("sprigan: drops: nothing (rule 31)").count(),
        1
    );
    let coins: Vec<_> = items
        .lines()
        .filter(|line| line.contains("sprigan1: drops (rule 31;"))
        .collect();
    assert_eq!(
        coins.len(),
        1,
        "spriggan coin drop; inspect {}",
        dir.display()
    );
    assert!(coins[0].contains("INVITEM_SilverCoins x6"));
    let mushrooms: Vec<_> = items
        .lines()
        .filter(|line| line.contains("zograv2: drops (rule 31;"))
        .collect();
    assert_eq!(mushrooms.len(), 1, "zombie drop; inspect {}", dir.display());
    assert_eq!(mushrooms[0].matches("INVITEM_Mushroom x1").count(), 2);
    assert!(
        characters.contains("zograv2: rises (rule 30):")
            && characters.contains("zograv2: 'birth' ended in Wait:")
            && characters
                .lines()
                .any(|line| line.contains("zograv2: Do (Attack") && line.contains("hits Connor"))
    );
    assert!(
        fight.lines().any(|line| line.contains("sprigan: fights (rule 2): stats [130, 8, 13, 15, 12]")
            && line.contains("from type sprgnbowtype2 [30, 10, 5, 7] + extra [100, -2, 10, 5]")
            && line.contains("holds INVITEM_SmCrossbow (LongWeapon: ranged +5")),
        "the placed bow Spriggan did not add its held crossbow to ranged damage; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| line.contains("bossprig: fights (rule 2): stats [230, 23, 23, 32, 7]")
            && line.contains("from type sprigan [30, 10, 5, 7] + extra [200, 13, 25, 0]")
            && line.contains("holds INVITEM_LeatherArmor (hand +0, health +0, protection +2)")),
        "the placed boss Spriggan did not add its held armour to protection; inspect {}",
        dir.display()
    );
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
