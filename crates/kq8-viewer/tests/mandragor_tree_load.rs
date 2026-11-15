//! Unattended original-data check for the Mandragor tree's shared visual load.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_mandragor_trees_load_as_animated_characters() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-load/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "157904,140178,0", "--god", "--sound", "off",
            "--windowed", "--updates", "20",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        characters.matches("KQMandragorTree: from").count(),
        11,
        "expected all Swamp trees; inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("object [mand3t] class=KQMandragorTree shape=mt_stop.kqs"),
        "expected an animated tree model; inspect {}",
        dir.display()
    );
    assert!(
        !viewer.contains("marker [mand3t] class=KQMandragorTree")
            && viewer.contains("marker [mand3r] class=KQMandragorRoot")
            && viewer.contains("marker [mand3l] class=KQMandragorRoot"),
        "root visual behavior remains pending; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
