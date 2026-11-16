//! A live goblin attack and injured health survive a fresh-process disk load.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn injured_goblin_resumes_its_attack_and_loses_the_fight_after_reload() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/goblin-live-checkpoint/{nonce}"
    ));
    let slots = root.join("slots");

    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--windowed", "--sound", "off", "--key", "popup:autohelp,0"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stdout.log"), output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, fight)
    };

    let (before, first_fight) = run("save", &[
        "--world", "Daventry", "--at", "289284,190000,0",
        "--command", "KQMonster::inventory Connor INVITEM_SmCrossbow 1,20",
        "--command", "KQConner::setDoMode 1,35",
        "--aim", "goblin22,60", "--aim", "goblin22,100",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=GoblinLive:enter:release:escape,110",
        "--updates", "120",
    ]);
    assert!(
        before.contains("Save screen -> Save(1, \"GoblinLive\")"),
        "fight was not saved; inspect {}",
        root.display()
    );
    assert_eq!(
        first_fight
            .matches("goblin22: hit by Connor, Projectile:")
            .count(),
        2
    );
    assert!(
        first_fight.contains("goblin22: attack (rule 7, standing):")
            && !first_fight.contains("goblin22: dies")
    );

    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let actors = saved["kept"]["daventry"]["characters"].as_array().unwrap();
    let goblin = actors
        .iter()
        .find(|actor| actor["name"] == "goblin22")
        .unwrap();
    assert_eq!(goblin["character"]["fight"]["stats"][0], 6);
    assert_eq!(goblin["animation"]["movement"], "attack");
    assert_eq!(goblin["character"]["ai"]["active"], true);
    assert_eq!(goblin["character"]["ai"]["alarmed"], true);
    assert_eq!(saved["journey"]["stats"]["stats"][0], 8);
    assert_eq!(saved["journey"]["weapon_ready"], "INVITEM_SmCrossbow");

    let (after, fight) = run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--aim",
            "goblin22,35",
            "--aim",
            "goblin22,85",
            "--updates",
            "140",
        ],
    );
    assert!(
        after.contains("save restored in Daventry without replaying Begin"),
        "slot did not load; inspect {}",
        root.display()
    );
    assert!(
        fight.contains("update 7: goblin22: Do (Attack") && fight.contains("health 8 -> 7"),
        "saved goblin attack did not resume; inspect {}",
        root.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("goblin22: hit by Connor, Projectile:"))
        .collect();
    assert_eq!(
        hits.len(),
        2,
        "two restored fight shots did not land; inspect {}",
        root.display()
    );
    assert!(
        hits[0].contains("health 6 -> 1") && hits[1].contains("health 1 -> -4"),
        "the injured health did not persist; inspect {}",
        root.display()
    );
    assert_eq!(
        fight
            .matches("goblin22: dies (rule 30): kind Projectile")
            .count(),
        1
    );
    assert_eq!(fight.matches("goblin22: drops (rule 31;").count(), 1);
    assert_eq!(
        fight
            .matches("goblin22: its death ended (rule 32): Connor gets its experience 3:")
            .count(),
        1,
        "death did not finish once; inspect {}",
        root.display()
    );
    assert!(!fight.contains("god mode is on") && !fight.contains("Connor: dies"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
