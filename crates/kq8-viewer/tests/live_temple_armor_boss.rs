//! Temple Armour's late-loaded weapon bank works through a live boss fight.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn temple_armor_flame_sword_defeats_live_skeleton_king() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/live-temple-armor-boss/{nonce}"
    ));
    let dir = root.join("save");
    let slots = root.join("slots");
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .env("KQ8_SAVE_DIR", &slots)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--combat-level",
            "1",
            "--world",
            "DeadCity",
            "--command",
            "move Skelking 0 183300 45000 864 0 0 0,5",
            "--command",
            "KQMonster::setState Skelking Wait,10",
            "--command",
            "KQMonster::inventory Connor INVITEM_FlameSword 1,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_TempleArmor 1,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_Invulnerable 1,20",
            "--key",
            "5,25",
            "--command",
            "KQConner::setDoMode 0,35",
            "--aim",
            "Skelking,60",
            "--aim",
            "Skelking,78",
            "--aim",
            "Skelking,120",
            "--aim",
            "Skelking,138",
            "--aim",
            "Skelking,170",
            "--frames",
            "400",
            "--key",
            "menu:down:down:enter:release:enter:release:escape:type=KingVictory:enter:release:escape,240",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        items.contains("Some(\"INVITEM_FlameSword\") -> contpsd2.anm")
            && items.contains("Some(\"INVITEM_SmCrossbow\") -> conicbw2.anm")
            && items.contains("INVITEM_TempleArmor, put on")
            && items.contains("INVITEM_Invulnerable, slot used): the shield: protection 42 -> 63"),
        "the original armour and shield did not apply; inspect {}",
        dir.display()
    );
    for movement in [
        "contpsd2.anm/attack",
        "contpsd2.anm/attackb",
        "contpsd2.anm/away",
        "conicbw2.anm/attack",
        "conicbw2.anm/away",
    ] {
        assert!(
            viewer.contains(&format!("connor: movement {movement} = movement")),
            "late weapon list omitted {movement}; inspect {}",
            dir.display()
        );
    }
    assert!(
        fight.matches("Skelking: hit by Connor, Fire:").count() >= 4,
        "authored Flame Sword cues did not defeat the boss; inspect {}",
        dir.display()
    );
    let incoming: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by Skelking, Normal:"))
        .collect();
    assert!(
        incoming.len() >= 2
            && incoming.iter().all(|line| line.contains("P 63"))
            && incoming.iter().all(|line| {
                line.split_once("; health ")
                    .and_then(|(_, pair)| pair.split_once(" -> "))
                    .and_then(|(before, after)| {
                        Some((before.parse::<i32>().ok()?, after.parse::<i32>().ok()?))
                    })
                    .is_some_and(|(before, after)| before > after && after > 0)
            })
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "the live boss did not damage a surviving Connor; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("Skelking: dies (rule 30): kind Fire").count(),
        1
    );
    assert_eq!(
        fight
            .matches("Skelking: its death ended (rule 32): Connor gets its experience 20000:")
            .count(),
        1
    );
    assert!(
        fight.contains("no corpse (Fire: burnt away)")
            && doors.contains("script skelking.cs Skelking Die Fire")
            && doors.contains("set DeadCity::Reaper::ConFoundBridge = 'true'"),
        "the authored Fire death did not complete; inspect {}",
        dir.display()
    );
    let drops = fight
        .lines()
        .filter(|line| line.contains("Skelking: drops (rule 31;"))
        .collect::<Vec<_>>();
    assert_eq!(drops.len(), 1, "inspect {}", dir.display());
    for item in [
        "INVITEM_SkelKingSword x1",
        "INVITEM_SilverCoins x25",
        "INVITEM_invulnerable x1",
    ] {
        assert!(
            drops[0].contains(item),
            "missing {item}; inspect {}",
            dir.display()
        );
    }
    assert_eq!(drops[0].matches("INVITEM_crystal x1").count(), 2);
    for item in [
        "INVITEM_SkelKingSword",
        "INVITEM_SilverCoins",
        "INVITEM_Invulnerable",
    ] {
        assert!(
            items.contains(&format!("({item}) flight ends")),
            "{item} did not finish its throw; inspect {}",
            dir.display()
        );
    }
    assert_eq!(items.matches("(INVITEM_Crystal) flight ends").count(), 2);
    assert!(
        viewer.contains("Save screen -> Save(1, \"KingVictory\")"),
        "victory did not reach the save menu; inspect {}",
        dir.display()
    );
    let slot: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let kept = &slot["kept"]["deadcity"];
    assert!(
        kept["gone"]
            .as_array()
            .is_some_and(|names| names.iter().any(|name| {
                name.as_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case("Skelking"))
            })),
        "the King was not removed in the disk checkpoint; inspect {}",
        root.display()
    );
    assert!(
        kept["vars"].as_array().is_some_and(|pairs| pairs
            .iter()
            .any(|pair| { pair[0] == "DeadCity::Reaper::ConFoundBridge" && pair[1] == "true" })),
        "the death-script flag was not saved; inspect {}",
        root.display()
    );
    for (kind, count) in [
        ("INVITEM_SkelKingSword", 1),
        ("INVITEM_SilverCoins", 1),
        ("INVITEM_Crystal", 2),
        ("INVITEM_Invulnerable", 1),
    ] {
        assert_eq!(
            slot["items"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|item| {
                    let at = &item["place"]["World"]["position"];
                    item["kind"] == kind
                        && at[0]
                            .as_f64()
                            .is_some_and(|x| (x - 183300.0).abs() < 5000.0)
                        && at[1].as_f64().is_some_and(|y| (y - 45000.0).abs() < 5000.0)
                })
                .count(),
            count,
            "{kind} not saved at the victory site; inspect {}",
            root.display()
        );
    }
    let saved_health = slot["journey"]["stats"]["stats"][0].as_i64().unwrap();
    assert!(saved_health > 0);
    assert_eq!(slot["journey"]["weapon_mode"], "Hand");
    assert_eq!(slot["journey"]["weapon_ready"], "INVITEM_FlameSword");
    let restored_dir = root.join("restore");
    fs::create_dir_all(&restored_dir).unwrap();
    let restored = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&restored_dir)
        .env("KQ8_SAVE_DIR", &slots)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--menu-input",
            "down,enter,release,enter,release",
            "--key",
            "menu:down:down:down:enter:release:enter:release:escape:type=KingRestored:enter:release:escape,30",
            "--updates",
            "60",
            "--frames",
            "1000",
        ])
        .output()
        .unwrap();
    fs::write(restored_dir.join("stderr.log"), &restored.stderr).unwrap();
    let restored_viewer =
        fs::read_to_string(restored_dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
    let restored_items =
        fs::read_to_string(restored_dir.join("kq8-artifacts/logs/items.log")).unwrap();
    assert!(
        restored.status.success()
            && restored_viewer.contains("RESULT: PASS")
            && restored_viewer.contains("save restored in DeadCity without replaying Begin")
            && restored_viewer.contains("1 objects that left it not loaded again: [\"Skelking\"]")
            && restored_viewer.contains("Save screen -> Save(2, \"KingRestored\")")
            && restored_viewer.contains(&format!(
                "gui: update 1: the screen opens: Connor [{saved_health},"
            ))
            && restored_viewer.contains("connor: weapon set contpsd2.anm")
            && restored_items.contains("INVITEM_FlameSword x1 carried by Connor")
            && restored_items.contains("INVITEM_TempleArmor x1 carried by Connor"),
        "victory did not reopen in a fresh process; inspect {}",
        restored_dir.display()
    );
    let second: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-2.json")).unwrap()).unwrap();
    assert!(
        second["kept"]["deadcity"]["vars"]
            .as_array()
            .is_some_and(|pairs| pairs.iter().any(|pair| {
                pair[0] == "DeadCity::Reaper::ConFoundBridge" && pair[1] == "true"
            })),
        "restored bridge flag was not saved again; inspect {}",
        root.display()
    );
    for item in slot["items"].as_array().unwrap().iter().filter(|item| {
        let at = &item["place"]["World"]["position"];
        at[0]
            .as_f64()
            .is_some_and(|x| (x - 183300.0).abs() < 5000.0)
            && at[1].as_f64().is_some_and(|y| (y - 45000.0).abs() < 5000.0)
    }) {
        let name = item["name"].as_str().unwrap();
        let kind = item["kind"].as_str().unwrap();
        let quantity = item["quantity"].as_u64().unwrap();
        assert!(
            restored_items.contains(&format!("{name}: {kind} x{quantity} in the world")),
            "restored drop missing {name}: {kind}; inspect {}",
            restored_dir.display()
        );
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
