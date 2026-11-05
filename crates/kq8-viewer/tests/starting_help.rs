use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires the original GOG game and a graphics device"]
fn scripts_show_the_installed_help_and_respect_the_saved_preference() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kq8-artifacts/starting-help")
        .join(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string(),
        );
    fs::create_dir_all(&dir).unwrap();
    let run = |extra: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .arg("--game")
            .arg(&game)
            .args([
                "--world",
                "Daventry",
                "--sound",
                "off",
                "--no-ai",
                "--command",
                "sendEvent 0 fountain KQCueEvent 30 0,4",
                "--updates",
                "120",
                "--frames",
                "1400",
                "--key",
                "menu:click=314x365:left:enter:release,100",
            ])
            .args(extra)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap()
    };
    let first = run(&["--key", "popup:checkbox,4", "--key", "popup:enter,4"]);
    let gui=fs::read_to_string(dir.join("kq8-artifacts/logs/gui.log")).unwrap().to_ascii_lowercase();
    for art in ["buywin4.pbm", "buyhibtn.pba", "buybtndn.pbm", "console.pft"] {
        assert!(gui.contains(&format!("vol:daventry/8gui/{art}")), "missing original {art}");
    }
    let popup=gui.lines().find(|l|l.contains("idctl_popup drawn")).unwrap();
    let alpha=popup.split_once(": ").unwrap().1.split_whitespace().next().unwrap().parse::<usize>().unwrap();
    assert!(alpha<370*200/2, "transparent popup: {popup}");
    let open = first
        .lines()
        .find(|l| l.contains("starting help: 500.MSG case 127 opened"))
        .unwrap();
    assert!(
        open.contains("CAMERA:"),
        "the original 500.MSG caption: {open}"
    );
    let tick = open
        .split_once("opened on update ")
        .unwrap()
        .1
        .split(':')
        .next()
        .unwrap();
    assert!(
        tick.parse::<u64>().unwrap() >= 49,
        "the script waits three seconds of game time"
    );
    assert!(first.contains(&format!(
        "acknowledged case 127 on update {tick}, help enabled false"
    )));
    assert!(fs::read_to_string(dir.join("kq8-artifacts/Options.cs"))
        .unwrap()
        .contains("notStartingHelp 1"));
    let template = fs::read_to_string(game.join("movekeys.kq")).unwrap();
    let mut keys = kq8_core::keyboard_menu::Bindings::read(template.as_bytes()).unwrap();
    keys.0[8] = b'G';
    fs::write(dir.join("kq8-artifacts/movekeys.kq"), keys.file(&template)).unwrap();
    let second = run(&[
        "--key",
        "f1,10",
        "--key",
        "popup:right,10",
        "--key",
        "popup:enter,10",
        "--key",
        "popup:right,10",
        "--key",
        "popup:enter,10",
        "--key",
        "popup:left,10",
        "--key",
        "popup:left,10",
        "--key",
        "popup:f1,10",
        "--key",
        "f1,20",
        "--key",
        "popup:help=127,20",
        "--key",
        "left:IDCTL_MAPWIN_TELEPORT,45",
        "--key",
        "popup:help=151,45",
    ]);
    assert_eq!(
        second.matches("F1 help: 500.MSG case 127 opened").count(),
        2
    );
    let wrap = second
        .lines()
        .find(|l| l.contains("F1 help: selected case 153"))
        .unwrap();
    assert!(wrap.contains("[G]") && wrap.contains("focus <<"), "{wrap}");
    assert!(second.contains("F1 help: selected case 127") && second.contains("focus >>"));
    assert!(second
        .lines()
        .any(|line| line.contains("map Go help: 500.MSG case 151")
            && line.contains("checkbox false")));

    assert!(!second.contains("starting help: 500.MSG case 127 opened"));
    let script = fs::read_to_string(dir.join("kq8-artifacts/logs/doors.log")).unwrap();
    assert!(script.contains("requested starting help, 500.MSG case 127"));
}
