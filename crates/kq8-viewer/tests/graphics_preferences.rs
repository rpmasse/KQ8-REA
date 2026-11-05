//! Local graphics preferences override the GOG defaults on a fresh launch.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn local_resolution_lighting_and_shadow_settings_reach_world_loading() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/graphics-preferences/{nonce}"));
    for (name, local, screen, dynamic, mask) in [
        ("installed", None, "640x480", "useDynamicLighting 0", "world mask 1"),
        ("local", Some(b"set800mode 1\nuseDynamicLighting 1\nsetShadowMask 15\n".as_slice()),
            "800x600", "useDynamicLighting 1", "world mask 15"),
    ] {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        if let Some(script) = local { fs::write(dir.join("kq8-artifacts/Options.cs"), script).unwrap(); }
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game)
            .args(["--world", "DeadCity", "--frames", "80", "--sound", "off", "--no-ai", "--god"])
            .output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        assert!(viewer.contains(&format!("screen {screen}")) && viewer.contains(dynamic),
            "resolution or terrain light map ignored the options; inspect {}", logs.display());
        let shadows = fs::read_to_string(logs.join("shadows.log")).unwrap();
        assert!(shadows.contains(mask), "shadows ignored the options; inspect {}", logs.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original, "the GOG installation was modified");
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn graphics_menu_saves_choices_for_a_fresh_world_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/graphics-menu/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game).args(["--sound", "off"]).args(args)
            .output().unwrap();
        fs::write(dir.join(format!("{name}.stderr.log")), &output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        fs::write(dir.join(format!("{name}.viewer.log")), &log).unwrap();
        assert!(output.status.success() && log.contains("RESULT: PASS"), "inspect {}", dir.display());
        log
    };
    let menu = run("menu", &["--menu-input", "down,enter,release,enter,click=350x190,click=350x232,click=350x263,click=525x387,mouse-up,click=310x450,click=315x451", "--frames", "70"]);
    assert!(menu.contains("graphics: opened original Graphics.gui") && menu.contains("graphics: returned to Options; saved local preferences"), "inspect {}", dir.display());
    let saved = kq8_core::options_menu::Settings::from_script(&fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap());
    assert!(saved.graphics.high_resolution && saved.graphics.dynamic_lighting);
    assert_eq!((saved.graphics.shadow_mask, saved.graphics.brightness), (0, 100));
    assert_eq!(saved.graphics.mode, "D3D-display");
    let world = run("world", &["--world", "DeadCity", "--frames", "80", "--no-ai", "--god"]);
    assert!(world.contains("screen 800x600") && world.contains("useDynamicLighting 1"), "inspect {}", dir.display());
    assert!(fs::read_to_string(dir.join("kq8-artifacts/logs/shadows.log")).unwrap().contains("world mask 0"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}
