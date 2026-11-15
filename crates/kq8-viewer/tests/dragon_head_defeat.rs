//! Unattended original-data check of the Dragon heads' two defeat routes.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn dragon_heads_rebirth_or_stay_defeated_by_equipped_sword() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/dragon-defeat/{nonce}"));
    for (label, commands, updates) in [
        ("temporary", vec!["--console", "kill draglhed,2"], "65"),
        ("permanent", vec![
            "--command", "KQMonster::inventory Connor INVITEM_FlameSword 1,2",
            "--command", "KQConner::setDoMode 0,8",
            "--console", "kill dragrhed,15",
        ], "30"),
        ("both-permanent", vec![
            "--command", "KQMonster::inventory Connor INVITEM_FlameSword 1,2",
            "--command", "KQConner::setDoMode 0,8",
            "--console", "kill draglhed,15",
            "--console", "kill dragrhed,35",
            "--command", "move Connor 0 182974 490884 30902 0 0 0,154",
            "--click", "dragdbox,155",
        ], "175"),
    ] {
        let run = root.join(label);
        fs::create_dir_all(&run).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&run).arg("--game").arg(&game)
            .args(["--world", "IceWorld", "--at", "182334,490176,30902",
                "--windowed", "--sound", "off", "--no-ai", "--god"])
            .args(&commands).args(["--updates", updates]).output().unwrap();
        fs::write(run.join("stderr.log"), &output.stderr).unwrap();
        let logs = run.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", run.display());
        assert!(fight.contains("draglhed: fights (rule 2): stats [600, 200, 200, 25, 0]")
            && fight.contains("dragrhed: fights (rule 2): stats [600, 200, 200, 25, 0]"),
            "heads did not load their authored stats; inspect {}", run.display());
        if label == "temporary" {
            assert!(fight.contains("draglhed: Dragon head Die: Normal, health 0, non-solid; lopt starts; rebirth due at"), "inspect {}", run.display());
            assert!(fight.contains("draglhed: Dragon head rebirth starts; health restored to 600"), "inspect {}", run.display());
            assert!(!fight.contains("draglhed: its death ended (rule 32)"), "ordinary death ran; inspect {}", run.display());
        } else if label == "permanent" {
            assert!(fight.contains("dragrhed: Dragon head Die: Normal, health 0, non-solid; synge starts; permanently defeated"), "inspect {}", run.display());
            assert!(!fight.contains("dragrhed: Dragon head rebirth starts"), "permanent head rebirthed; inspect {}", run.display());
        } else {
            assert!(fight.contains("dragbody: both heads permanently defeated: body enters Die, becomes non-solid and starts 'die'"),
                "Dragon body did not enter its death movement; inspect {}", run.display());
            assert!(fight.contains("dragbody: dragdbox.kq load result 0")
                && ["draglhed", "dragrhed", "dragwall"].iter().all(|n| fight.contains(&format!("dragbody: remove {n}: true"))),
                "Dragon finish did not make the box and remove the old actors; inspect {}", run.display());
            assert!(fight.contains("dragbody: Dragon body 'die' movement ended"),
                "Dragon body did not finish its authored death movement; inspect {}", run.display());
            assert!(doors.contains("loadKQ: dragdbox came in: 1 drawn part(s) shown; registered collision [\"object box\"]")
                && doors.contains("deleteObject dragwall: clickable object deleted"),
                "Dragon box or wall collision transition failed; inspect {}", run.display());
            assert!(doors.contains("click on dragdbox (test --click) -> Script")
                && doors.contains("dragwall.cs line 7: KQMonster::speak Connor to Connor: 6000 35 14 106"),
                "post-fight box did not use the wall's authored callback; inspect {}", run.display());
        }
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
