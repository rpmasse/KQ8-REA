//! Four placed Swamp types share one controlled original-data encounter.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_plant_rises_and_small_creatures_follow_their_combat_rules() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/swamp-plant-small-families/{nonce}"
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
            "move crvplant8 0 166500 72961 5355 0 0 0,20",
            "--command",
            "sendEvent 0 crvplant8 KQAttackEvent Connor 0 100000 0,180",
            "--command",
            "move frog0 0 167100 72961 5355 0 0 0,220",
            "--command",
            "move weed2 0 167000 72961 5355 0 0 0,220",
            "--command",
            "move bat1 0 166800 72961 5355 0 0 0,220",
            "--command",
            "sendEvent 0 frog0 KQAttackEvent Connor 0 100000 0,290",
            "--command",
            "sendEvent 0 weed2 KQAttackEvent Connor 0 100000 0,290",
            "--command",
            "sendEvent 0 bat1 KQAttackEvent Connor 0 100000 0,290",
            "--updates",
            "400",
            "--frames",
            "12000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "viewer failed; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("crvplant8: rises (rule 30):")
            && characters
                .lines()
                .any(|line| line.contains("crvplant8: Do (Attack") && line.contains("hits Connor")),
        "plant did not rise and land a natural hit; inspect {}",
        dir.display()
    );
    let plant_hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by crvplant8, Normal: D 35,"))
        .collect();
    assert!(
        !plant_hits.is_empty(),
        "no plant health loss; inspect {}",
        dir.display()
    );
    for hit in plant_hits {
        let (before, after) = hit
            .rsplit_once("health ")
            .unwrap()
            .1
            .split_once(" -> ")
            .unwrap();
        assert!(after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap());
    }
    for (name, kind, stats, alarm, reach, xp) in [
        (
            "crvplant8",
            "crvplant",
            "[70, 35, 35, 26, 2000]",
            "15000",
            "2000",
            2000,
        ),
        ("frog0", "frog", "[25, 5, 5, 3, 500]", "1.5", "1500", 500),
        ("weed2", "weedtype", "[5, 5, 5, 2, 1]", "0", "0", 1),
        ("bat1", "bat", "[25, 5, 5, 3, 500]", "0", "0", 500),
    ] {
        let placement = format!("[{name}] KQMonster:");
        assert!(
            characters.lines().any(|line| line.contains(&placement)
                && line.contains(&format!("(type {kind},"))
                && line.contains(&format!("alarm radius {alarm};"))
                && line.contains(&format!("attack distance {reach};"))),
            "placement/radii mismatch for {name}; inspect {}",
            dir.display()
        );
        assert!(
            viewer.contains(&format!("test --command move {name}"))
                && fight.contains(&format!("{name}: fights (rule 2): stats {stats}"))
        );
        if name != "crvplant8" {
            assert!(
                !fight.contains(&format!("{name}: attack (rule 7"))
                    && !fight.contains(&format!("Connor: hit by {name},")),
                "small creature {name} attacked; inspect {}",
                dir.display()
            );
        }
        assert_eq!(
            fight
                .matches(&format!("{name}: dies (rule 30): kind Normal"))
                .count(),
            1
        );
        assert_eq!(
            fight
                .matches(&format!("{name}: drops: nothing (rule 31)"))
                .count(),
            1
        );
        if name == "weed2" {
            assert!(
                !fight.contains("weed2: its death ended (rule 32):"),
                "repeating weed Die unexpectedly finalized; inspect {}",
                dir.display()
            );
            continue;
        }
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience {xp}");
        assert_eq!(
            fight.matches(&finish).count(),
            1,
            "death did not finish for {name}; inspect {}",
            dir.display()
        );
        assert!(fight
            .lines()
            .any(|line| line.contains(&finish)
                && line.contains("no corpse (no shape for this kind)")));
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
