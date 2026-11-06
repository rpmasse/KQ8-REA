//! Original angular shake calls, disk continuation and shared camera updates.
use kq8_core::camera::wrap;
use kq8_core::camera_shake::{Parameters, Shake, Tick};
use std::{fs, path::Path, process::Command};

const GONG: Parameters = Parameters {
    seconds: 4.0,
    maximum: [0.02, 0.03],
    rate: [4.0, 5.0],
};

fn expected(count: u32) -> (f32, f32) {
    let mut shake = Shake::default();
    shake.start(GONG).unwrap();
    let (mut tilt, mut heading) = (-0.3, 0.7);
    for _ in 0..count {
        let Tick::Sample(sample) = shake.tick() else {
            panic!("missing sample");
        };
        tilt += sample.angle[0];
        heading = wrap(heading + sample.angle[1]);
    }
    (tilt, heading)
}
fn view(line: &str) -> (f32, f32) {
    let values: Vec<f32> = line
        .split(", view ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    (values[0], values[1])
}
fn close(actual: (f32, f32), wanted: (f32, f32)) {
    assert!(
        (actual.0 - wanted.0).abs() < 0.000002 && (actual.1 - wanted.1).abs() < 0.000002,
        "{actual:?} vs {wanted:?}"
    );
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_shake_continues_after_disk_save_and_through_cut_follow_and_flight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/camera-shake/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
        fs::write(dir.join("stdout.log"), &result.stdout).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            result.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, fs::read_to_string(logs.join("camera.log")).unwrap())
    };
    let (_, first_log) = run("save", &["--world","Daventry","--frames","1200",
        "--command","set Daventry::Wizard::Clicked true,3",
        "--command","KQCamera::follow none,5",
        "--command","move KQCamera 0 192100 232500 9255 -.3 .2 .7,6",
        "--command","KQCamera::shake 4 .02 4 .03 5,10",
        "--key","menu:down:down:enter:release:enter:release:escape:type=Shake:enter:release:down:down:down:enter:release:left:enter:release,20"]);
    let first: serde_json::Value = serde_json::from_slice(
        &fs::read(slots.join("slot-1.json"))
            .unwrap_or_else(|e| panic!("{e}; inspect {}", root.display())),
    )
    .unwrap();
    let camera = &first["camera"];
    let count = camera["shake"]["count"].as_u64().unwrap() as u32;
    assert!(
        count > 0 && count < 30,
        "save must occur on the rising ramp; inspect {}",
        root.display()
    );
    assert_eq!(camera["shake"]["active"], true);
    let pose = &camera["cam"];
    close(
        (
            pose["view_tilt"].as_f64().unwrap() as f32,
            pose["heading"].as_f64().unwrap() as f32,
        ),
        expected(count),
    );
    assert_eq!(camera["view"]["roll"], 0.2);
    assert_eq!(
        pose["position"],
        serde_json::json!([192100.0, 232500.0, 9255.0])
    );
    let first_sample = first_log
        .lines()
        .find(|line| line.contains("shake sample 1:"))
        .unwrap();
    let amplitude: Vec<f32> = first_sample
        .split("amplitude ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    assert!((amplitude[0] - 0.0006666667).abs() < 1e-9 && (amplitude[1] - 0.001).abs() < 1e-9);
    close(view(first_sample), expected(1));
    let (viewer, resumed) = run(
        "resume",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--updates",
            "100",
            "--frames",
            "1600",
        ],
    );
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    assert!(resumed.contains(&format!("camera shake restored after sample {count},")));
    let next = resumed
        .lines()
        .find(|line| line.contains("shake sample "))
        .unwrap();
    assert!(
        next.contains(&format!("camera tick 1: shake sample {}:", count + 1)),
        "inspect {}",
        root.display()
    );
    close(view(next), expected(count + 1));
    let last = resumed
        .lines()
        .find(|line| line.contains("shake sample 60:"))
        .unwrap();
    close(view(last), expected(60));
    assert_eq!(
        resumed.matches("shake sample ").count(),
        (60 - count) as usize
    );
    assert_eq!(
        resumed
            .matches("shake stopped; shared angles reset, drawn pose retained")
            .count(),
        1
    );
    assert!(!resumed.contains("shake sample 61:"));

    let (shared_viewer, shared) = run(
        "shared",
        &[
            "--world",
            "Daventry",
            "--updates",
            "90",
            "--frames",
            "1800",
            "--command",
            "set Daventry::Wizard::Clicked true,3",
            "--command",
            "KQCamera::follow none,5",
            "--command",
            "move KQCamera 0 192100 232500 9255 -.3 .2 .7,6",
            "--command",
            "KQCamera::shake 4 .01 2 .01 2,10",
            "--command",
            "KQCamera::shake 4 .02 4 .03 5,20",
            "--command",
            "KQCamera::addCamLoc 190000 230000 10600 1,28",
            "--command",
            "KQCamera::cutToCamLoc 186960 228000 10000,29",
            "--command",
            "KQCamera::follow Connor,34",
            "--command",
            "KQCamera::flyToCamLoc 186960 228000 10000 1,40",
            "--command",
            "KQCamera::cutToCamLoc 186960 228000 10000 .1,47",
            "--command",
            "KQCamera::follow Connor,50",
        ],
    );
    assert_eq!(
        shared.matches("shake sample 1:").count(),
        2,
        "replacement restarts the wave, inspect {}",
        root.display()
    );
    assert_eq!(shared.matches("shake sample 60:").count(), 1);
    assert_eq!(shared.matches("shake stopped;").count(), 1);
    assert!(
        shared.contains("the flight goes on (")
            && shared_viewer.contains("the flight continues from update")
    );
    assert_eq!(
        shared
            .lines()
            .filter(|line| line.contains(": flight ") && line.contains("/16:"))
            .count(),
        16
    );
    assert_eq!(shared.matches("last update: its last step").count(), 1);
    let rows: Vec<&str> = shared.lines().collect();
    let mut paired = 0;
    for (index, row) in rows.iter().enumerate() {
        if row.contains("| shake |") {
            let update = row.split_whitespace().next().unwrap();
            if let Some(flight) = rows[index + 1..]
                .iter()
                .take_while(|next| !next.contains("shake sample "))
                .find(|next| next.contains("| flight |"))
            {
                assert_eq!(flight.split_whitespace().next().unwrap(), update);
                paired += 1;
            }
        }
    }
    assert_eq!(
        paired,
        16,
        "each flight sync samples once, then performs its separate full update; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
