//! The installed Temple 4 two-form fighter continues across a disk checkpoint.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn lucreto_near_form_survives_a_fresh_process_then_switches_back() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/lucreto-form-checkpoint/{nonce}"
    ));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--windowed", "--sound", "off", "--god"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, characters, fight)
    };
    let (first, first_characters, _) = run(
        "save",
        &[
            "--world", "temple4",
            "--command", "sendEvent 0 altar KQCueEvent 6 7,5",
            "--command", "move Connor 0 101180 96100 5938 0 0 0,50",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=LucretoNear:enter:release:escape,75",
            "--updates", "95", "--frames", "1100",
        ],
    );
    assert!(
        first_characters.contains("LucEndFar: Connor 1350")
            && first.contains("Save screen -> Save(1, \"LucretoNear\")"),
        "near form or checkpoint missing; inspect {}",
        root.display()
    );
    let slot: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let actors = slot["kept"]["temple4"]["characters"].as_array().unwrap();
    let actor = |name: &str| {
        actors
            .iter()
            .find(|actor| actor["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing from checkpoint; inspect {}", root.display()))
    };
    assert_eq!(actor("LucEnd")["hidden"], false);
    assert_eq!(actor("LucEndFar")["hidden"], true);
    assert_eq!(
        actor("LucEnd")["animation"]["placement"]["position"],
        actor("LucEndFar")["animation"]["placement"]["position"]
    );
    let (restored, restored_characters, restored_fight) = run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--command",
            "move Connor 0 100032 97535 5938 0 0 0,35",
            "--updates",
            "95",
            "--frames",
            "1100",
        ],
    );
    assert!(
        restored.contains("save restored in temple4 without replaying Begin")
            && restored_characters.contains("LucEnd: status: Wait, active")
            && restored_fight.contains("LucEnd: Do (Attack")
            && restored_characters.contains("LucEnd: Connor 3012")
            && restored_fight.contains("LucEndFar: Do (Attack")
            && restored_fight.contains("shoots bolt"),
        "the saved fighter did not resume and switch forms; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
