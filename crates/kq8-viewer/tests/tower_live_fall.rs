//! Controlled original-data Tower Skeleton fall with ordinary hits and damage.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_tower_fall_drops_mail_and_returns_control() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/tower-live-fall/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(&game)
        .args([
            "--windowed", "--sound", "off", "--world", "DeadCity",
            "--command", "move Connor 0 183616 123639 9714 0 0 1.57,100",
            "--command", "sendEvent 0 Skeltowr KQCueEvent 201,101",
            "--aim", "Skeltowr,212",
            // This controlled geometry fixture keeps the same GOG actor.
            // It does not establish an earned route to the tower edge.
            "--command", "move Skeltowr 0 188400 123776 9593 0 0 -1.57,268",
            "--command", "move Connor 0 186600 123776 9593 0 0 1.57,268",
            "--aim", "Skeltowr,270",
            "--updates", "520", "--frames", "2700",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let read = |name: &str| fs::read_to_string(logs.join(name)).unwrap();
    let viewer = read("viewer.log");
    let characters = read("characters.log");
    let fight = read("fight.log");
    let items = read("items.log");
    let traps = read("traps.log");
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("[Skeltowr] KQTowerSkeleton: from vol:DeadCity/kq/MONSTERS.KQ")
        && characters.contains("Skeltowr: rises (rule 30)")
        && fight.contains("Skeltowr: tower Cue 201: enabled its real alarm radius"),
        "the installed actor did not enter combat; inspect {}", dir.display());
    assert_eq!(fight.matches("Connor: Do (conaxe.anm/").count(), 2,
        "two authored Small Axe swings did not complete; inspect {}", dir.display());
    assert_eq!(fight.matches("Skeltowr: hit by Connor, Normal:").count(), 2,
        "both ordinary hits did not land; inspect {}", dir.display());
    assert!(fight.matches("Connor: hit by Skeltowr, Normal:").count() >= 1
        && !fight.contains("god mode is on") && !fight.contains("Connor: dies"),
        "normal incoming damage was not active or Connor died; inspect {}", dir.display());
    assert!(fight.contains("Skeltowr: reactback: no support 400 behind, starts the fall")
        && fight.contains("Skeltowr: dies (rule 30): kind Normal, 'teeter' starts")
        && fight.contains("tower line 2000 16 12 22 1")
        && fight.contains("tower line 2000 16 12 22 2")
        && fight.contains("tower teeter ended, next Some(\"die\")")
        && fight.contains("tower die ended, next Some(\"die2\")")
        && fight.contains("tower die2 ended, next Some(\"fallend\")")
        && fight.contains("tower fallend ended, next None"),
        "the scripted fall did not finish; inspect {}", dir.display());
    assert_eq!(fight.matches("Skeltowr: drops (rule 31;").count(), 1);
    assert!(fight.contains("INVITEM_ChainMailShirt x1")
        && items.contains("(INVITEM_ChainMailShirt) flight ends"),
        "the chain mail was not dropped; inspect {}", dir.display());
    assert!(traps.contains("tower skeleton: applied: handsOff true")
        && traps.matches("tower fall: applied: camera moved").count() == 3
        && traps.contains("tower skeleton: applied: the camera follows Connor again")
        && traps.contains("tower skeleton: applied: handsOff false"),
        "camera or controls did not return; inspect {}", dir.display());
    assert_eq!(fight.matches("Skeltowr: its death ended (rule 32): Connor gets its experience 4000:").count(), 1);
    assert!(viewer.contains("Skeltowr died (fighting.md rule 32): applied: Skeltowr deleted"),
        "the tower actor remained after its reward; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
