//! Windows world loading and quest-state checks against the installed GOG data.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn run_world(case: &str, args: &[&str]) -> PathBuf {
    let game = fs::canonicalize(env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kq8-artifacts/region-tests")
        .join(format!("{}-{case}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(game)
        .args(["--sound", "off"]).args(args)
        .stdout(Stdio::null())
        .stderr(fs::File::create(dir.join("stderr.log")).unwrap())
        .spawn().unwrap();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{case} failed: see {}", dir.display());
            break;
        }
        if start.elapsed() > Duration::from_secs(180) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("{case} timed out: see {}", dir.display());
        }
        thread::sleep(Duration::from_millis(200));
    }
    dir.join("kq8-artifacts/logs")
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn every_region_builds_and_presents_frames() {
    for region in ["Daventry", "CastleD", "DeadCity", "Swamp", "Gnome", "Barren", "IceWorld",
                   "Snowexit", "temple1", "temple2", "temple3", "temple4"] {
        let logs = run_world(region, &["--world", region, "--frames", "30"]);
        let log = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(log.lines().any(|l| l.starts_with(&format!("world built: {region}:"))),
            "{region} never finished building: see {}", logs.display());
        assert!(log.contains("window closed: frames=30 ") && log.contains("RESULT: PASS"),
            "{region} failed to render 30 frames: see {}", logs.display());
        println!("{region}: loaded and rendered 30 frames");
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn daventry_girl_uses_carried_quest_flag_during_init() {
    let fresh = run_world("girl-unmet", &["--world", "Daventry", "--frames", "30"]);
    let before = fs::read_to_string(fresh.join("doors.log")).unwrap();
    assert!(before.contains("echo Has not met girl in DC - hide"), "see {}", fresh.display());
    assert!(!before.contains("echo Met Girl in DC - move her home"));

    // Start at the point after Dead City's conversation sets its flag. Ordinary
    // world travel must make it visible to Daventry's own girl.cs during Init.
    let travel = run_world("girl-met", &[
        "--world", "DeadCity", "--command", "setConnorFlag MetGirlInDC 1,2",
        "--console", "goworld Daventry,6", "--frames", "500",
    ]);
    let after = fs::read_to_string(travel.join("doors.log")).unwrap();
    assert!(after.contains("echo Met Girl in DC - move her home"), "see {}", travel.display());
    assert!(after.contains("setHome: home (135240, 279172, 11200)"));
    assert!(!after.contains("echo Has not met girl in DC - hide"));
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn gnome_rolling_rock_hits_connor_through_its_own_script() {
    // Wait in the moving rock's path. Its original movement supplies the
    // contact, and its original rollside.cs supplies the attack event.
    let logs = run_world("rolling-rock", &[
        "--world", "Gnome", "--at", "57500,104693,0",
        "--command", "move Connor 0 57500 104693 17650 0 0 0,4",
        "--frames", "400",
    ]);
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(doors.contains("rollside.cs rrock1 ConnorCollision"), "see {}", logs.display());
    assert!(fight.lines().any(|l| l.contains("hit by rrock1") && l.contains("health ")), "see {}", logs.display());
}
