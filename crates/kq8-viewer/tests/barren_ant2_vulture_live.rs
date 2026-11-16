//! Barren's second ant type and vulture complete combat in one original-data process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_barren_ant2_attack_and_two_family_deaths() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/barren-ant2-vulture-live/{nonce}"
    ));
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
            "155453,54700,0",
            // Allow an ordinary ant attack without a terrain or neighboring ant
            // ending the route before both controlled death movements finish.
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "sendEvent 0 ant11 KQAttackEvent Connor 0 100000 0,100",
            "--command",
            "sendEvent 0 vulture2 KQAttackEvent Connor 0 100000 0,130",
            "--updates",
            "240",
            "--frames",
            "5000",
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
        viewer.contains("test --command KQConner::setLevel 1000: Some(\"0\")")
            && viewer.contains(
                "test --command sendEvent 0 ant11 KQAttackEvent Connor 0 100000 0: Some(\"0\")"
            )
            && viewer.contains(
                "test --command sendEvent 0 vulture2 KQAttackEvent Connor 0 100000 0: Some(\"0\")"
            ),
        "the original-data scenario did not run; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("ant11: fights (rule 2): stats [220, 40, 40, 41, 5000]")
            && fight.contains("from type ant2 [200, 40, 40, 5000]"),
        "the placed second ant type has wrong stats; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("vulture2: fights (rule 2): stats [25, 5, 5, 3, 500]")
            && fight.contains("from type vulture [5, 5, 2, 500]"),
        "the placed vulture has wrong stats; inspect {}",
        dir.display()
    );
    assert!(characters.lines().any(|line| line.contains("[vulture2] KQMonster:")
        && line.contains("alarm radius 0;")
        && line.contains("attack distance 0;"))
        && characters.contains("vulture2: status: Move, active")
        && !fight.contains("vulture2: attack (rule 7,")
        && !fight.contains("Connor: hit by vulture2,"),
        "the active Vulture did not follow its zero-range data; inspect {}", dir.display());
    assert!(
        fight.contains("ant11: attack (rule 7,")
            && fight
                .lines()
                .any(|line| line.contains("ant11: Do (Attack") && line.contains("hits Connor:")),
        "normal ant AI did not attack; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by ant11, Normal: D 40,"))
        .collect();
    assert!(
        !hits.is_empty(),
        "the ant caused no normal damage; inspect {}",
        dir.display()
    );
    for hit in hits {
        let health = hit.rsplit_once("health ").unwrap().1;
        let (before, after) = health.split_once(" -> ").unwrap();
        let before: i64 = before.parse().unwrap();
        let after: i64 = after.parse().unwrap();
        assert!(
            before > after && after > 0,
            "Connor did not survive the ant hit: {hit}"
        );
    }
    assert_eq!(
        fight.matches("ant11: dies (rule 30): kind Normal").count(),
        1
    );
    assert_eq!(items.matches("ant11: drops (rule 31;").count(), 1);
    assert!(
        items
            .lines()
            .any(|line| line.contains("ant11: drops (rule 31;")
                && line.contains("INVITEM_crystal x1")),
        "the ant did not drop its Crystal; inspect {}",
        dir.display()
    );
    let ant_end = "ant11: its death ended (rule 32): Connor gets its experience 5000";
    assert_eq!(fight.matches(ant_end).count(), 1);
    assert!(
        fight
            .lines()
            .any(|line| line.contains(ant_end)
                && line.contains("no corpse (no shape for this kind)")),
        "the ant has the wrong corpse; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("vulture2: dies (rule 30): kind Normal")
            .count(),
        1
    );
    assert_eq!(
        fight.matches("vulture2: drops: nothing (rule 31)").count(),
        1
    );
    let vulture_end = "vulture2: its death ended (rule 32): Connor gets its experience 500";
    assert_eq!(fight.matches(vulture_end).count(), 1);
    assert!(
        fight.lines().any(|line| line.contains(vulture_end)
            && line.contains("no corpse (no shape for this kind)")),
        "the vulture has the wrong corpse; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("god mode is on"),
        "god mode obscured damage; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
