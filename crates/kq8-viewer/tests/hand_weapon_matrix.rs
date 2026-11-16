//! Every installed hand weapon lands a live hit during one continuous fight.
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
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/hand-weapon-matrix/{nonce}"));
    fs::create_dir_all(&dir).unwrap();

    // The fixed Dead City start has 45 hand damage before Invinfo.itl's SDam.
    let weapons = [
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
    ];
    let mut args: Vec<String> = [
        "--windowed", "--sound", "off", "--no-ai", "--world", "DeadCity",
        "--key", "popup:autohelp,0",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--command", "sendEvent 0 skeleton1 KQStatsChangeRequestEvent 10000 0 0 0 0,6",
        "--command", "KQConner::setDoMode 0,35",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for (i, (item, _, _)) in weapons.iter().enumerate() {
        let at = 20 + i as u64 * 60;
        args.extend([
            "--command".into(),
            format!("KQMonster::inventory Connor INVITEM_{item} 1,{at}"),
            "--swing".into(),
            format!("skeleton1,{}", at + 40),
        ]);
    }
    args.extend(["--updates".into(), "760".into(), "--frames".into(), "15000".into()]);
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

    let hit_lines: Vec<_> = fight.lines().filter(|line| line.contains("skeleton1: hit by Connor, ")).collect();
    assert_eq!(hit_lines.len(), weapons.len(), "not one hit per weapon; inspect {}", dir.display());
    for (i, (item, kind, damage)) in weapons.iter().enumerate() {
        let swing_at = 60 + i as u64 * 60;
        let ready = if i == 0 {
            fight.contains(&format!("-> hand (Some(\"INVITEM_{item}\"))"))
        } else {
            items.contains(&format!("INVITEM_{item} is ready now"))
        };
        assert!(ready,
            "{item} was not readied; inspect {}", dir.display());
        let hit = hit_lines.iter().find(|line| {
            line.strip_prefix("update ")
                .and_then(|rest| rest.split_once(':'))
                .and_then(|(n, _)| n.parse::<u64>().ok())
                .is_some_and(|n| n >= swing_at && n < swing_at + 60)
        }).unwrap_or_else(|| panic!("{item}: no hit after update {swing_at}; inspect {}", dir.display()));
        assert!(hit.contains(&format!("skeleton1: hit by Connor, {kind}: D {damage}, ")),
            "{item}: wrong damage or hit kind: {hit}");
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
