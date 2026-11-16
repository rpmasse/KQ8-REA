//! Unattended GOG-data check for the three separately authored IceWorld dragon actors.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn iceworld_dragon_body_and_heads_load_their_own_models() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/dragon-load/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args(["--world", "IceWorld", "--at", "182334,490176,30902",
            "--windowed", "--sound", "off", "--no-ai", "--god",
            "--command", "sendEvent 0 dragbody KQAttackEvent Connor 0 5000 0,10",
            "--updates", "35"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    for (name, class, shape, position) in [
        ("dragbody", "KQDragonBody", "db_", "182334, 490176, 30902"),
        ("draglhed", "KQDragonHead", "dl_", "182734, 488910, 32300"),
        ("dragrhed", "KQDragonHead", "dr_", "182000, 488910, 32200"),
    ] {
        assert!(characters.contains(&format!("[{name}] {class}: from")),
            "{name} did not load as an animated character; inspect {}", dir.display());
        assert!(viewer.lines().any(|line| line.contains(&format!("object [{name}] class={class} shape={shape}"))
            && line.contains(&format!("at ({position})"))),
            "{name} has no separately placed authored model; inspect {}", dir.display());
        assert!(!viewer.contains(&format!("marker [{name}] class={class}")),
            "{name} remained a marker; inspect {}", dir.display());
    }
    assert!(fight.contains("dragbody: fights (rule 2): stats [300, 25, 25, 100")
        && fight.contains("dragbody: hit by Connor, Normal:")
        && fight.contains("health 300 -> 300"),
        "Dragon body did not retain shared fighter stats or protection; inspect {}", dir.display());
    for (name, radius, height, kind) in [
        ("dragbody", 1365, 2730, "0x1"),
        ("draglhed", 460, 2170, "0x4042"),
        ("dragrhed", 485, 2150, "0x4042"),
    ] {
        assert!(doors.lines().any(|line| line.contains(&format!(
            "{name}: solid for Connor: cylinder radius {radius} (+ his ball radius), height {height}"
        )) && line.contains(&format!("type {kind} mask 0x0"))),
            "{name} did not resolve its authored type to active Dragon bounds; inspect {}", dir.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
