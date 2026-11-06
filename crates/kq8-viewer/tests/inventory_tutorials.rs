//! The original candle and watermill scripts give items in normal play.
#[path = "support/pickup_help.rs"]
mod pickup_help;
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn candle_and_watermill_rope_clicks_show_their_own_delayed_help() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/rope-tutorial/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .env("KQ8_SAVE_DIR", dir.join("slots"))
        .arg("--game")
        .arg(&game)
        .args([
            "--menu-input",
            "enter,release,enter,release",
            "--sound",
            "off",
            "--no-ai",
            "--key",
            "popup:help=144,0",
            "--console",
            "move Connor 0 201801 184793 9125 0 0 0,300",
            "--click",
            "candle,307",
            "--key",
            "popup:help=132,0",
            "--console",
            "move Connor 0 187750 227800 10350 0 0 0,390",
            "--hold",
            "1,0,0,390,393",
            "--click",
            "ropenhok,393",
            "--console",
            "inv,402",
            "--key",
            "popup:help=128,0",
            "--updates",
            "495",
            "--frames",
            "2400",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &out.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(
        out.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    let traps = fs::read_to_string(logs.join("traps.log"))
        .unwrap()
        .to_lowercase();
    let events = fs::read_to_string(logs.join("doors.log"))
        .unwrap()
        .to_lowercase();
    assert!(
        traps.contains("miltrap2 on connor")
            && traps.contains("set daventry::mill::ontrap2 = 'true'")
            && events.contains("script ropenhok.cs ropenhok connoraction do do"),
        "inspect {}",
        dir.display()
    );
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(
        events.contains("script candle.cs candle connoraction"),
        "inspect {}",
        dir.display()
    );
    pickup_help::check(&items, &viewer, "INVITEM_Candle", 132, 45, "INVENTORY:");
    pickup_help::check(
        &items,
        &viewer,
        "INVITEM_RopeAndHook",
        128,
        68,
        "ROPE AND HOOK:",
    );
    assert!(items.contains("INVITEM_RopeAndHook added 0.3 s ago: +10 experience"));
    let console = fs::read_to_string(logs.join("console.log"))
        .unwrap()
        .to_lowercase();
    assert!(
        console
            .lines()
            .any(|l| l.trim_start().starts_with("ropeandhook") && l.contains("x1")),
        "inspect {}",
        dir.display()
    );
    assert!(console
        .lines()
        .any(|l| l.trim_start().starts_with("candle") && l.contains("x1")));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
