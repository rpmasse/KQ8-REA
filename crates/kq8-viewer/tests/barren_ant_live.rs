//! Barren's hidden ant rises, fights, and completes its Fire death.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_barren_ant_birth_attack_and_death() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/barren-ant-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Barren",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--at",
            "167105,104868,0",
            "--command",
            "KQMonster::inventory Connor INVITEM_FlameSword 1,20",
            "--command",
            "KQConner::setDoMode 0,35",
            "--swing",
            "ant1,85",
            "--swing",
            "ant1,115",
            "--swing",
            "ant1,145",
            "--swing",
            "ant1,175",
            "--updates",
            "220",
            "--frames",
            "4000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("ant1: fights (rule 2): stats [220, 40, 40, 41, 5000]"),
        "inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("ant1: joins the active list")
            && characters.contains("ant1: rises (rule 30)")
            && characters.contains("ant1: moving:"),
        "ant did not rise and approach; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("ant1: Do (Attack") && fight.contains("Connor: hit by ant1, Normal:"),
        "ant did not injure Connor; inspect {}",
        dir.display()
    );
    let sword_hits = fight
        .lines()
        .filter(|line| line.contains("Connor: Do (") && line.contains("hits ant1:"))
        .count();
    assert_eq!(
        sword_hits,
        4,
        "unexpected direct-picked sword cues; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("ant1: dies (rule 30): kind Fire").count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        items.matches("ant1: drops (rule 31;").count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert!(
        items.contains("INVITEM_crystal x1"),
        "Crystal missing; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("ant1: its death ended (rule 32): Connor gets its experience 5000")
            .count(),
        1,
        "one reward required; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("no corpse (Fire: burnt away)"),
        "inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
