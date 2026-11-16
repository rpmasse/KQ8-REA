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
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/live-temple-armor-boss/{nonce}"
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
            "210",
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
            && doors.contains("script skelking.cs Skelking Die Fire"),
        "the authored Fire death did not complete; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
