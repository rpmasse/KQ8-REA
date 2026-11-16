//! Unattended original-data route for a ranged Swamp slime's attack and retreat.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_slime_shoots_then_keeps_distance_and_respects_its_leash() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/swamp-slime-retreat/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let slots = dir.join("slots");
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
        .args(["--world", "Swamp", "--at", "168074,72961,0",
            "--god", "--sound", "off", "--windowed",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=Retreat:enter:release:escape,50",
            "--updates", "120", "--frames", "3000"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(fight.lines().any(|line| line.contains("slime1: fights (rule 2)")
        && line.contains("likelyToFlee 0.75; tooClose 0.4 farEnough 3")
        && line.contains("ranged with INVITEM_SlmSlime")), "original slime data: inspect {}", dir.display());
    let shot = fight.lines().find(|line| line.contains("slime1: Do")
        && line.contains("shoots bolt1 (Prslime.kq"))
        .unwrap_or_else(|| panic!("original ranged shot: inspect {}", dir.display()));
    assert!(fight.lines().any(|line| line.contains("slime1: bolt1")
        && line.contains("touches Connor")), "projectile impact: inspect {}", dir.display());
    let start = characters.lines().find(|line| line.contains("slime1: keeping distance starts: chance")
        && line.contains("tooClose 2000"))
        .unwrap_or_else(|| panic!("retreat entry: inspect {}", dir.display()));
    let stop = characters.lines().find(|line| line.contains("slime1: keeping distance ends:")
        && line.contains("beyond 15000"))
        .unwrap_or_else(|| panic!("retreat stop: inspect {}", dir.display()));
    let give_up = characters.lines().find(|line| line.contains("slime1: gives up:")
        && line.contains("beyond its chase radius 6000"))
        .unwrap_or_else(|| panic!("chase leash: inspect {}", dir.display()));
    let update = |line: &str| -> u64 { line.strip_prefix("update ").unwrap()
        .split(':').next().unwrap().parse().unwrap() };
    assert!(update(shot) < update(start) && update(start) < update(stop)
        && update(stop) < update(give_up), "wrong shot/retreat/leash order: inspect {}", dir.display());
    assert!(viewer.contains("Save screen -> Save(1, \"Retreat\")"), "retreat was not saved; inspect {}", dir.display());
    let slot: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let slime = slot["kept"]["swamp"]["characters"].as_array().unwrap().iter()
        .find(|actor| actor["name"] == "slime1").expect("placed slime missing from saved world");
    assert_eq!(slime["character"]["state"], "Move");
    assert_eq!(slime["character"]["walk"]["run"], true);
    assert!(slime["character"]["walk"]["steer"].is_array());
    assert_eq!(slime["character"]["ai"]["active"], true);
    assert_eq!(slime["character"]["ai"]["keeping_distance"], true);
    assert_eq!(slime["character"]["fight"]["must_flee"], false);
    assert!(slot["kept"]["swamp"]["active_list"].as_array().unwrap().iter().any(|name| name == "slime1"));

    let restored_dir = dir.join("restore");
    fs::create_dir_all(&restored_dir).unwrap();
    let restored = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&restored_dir).env("KQ8_SAVE_DIR", &slots)
        .arg("--game").arg(&game)
        .args(["--windowed", "--sound", "off", "--god",
            "--menu-input", "down,enter,release,enter,release",
            "--updates", "100", "--frames", "2500"])
        .output().unwrap();
    fs::write(restored_dir.join("stderr.log"), &restored.stderr).unwrap();
    let logs = restored_dir.join("kq8-artifacts/logs");
    let restored_viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let resumed = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(restored.status.success() && restored_viewer.contains("RESULT: PASS")
        && restored_viewer.contains("save restored in Swamp without replaying Begin"),
        "retreat save did not load; inspect {}", restored_dir.display());
    let resumed_move = resumed.lines().find(|line| line.contains("slime1: status: Move, active"))
        .unwrap_or_else(|| panic!("saved retreat did not resume; inspect {}", restored_dir.display()));
    let resumed_stop = resumed.lines().find(|line| line.contains("slime1: keeping distance ends:")
        && line.contains("beyond 15000"))
        .unwrap_or_else(|| panic!("saved retreat did not end; inspect {}", restored_dir.display()));
    let resumed_give_up = resumed.lines().find(|line| line.contains("slime1: gives up:")
        && line.contains("beyond its chase radius 6000"))
        .unwrap_or_else(|| panic!("saved chase leash did not run; inspect {}", restored_dir.display()));
    assert!(update(resumed_move) < update(resumed_stop)
        && update(resumed_stop) < update(resumed_give_up),
        "restored retreat/stop/leash order changed; inspect {}", restored_dir.display());
    assert!(!resumed.lines().any(|line| line.contains("slime1: keeping distance starts:")
        && update(line) < update(resumed_stop)),
        "retreat restarted instead of continuing; inspect {}", restored_dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
