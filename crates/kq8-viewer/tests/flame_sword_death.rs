//! The original Flame Sword movement and Fire hit burn a Dead City skeleton.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn flame_sword_swing_uses_fire_death_and_leaves_no_corpse() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let run = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/flame-sword-tests/{nonce}"));
    fs::create_dir_all(&run).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&run)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--no-ai",
            "--god",
            "--world",
            "DeadCity",
        ])
        .args([
            "--command",
            "KQMonster::inventory Connor INVITEM_FlameSword 1,20",
            "--command",
            "move skeleton2 0 183300 45000 864 0 0 0,5",
            "--command",
            "KQConner::setDoMode 0,35",
            "--swing",
            "skeleton2,60",
            "--frames",
            "130",
        ])
        .output()
        .unwrap();
    fs::write(run.join("stderr.log"), &output.stderr).unwrap();
    let logs = run.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        run.display()
    );
    assert!(
        viewer.contains("made NoName30: INVITEM_FlameSword x1 carried by Connor"),
        "inspect {}",
        run.display()
    );

    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert_eq!(
        fight
            .matches("Connor: Do (confswd.anm/attack): hits skeleton2")
            .count(),
        1,
        "the installed sword movement did not land once; inspect {}",
        run.display()
    );
    assert_eq!(
        fight.matches("skeleton2: hit by Connor, Fire:").count(),
        1,
        "the sword did not deal Fire damage; inspect {}",
        run.display()
    );
    assert_eq!(
        fight
            .matches("skeleton2: dies (rule 30): kind Fire, 'firedie' starts")
            .count(),
        1,
        "the original Fire death did not start; inspect {}",
        run.display()
    );
    assert_eq!(
        fight
            .matches("skeleton2: its death ended (rule 32):")
            .count(),
        1,
        "the death did not finish exactly once; inspect {}",
        run.display()
    );
    assert!(
        fight.contains("no corpse (Fire: burnt away)"),
        "inspect {}",
        run.display()
    );
    assert!(
        !viewer.contains("corpse of skeleton2:"),
        "inspect {}",
        run.display()
    );
}
