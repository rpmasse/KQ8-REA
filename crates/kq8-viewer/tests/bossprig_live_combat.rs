//! A normal AI fight with the installed Daventry Bossprig.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn bossprig_attacks_dies_and_leaves_its_authored_rewards() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/bossprig-live-combat/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args([
            "--windowed", "--sound", "off", "--god", "--world", "Daventry",
            "--at", "289284,190000,0", "--key", "popup:autohelp,0",
            // Keep the installed actor and its AI, but use the known open
            // encounter ground rather than its wooded original placement.
            "--command", "KQObject::delete goblin22,5",
            "--command", "move bossprig 0 289284 192527 13770 0 0 0,5",
            "--command", "KQMonster::inventory Connor INVITEM_SkelKingSword 1,20",
            "--command", "KQConner::setDoMode 0,35",
            "--swing", "bossprig,80", "--swing", "bossprig,120",
            "--swing", "bossprig,160", "--swing", "bossprig,200",
            "--swing", "bossprig,240", "--swing", "bossprig,280",
            "--swing", "bossprig,320", "--swing", "bossprig,360",
            "--swing", "bossprig,400", "--updates", "500", "--frames", "2100",
        ])
        .output().unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let read = |name: &str| fs::read_to_string(logs.join(name)).unwrap();
    let viewer = read("viewer.log");
    let fight = read("fight.log");
    let characters = read("characters.log");
    let items = read("items.log");
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("bossprig: alarmed: sees Connor")
        && fight.contains("Connor: hit by bossprig, Normal, damage 23")
        && fight.contains("hand (Some(\"INVITEM_SkelKingSword\"))"),
        "the original actor or its normal attack did not engage; inspect {}", dir.display());
    let hits: Vec<_> = fight.lines().filter(|line| line.contains("bossprig: hit by Connor, Normal:")).collect();
    assert!(hits.len() >= 9
        && hits.iter().any(|line| line.contains(" -> 0;"))
        && hits.iter().any(|line| line.contains("health 0 -> -") && line.contains("below 0: it dies")),
        "ordinary sword hits did not pass through the zero-health boundary; inspect {}", dir.display());
    assert_eq!(fight.matches("bossprig: dies (rule 30): kind Normal").count(), 1,
        "boss death must start once; inspect {}", dir.display());
    let drops = fight.lines().find(|line| line.contains("bossprig: drops (rule 31;")).unwrap();
    for kind in ["INVITEM_sacredwater x1", "INVITEM_SilverCoins x50",
                 "INVITEM_Mushroom x1", "INVITEM_LeatherArmor x1"] {
        assert!(drops.contains(kind), "missing {kind}; inspect {}", dir.display());
    }
    assert!(items.contains("(INVITEM_LeatherArmor) flight ends")
        && viewer.contains("corpse of bossprig: spdeath.kqs drawn")
        && viewer.contains("skin pairs [(\"test2.bmp\", \"test3.bmp\")], 1 material(s) changed")
        && viewer.contains("bossprig died (fighting.md rule 32): applied: bossprig deleted"),
        "drops, skinned corpse or actor removal failed; inspect {}", dir.display());
    assert_eq!(fight.matches("bossprig: its death ended (rule 32): Connor gets its experience 7:").count(), 1,
        "the reward must be granted once; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
