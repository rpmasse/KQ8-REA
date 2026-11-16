//! The original outhouse door reveals the placed sprdump before its fight.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn outhouse_dump_enables_placed_spriggan_fight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/daventry-outhouse-live/{nonce}"
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
            "--key",
            "popup:autohelp,0",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "move Connor 0 122871 286239 10965 0 0 0,3",
            "--click",
            "outdoor,8",
            "--command",
            "sendEvent 0 sprdump KQAttackEvent Connor 0 100000 0,165",
            "--updates",
            "235",
            "--frames",
            "11000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "viewer failed; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("[sprdump] KQMonster: from vol:Daventry/kq/MONSTERS.KQ monster51")
            && characters.contains("alarm radius 0; home radius 0; chase radius 0")
            && fight.contains("sprdump: fights (rule 2): stats [140, 15, 15, 15, 23]"),
        "placed actor or initial combat rules differ; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("click on outdoor (test --click) -> Script([\"outdoor\", \"ConnorAction\", \"do\", \"do\"])")
            && doors.contains("setHidden sprdump: placed object hidden false")
            && doors.contains("KQMonster::setState special dump")
            && doors.contains("script sprdump.cs sprdump EndLoop dump"),
        "door-to-dump script chain did not complete; inspect {}",
        dir.display()
    );
    for (name, value) in [
        ("KQMonster::setAlarmRadius", "alarm radius 8000"),
        ("KQMonster::setHomeRadius", "home radius 8000"),
        ("KQMonster::setChaseRadius", "chase radius 8000"),
    ] {
        assert!(
            characters.lines().any(|line| {
                line.contains("update 88: sprdump:") && line.contains(name) && line.contains(value)
            }),
            "{name} was not enabled by EndLoop dump; inspect {}",
            dir.display()
        );
    }
    assert!(!fight.lines().any(|line| {
        line.contains("sprdump: attack (rule 7")
            && line
                .split_once("update ")
                .and_then(|(_, rest)| rest.split_once(':'))
                .and_then(|(number, _)| number.parse::<u64>().ok())
                .is_some_and(|update| update < 88)
    }));
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by sprdump, Normal: D 15,"))
        .collect();
    assert!(
        !hits.is_empty(),
        "no live sprdump hit; inspect {}",
        dir.display()
    );
    for hit in hits {
        let (before, after) = hit
            .rsplit_once("health ")
            .unwrap()
            .1
            .split_once(" -> ")
            .unwrap();
        assert!(after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap());
    }
    let death = fight
        .lines()
        .filter(|line| line.contains("sprdump: dies (rule 30): kind Normal"))
        .collect::<Vec<_>>();
    assert_eq!(death.len(), 1);
    assert!(
        characters.contains("sprdump: status: Wait, active")
            && death[0].contains("off the active list; gravity off; not solid"),
        "Normal death did not remove an active fighter; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("sprdump: drops: nothing (rule 31)").count(),
        1
    );
    let reward = "sprdump: its death ended (rule 32): Connor gets its experience 23";
    assert_eq!(fight.matches(reward).count(), 1);
    assert!(fight
        .lines()
        .any(|line| { line.contains(reward) && line.contains("a corpse spd_deth.kqs") }));
    assert!(!fight.contains("Connor: hit by world") && !fight.contains("god mode is on"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
