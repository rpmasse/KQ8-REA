//! A scripted camera flight and its retained destination across disk reload.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_windmill_camera_flight_and_later_cut_survive_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/camera-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
        fs::write(dir.join("stdout.log"), &result.stdout).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(result.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("camera.log")).unwrap())
    };
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap_or_else(|e| panic!("{e}; inspect {}", root.display()))).unwrap()
    };
    // The installed wtrmwalk.cs supplies these camera coordinates and duration.
    run("save", &["--world", "Daventry", "--frames", "1200",
        "--command", "set Daventry::Wizard::Clicked true,3",
        "--command", "KQCamera::follow none,5",
        "--command", "move KQCamera 0 192100 232500 9255 -.2 0 2.632,6",
        "--command", "KQCamera::setRange 1000 2500,7",
        "--command", "KQCamera::addCamLoc 190000 230000 10600 1,9",
        "--command", "KQCamera::flyToCamLoc 186960 228000 10000 2,10",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Flight:enter:release:down:down:down:enter:release:left:enter:release,20"]);
    let first = read(1);
    let flight = &first["camera"]["flight"];
    let k = flight["k"].as_u64().unwrap();
    assert!(k > 0 && k < 31);
    assert_eq!(flight["n"], 31);
    let (viewer, camera) = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1800",
        "--command", "KQCamera::cutToCamLoc 186960 228000 10000,50",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Cut:enter:release:down:down:down:enter:release:left:enter:release,60"]);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    assert!(!camera.contains("ERROR cutToCamLoc: no spot"), "camera destination list was lost; inspect {}", root.display());
    assert!(camera.contains(&format!("camera update 1: flight {}/31:", k + 1)), "flight restarted or skipped a step; inspect {}", root.display());
    let row = camera.lines().find(|l| l.contains(&format!("camera update 1: flight {}/31:", k + 1))).unwrap();
    let position: Vec<f64> = row.split("at (").nth(1).unwrap().split(')').next().unwrap()
        .split(',').map(|v| v.trim().parse().unwrap()).collect();
    for (axis, actual) in position.iter().enumerate() {
        let expected = first["camera"]["cam"]["position"][axis].as_f64().unwrap() + flight["step"][axis].as_f64().unwrap();
        assert!((actual - expected).abs() < 0.11, "flight resumed from the wrong position; inspect {}", root.display());
    }
    assert_eq!(camera.matches("last update: its last step").count(), 1);
    assert_eq!(first["camera"]["spots"], serde_json::json!([[190000.0, 230000.0, 10600.0]]));
    let second = read(2);
    assert_eq!(second["camera"]["spots"], first["camera"]["spots"]);
    assert!(second["camera"]["flight"].is_null());
    assert_eq!(second["camera"]["following"], false);
    assert!(camera.contains("cut to the spot (190000, 230000, 10600)"), "later cut did not execute; inspect {}", root.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
