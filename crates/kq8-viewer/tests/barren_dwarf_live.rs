//! Barren's hammer dwarf fights and completes its original-data death drops.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_barren_hammer_dwarf_camera_fight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/barren-dwarf-live/{nonce}"));
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
            "27891,18000,3.14159",
            "--lift",
            "2000",
            "--command",
            "KQMonster::inventory Connor INVITEM_TempleSword 1,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_Invulnerable 1,20",
            "--key",
            "5,25",
            "--command",
            "KQConner::setDoMode 0,35",
            "--aim",
            "hidwarf,60",
            "--aim",
            "hidwarf,85",
            "--aim",
            "hidwarf,110",
            "--updates",
            "200",
            "--frames",
            "3600",
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
        fight.contains("hidwarf: fights (rule 2): stats [1100, 110, 110, 31, 30000]")
            && characters.contains("hidwarf: status: Wait, active")
            && characters.contains("hidwarf: status: Wait, not active")
            && characters.contains("sees him (0 updates unseen)"),
        "dwarf did not activate with its authored combat stats; inspect {}",
        dir.display()
    );
    assert!(
        items.contains("the shield: protection 45 -> 67")
            && fight.contains("hidwarf: Do (Attack")
            && fight.contains("Connor: hit by hidwarf, Normal:")
            && !fight.contains("god mode is on"),
        "ordinary attack and shielded damage required; inspect {}",
        dir.display()
    );
    assert_eq!(
        viewer.matches("swing pick meets character hidwarf").count(),
        3,
        "all camera-selected swings must pick the dwarf; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .lines()
            .filter(|line| line.contains("Connor: Do (") && line.contains("hits hidwarf:"))
            .count(),
        3,
        "three sword hits required; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("hidwarf: dies (rule 30): kind Normal")
            .count(),
        1,
        "inspect {}",
        dir.display()
    );
    assert!(
        items.contains("hidwarf: drops (rule 31;")
            && items.contains("INVITEM_sacredwater x1")
            && items.contains("hidwarf: kept weapon (rule 31a)")
            && items.contains("INVITEM_WarHammer x1) laid"),
        "the authored drops did not complete; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("hidwarf: its death ended (rule 32): Connor gets its experience 30000")
            .count(),
        1,
        "one experience award required; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("a corpse Bd_death.kqs where it stood"),
        "inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
