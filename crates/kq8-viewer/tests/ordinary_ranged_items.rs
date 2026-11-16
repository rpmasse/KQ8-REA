//! Five ordinary installed bows and crossbows fire in one continuous encounter.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn ordinary_bows_and_crossbows_use_their_authored_projectiles() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/ordinary-ranged-items/{nonce}"));
    fs::create_dir_all(&dir).unwrap();

    let weapons = [
        ("SmCpndCBow", "Prsccblt.kq"),
        ("ShortBow", "Prarrow.kq"),
        ("LongBow", "Prarrow.kq"),
        ("CrossBow", "Prcbolt.kq"),
        ("CpndCrossbow", "Prccbolt.kq"),
    ];
    let mut args: Vec<String> = [
        "--windowed", "--sound", "off", "--no-ai", "--world", "DeadCity",
        "--key", "popup:autohelp,0",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--command", "sendEvent 0 skeleton1 KQStatsChangeRequestEvent 10000 0 0 0 0,6",
        "--command", "KQConner::setDoMode 1,35",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for (i, (item, _)) in weapons.iter().enumerate() {
        let at = 20 + i as u64 * 70;
        args.extend([
            "--command".into(),
            format!("KQMonster::inventory Connor INVITEM_{item} 1,{at}"),
            "--aim".into(),
            format!("skeleton1,{}", at + 40),
        ]);
    }
    args.extend(["--updates".into(), "390".into(), "--frames".into(), "9000".into()]);
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args(&args)
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(!fight.contains("god mode is on"), "god mode was used; inspect {}", dir.display());
    assert_eq!(fight.matches("skeleton1: hit by Connor, Projectile:").count(), weapons.len(),
        "not one Projectile hit per weapon; inspect {}", dir.display());

    for (i, (item, projectile)) in weapons.iter().enumerate() {
        let shot_at = 60 + i as u64 * 70;
        let bolt = format!("bolt{}", i + 1);
        assert!(items.contains(&format!("INVITEM_{item}: shoots Some(\"{projectile}\")")),
            "{item}: original projectile was not selected; inspect {}", dir.display());
        assert_eq!(fight.matches(&format!("shoots {bolt} ({projectile},")).count(), 1,
            "{item}: wrong projectile launch; inspect {}", dir.display());
        assert!(fight.lines().any(|line| line.contains(&format!("{bolt} update "))
            && line.contains("touches skeleton1")),
            "{item}: projectile did not touch the skeleton; inspect {}", dir.display());
        let hit = fight.lines().find(|line| {
            line.contains("skeleton1: hit by Connor, Projectile:")
                && line.strip_prefix("update ")
                    .and_then(|rest| rest.split_once(':'))
                    .and_then(|(n, _)| n.parse::<u64>().ok())
                    .is_some_and(|n| n >= shot_at && n < shot_at + 70)
        }).unwrap_or_else(|| panic!("{item}: no Projectile hit after update {shot_at}; inspect {}", dir.display()));
        let (before, after) = hit
            .split_once("; health ")
            .and_then(|(_, tail)| tail.split(';').next())
            .and_then(|health| health.split_once(" -> "))
            .map(|(before, after)| (before.parse::<i64>().unwrap(), after.parse::<i64>().unwrap()))
            .unwrap();
        assert!(after < before && after > 0,
            "{item}: target health did not fall while staying alive ({before} -> {after}); inspect {}", dir.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
