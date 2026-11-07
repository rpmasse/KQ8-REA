//! Inventory rules 10/12: a blank quest cell is silent on right click,
//! while left click can return the quest item currently held as a pointer.
use serde_json::Value;
use std::{fs, path::{Path, PathBuf}, process::Command};

#[test]
#[ignore = "requires GOG assets, graphics and earned KQ8_INVENTORY_SLOT"]
fn empty_quest_cell_is_silent_on_right_click_and_returns_item_on_left_click() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let source = PathBuf::from(std::env::var_os("KQ8_INVENTORY_SLOT").expect("set KQ8_INVENTORY_SLOT to GoodSkullReload"));
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let start: Value = serde_json::from_slice(&fs::read(&source).unwrap()).unwrap();
    assert_eq!(start["label"], "GoodSkullReload");
    assert!(start["journey"]["hand"].is_null());
    let carried = start["journey"]["items"].as_array().unwrap();
    assert!(carried.iter().any(|item| item["kind"] == "INVITEM_GoodSkull" && item["quantity"] == 1));
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/inventory-empty-slot/{nonce}"));
    let slots = root.join("slots");
    fs::create_dir_all(&slots).unwrap();
    fs::copy(&source, slots.join("slot-1.json")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
        .args(["--sound", "off", "--menu-input", "down,enter,release,enter,release",
            "--key", "right:IDCTL_INVBAR_QUEST2,10",
            "--key", "left:IDCTL_INVBAR_QUEST2,15",
            "--key", "right:IDCTL_INVBAR_QUEST7,20",
            "--key", "left:IDCTL_INVBAR_QUEST7,25",
            "--key", "right:IDCTL_INVBAR_QUEST7,30",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=EmptyQuestSlot:enter:release:escape,45",
            "--key", "menu:up:enter:release:left:enter:release,60",
            "--frames", "900"])
        .output().unwrap();
    fs::write(root.join("stderr.log"), output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", root.display());
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    let descriptions: Vec<_> = items.lines().filter(|line| line.contains("describes it:")).collect();
    assert_eq!(descriptions.len(), 1, "only the occupied right-click should describe an item; inspect {}", root.display());
    assert!(descriptions[0].contains("INVITEM_GoodSkull"));
    let held = items.lines().find(|line| line.contains("left:IDCTL_INVBAR_QUEST2") && line.contains("now the item in hand")).unwrap();
    let returned = items.lines().find(|line| line.contains("left:IDCTL_INVBAR_QUEST7") && line.contains("it is the item in hand: put back")).unwrap();
    assert!(held.contains("INVITEM_GoodSkull") && returned.contains("INVITEM_GoodSkull"));
    let gui = fs::read_to_string(logs.join("gui.log")).unwrap();
    assert_eq!(gui.lines().filter(|line| line.contains("right:IDCTL_INVBAR_QUEST7")
        && line.contains(": nothing there; nothing")).count(), 2);
    let saved: Value = serde_json::from_slice(&fs::read(slots.join("slot-2.json")).unwrap()).unwrap();
    assert_eq!(saved["label"], "EmptyQuestSlot");
    assert!(saved["journey"]["hand"].is_null());
    for field in ["items", "stats", "flags", "god_mode", "ai_off"] {
        assert_eq!(saved["journey"][field], start["journey"][field], "{field}");
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
