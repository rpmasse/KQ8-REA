//! Isolated GOG-data Swamp fight with ordinary root and Connor damage.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn sword_defeats_a_live_root_and_it_rebirths_under_normal_damage() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-live-combat/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--world",
            "Swamp",
            "--at",
            "159904,140178,0",
            "--command",
            "KQConner::setDoMode 0,15",
            "--aim",
            "mand3r,45",
            "--aim",
            "mand3r,75",
            "--aim",
            "mand3r,105",
            "--aim",
            "mand3r,135",
            "--updates",
            "290",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("weapon set conKng.anm (SkelKingSword): named in conStart.cs")
            && fight.contains("hand (Some(\"INVITEM_SkelKingSword\"))"),
        "original Swamp sword was not readied; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("test --aim mand3r (camera to the middle of its box):")
            && doors.contains("swing pick meets character mand3l (type 0x4042)"),
        "the nearer root was not selected through the camera ray; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("'conkng.anm/react' can't be cut short (mode free): no swing"),
        "the first aim did not respect Connor's ordinary hit reaction; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("Connor: Do (conkng.anm/").count(),
        3,
        "expected three sword hit cues; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("mand3l: hit by Connor, Normal: D 110")
            .count(),
        3,
        "the left root did not take all three ordinary hits; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("mand3l: Mandragor root Die: Normal, non-solid, linked tree mand3t holds 1 down root(s)")
            && fight.contains("mand3l: Mandragor root die ended: rebirth due at")
            && fight.contains("mand3l: Mandragor root rebirth due: restores health 250, starts birth")
            && fight.contains("mand3l: Mandragor root birth ended: collision 0x4042/0x201f"),
        "temporary root death or timed rebirth failed; inspect {}",
        dir.display()
    );
    let connor_hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by mand3") && line.contains("; health "))
        .collect();
    assert!(
        connor_hits
            .iter()
            .any(|line| line.contains("hit by mand3r"))
            && connor_hits
                .iter()
                .any(|line| line.contains("hit by mand3l"))
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "normal root attacks did not damage a surviving Connor; inspect {}",
        dir.display()
    );
    let final_health = connor_hits
        .last()
        .and_then(|line| line.split("; health ").nth(1))
        .and_then(|health| health.split(" -> ").nth(1))
        .and_then(|health| health.parse::<i32>().ok())
        .expect("a root hit must log Connor's final health");
    assert!(
        final_health > 0,
        "Connor did not survive; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
