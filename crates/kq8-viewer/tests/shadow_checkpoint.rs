//! Script suppression survives a disk save without changing Graphics settings.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn scripted_shadow_suppression_survives_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/shadow-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        fs::write(dir.join("kq8-artifacts/Options.cs"), b"setShadowMask 15\n").unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(result.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        let preferences = kq8_core::options_menu::Settings::from_script(&fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap());
        assert_eq!(preferences.graphics.shadow_mask, 15, "the script changed Graphics preferences");
        viewer
    };
    let first = run("save", &["--world", "DeadCity", "--frames", "180",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--command", "allowShadows 0,30",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=NoShadows:enter:release:escape,50"]);
    assert!(first.contains("mask 15, Connor 0 pieces, 0 character shadows shown; allowShadows 0"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    assert_eq!(read(1)["shadows_allowed"], false);
    let second = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "220",
        "--command", "allowShadows 1,30",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ShadowsRestored:enter:release:escape,55"]);
    let (_, resumed) = second.split_once("save restored in DeadCity without replaying Begin").expect("save did not resume");
    assert!(resumed.contains("mask 15, Connor 0 pieces, 0 character shadows shown; allowShadows 0"), "suppression lost on load; inspect {}", root.display());
    let (_, enabled) = resumed.split_once("allowShadows 1").expect("enable command did not run");
    assert!(enabled.lines().any(|l| l.starts_with("shadow renderer: mask 15,") && !l.contains("Connor 0 pieces")
        && l.contains("1 character shadows shown; allowShadows 1")), "projected characters did not return; inspect {}", root.display());
    assert_eq!(read(2)["shadows_allowed"], true);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}
