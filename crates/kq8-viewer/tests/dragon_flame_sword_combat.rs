//! Isolated IceWorld fight through real Flame Sword and Dragon animation cues.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn flame_sword_defeats_both_dragon_heads_with_normal_damage() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/dragon-flame-sword-combat/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--combat-level",
            "1",
            "--world",
            "IceWorld",
            "--at",
            "182334,490176,0",
            // The regional start script equips a Long Sword first; supply
            // the Flame Sword after it and ready the hand slot before swings.
            "--command",
            "KQMonster::inventory Connor INVITEM_FlameSword 1,20",
            "--command",
            "KQConner::setDoMode 0,25",
            "--swing",
            "draglhed,40",
            "--swing",
            "draglhed,75",
            "--swing",
            "dragrhed,135",
            "--swing",
            "dragrhed,170",
            "--updates",
            "230",
        ])
        .output()
        .unwrap();
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
        items.contains("INVITEM_FlameSword x1 carried by Connor")
            && fight.contains("confswd.anm/attack"),
        "Flame Sword not equipped; inspect {}",
        dir.display()
    );
    for head in ["draglhed", "dragrhed"] {
        assert_eq!(
            fight.matches(&format!("hits {head}:")).count(),
            2,
            "{head} did not take two Flame Sword cues; inspect {}",
            dir.display()
        );
        assert_eq!(fight.matches(&format!("{head}: Dragon head Die: Fire, health 0, non-solid; synge starts; permanently defeated")).count(), 1,
            "{head} did not stay permanently defeated; inspect {}", dir.display());
        assert!(
            !fight.contains(&format!("{head}: Dragon head rebirth starts")),
            "{head} rebirthed after Flame Sword defeat; inspect {}",
            dir.display()
        );
    }
    assert!(
        fight.contains("Connor: hit by dragrhed, Fire: D")
            && fight.contains("Connor: hit by draglhed, Fire: D")
            && !fight.contains("god mode is on"),
        "ordinary enemy damage was bypassed; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("dragbody: both heads permanently defeated: body enters Die")
            && fight.contains("dragbody: dragdbox.kq load result 0"),
        "two-head Dragon finish missing; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
