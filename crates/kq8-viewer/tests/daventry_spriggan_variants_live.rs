//! Two distinct placed Daventry Spriggan combat variants in one original-data run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn daventry_second_bow_and_melee_spriggan_fight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/daventry-spriggan-variants/{nonce}"
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
            "move sprigan 0 289284 193500 13770 0 0 0,25",
            "--command",
            "sendEvent 0 sprigan KQAttackEvent Connor 0 100000 0,155",
            "--command",
            "move sprigan1 0 287855 190454 13770 0 0 0,190",
            "--command",
            "KQMonster::setHome sprigan1 287855 190454 13770,190",
            "--command",
            "sendEvent 0 sprigan1 KQAttackEvent Connor 0 100000 0,270",
            "--updates",
            "340",
            "--frames",
            "10000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
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
    for (name, kind, damage) in [("sprigan", "Projectile", 13), ("sprigan1", "Normal", 7)] {
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
    let bow_finish = "sprigan: its death ended (rule 32): Connor gets its experience 12";
    assert_eq!(fight.matches(bow_finish).count(), 1);
    assert!(fight
        .lines()
        .any(|line| line.contains(bow_finish) && line.contains("a corpse spbdeath.kqs")));
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
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
