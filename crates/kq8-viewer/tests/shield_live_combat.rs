//! Original-data potion expiry while live Swamp roots damage Connor.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn shield_changes_live_root_damage_then_expires() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/shield-live-combat/{nonce}"));
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
            "KQMonster::inventory Connor INVITEM_Invulnerable 1,20",
            "--key",
            "5,25",
            "--updates",
            "350",
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
        items.contains("bar click left on")
            && items.contains("slot 5, INVITEM_Invulnerable, HealthItem")
            && items.contains("update 25:")
            && items.contains("the shield: protection 21 -> 50")
            && items.contains("Dur 20 s:"),
        "the original shield was not applied through the inventory key; inspect {}",
        dir.display()
    );
    assert!(
        items.contains("update 325:")
            && items.contains("its time is up at game time")
            && items.contains("the shield ends: protection 50 -> 21"),
        "the 20-second shield did not expire; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by mand3") && line.contains("; health "))
        .collect();
    assert!(
        hits.iter().any(|line| line.contains("hit by mand3r"))
            && hits.iter().any(|line| line.contains("hit by mand3l"))
            && !fight.contains("god mode is on"),
        "both roots must deal ordinary damage; inspect {}",
        dir.display()
    );
    let update = |line: &str| -> u32 {
        line.strip_prefix("update ")
            .and_then(|line| line.split_once(':'))
            .and_then(|(number, _)| number.parse().ok())
            .expect("combat hit update")
    };
    assert!(
        hits.iter()
            .any(|line| (25..325).contains(&update(line)) && line.contains("P 50 -> P'"))
            && hits
                .iter()
                .any(|line| update(line) > 325 && line.contains("P 21 -> P'")),
        "live hits did not use raised then restored protection; inspect {}",
        dir.display()
    );
    let final_health = hits
        .last()
        .and_then(|line| line.split("; health ").nth(1))
        .and_then(|health| health.split(" -> ").nth(1))
        .and_then(|health| health.parse::<i32>().ok())
        .expect("a root hit must log Connor's final health");
    assert!(
        final_health > 0 && !fight.contains("Connor: dies"),
        "Connor did not survive the expiry boundary; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
