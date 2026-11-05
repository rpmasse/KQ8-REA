//! Exercise the real HUD, click ray, Connor's animation cue, and landing script.
use std::{env, fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn throw_and_recover_a_rock_in_daventry() {
    let game = fs::canonicalize(env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kq8-artifacts/rock-tests")
        .join(std::process::id().to_string());
    fs::create_dir_all(&dir).unwrap();
    // --item gives one genuine inventory object a stable name. The two
    // bar clicks put it back, then take it from the ordinary rock slot.
    let run = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(game)
        .args(["--world", "Daventry", "--at", "277744,199295,0", "--no-ai", "--god",
            "--sound", "off", "--item", "Rock", "--key", "left:IDCTL_INVBAR_ROCKS,10",
            "--key", "left:IDCTL_INVBAR_ROCKS,14", "--aim-point", "277744,198000,13580,24",
            "--aim", "TestItem,110", "--frames", "900"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), run.stderr).unwrap();
    assert!(run.status.success(), "see {}", dir.display());
    let logs = dir.join("kq8-artifacts/logs");
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(fight.contains("ThrowRock: started=true") && fight.contains("shoots rock"), "see {}", logs.display());
    assert!(items.to_ascii_lowercase().contains("rockhit.cs"), "see {}", logs.display());
    assert!(items.contains("picked up; Connor's INVITEM_Rock 0 -> 1"), "see {}", logs.display());
}
