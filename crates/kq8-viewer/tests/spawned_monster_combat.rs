//! Script-created Dead City monsters fight under normal AI in one encounter.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn script_created_monsters_deal_damage_die_and_award_experience() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/spawned-monster-combat/{nonce}"
    ));
    fs::create_dir_all(&dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed", "--sound", "off", "--world", "DeadCity",
            "--item", "INVITEM_SmallAx", "--key", "popup:autohelp,0",
            "--command", "KQMonster::add testRat rat @ @ @ @ @ 183300 45000 864 0 @ @ @ @ @ @ @ @ @,10",
            "--swing", "testRat,100",
            "--swing", "testRat,130",
            "--command", "KQMonster::add testSkeleton skeleton @ @ @ @ @ 183300 45000 864 0 @ @ @ @ @ @ @ @ @,160",
            "--updates", "215", "--frames", "7000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("god mode is on"),
        "god mode was used; inspect {}",
        dir.display()
    );
    for (name, kind) in [("testRat", "rat"), ("testSkeleton", "skeleton")] {
        assert!(
            doors.contains(&format!("KQMonster::add {name} type {kind}")),
            "{name} did not spawn; inspect {}",
            dir.display()
        );
    }
    assert!(
        fight.contains("testRat: hit by Connor, Normal:")
            && fight.contains("testRat: its death ended")
            && fight.contains("Connor gets its experience 1")
            && viewer.contains("corpse of testRat: rt_dead.kqs drawn"),
        "rat did not complete its ordinary death and reward; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by testSkeleton, Normal:"))
        .collect();
    assert!(
        !hits.is_empty()
            && hits.iter().all(|line| {
                line.split_once("; health ")
                    .and_then(|(_, pair)| pair.split_once(" -> "))
                    .and_then(|(before, after)| {
                        Some((before.parse::<i32>().ok()?, after.parse::<i32>().ok()?))
                    })
                    .is_some_and(|(before, after)| before > after && after > 0)
            }),
        "the spawned skeleton did not damage a surviving Connor; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("Connor: dies"),
        "Connor died before the encounter ended; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
