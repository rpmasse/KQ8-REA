//! One original-data fight delivers the hit, low-health and first-kill help.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn goblin_hit_and_death_deliver_the_four_combat_help_pages() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/combat-tutorial-delivery/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .env("KQ8_SAVE_DIR", dir.join("slots"))
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--world",
            "Daventry",
            "--at",
            "289284,190000,0",
            "--key",
            "popup:autohelp,0",
            "--command",
            "KQConner::reduceHealth .5 0,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_Mushroom 1,85",
            "--key",
            "1,90",
            "--command",
            "KQMonster::inventory Connor INVITEM_SmCrossbow 1,105",
            "--command",
            "KQConner::setDoMode 1,115",
            "--aim",
            "goblin22,140",
            "--aim",
            "goblin22,175",
            "--aim",
            "goblin22,225",
            "--aim",
            "goblin22,245",
            "--updates",
            "480",
            "--frames",
            "12000",
        ])
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
    assert!(
        fight.contains("Connor: hit by goblin22, Normal:")
            && fight.contains("; health 5 -> ")
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: hit by goblin22: dies"),
        "the unarmed low-health hit or ordinary damage was missed; inspect {}",
        dir.display()
    );
    assert!(
        items.contains("slot 1, INVITEM_Mushroom, HealthItem")
            && items.contains("INVITEM_Mushroom, slot used"),
        "the original Mushroom was not used through inventory; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("goblin22: dies (rule 30): kind Projectile")
            .count(),
        1,
        "the goblin did not die once from the regular crossbow; inspect {}",
        dir.display()
    );
    let mut previous = 0;
    for case in [140, 130, 137, 149] {
        let opened = format!("starting help: 500.MSG case {case} opened");
        let acknowledged = format!("starting help: acknowledged case {case}");
        let at = viewer
            .find(&opened)
            .unwrap_or_else(|| panic!("case {case} did not open; inspect {}", dir.display()));
        let ack = viewer.find(&acknowledged).unwrap_or_else(|| {
            panic!(
                "case {case} was not acknowledged; inspect {}",
                dir.display()
            )
        });
        assert!(
            previous <= at && at < ack,
            "combat help order failed at case {case}; inspect {}",
            dir.display()
        );
        previous = ack;
        assert_eq!(viewer.matches(&opened).count(), 1);
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
