//! Exercise a placed Weeper's ordinary attack and its original assets.
use std::{env, fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn a_dead_city_weeper_drains_connor() {
    let game = fs::canonicalize(env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kq8-artifacts/weeper-tests")
        .join(std::process::id().to_string());
    fs::create_dir_all(&dir).unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(game)
        .args(["--world", "DeadCity", "--at", "170920,149369,0", "--sound", "off",
            "--command", "KQConner::setLevel 8,1",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 80 0 0 0 0,2",
            "--frames", "800"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), run.stderr).unwrap();
    assert!(run.status.success(), "see {}", dir.display());
    let logs = dir.join("kq8-artifacts/logs");
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let sound = fs::read_to_string(logs.join("sound.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert!(fight.contains("weeper14: bolt1 starts drain on Connor:") && fight.contains("shoots bolt1 (Prweeper.kq"), "see {}", logs.display());
    assert!(fight.contains("drain pulse 12 on Connor:") && fight.contains("ends drain after 12 pulses; suckstop"), "see {}", logs.display());
    assert!(doors.contains("copy WeepLink.weeper14.0 of LifeSuck: drawn") && doors.contains("copy WeepEnd.weeper14 of LifeSuck2: drawn"), "see {}", logs.display());
    assert!(connor.contains("plays magicReact (magicReact)"), "see {}", logs.display());
    assert!(sound.contains("data/DeadCity/Sound/LIFESUCK.AUD") && sound.contains("Weeper drain ended"), "see {}", logs.display());
}
