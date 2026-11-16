//! Original-data Dead City crossbow fight with normal enemy AI and damage.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn small_crossbow_defeats_a_live_bow_skeleton() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/live-crossbow-combat/{nonce}"));
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
            "DeadCity",
            "--command",
            "move skeleton2 0 183300 45000 864 0 0 0,5",
            "--command",
            "KQConner::setDoMode 1,35",
            "--aim",
            "skeleton2,60",
            "--aim",
            "skeleton2,100",
            "--aim",
            "skeleton2,140",
            "--updates",
            "190",
            "--frames",
            "9000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("taken, none (None) -> ranged (Some(\"INVITEM_SmCrossbow\"))")
            && doors
                .matches("aim pick meets character skeleton2 (type 0x42)")
                .count()
                == 3,
        "the original crossbow was not readied or all three camera aims missed; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .lines()
            .filter(|line| {
                line.contains("Connor: do (conscbow.anm/attack") && line.contains("Prscbolt.kq")
            })
            .count(),
        3,
        "the original unlimited-ammunition crossbow did not fire three times; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("skeleton2: hit by Connor, Projectile:"))
        .collect();
    assert_eq!(
        hits.len(),
        3,
        "expected three real projectile hits; inspect {}",
        dir.display()
    );
    assert!(
        hits[0].contains("health 50 -> 33")
            && hits[1].contains("health 33 -> 16")
            && hits[2].contains("health 16 -> -5"),
        "the skeleton did not lose its original health through crossbow hits; inspect {}",
        dir.display()
    );
    let incoming: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by skeleton2, Projectile:"))
        .collect();
    assert!(
        incoming.len() >= 2
            && incoming[0].contains("health 80 -> ")
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
        "live bow arrows did not damage a surviving Connor; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("skeleton2: dies (rule 30): kind Projectile")
            .count(),
        1
    );
    let death = fight.lines().find(|line| line.contains("skeleton2: dies (rule 30): kind Projectile"))
        .unwrap();
    assert!(characters.contains("skeleton2: joins the active list (rule 20)")
        && death.contains("off the active list; gravity off; not solid (collision type and mask 0); its state never changes again"),
        "the live skeleton's ordinary death did not remove its active and solid states; inspect {}", dir.display());
    let drops = fight
        .lines()
        .find(|line| line.contains("skeleton2: drops (rule 31;"))
        .expect("bow skeleton must drop its loot");
    assert_eq!(drops.matches("INVITEM_Mushroom x1").count(), 2);
    assert_eq!(
        fight
            .matches("skeleton2: its death ended (rule 32): Connor gets its experience 150:")
            .count(),
        1
    );
    assert!(fight.contains("a corpse skbdead.kqs where it stood"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
