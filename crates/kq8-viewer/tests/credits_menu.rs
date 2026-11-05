//! Both normal entry paths activate the original credits renderer.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires the original GOG game and a graphics device"]
fn settings_and_finale_command_reach_the_same_original_credits() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/menu-tests/credits-{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, args: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .env("KQ8_SAVE_DIR", dir.join("saves"))
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off"])
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        fs::write(dir.join(format!("{name}.stderr.log")), &result.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        fs::write(dir.join(format!("{name}.viewer.log")), &log).unwrap();
        assert!(
            result.status.success() && log.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        log
    };
    let mut input = String::from("down,enter,release,click=310x362");
    for _ in 0..25 {
        input.push_str(",enter");
    }
    input.push_str(",escape");
    let title = run("options", &["--menu-input", &input, "--frames", "90"]);
    assert!(title.contains("credits: Options 400.MSG card 1/25"));
    for page in 2..=25 {
        assert!(title.contains(&format!("credits: advance 400.MSG card {page}/25")));
    }
    assert!(title.contains("card 25/25, team photo true, score None"));
    assert!(
        title.contains("credits: returned to Options") && title.contains("options: back to main")
    );
    let ending = run(
        "ending",
        &[
            "--world",
            "Temple4",
            "--command",
            "doCredits,4",
            "--frames",
            "105",
        ],
    );
    assert!(ending.contains("credits: doCredits 400.MSG card 1/25, team photo false, score Some("));
    assert!(!ending.contains("credits: Options"));
}
