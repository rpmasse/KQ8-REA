//! The installed Hammer hits once, returns, and releases its menu lock after catch.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn dead_city_hammer_hits_returns_and_allows_a_save_after_catch() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/hammer-hit-return/{nonce}"));
    let slots = root.join("slots");
    let run = root.join("run");
    fs::create_dir_all(&run).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&run).env("KQ8_SAVE_DIR", &slots)
        .arg("--game").arg(&game)
        .args(["--windowed", "--sound", "off", "--no-ai", "--god",
            "--world", "DeadCity"])
        // Dead City's start script gives a crossbow at update 0; the
        // ordinary give at update 5 replaces it with Azriel's Hammer.
        .args(["--command", "give AzrielHammer 1,5",
            "--command", "move skeleton1 0 184500 43091 864 0 0 0,5",
            "--command", "KQConner::setDoMode 1,25",
            "--aim", "skeleton1,50",
            "--key", "menu,63",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=AfterHammer:enter:release:escape,80",
            "--updates", "110"])
        .output().unwrap();
    fs::write(run.join("stdout.log"), &output.stdout).unwrap();
    fs::write(run.join("stderr.log"), &output.stderr).unwrap();
    let logs = run.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        run.display()
    );
    assert_eq!(
        viewer
            .matches("menu: gameplay menu request blocked by handsOff at Connor update 63")
            .count(),
        1
    );
    assert!(
        viewer.contains("Save screen -> Save(1, \"AfterHammer\")"),
        "the menu stayed locked after catch"
    );

    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert_eq!(fight.matches("shoots hammer1 (Prhammer.kq").count(), 1);
    assert_eq!(
        fight
            .matches("hammer1 update 2 (out): touches skeleton1")
            .count(),
        1
    );
    assert_eq!(
        fight
            .matches("skeleton1: hit by Connor, Projectile:")
            .count(),
        1
    );
    assert_eq!(fight.matches("hammer1: turns back (its touch)").count(), 1);
    assert_eq!(
        fight
            .matches("hammer1: the way back ends (touching Connor)")
            .count(),
        1
    );
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert_eq!(
        connor
            .matches("throw: 'conhamr.anm/catch' ended: handsOff off")
            .count(),
        1
    );

    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let skeleton = saved["kept"]["deadcity"]["characters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|actor| actor["name"] == "skeleton1")
        .unwrap();
    assert_eq!(skeleton["character"]["fight"]["stats"][0], 23);
    assert!(saved["bolts"]["bolts"].as_array().unwrap().is_empty());
    assert_eq!(saved["bolts"]["bolts_shot"], 1);
    assert_eq!(saved["journey"]["weapon_ready"], "INVITEM_AzrielHammer");
    assert_eq!(saved["weapon_state"]["set_mode"], "Ranged");
    assert_eq!(saved["panels"]["hands_off"], false);
}
