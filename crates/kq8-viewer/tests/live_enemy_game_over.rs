//! A normal enemy attack can take Connor through death to the title menu.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_roots_kill_connor_and_open_the_title_menu_once() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/live-enemy-game-over/{nonce}"));
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
            "--frames",
            "210",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
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
    let final_hit = hits.last().expect("at least one root hit");
    let (before, after) = final_hit
        .split_once("; health ")
        .and_then(|(_, health)| health.split_once(" -> "))
        .expect("root health transition");
    let before: i32 = before.parse().unwrap();
    let after: i32 = after.parse().unwrap();
    assert!(
        before > 0 && after <= 0,
        "the last root hit did not kill Connor: {final_hit}"
    );
    assert_eq!(
        fight
            .lines()
            .filter(|line| line.contains("Connor: hit by mand3") && line.contains("dies (Normal)"))
            .count(),
        1,
        "Connor did not enter Normal death exactly once; inspect {}",
        dir.display()
    );
    let die_end = viewer
        .find("die finished: Die state holds until update")
        .expect("death movement must finish");
    let hold_end = viewer
        .find("scripted Die hold finished;")
        .expect("death hold must finish");
    let menu_marker = "menu: shown Choice { game: false, row: 0";
    let menu = viewer.find(menu_marker).expect("title menu must open");
    assert!(
        die_end < hold_end && hold_end < menu && viewer.matches(menu_marker).count() == 1,
        "death animation, hold and menu did not occur once in order; inspect {}",
        dir.display()
    );
    assert!(
        connor.contains("die finished: Die state holds until update")
            && viewer[menu..].contains("world clock stopped at Some("),
        "the death did not settle at the title menu; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
