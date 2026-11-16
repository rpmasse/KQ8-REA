//! The installed Temple 4 altar starts both of Lucreto's fighting forms.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn temple4_lucreto_switches_forms_and_uses_both_attacks() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/live-lucreto-forms/{nonce}"));
    fs::create_dir_all(&root).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root)
        .arg("--game")
        .arg(&game)
        .args(["--windowed", "--sound", "off", "--world", "temple4"])
        .args([
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "sendEvent 0 altar KQCueEvent 6 7,5",
            "--command",
            "move Connor 0 101180 97050 5938 0 0 0,10",
            "--command",
            "move Connor 0 101180 96100 5938 0 0 0,50",
            "--command",
            "move Connor 0 100032 97535 5938 0 0 0,105",
            "--updates",
            "160",
            "--frames",
            "1300",
        ])
        .output()
        .unwrap();
    fs::write(root.join("stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("stderr.log"), &output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let traps = fs::read_to_string(logs.join("traps.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        root.display()
    );
    let far_first = characters
        .find("LucEnd: Connor 3012")
        .expect("close-to-far handoff");
    let near = characters
        .find("LucEndFar: Connor 1350")
        .expect("far-to-close handoff");
    let far_again = characters
        .rfind("LucEnd: Connor 3012")
        .expect("return to far form");
    assert!(
        far_first < near && near < far_again,
        "form handoffs out of order; inspect {}",
        root.display()
    );
    assert!(
        fight.contains("LucEndFar: Do (Attack")
            && fight.contains("shoots bolt1 (PrLuc.kq")
            && fight.contains("LucEnd: Do (Attack")
            && fight.contains("hits Connor: distance 1350"),
        "an authored ranged or close attack did not run; inspect {}",
        root.display()
    );
    assert!(
        fight
            .lines()
            .any(|line| line.contains("LucEndFar: bolt1") && line.contains("touches Connor"))
            && fight.lines().any(
                |line| line.contains("Connor: hit by LucEndFar, Projectile:")
                    && line.contains("; health ")
                    && !line.contains("loss 0")
            )
            && fight
                .lines()
                .any(|line| line.contains("Connor: hit by LucEnd, Normal:")
                    && line.contains("; health ")
                    && !line.contains("loss 0"))
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "the far bolt and near swing did not damage living Connor; inspect {}",
        root.display()
    );
    assert!(
        traps
            .matches("test --command: applied: moved Connor")
            .count()
            == 3,
        "the distance crossings did not run; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
