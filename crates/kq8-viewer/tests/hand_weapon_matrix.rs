//! Each player hand weapon in the installed item list lands a live swing.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_hand_weapons_hit_with_their_authored_damage_kind() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/hand-weapon-matrix/{nonce}"));

    // The fixed Dead City start has 45 hand damage before Invinfo.itl's SDam.
    for (item, kind, damage) in [
        ("Dagger", "Normal", 50),
        ("SmallAx", "Normal", 60),
        ("BroadSword", "Normal", 62),
        ("Mace", "Normal", 65),
        ("LakeSword", "Normal", 70),
        ("SkelKingSword", "Normal", 75),
        ("BattleAx", "Normal", 110),
        ("WarHammer", "Normal", 95),
        ("Pike", "Normal", 75),
        ("LongSword", "Normal", 145),
        ("FlameSword", "Fire", 195),
        ("TempleSword", "Normal", 545),
    ] {
        let run = root.join(item);
        fs::create_dir_all(&run).unwrap();
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&run)
            .arg("--game")
            .arg(&game)
            .args([
                "--windowed",
                "--sound",
                "off",
                "--no-ai",
                "--god",
                "--world",
                "DeadCity",
            ])
            .args([
                "--command",
                &give,
                "--command",
                "move skeleton1 0 183300 45000 864 0 0 0,5",
                "--command",
                "KQConner::setDoMode 0,35",
                "--swing",
                "skeleton1,60",
                "--updates",
                "82",
            ])
            .output()
            .unwrap();
        fs::write(run.join("stderr.log"), &output.stderr).unwrap();
        let logs = run.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "{item}: inspect {}",
            run.display()
        );
        assert!(
            fight.contains(&format!("-> hand (Some(\"INVITEM_{item}\"))")),
            "{item} was not readied; inspect {}",
            run.display()
        );
        assert_eq!(
            fight
                .lines()
                .filter(|line| line.contains("Connor: Do (") && line.contains("hits skeleton1"))
                .count(),
            1,
            "{item} missed its authored swing cue; inspect {}",
            run.display()
        );
        let hit = format!("skeleton1: hit by Connor, {kind}:");
        let hits: Vec<_> = fight.lines().filter(|line| line.contains(&hit)).collect();
        assert_eq!(
            hits.len(),
            1,
            "{item} did not apply one {kind} hit; inspect {}",
            run.display()
        );
        assert!(
            hits[0].contains(&format!(": D {damage}, ")),
            "{item} did not apply its installed damage bonus; inspect {}",
            run.display()
        );
        let (before, after) = hits[0]
            .split_once("; health ")
            .and_then(|(_, tail)| tail.split(';').next())
            .and_then(|health| health.split_once(" -> "))
            .map(|(before, after)| {
                (
                    before.parse::<i64>().unwrap(),
                    after.parse::<i64>().unwrap(),
                )
            })
            .unwrap();
        assert!(
            after < before,
            "{item} did not reduce target health ({before} -> {after}); inspect {}",
            run.display()
        );
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
