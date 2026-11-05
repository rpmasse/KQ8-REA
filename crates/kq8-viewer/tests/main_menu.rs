//! Launch the actual Windows executable, rendering the GOG menu and using
//! the same input handlers as a player. Requires assets and a graphics device.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn windows_menu_routes() {
    let game =
        std::fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kq8-artifacts/menu-tests");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(format!("{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
        assert!(
            result.status.success(),
            "{name} failed: inspect {}",
            dir.display()
        );
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(
            log.contains("RESULT: PASS"),
            "{name} failed: inspect {}",
            dir.display()
        );
        log
    };
    let startup = run("startup", &["--menu", "--frames", "12"]);
    assert!(
        startup.contains("game: false, row: 0") && startup.contains("window closed: frames=12 ")
    );
    assert!(
        !startup.contains("world built:"),
        "menu starts before loading Daventry"
    );

    let new_game = run(
        "new",
        &[
            "--menu-input",
            "enter,release,enter,release",
            "--frames",
            "450",
        ],
    );
    assert!(new_game.contains("Start(2)") && new_game.contains("(factor 1)"));
    assert!(
        new_game.contains("world built: Daventry")
            && new_game.contains("world.cs Daventry Begin (world-change.md rule 9): run")
    );

    let resume = run(
        "resume",
        &[
            "--world",
            "Daventry",
            "--key",
            "menu:escape,12",
            "--frames",
            "125",
        ],
    );
    assert!(resume.contains("game: true, row: 6") && resume.contains("Resume, cue="));
    let line = resume
        .lines()
        .find(|s| s.starts_with("menu: leaving at update Some("))
        .expect("resume tick in viewer.log");
    let (after, before) = line
        .trim_start_matches("menu: leaving at update Some(")
        .split_once("), entered at ")
        .unwrap();
    assert_eq!(
        after, before,
        "the game clock must stop throughout the menu"
    );

    let death = run(
        "death",
        &["--world", "Daventry", "--lift", "6500", "--frames", "240"],
    );
    let died = death.find("state fall -> dead").expect("fatal landing");
    assert!(death[died..].contains("menu: shown Choice { game: false, row: 0"));
}
