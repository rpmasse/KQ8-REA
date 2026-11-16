//! A complete Daventry fight using the original goblin's location and live AI.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_goblin_attacks_and_dies_in_a_live_crossbow_fight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/goblin-live-combat/{nonce}"));
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
            "Daventry",
            "--at",
            "289284,190000,0",
            "--key",
            "popup:autohelp,0",
            "--command",
            "KQMonster::inventory Connor INVITEM_SmCrossbow 1,20",
            "--command",
            "KQConner::setDoMode 1,35",
            "--aim",
            "goblin22,60",
            "--aim",
            "goblin22,100",
            "--aim",
            "goblin22,140",
            "--aim",
            "goblin22,165",
            "--command",
            "move Connor 0 192171 198091 6700 0 0 0,200",
            "--aim",
            "Wizard,250",
            "--updates",
            "300",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let sound = fs::read_to_string(logs.join("sound.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("goblin22: alarmed: sees Connor"),
        "the original goblin did not engage; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("taken, none (None) -> ranged (Some(\"INVITEM_SmCrossbow\"))"),
        "the crossbow did not ready; inspect {}",
        dir.display()
    );
    let incoming: Vec<_> = fight
        .lines()
        .filter(|line| {
            line.contains("Connor: hit by goblin22, Normal:") && line.contains("; health ")
        })
        .collect();
    assert!(
        !incoming.is_empty()
            && incoming
                .iter()
                .all(|line| line.contains("loss ") && !line.contains("loss 0")),
        "the goblin did not injure Connor; inspect {}",
        dir.display()
    );
    let outgoing: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("goblin22: hit by Connor, Projectile:"))
        .collect();
    assert_eq!(
        outgoing.len(),
        4,
        "Connor's four authored shots did not contact; inspect {}",
        dir.display()
    );
    assert!(
        outgoing.first().unwrap().contains("health 15 -> ")
            && outgoing.last().unwrap().contains("below 0: it dies"),
        "the goblin did not lose its original health; inspect {}",
        dir.display()
    );
    assert!(sound.lines().any(|line| line.contains("start #")
        && line.contains("gblnouch.aud (data/Daventry/Sound/GBLNOUCH.AUD")
        && line.contains("group 0 placed at (")
        && line.contains("[goblin22's ouch sound (fighting.md rule 24)]")),
        "the installed goblin ouch sound was not dispatched at its position; inspect {}", dir.display());
    assert_eq!(
        fight
            .matches("goblin22: dies (rule 30): kind Projectile")
            .count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("goblin22: its death ended (rule 32): Connor gets its experience 3:")
            .count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("goblin22: drops (rule 31;")
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "the fight did not finish normally; inspect {}",
        dir.display()
    );
    assert!(doors.contains("the ranged weapon is ready, the talk pick meets Wizard")
        && fight.matches("Connor: do (conscbow.anm/attack").count() == 4,
        "the ranged Wizard click did not take the plain interaction path; inspect {}",
        dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
