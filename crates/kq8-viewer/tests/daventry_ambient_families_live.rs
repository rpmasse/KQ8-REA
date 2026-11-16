//! Four placed Daventry ambient families in one original-data process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn daventry_ambient_actors_do_not_attack_and_have_distinct_deaths() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/daventry-ambient-families/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Daventry",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--at",
            "289284,188000,0",
            "--key",
            "popup:autohelp,0",
            "--command",
            "move Connor 0 287855 189554 13770 0 0 0,20",
            "--command",
            "deleteObject goblin22,21",
            "--command",
            "move bat5 0 287855 190454 13770 0 0 0,25",
            "--command",
            "move chicken1 0 288555 190454 13770 0 0 0,25",
            "--command",
            "move chicken2 0 287855 188854 13770 0 0 0,25",
            "--command",
            "move piggy1 0 287155 190454 13770 0 0 0,25",
            "--command",
            "sendEvent 0 bat5 KQAttackEvent Connor 0 100000 0,70",
            "--command",
            "sendEvent 0 chicken1 KQAttackEvent Connor 0 100000 0,70",
            "--command",
            "sendEvent 0 chicken2 KQAttackEvent Connor 0 100000 0,70",
            "--command",
            "sendEvent 0 piggy1 KQAttackEvent Connor 0 100000 0,70",
            "--updates",
            "180",
            "--frames",
            "6000",
        ])
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
        "viewer failed; inspect {}",
        dir.display()
    );
    for (name, kind, corpse) in [
        ("bat5", "bat", None),
        ("chicken1", "chicken", Some("Chk_rded.KQS")),
        ("chicken2", "chicken2", Some("Chk_rded.KQS")),
        ("piggy1", "piggy", Some("pigdead2.kqs")),
    ] {
        let placement = format!("[{name}] KQMonster:");
        assert!(
            characters.lines().any(|line| line.contains(&placement)
                && line.contains(&format!("(type {kind},"))
                && line.contains("alarm radius 0;")
                && line.contains("attack distance 0;")),
            "installed zero-attack placement missing for {name}; inspect {}",
            dir.display()
        );
        assert!(
            viewer.contains(&format!("test --command move {name}"))
                && fight.contains(&format!("{name}: fights (rule 2): stats [5, 5, 5, 3, 0]")),
            "relocated {name} did not load; inspect {}",
            dir.display()
        );
        assert!(
            !fight.contains(&format!("{name}: attack (rule 7"))
                && !fight.contains(&format!("Connor: hit by {name},")),
            "ambient {name} attacked; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("{name}: dies (rule 30): kind Normal"))
                .count(),
            1,
            "death count for {name}; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("{name}: drops: nothing (rule 31)"))
                .count(),
            1,
            "drop count for {name}; inspect {}",
            dir.display()
        );
        let finish = format!("{name}: its death ended (rule 32): Connor gets its experience 0");
        assert_eq!(
            fight.matches(&finish).count(),
            1,
            "experience count for {name}; inspect {}",
            dir.display()
        );
        let corpse_description = corpse
            .map(|shape| format!("a corpse {shape}"))
            .unwrap_or_else(|| "no corpse (no shape for this kind)".into());
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&finish) && line.contains(&corpse_description)),
            "corpse mismatch for {name}; inspect {}",
            dir.display()
        );
    }
    assert_eq!(
        fight.matches("first qualifying character death:").count(),
        1
    );
    assert!(fight.contains("chicken2: first qualifying character death:"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
