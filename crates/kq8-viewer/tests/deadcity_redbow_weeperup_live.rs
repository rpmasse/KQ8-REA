//! Two off-map Dead City ranged families fight and die in one original-data process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_red_bow_skeleton_and_raised_weeper() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/deadcity-redbow-weeperup-live/{nonce}"
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
            // The world script initially parks these installed actors off-map.
            // The already measured crossbow lane permits both ranged attacks.
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "move sneezy11 0 184200 45000 864 0 0 0,20",
            "--command",
            "sendEvent 0 sneezy11 KQAttackEvent Connor 0 100000 0,130",
            "--command",
            "move weepy 0 184200 45000 864 0 0 0,155",
            "--command",
            "sendEvent 0 weepy KQAttackEvent Connor 0 100000 0,300",
            "--updates",
            "380",
            "--frames",
            "8000",
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
    for name in ["sneezy11", "weepy"] {
        assert!(
            viewer.contains(&format!("test --command move {name}")),
            "{name} was not explicitly moved from its scripted placement; inspect {}",
            dir.display()
        );
        assert!(
            viewer.contains(&format!(
                "test --command sendEvent 0 {name} KQAttackEvent Connor 0 100000 0: Some(\"0\")"
            )),
            "{name}'s controlled death did not run; inspect {}",
            dir.display()
        );
    }
    assert!(
        fight.contains("sneezy11: fights (rule 2): stats [110, 15, 15, 10, 500]")
            && fight.contains("from type skelrdbw [60, 15, 10, 200]"),
        "the red bow skeleton's installed stats were not loaded; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("weepy: fights (rule 2): stats [100, 65, 65, 10, 500]")
            && fight.contains("from type weeperup [100, 25, 10, 500]"),
        "the raised Weeper's installed stats were not loaded; inspect {}",
        dir.display()
    );
    assert!(
        fight
            .lines()
            .any(|line| line.contains("sneezy11: Do (Attack")
                && line.contains("its part 'arrow' in frame")
                && line.contains("Prmonarw.kq")
                && line.contains("Projectile")),
        "the red bow skeleton did not fire its authored arrow; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| line.contains("sneezy11: bolt")
            && line.contains("touches Connor")
            && line.contains("a hit of 15, Projectile")),
        "the red bow arrow did not contact Connor; inspect {}",
        dir.display()
    );
    let bow_hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by sneezy11, Projectile: D 15,"))
        .collect();
    assert!(
        !bow_hits.is_empty(),
        "the bow made no health loss; inspect {}",
        dir.display()
    );
    for hit in bow_hits {
        let (before, after) = hit
            .rsplit_once("health ")
            .unwrap()
            .1
            .split_once(" -> ")
            .unwrap();
        assert!(
            before.parse::<i64>().unwrap() > after.parse::<i64>().unwrap()
                && after.parse::<i64>().unwrap() > 0,
            "Connor did not survive red bow damage: {hit}"
        );
    }
    assert!(
        fight.lines().any(|line| line.contains("weepy: Do (Attack")
            && line.contains("no part 'flsh' in frame")
            && line.contains("shootOffsets fallback")
            && line.contains("Prweeper.kq")),
        "the raised Weeper did not use its authored attack fallback; inspect {}",
        dir.display()
    );
    assert!(
        fight
            .lines()
            .any(|line| line.contains("weepy: bolt") && line.contains("starts drain on Connor")),
        "the raised Weeper did not link to Connor; inspect {}",
        dir.display()
    );
    let pulses: Vec<_> = fight
        .lines()
        .filter(|line| {
            line.contains("weepy: bolt")
                && line.contains("drain pulse")
                && line.contains("on Connor: health ")
        })
        .collect();
    assert_eq!(
        pulses.len(),
        12,
        "the raised Weeper's drain did not complete; inspect {}",
        dir.display()
    );
    let first: i64 = pulses[0].rsplit_once("health ").unwrap().1.parse().unwrap();
    let last: i64 = pulses[11]
        .rsplit_once("health ")
        .unwrap()
        .1
        .parse()
        .unwrap();
    assert!(
        first > last && last > 0,
        "the raised Weeper did not drain living Connor"
    );
    assert!(
        fight.lines().any(|line| line.contains("weepy: bolt")
            && line.contains("ends drain after 12 pulses; suckstop")),
        "the raised Weeper did not stop its drain; inspect {}",
        dir.display()
    );
    for (name, corpse) in [("sneezy11", "skbdead.kqs"), ("weepy", "weepdead.kqs")] {
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
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience 500");
        assert_eq!(fight.matches(&finish).count(), 1);
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&finish) && line.contains(&format!("a corpse {corpse}"))),
            "{name} has the wrong Normal corpse; inspect {}",
            dir.display()
        );
    }
    assert!(
        !fight.contains("god mode is on"),
        "god mode obscured damage; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
