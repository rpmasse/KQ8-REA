//! Original AVI acceptance on the normal menu, world and credits render paths.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn run(
    dir: &Path,
    game: &Path,
    decoder: &Path,
    args: &[&str],
    final_marker: Option<&str>,
) -> String {
    fs::create_dir_all(dir).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(dir)
        .env("KQ8_FFMPEG", decoder)
        .env("KQ8_SAVE_DIR", dir.join("saves"))
        .args(["--sound", "on", "--game"])
        .arg(game)
        .args(args)
        .stderr(Stdio::from(
            fs::File::create(dir.join("stderr.log")).unwrap(),
        ))
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let log = dir.join("kq8-artifacts/logs/viewer.log");
    let deadline = Instant::now() + Duration::from_secs(200);
    loop {
        let status = child.try_wait().unwrap();
        let text = fs::read_to_string(&log).unwrap_or_default();
        if let Some(status) = status {
            assert!(
                status.success() && text.contains("RESULT: PASS"),
                "inspect {}",
                dir.display()
            );
            return text;
        }
        if final_marker.is_some_and(|marker| text.contains(marker)) {
            // All ending assertions are observable by the first credits card.
            // Bound the test to that checkpoint instead of waiting for 25 cards.
            child.kill().unwrap();
            child.wait().unwrap();
            return text;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("movie acceptance timed out: {}", dir.display());
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

#[test]
#[ignore = "requires original movies, KQ8_FFMPEG, a display and sound device"]
fn titles_portal_ending_and_decoder_failure() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let decoder = fs::canonicalize(std::env::var_os("KQ8_FFMPEG").unwrap()).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/movie-tests/{nonce}"));
    let missing = run(
        &dir.join("missing"),
        &game,
        &dir.join("absent-ffmpeg.exe"),
        &[
            "--menu-input",
            "click=314x304,enter,click=314x365,left,enter,release",
            "--frames",
            "2000",
        ],
        None,
    );
    assert!(
        missing.contains("unavailable: opening FFmpeg")
            && missing.contains("acknowledged, returning to Menu")
            && missing.contains("Quit, cue=")
    );
    assert!(!missing.contains("first picture"));
    let title = run(
        &dir.join("title"),
        &game,
        &decoder,
        &[
            "--menu-input",
            "click=314x304,click=314x304,escape,click=314x365,left,enter,release",
            "--frames",
            "30000",
        ],
        None,
    );
    assert_eq!(
        title
            .matches("W32opn_1.dll first picture, PCM start 2364663")
            .count(),
        2
    );
    assert!(
        title.contains("finished after 1609 intervals, 1593 decoded pictures, returning to Menu")
    );
    assert!(title.contains("skipped after") && title.contains("Quit, cue="));
    let ending = run(
        &dir.join("ending"),
        &game,
        &decoder,
        &[
            "--world",
            "Temple4",
            "--command",
            "playAvi w32mdl_1.dll,4",
            "--command",
            "killGame,40",
            "--command",
            "playAvi w32ndg_1.dll,40",
            "--command",
            "doCredits,40",
            "--frames",
            "30000",
        ],
        Some("credits: doCredits 400.MSG card 1/25"),
    );
    let events = [
        "w32mdl_1.dll first picture, PCM start 373762",
        "finished after 255 intervals, 237 decoded pictures, returning to Gameplay",
        "ending: killGame removed",
        "w32ndg_1.dll first picture, PCM start 1136116",
        "finished after 774 intervals, 774 decoded pictures, returning to Title",
        "credits: doCredits 400.MSG card 1/25, team photo false, score Some(",
    ];
    let mut rest = ending.as_str();
    for event in events {
        let (_, after) = rest
            .split_once(event)
            .unwrap_or_else(|| panic!("missing/out of order {event} in {}", dir.display()));
        rest = after;
    }
}
