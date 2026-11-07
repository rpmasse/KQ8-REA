//! One representative native credits scheduler/input/exit flow.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires the original GOG game and a graphics device"]
fn credits_no_skip_escape_and_main_after_fade() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/menu-tests/credits-{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .env("KQ8_SAVE_DIR", dir.join("saves"))
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off"])
            .args([
                "--menu-input",
                "down,enter,release,click=310x362,enter,release,left,right,up,down,click=310x362,type= ,escape",
                "--frames", "360",
            ])
            .current_dir(&dir)
            .output()
            .unwrap();
    fs::write(dir.join("credits.stderr.log"), &result.stderr).unwrap();
    let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
    fs::write(dir.join("credits.viewer.log"), &log).unwrap();
    assert!(
        result.status.success() && log.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(log.contains("credits: Options 400.MSG card 1/25, team photo false, score None"));
    assert!(
        !log.contains("credits: advance") && !log.contains("credits: timer"),
        "keys/clicks must not skip"
    );
    let fade = log
        .find("credits: fading menu theme for one second before Main")
        .unwrap();
    let main = log
        .find("credits: returned to Main after one-second theme fade")
        .unwrap();
    assert!(fade < main);
    assert!(!log.contains("credits: returned to Options"));
    println!("credits native flow passed; inspect {}", dir.display());
}
