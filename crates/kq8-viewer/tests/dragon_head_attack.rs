//! Unattended original-data check of both IceWorld Dragon head attacks.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn dragon_heads_alarm_and_deliver_bite_and_fire_cues() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/dragon-attack/{nonce}"));
    fs::create_dir_all(&root).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root).arg("--game").arg(&game)
        .args(["--world", "IceWorld", "--at", "182334,490176,30902",
            "--windowed", "--sound", "off", "--god", "--console", "kill draglhed,90", "--updates", "220"])
        .output().unwrap();
    fs::write(root.join("stderr.log"), &output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let effects = fs::read_to_string(logs.join("effects.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", root.display());
    assert!(characters.contains("draglhed: joins the active list")
        && characters.contains("dragrhed: alarmed: told by its team member draglhed"),
        "head sight/team alarm missing; inspect {}", root.display());
    assert!(!characters.contains("dragbody: joins the active list"), "body contact is not studied; inspect {}", root.display());
    assert!(characters.lines().filter(|line| line.contains("draglhed: joins the active list")).count() >= 2
        && fight.contains("draglhed: Dragon head rebirth ended; original collision"),
        "temporarily defeated head did not rejoin after rebirth; inspect {}", root.display());
    for head in ["draglhed", "dragrhed"] {
        assert!(fight.contains(&format!("{head}: Dragon head Do (Attack at phase"))
            && fight.contains(&format!("{head}: Dragon head Do (attack2 at phase")),
            "{head} did not deliver both authored Do cues; inspect {}", root.display());
    }
    assert!(fight.contains("within 3000; 200 Normal damage")
        && fight.contains("within 6000; 200 Fire damage"),
        "bite/Fire radius or kind missing; inspect {}", root.display());
    assert!(viewer.contains("effects: 2 Dragon fire emitter recipes prepared")
        && viewer.contains("effects: 2 emitters running, 375 particles"),
        "fire recipes were not prepared separately from startup emitters; inspect {}", root.display());
    for (head, fire, file) in [("draglhed", "draglfire", "draglfir.kq"), ("dragrhed", "dragrfire", "dragrfir.kq")] {
        assert!(effects.contains(&format!("[{head}]: started {fire} from {file} with 100 particles")),
            "{head} did not start its authored emitter; inspect {}", root.display());
        let positions: Vec<&str> = effects.lines().filter(|line| line.contains(&format!("[{fire}] follows {head} part Some(\"fire\") frame")))
            .filter_map(|line| line.split("loc ").nth(1)).collect();
        assert!(positions.len() >= 2 && positions[0] != positions[1],
            "{fire} did not follow changing animated part positions; inspect {}", root.display());
        let visible = effects.lines().filter(|line| line.contains(&format!("[{fire}] follows {head} part")))
            .filter_map(|line| line.rsplit(", ").next()?.split_whitespace().next()?.parse::<usize>().ok())
            .any(|count| count > 0);
        assert!(visible && effects.contains(&format!("draw [{fire}] activity Some(Near), draw groups true")),
            "{fire} did not have visible particles within the draw distance; inspect {}", root.display());
        assert!(effects.contains(&format!("[{fire}] part or target ended: dying"))
            && effects.contains(&format!("[{fire}] all particles expired: removed")),
            "{fire} did not finish its particles and leave the world; inspect {}", root.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
