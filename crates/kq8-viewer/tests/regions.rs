//! Per-region smoke check for the complete Windows display and world-loading path.
use std::{env, fs, path::Path, process::{Command, Stdio}, thread, time::{Duration, Instant}};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn every_region_builds_and_presents_frames() {
    let game = fs::canonicalize(env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kq8-artifacts/region-tests");
    for region in ["Daventry", "CastleD", "DeadCity", "Swamp", "Gnome", "Barren", "IceWorld",
                   "Snowexit", "temple1", "temple2", "temple3", "temple4"] {
        let dir = root.join(format!("{}-{region}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game)
            .args(["--world", region, "--frames", "30", "--sound", "off"])
            .stdout(Stdio::null()).stderr(fs::File::create(dir.join("stderr.log")).unwrap())
            .spawn().unwrap();
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "{region} failed: see {}", dir.display());
                break;
            }
            if start.elapsed() > Duration::from_secs(180) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{region} timed out: see {}", dir.display());
            }
            thread::sleep(Duration::from_millis(200));
        }
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(log.lines().any(|l| l.starts_with(&format!("world built: {region}:"))),
            "{region} never finished building: see {}", dir.display());
        assert!(log.contains("window closed: frames=30 ") && log.contains("RESULT: PASS"),
            "{region} failed to render 30 frames: see {}", dir.display());
        println!("{region}: loaded and rendered 30 frames");
    }
}
