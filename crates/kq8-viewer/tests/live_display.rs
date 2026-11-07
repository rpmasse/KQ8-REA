//! One bounded native acknowledgment flow; no world or manual inspection.
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[test]
#[ignore = "requires read-only KQ8_GAME_DIR and a native graphics device"]
fn live_original_resolution_round_trip() {
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
    fs::write(dir.join("kq8-artifacts/Options.cs"), "set800mode 0\n").unwrap();
    println!("live display native evidence: {}", dir.display());

    let mut command = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
    command
        .current_dir(&dir)
        .env("KQ8_SAVE_DIR", dir.join("saves"))
        .arg("--game")
        .arg(&game)
        .args([
            "--sound",
            "off",
            "--menu-input",
            "down,enter,release,enter,release,down,enter,enter,up,enter,escape,escape",
            "--frames",
            "180",
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
    let lines: Vec<_> = log.lines().filter(|l| l.starts_with("display:")).collect();
    assert_eq!(
        lines.len(),
        4,
        "duplicate selection must add no request; inspect {}",
        dir.display()
    );
    for (line, expected) in lines.iter().zip([
        "display: requested [800, 600] physical pixels; waiting for window and camera acknowledgement",
        "display: confirmed [800, 600], interface revision 1; retained renderer, projection and game state",
        "display: requested [640, 480] physical pixels; waiting for window and camera acknowledgement",
        "display: confirmed [640, 480], interface revision 2; retained renderer, projection and game state",
    ]) {
        assert_eq!(*line, expected, "inspect {}", dir.display());
    }
    assert!(log.contains("graphics: returned to Options; saved local preferences"));
    assert!(log.contains("options: back to main"));
    assert!(
        !log.contains("world built:"),
        "this flow must not load a world"
    );
    let saved = kq8_core::options_menu::Settings::from_script(
        &fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap(),
    );
    assert!(!saved.graphics.high_resolution);
    assert_eq!(
        fs::read(game.join("Options.cs")).unwrap(),
        original_options,
        "the original installation remains read-only"
    );
    println!("LIVE DISPLAY PASS: real 800/640 window and camera acknowledgements, repeat800 no-op, menu return, isolated preference false; original Options bytes unchanged; {:.2}s; {}",
        started.elapsed().as_secs_f64(), dir.display());
}
