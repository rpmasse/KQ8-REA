//! One bounded exact-fullscreen acknowledgment flow; no manual inspection.
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[test]
#[ignore = "requires read-only KQ8_GAME_DIR and a native graphics device"]
fn live_original_exclusive_resolution_round_trip() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/live-display-native/{nonce}"));
    fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
    fs::write(
        dir.join("kq8-artifacts/Options.cs"),
        "assignGModeName Glide\nset800mode 0\n",
    )
    .unwrap();
    println!("live display native evidence: {}", dir.display());

    let mut command = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
    command
        .current_dir(&dir)
        .env("KQ8_SAVE_DIR", dir.join("saves"))
        .arg("--game")
        .arg(&game)
        .args([
            "--display-profile",
            "glide",
            "--fullscreen",
            "--sound",
            "off",
            "--menu-input",
            // Quit through the menu after the route. A frame cap can end
            // before the backend's bounded acknowledgement deadline.
            "down,enter,release,enter,release,down,down,down,enter,enter,up,enter,escape,escape,down,down,enter,left,enter,release",
        ])
        .stdout(Stdio::from(
            fs::File::create(dir.join("stdout.log")).unwrap(),
        ))
        .stderr(Stdio::from(
            fs::File::create(dir.join("stderr.log")).unwrap(),
        ));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Suppress a separate console helper; the native renderer owns its window.
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(45) {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "native resize child exceeded 45 seconds; inspect {}",
                dir.display()
            );
        }
        thread::sleep(Duration::from_millis(100));
    };
    let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
    assert!(
        status.success() && log.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    let lines: Vec<_> = log
        .lines()
        .filter(|l| l.starts_with("graphics runtime: confirmed"))
        .collect();
    assert_eq!(
        lines.len(),
        3,
        "duplicate selection must add no request; inspect {}",
        dir.display()
    );
    for (line, pixels) in lines.iter().zip(["[640, 480]", "[800, 600]", "[640, 480]"]) {
        for expected in [
            "glide profile, active \"Glide\" Fullscreen",
            "presentation: Exclusive",
            "saved fullscreen \"Glide\"",
            &format!("window_pixels: {pixels}"),
            &format!("monitor_pixels: Some({pixels})"),
        ] {
            assert!(
                line.contains(expected),
                "missing {expected:?} in {line}; inspect {}",
                dir.display()
            );
        }
    }
    let submitted: Vec<_> = log.lines()
        .filter(|line| line.starts_with("graphics runtime: accepted frame submission epoch "))
        .collect();
    assert_eq!(submitted.len(), 3, "each native change requires an actual submitted target frame");
    for (index, (line, pixels)) in submitted.iter()
        .zip(["[640, 480]", "[800, 600]", "[640, 480]"]).enumerate()
    {
        assert!(line.contains(&format!("epoch {} ", index + 1)) && line.contains(pixels),
            "stale or wrong-sized submission: {line}");
    }
    assert_eq!(
        log.lines()
            .filter(|line| line.starts_with("graphics runtime: requested exact host"))
            .count(),
        3,
        "startup plus two real changes; selecting active800 must add no request"
    );
    assert!(
        !log.contains("request 1 failed:") && !log.contains("presentation was not acknowledged")
    );
    assert!(log.contains("graphics: returned to Options; saved local preferences"));
    assert!(log.contains("options: back to main"));
    assert!(log.contains(", Quit,"), "the route must finish through the ordinary Quit action");
    assert!(
        !log.contains("world built:"),
        "this flow must not load a world"
    );
    let saved = kq8_core::options_menu::Settings::from_script(
        &fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap(),
    );
    assert!(!saved.graphics.high_resolution);
    assert_eq!(saved.graphics.mode, "Glide");
    assert_eq!(
        fs::read(game.join("Options.cs")).unwrap(),
        original_options,
        "the original installation remains read-only"
    );
    assert!(
        child.try_wait().unwrap().is_some(),
        "the bounded child has exited"
    );
    println!("LIVE DISPLAY PASS: actual Winit Exclusive/window/monitor/camera 640/800/640 acknowledgements, repeat800 no-op, menu return, isolated Glide640 preferences; original Options bytes unchanged; {:.2}s; {}",
        started.elapsed().as_secs_f64(), dir.display());
}
