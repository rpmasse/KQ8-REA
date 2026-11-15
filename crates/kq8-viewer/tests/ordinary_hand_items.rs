//! Unattended original-data matrix for Connor's ordinary hand weapons.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn ten_hand_weapons_use_their_authored_attacks() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/ordinary-hand-items/{nonce}"));

    for (item, movement) in [
        ("Dagger", "condag.anm"),
        ("BroadSword", "conbswd.anm"),
        ("Mace", "conmace.anm"),
        ("LakeSword", "consword.anm"),
        ("SkelKingSword", "conkng.anm"),
        ("BattleAx", "conbax.anm"),
        ("WarHammer", "conwhamr.anm"),
        ("Pike", "conbpike.anm"),
        ("LongSword", "conlngsw.anm"),
        ("TempleSword", "contpswd.anm"),
    ] {
        let run = root.join(item);
        fs::create_dir_all(&run).unwrap();
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&run).arg("--game").arg(&game)
            .args(["--windowed", "--sound", "off", "--no-ai", "--god", "--world", "DeadCity"])
            .args([
                "--command", &give,
                "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
                "--command", "KQConner::setDoMode 0,35",
                "--swing", "skeleton1,60", "--frames", "130",
            ])
            .output().unwrap();
        fs::write(run.join("stderr.log"), &output.stderr).unwrap();
        let logs = run.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"),
            "{item}: inspect {}", run.display());
        assert_eq!(fight.matches(&format!("Connor: Do ({movement}/attack): hits skeleton1")).count(), 1,
            "{item}: authored swing did not hit once; inspect {}", run.display());
        assert_eq!(fight.matches("skeleton1: hit by Connor, Normal:").count(), 1,
            "{item}: Normal damage did not arrive once; inspect {}", run.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
