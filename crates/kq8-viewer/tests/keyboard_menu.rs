//! Two fresh processes. Configure keys at the title, exercise them through
//! gameplay input, then rebind one of those actions while the game is paused.
use kq8_core::keyboard_menu::Bindings;
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires the original game and a graphics device"]
fn remapped_physical_keys_reach_connor_and_the_interface_after_restart() {
    let path = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let original_path = kq8_data::GameDir::open(&path)
        .unwrap()
        .find("movekeys.kq")
        .unwrap();
    let original = fs::read(&original_path).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/menu-tests/keyboard-{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", dir.join("saves"))
            .arg("--game")
            .arg(&path)
            .args(["--sound", "off"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join(format!("{name}.stderr.log")), &output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        fs::write(dir.join(format!("{name}.viewer.log")), &log).unwrap();
        assert!(
            output.status.success() && log.contains("RESULT: PASS"),
            "see {}",
            dir.display()
        );
        log
    };
    let startup=run("startup",&["--menu-input",
        "down,enter,release,click=310x145,click=344x181,bind=up,bind=r,click=344x407,bind=q,click=360x450,click=315x451",
        "--frames","66"]);
    let prefs = dir.join("kq8-artifacts/movekeys.kq");
    let saved = Bindings::read(&fs::read(&prefs).unwrap()).unwrap();
    assert_eq!((saved.0[4], saved.0[12]), (b'R', b'Q'));
    for s in [
        "keyboard: opened original movekey.gui (13 actions)",
        "keyboard: key in use; retry",
        "keyboard: assigned row 4 to Windows key 82",
        "keyboard: assigned row 12 to Windows key 81",
        "keyboard: returned to Options",
        "options: back to main",
    ] {
        assert!(startup.contains(s), "{s}");
    }
    let world=run("world",&["--world","Daventry",
        "--key","physical:end,9","--key","physical:r,17","--key","physical:space,23","--key","physical:q,29",
        "--key","menu:down:down:down:enter:release:click=310x145:click=344x181:bind=g:escape:escape:escape,36",
        "--key","physical:r,44","--key","physical:g,51","--frames","280"]);
    let (before, after) = world
        .split_once("keyboard: assigned row 4 to Windows key 71")
        .expect("rebound Run while paused");
    assert_eq!(before.matches(": run switch ").count(), 1);
    assert_eq!(after.matches(": run switch ").count(), 1);
    // Connor starts with Run enabled. R turns it off and the reassigned G
    // turns it on again; End and the vacated R have no effect.
    assert!(before.contains(": run switch off") && after.contains(": run switch on"));
    assert!(
        !world.contains("key KeyG: lifted"),
        "configured G also triggered a viewer shortcut"
    );
    let gui = fs::read_to_string(dir.join("kq8-artifacts/logs/gui.log")).unwrap();
    assert_eq!(
        gui.lines()
            .filter(|s| s.contains("Space: intrup.aud") || s.contains("Space: intrdown.aud"))
            .count(),
        1
    );
    assert_eq!(
        Bindings::read(&fs::read(&prefs).unwrap()).unwrap().0[4],
        b'G'
    );
    assert_eq!(fs::read(original_path).unwrap(), original);
}
#[test]
#[ignore = "requires GOG assets and graphics"]
fn original_selection_masks_advance_on_each_menu_screen() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/menu-masks/{nonce}"));
    for (page, input) in [
        ("Main", None),
        ("Options", Some("down,enter,release")),
        ("Graphics", Some("down,enter,release,enter,release")),
        ("Keyboard", Some("down,enter,release,click=310x145")),
    ] {
        let dir = root.join(page);
        fs::create_dir_all(&dir).unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        cmd.current_dir(&dir).arg("--game").arg(&game).args(["--sound", "off", "--menu", "--frames", "85"]);
        if let Some(keys) = input { cmd.args(["--menu-input", keys]); }
        let output = cmd.output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && log.contains("RESULT: PASS"), "{}", dir.display());
        assert!(log.contains(&format!("menu: {page} mask advanced on its 100 ms clock")), "{}", dir.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}
