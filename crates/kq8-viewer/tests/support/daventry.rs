//! Daventry campaign replays beginning in the original Normal New Game.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn game() -> PathBuf {
    fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap()
}

pub fn test_root(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/{name}/{nonce}"))
}

pub fn earn_magic_map(game: &Path, root: &Path) {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let route = manifest.join("tests/routes/daventry_wizard.json");
    fs::create_dir_all(root).unwrap();
    let slots = root.join("slots");
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(root).env("KQ8_SAVE_DIR", &slots)
        .arg("--game").arg(game).args(["--sound", "off", "--menu-input", "enter,release,enter,release"])
        // The original introduction, movement, camera and map requests.
        .args(["--key", "popup:help=144,0", "--key", "popup:help=141,0",
            "--key", "popup:help=127,0", "--key", "popup:help=153,0"])
        .arg("--walk-route").arg(&route)
        // The 13 original English voice lines last about 111 seconds.
        // Leave time for the five map movements and the final spoken line.
        .args(["--key", "menu:down:down:enter:release:enter:release:escape:type=MagicMap:enter:release:escape,3030",
            "--key", "menu:up:enter:release:left:enter:release,3040", "--frames", "12000"])
        .output().unwrap();
    fs::write(root.join("stderr.log"), &output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(
        output.status.success()
            && viewer.contains("RESULT: PASS")
            && viewer.contains("walk-route at Wizard on update"),
        "inspect {}",
        root.display()
    );
    assert!(
        viewer.contains("Save screen -> Save(1, \"MagicMap\")"),
        "inspect {}",
        root.display()
    );
    for case in [144, 141, 127, 153] {
        assert!(
            viewer.contains(&format!("starting help: acknowledged case {case}")),
            "case {case}, inspect {}",
            root.display()
        );
    }
    let map_help = viewer
        .lines()
        .find(|line| line.contains("starting help: 500.MSG case 153 opened"))
        .expect("the wizard's Magic Map help opened");
    assert!(
        map_help.to_ascii_lowercase().contains("[tab]"),
        "{map_help}"
    );
    let snapshot: Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let items = snapshot["journey"]["items"].as_array().unwrap();
    let carried = |name: &str| {
        items.iter().any(|i| {
            i["kind"]
                .as_str()
                .is_some_and(|k| k.eq_ignore_ascii_case(name))
                && i["quantity"].as_i64().unwrap_or(0) > 0
        })
    };
    assert!(
        carried("INVITEM_Mask1") && carried("INVITEM_Piece1") && carried("INVITEM_MagicMap"),
        "inspect {}",
        root.display()
    );
    assert!(
        !carried("INVITEM_Piece1A"),
        "the wizard exchanges the starting fragment"
    );
    let vars = snapshot["kept"]["daventry"]["vars"].as_array().unwrap();
    assert!(vars.iter().any(|v| v[0]
        .as_str()
        .is_some_and(|k| k.eq_ignore_ascii_case("Daventry::Wizard::metConnor"))
        && v[1] == "1"));
}
