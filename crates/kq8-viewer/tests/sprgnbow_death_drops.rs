//! Daventry's bow spriggan drops its crossbow differently after fire.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_bow_spriggan_lays_or_throws_crossbow_by_death_kind() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/sprgnbow-death-drops/{nonce}"));
    let run = |label: &str, action: &[&str]| {
        let dir = root.join(label);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .arg("--game")
            .arg(&game)
            .args([
                "--world",
                "Daventry",
                "--at",
                "277744,199295,0",
                "--windowed",
                "--sound",
                "off",
                "--no-ai",
                "--god",
                "--key",
                "popup:autohelp,0",
                "--command",
                "move sprgnbow 0 277744 199295 13633 0 0 0,5",
            ])
            .args(action)
            .args(["--updates", "140", "--frames", "900"])
            .output()
            .unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        let items = fs::read_to_string(logs.join("items.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, fight, items)
    };

    let (normal_view, normal_fight, normal_items) =
        run("normal", &["--console", "kill sprgnbow,10"]);
    assert!(
        normal_fight.contains("sprgnbow: dies (rule 30): kind Normal, 'Die' starts")
            && normal_fight.contains("update 103: sprgnbow: its death ended (rule 32)")
            && normal_fight.contains("Connor gets its experience 15")
            && normal_fight
                .matches("sprgnbow: its death ended (rule 32)")
                .count()
                == 1
            && normal_fight.contains("a corpse spbdeath.kqs"),
        "normal death did not finish; inspect {}",
        root.display()
    );
    assert!(
        normal_items.contains("kept for the die movement's end, not thrown")
            && normal_items.contains("update 103: sprgnbow: kept weapon (rule 31a):")
            && normal_items.contains("on part 'SmCrossbow' of 'die' frame 93")
            && !normal_items.contains("INVITEM_SmCrossbow x1) thrown"),
        "normal death did not lay the crossbow; inspect {}",
        root.display()
    );
    assert!(
        normal_view.contains("corpse of sprgnbow: spbdeath.kqs drawn"),
        "inspect {}",
        root.display()
    );

    let (fire_view, fire_fight, fire_items) = run(
        "fire",
        &[
            "--command",
            "sendEvent 0 sprgnbow KQAttackEvent Connor 2 10000 0,10",
        ],
    );
    assert!(
        fire_fight.contains("sprgnbow: dies (rule 30): kind Fire, 'firedie' starts")
            && fire_fight.contains("update 50: sprgnbow: its death ended (rule 32)")
            && fire_fight.contains("Connor gets its experience 15")
            && fire_fight
                .matches("sprgnbow: its death ended (rule 32)")
                .count()
                == 1
            && fire_fight.contains("no corpse (Fire: burnt away)"),
        "fire death did not finish; inspect {}",
        root.display()
    );
    assert!(
        fire_items.contains("update 10: sprgnbow: drops (rule 31; parts of 'firedie' frame 0:")
            && fire_items.contains("INVITEM_SmCrossbow x1) thrown from")
            && !fire_items.contains("sprgnbow: kept weapon (rule 31a)"),
        "fire death did not throw the crossbow; inspect {}",
        root.display()
    );
    assert!(
        !fire_view.contains("corpse of sprgnbow:"),
        "fire left a corpse; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
