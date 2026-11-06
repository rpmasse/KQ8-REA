//! Earn the village shield with ordinary controls, save, then use it after a fresh launch.
#[path = "support/pickup_help.rs"]
mod pickup_help;
use serde_json::Value;
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn cottage_shield_is_visible_earned_saved_and_used_on_normal() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = manifest.join(format!("../../kq8-artifacts/cottage-shield/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, inputs: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off"])
            .args(inputs)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, fs::read_to_string(logs.join("items.log")).unwrap())
    };
    let slot = |number: u32| -> Value {
        let path = slots.join(format!("slot-{number}.json"));
        serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
        )
        .unwrap()
    };

    let route = manifest.join("tests/routes/daventry_cottage_shield.json");
    let (walk, items) = run("new-game", &[
        "--menu-input", "enter,release,enter,release",
        "--walk-route", route.to_str().unwrap(),
        "--key", "popup:help=144,0", "--key", "popup:help=141,0",
        "--key", "popup:help=127,0", "--key", "popup:help=131,0",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=VillageShield:enter:release:escape,920",
        "--key", "menu:up:enter:release:left:enter:release,934", "--frames", "4500",
    ]);
    for name in ["saradoor", "NoName20"] {
        assert!(
            walk.lines()
                .any(|line| line.contains(&format!("test --aim {name} (camera"))
                    && line.contains(&format!("hits {name} first"))),
            "{name}: inspect {}",
            root.display()
        );
    }
    assert!(walk.contains("walk-route at Shield beside the east wall"));
    assert!(items.contains("IN REACH: picked up; Connor's INVITEM_Invulnerable 0 -> 1"));
    pickup_help::check(
        &items,
        &walk,
        "INVITEM_Invulnerable",
        131,
        45,
        "HEALTH ITEMS:",
    );
    let earned = slot(1);
    assert_eq!(earned["label"], "VillageShield");
    assert_eq!(earned["journey"]["ai_off"], false);
    assert_eq!(earned["journey"]["god_mode"], false);
    let shield = earned["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "INVITEM_Invulnerable")
        .unwrap();
    assert_eq!(shield["name"], "NoName20");
    assert_eq!(shield["quantity"], 1);
    assert_eq!(shield["place"]["Carried"], "Connor");

    let (reload, timer) = run("dose", &[
        "--menu-input", "down,enter,release,enter,release", "--key", "5,12",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ShieldActive:enter:release:escape,45",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ShieldEnded:enter:release:escape,340",
        "--key", "menu:up:enter:release:left:enter:release,355", "--frames", "2200",
    ]);
    assert!(reload.contains("save restored in Daventry without replaying Begin"));
    assert!(timer.contains("update 12: bar click left on NoName20 (slot 5, INVITEM_Invulnerable"));
    assert!(timer.contains("the shield: protection 0 -> 50"));
    assert!(timer.contains("update 312: NoName20: its time is up"));
    assert!(timer.contains("the shield ends: protection 50 -> 0"));
    let active = slot(2);
    assert_eq!(active["label"], "ShieldActive");
    assert_eq!(active["journey"]["stats"]["stats"][3], 50);
    assert_eq!(
        active["journey"]["potions"][0][0]["kind"],
        "INVITEM_Invulnerable"
    );
    assert_eq!(active["journey"]["potions"][0][0]["duration"], 20);
    let ended = slot(3);
    assert_eq!(ended["label"], "ShieldEnded");
    assert_eq!(ended["journey"]["stats"]["stats"][3], 0);
    assert!(ended["journey"]["potions"].as_array().unwrap().is_empty());
    assert!(ended["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["kind"] != "INVITEM_Invulnerable"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
