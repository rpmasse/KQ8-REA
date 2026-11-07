//! Isolated original Ferryman conversation, not an earned campaign route.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires GOG assets and graphics"]
fn held_route_turn_is_pressed_again_after_conversation_turns() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/route-conversation-input/{nonce}"
    ));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("leave.json");
    fs::write(&path, r#"{"start":5,"landmarks":[{"name":"Leave Ferryman's stay area","xy":[56954,190000],"radius":200,"run":false}]}"#).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root).env("KQ8_SAVE_DIR", root.join("slots"))
        .arg("--game").arg(&game)
        .args(["--world","DeadCity","--sound","off","--frames","1200",
            "--command","move Connor 0 56954 192325 1042 0 0 3.05,3",
            "--command","KQConner::conversation Ferryman 2000 29 14 41 1 0 0,8"])
        .arg("--walk-route").arg(&path)
        // Load is disabled in this fresh directory, so two Down presses
        // select Save. The campaign helper starts with an existing slot.
        .args(["--key","menu:down:down:enter:release:type=LeftFerryman:enter:release:escape,100",
            "--key","menu:up:enter:release:left:enter:release,115"])
        .output().unwrap();
    fs::write(root.join("stderr.log"), &result.stderr).unwrap();
    fs::write(root.join("stdout.log"), &result.stdout).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(
        result.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        root.display()
    );
    assert!(viewer.contains("walk-route releases a held turn key after its turn intent was cleared"));
    assert!(viewer.contains("walk-route at Leave Ferryman's stay area"));
    let conversation = fs::read_to_string(logs.join("conversation.log")).unwrap();
    assert!(conversation.contains("walked out of the stay area"));
    assert_eq!(
        conversation
            .matches("Ferryman: TalkExit: its script ran")
            .count(),
        1
    );
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("slots/slot-1.json")).unwrap()).unwrap();
    assert_eq!(saved["journey"]["god_mode"], false);
    assert_eq!(saved["journey"]["ai_off"], false);
    assert!(saved["position"][1].as_f64().unwrap() < 190500.0);
    assert!(saved["conversation"]["active"].is_null());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
