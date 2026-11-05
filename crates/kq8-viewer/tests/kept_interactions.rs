//! Persist actual Daventry door/chest instances between two Windows game
//! processes and collect the chest's original three crystals once.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn daventry_door_and_open_chest_continue_after_loading() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/interaction-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &out.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(
            out.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        viewer
    };
    // Use the game's move command to put the pub chest within reach of
    // Connor's starting position. The test can then use the normal chest
    // click (including its distance check) without walking through town.
    run(
        "save",
        &[
            "--world",
            "Daventry",
            "--frames",
            "230",
            "--console",
            "move smlchst3 0 277744 198295 13633,2",
            "--console",
            "sendEvent 0 Condoor KQDoorEvent 1,2",
            "--click",
            "smlchst3,5",
            "--key",
            "menu:down:down:enter:release:enter:release:escape:type=Pub:enter:release:escape,30",
        ],
    );
    let slot = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let saved = slot(1);
    let objects = &saved["kept"]["daventry"]["interactions"];
    assert_eq!(objects["doors"]["condoor"]["state"], "Opened");
    let chest = &objects["chests"]["smlchst3"];
    assert_eq!(chest["state"], "Open");
    assert_eq!(chest["loot"]["quantity"], 3);
    assert_eq!(chest["loot"]["name"], chest["remembered"]);
    let loot = chest["remembered"].as_str().unwrap();
    let viewer = run("restore", &["--menu-input","down,enter,release,enter,release","--frames","500",
        "--click","smlchst3,6","--click","smlchst3,12",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=PubAfter:enter:release:escape,24"]);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    assert!(viewer.lines().any(|s| s.starts_with("doors: [Condoor] at") && s.contains("(state opened)")));
    assert!(viewer.contains("Save screen -> Save(2, \"PubAfter\")"), "inspect {}", root.join("restore").display());
    let after = slot(2);
    let objects = &after["kept"]["daventry"]["interactions"];
    assert_eq!(objects["doors"]["condoor"]["state"], "Opened");
    assert_eq!(objects["chests"]["smlchst3"]["state"], "Empty");
    assert!(objects["chests"]["smlchst3"]["remembered"].is_null());
    let items = after["items"].as_array().unwrap();
    let matching: Vec<_> = items.iter().filter(|item| item["name"] == loot).collect();
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0]["place"]["Carried"], "Connor");
    assert_eq!(matching[0]["quantity"], 3);
}
