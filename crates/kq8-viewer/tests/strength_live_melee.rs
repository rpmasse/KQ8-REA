//! Original-data Strength potion raises a real Mace hit, then expires.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn strength_changes_melee_damage_until_its_timer_expires() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/strength-live-melee/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--no-ai",
            "--world",
            "DeadCity",
        ])
        .args([
            "--command",
            "move Skelking 0 183300 45000 864 0 0 0,5",
            "--command",
            "KQMonster::setState Skelking Wait,6",
            "--command",
            "KQMonster::inventory Connor INVITEM_Mace 1,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_Strength 1,20",
            "--command",
            "KQConner::setDoMode 0,35",
            "--key",
            "6,45",
            "--swing",
            "Skelking,80",
            "--swing",
            "Skelking,360",
            "--updates",
            "390",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        items.contains("slot 6, INVITEM_Strength")
            && items.contains("strength: hand number 65 -> 97")
            && items.contains("stats [80, 97, 19, 11, 10]")
            && items.contains("Dur 20 s:")
            && items
                .lines()
                .any(|line| line.starts_with("update 345:") && line.contains("its time is up"))
            && items.contains("strength ends: hand number 97 -> 65")
            && items.contains("count 0:")
            && items.contains("leaves the inventory"),
        "the potion did not start and expire through the original inventory path; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Skelking: hit by Connor, Normal:"))
        .collect();
    assert_eq!(
        hits.len(),
        2,
        "the Mace must land once before and once after expiry; inspect {}",
        dir.display()
    );
    let update = |line: &str| {
        line.strip_prefix("update ")
            .and_then(|tail| tail.split_once(':'))
            .and_then(|(n, _)| n.parse::<u32>().ok())
            .unwrap()
    };
    assert!((45..345).contains(&update(hits[0])) && update(hits[1]) > 345);
    assert!(
        hits[0].contains("D 97") && hits[1].contains("D 65"),
        "melee damage did not follow the Strength stat; inspect {}",
        dir.display()
    );
    let health = |line: &str| {
        let pair = line
            .split_once("; health ")
            .unwrap()
            .1
            .split(';')
            .next()
            .unwrap();
        let (before, after) = pair.split_once(" -> ").unwrap();
        (
            before.parse::<i64>().unwrap(),
            after.parse::<i64>().unwrap(),
        )
    };
    let (first_before, first_after) = health(hits[0]);
    let (second_before, second_after) = health(hits[1]);
    assert!(
        first_before == 800
            && first_after < 800
            && first_after == second_before
            && second_after < second_before
            && second_after > 0
            && !fight.contains("Skelking: dies"),
        "the original high-health target did not retain both real hits; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
