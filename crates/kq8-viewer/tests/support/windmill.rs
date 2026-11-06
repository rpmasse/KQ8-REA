//! Resume a Wizard save using the ordinary windmill input route.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn earn_axe(game: &Path, root: &Path) -> PathBuf {
    let slots = root.join("slots");
    let replay = root.join("windmill");
    fs::create_dir_all(&replay).unwrap();
    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_windmill.json");
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&replay).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(game)
        .args(["--sound", "off", "--menu-input", "down,enter,release,enter,release"])
        .args(["--key", "popup:help=131,0", "--key", "popup:help=142,0"])
        .arg("--walk-route").arg(route)
        .args(["--key", "menu:down:down:down:enter:release:enter:release:escape:type=Hillside:enter:release:escape,838",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Windmill:enter:release:escape,1800",
            "--key", "menu:up:enter:release:left:enter:release,1810", "--frames", "6500"])
        .output().unwrap();
    fs::write(replay.join("stderr.log"), output.stderr).unwrap();
    let log = fs::read_to_string(replay.join("kq8-artifacts/logs/viewer.log")).unwrap();
    assert!(
        output.status.success()
            && log.contains("RESULT: PASS")
            && log.contains("Save screen -> Save(3, \"Windmill\")"),
        "inspect {}",
        replay.display()
    );
    slots.join("slot-3.json")
}
