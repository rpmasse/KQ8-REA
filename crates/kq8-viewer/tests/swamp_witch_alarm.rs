//! One GOG-data Swamp route links Witch speech and Fire with Henchman melee.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_witch_and_henchman_use_their_authored_attacks_and_deaths() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/swamp-witch-alarm/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Swamp",
            "--at",
            "159699,19000,0",
            "--sound",
            "off",
            "--windowed",
            "--command",
            "KQConner::setLevel 50000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 500000 0 0 0 0,2",
            "--command",
            "sendEvent 0 Swmpwich KQAttackEvent Connor 0 100000 0,90",
            "--command",
            "move Henchman 0 159699 20500 6017 0 0 0,130",
            "--command",
            "KQMonster::setHome Henchman 159699 20500 6017,130",
            "--command",
            "sendEvent 0 Henchman KQAttackEvent Connor 0 100000 0,260",
            "--updates",
            "330",
            "--frames",
            "10000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let traps = fs::read_to_string(logs.join("traps.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        characters
            .matches("Swmpwich: alarm line (rule 23):")
            .count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        talk.matches("SwmpwichTalk: queued 3000 4 0 24 1…2").count(),
        1,
        "inspect {}",
        dir.display()
    );
    let witch = talk
        .find("SwmpwichTalk: line #0 3000 4 0 24 1 starts: speaker Swmpwich")
        .unwrap_or_else(|| panic!("witch line: inspect {}", dir.display()));
    let reply = talk
        .find("SwmpwichTalk: line #1 3000 4 0 24 2 starts: speaker Connor")
        .unwrap_or_else(|| panic!("Connor reply: inspect {}", dir.display()));
    assert!(
        witch < reply,
        "wrong speech order: inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("Swmpwich: fights (rule 2): stats [800, 70, 70, 51, 50000]")
            && fight.lines().any(|line| line.contains("Swmpwich: bolt1")
                && line.contains("touches Connor")
                && line.contains("Fire"))
            && fight
                .lines()
                .any(|line| line.contains("Connor: hit by Swmpwich, Fire: D 70,")
                    && line.contains("; health ")
                    && !line.contains("loss 0")),
        "the Witch's installed Fire bolt did not damage Connor; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("Henchman: fights (rule 2): stats [850, 75, 75, 55, 50005]")
            && viewer.contains("test --command move Henchman")
            && fight
                .lines()
                .any(|line| line.contains("Henchman: Do (Attack")
                    && line.contains("hits Connor:")
                    && line.contains("hand damage 75, Normal"))
            && fight.lines().any(
                |line| line.contains("Connor: hit by Henchman, Normal: D 75,")
                    && line.contains("; health ")
                    && !line.contains("loss 0")
            ),
        "the placed Swamp Henchman did not land a Normal hit; inspect {}",
        dir.display()
    );
    for (name, experience) in [("Swmpwich", 50000), ("Henchman", 50005)] {
        assert_eq!(
            fight
                .matches(&format!("{name}: dies (rule 30): kind Normal"))
                .count(),
            1,
            "{name} did not die once; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("{name}: drops: nothing (rule 31)"))
                .count(),
            1,
            "{name} unexpectedly dropped an item; inspect {}",
            dir.display()
        );
        assert!(
            fight.lines().any(|line| line.contains(&format!(
                "{name}: its death ended (rule 32): Connor gets its experience {experience}"
            )) && line.contains("no corpse")),
            "{name} did not finish its no-corpse death; inspect {}",
            dir.display()
        );
    }
    assert!(
        traps.contains("swhnchmn.cs line 12: applied: moved Connor")
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "the Swamp Henchman death script did not hand off; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
