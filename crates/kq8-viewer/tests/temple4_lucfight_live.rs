//! The first authored altar cue starts a placed Lucfight encounter.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn altar_first_lucfight_hits_then_completes_its_death_script() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/temple4-lucfight-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--world",
            "temple4",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "sendEvent 0 altar KQCueEvent 2 6,5",
            "--command",
            "sendEvent 0 Lucfight1 KQAttackEvent Connor 0 100000 0,120",
            "--updates",
            "430",
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
    let traps = fs::read_to_string(logs.join("traps.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        characters.contains(
            "Lucfight1: altar.cs line 186: KQMonster::setHome: home (100032, 101700, 4660)"
        ) && traps.contains("altar.cs line 179: applied: moved Connor")
            && traps.contains("altar.cs line 185: applied: loadKQ wall.kq"),
        "the authored altar cue did not stage the encounter; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("Lucfight1: rises (rule 30)")
            && fight.contains("Lucfight1: fights (rule 2): stats [3500, 800, 800, 5, 75000]"),
        "the original placed type did not enter combat; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| {
            line.contains("Lucfight1: Do (Attack")
                && line.contains("hits Connor:")
                && line.contains("hand damage 800, Normal")
        }) && fight.lines().any(|line| {
            line.contains("Connor: hit by Lucfight1, Normal: D 800,")
                && line.contains("; health ")
                && !line.contains("loss 0")
        }),
        "Lucfight1 did not damage living Connor; inspect {}",
        dir.display()
    );
    assert!(
        traps.contains("lucfight.cs line 6: applied: wall deleted")
            && traps.contains("update 391: altar.cs line 563: applied: handsOff true")
            && traps.contains("update 391: altar.cs line 571: applied: Connor scripted Point"),
        "the delayed altar cue after Lucfight1's death did not run; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("Lucfight1: dies (rule 30): kind Normal")
            .count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("Lucfight1: drops: nothing (rule 31)").count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| {
            line.contains("Lucfight1: its death ended (rule 32): Connor gets its experience 75000")
                && line.contains("no corpse")
        }),
        "Lucfight1's ordinary death reward did not complete; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("god mode is on") && !fight.contains("Connor: dies"),
        "the controlled encounter did not leave Connor alive; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
