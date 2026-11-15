//! Unattended original-data check for Mandragor tree and linked-root startup.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_mandragor_trees_and_roots_load_as_animated_characters() {
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
            "--windowed", "--command", "move mand3t 0 160000 140000 6120 0 0 1.5707963,10",
            "--updates", "20",
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
    assert_eq!(
        characters.matches("KQMandragorRoot: from").count(),
        22,
        "expected all Swamp roots; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("[mand3r] KQMandragorRoot: from")
            && characters.contains("at (158504, 139178, 6120)")
            && characters.contains("[mand3l] KQMandragorRoot: from")
            && characters.contains("at (157304, 139178, 6120)"),
        "root startup offsets not applied; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("has Birth: NOT DRAWN until born"),
        "root Unborn startup missing; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("mand3r: Mandragor root follows mand3t at (161000, 140600, 6120)")
            && characters.contains("mand3l: Mandragor root follows mand3t at (161000, 139400, 6120)"),
        "roots did not follow the moved, turned tree; inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("object [mand3t] class=KQMandragorTree shape=mt_stop.kqs"),
        "expected an animated tree model; inspect {}",
        dir.display()
    );
    assert!(
        !viewer.contains("marker [mand3t] class=KQMandragorTree")
            && !viewer.contains("marker [mand3r] class=KQMandragorRoot")
            && !viewer.contains("marker [mand3l] class=KQMandragorRoot"),
        "Mandragor actors must use animated models; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn a_swamp_root_rises_without_repeated_tree_pose_resets() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-root-rise/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "159904,140178,0", "--god", "--sound", "off",
            "--windowed", "--updates", "40",
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
    assert!(
        characters.contains("mand3r: rises in 0 s")
            && characters.contains("mand3r: rises (rule 30):")
            && characters.contains("mand3r: solid for Connor:")
            && characters.contains("type 0x4042 mask 0x281f"),
        "root did not rise with its authored collision; inspect {}",
        dir.display()
    );
    assert_eq!(
        characters.matches("mand3r: Mandragor root follows mand3t").count(),
        0,
        "a stationary tree should not reset the root's turn; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn an_original_swamp_tree_dies_with_its_authored_corpse_after_fire() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-fire/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "157904,140178,0", "--god", "--sound", "off",
            "--no-ai", "--windowed", "--command",
            "sendEvent 0 mand3t KQAttackEvent Connor 2 10000 0,10", "--updates", "85",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(fight.contains("mand3t: dies (rule 30): kind Fire, 'firedie' starts"), "inspect {}", dir.display());
    assert!(fight.contains("mand3t: its death ended (rule 32): Connor gets its experience 20000"), "inspect {}", dir.display());
    assert!(fight.contains("a corpse mt_dead.kqs") && fight.contains("Mandragor tree finalizer has no ordinary Die callback"), "inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
