//! Two installed Gnome melee families in one controlled original-data run.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn gnome_spriggan_and_zombie_attack_and_die() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/gnome-spriggan-zombie-live/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Gnome",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--at",
            "100819,59000,0",
            "--command",
            "move Connor 0 100819 58700 17575 0 0 0,4",
            "--command",
            "deleteObject batmanta1 Rock2 door01a door01b,5",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "move sprigsurprise 0 100819 59600 17575 0 0 3.14,20",
            "--command",
            "sendEvent 0 sprigsurprise KQAttackEvent Connor 0 100000 0,165",
            "--command",
            "move zombiepit1 0 100819 59600 17575 0 0 3.14,220",
            "--command",
            "KQMonster::setHome zombiepit1 100819 59600 17575,220",
            "--command",
            "sendEvent 0 zombiepit1 KQAttackEvent Connor 0 100000 0,440",
            "--updates",
            "530",
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
        "viewer failed; inspect {}",
        dir.display()
    );
    for (name, stats, damage, reach, xp, corpse) in [
        (
            "sprigsurprise",
            "[230, 30, 30, 15, 5007]",
            30,
            1200,
            5007,
            "spdeath.kqs",
        ),
        (
            "zombiepit1",
            "[210, 78, 78, 2, 2502]",
            78,
            1100,
            2502,
            "zdead.kqs",
        ),
    ] {
        assert!(
            viewer.contains(&format!("test --command move {name}")),
            "relocation missing for {name}; inspect {}",
            dir.display()
        );
        assert!(
            fight.contains(&format!("{name}: fights (rule 2): stats {stats}")),
            "installed stats missing for {name}; inspect {}",
            dir.display()
        );
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&format!("{name}: Do (Attack"))
                    && line.contains("hits Connor")
                    && line.contains(&format!("reach {reach}"))
                    && line.contains(&format!("hand damage {damage}, Normal"))),
            "authored attack missing for {name}; inspect {}",
            dir.display()
        );
        let hits: Vec<_> = fight
            .lines()
            .filter(|line| line.contains(&format!("Connor: hit by {name}, Normal: D {damage},")))
            .collect();
        assert!(
            !hits.is_empty(),
            "health hit missing for {name}; inspect {}",
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
        let death = format!("{name}: dies (rule 30): kind Normal");
        assert_eq!(
            fight.matches(&death).count(),
            1,
            "death count for {name}; inspect {}",
            dir.display()
        );
        assert!(fight.find(&format!("{name}: Do (Attack")).unwrap() < fight.find(&death).unwrap());
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {xp}");
        assert_eq!(
            fight.matches(&finish).count(),
            1,
            "XP count for {name}; inspect {}",
            dir.display()
        );
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&finish) && line.contains(&format!("a corpse {corpse}"))),
            "corpse mismatch for {name}; inspect {}",
            dir.display()
        );
    }
    assert!(
        characters.contains("zombiepit1: alarmed: seen in visible PreBirth")
            || characters.contains("zombiepit1: status: Birth, active"),
        "Zombie Birth did not activate; inspect {}",
        dir.display()
    );
    let sprig_drops: Vec<_> = items
        .lines()
        .filter(|line| line.contains("sprigsurprise: drops (rule 31;"))
        .collect();
    assert_eq!(
        sprig_drops.len(),
        1,
        "spriggan drops; inspect {}",
        dir.display()
    );
    assert!(sprig_drops[0].contains("INVITEM_SilverCoins x65"));
    assert_eq!(
        fight
            .matches("zombiepit1: drops: nothing (rule 31)")
            .count(),
        1
    );
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
