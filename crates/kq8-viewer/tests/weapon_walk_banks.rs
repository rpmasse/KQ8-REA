//! The drawn walking clip follows each installed weapon's animation bank.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn all_authored_weapons_draw_their_own_walk() {
    use kq8_data::characters::ConnorAnimLists;

    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let file = kq8_data::kq_text::parse(&fs::read(game.join("GAME/Kq/Connor.kq")).unwrap());
    let lists = ConnorAnimLists::from_file(&file, &file[0], false);
    let fists = lists
        .hand
        .iter()
        .find(|(item, _)| item.eq_ignore_ascii_case("Fists"))
        .unwrap();
    let hand: Vec<_> = lists
        .hand
        .iter()
        .filter(|(item, _)| !item.eq_ignore_ascii_case("Fists"))
        .collect();
    let ranged: Vec<_> = lists
        .ranged
        .iter()
        .filter(|(item, _)| !item.eq_ignore_ascii_case("NoRanged"))
        .collect();
    assert_eq!(1 + hand.len() + ranged.len(), 22);

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/weapon-walk-banks/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let mut args: Vec<String> = [
        "--windowed",
        "--renderer",
        "dx12",
        "--sound",
        "off",
        "--no-ai",
        "--god",
        "--world",
        "DeadCity",
        "--at",
        "190000,45000,0",
        "--key",
        "popup:autohelp,0",
        "--command",
        "KQMonster::inventory Connor INVITEM_Fists 1,20",
        "--command",
        "KQConner::setDoMode 0,35",
        "--hold",
        "1,0,0,50,70",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let mut checks = vec![(fists.0.clone(), fists.1.clone(), 50_u64, 70_u64)];
    for (i, (item, bank)) in hand.into_iter().enumerate() {
        let at = 70 + i as u64 * 26;
        args.extend([
            "--command".into(),
            format!("KQMonster::inventory Connor INVITEM_{item} 1,{at}"),
            "--hold".into(),
            format!("1,0,0,{},{}", at + 5, at + 25),
        ]);
        checks.push((item.clone(), bank.clone(), at, at + 26));
    }
    for (i, (item, bank)) in ranged.into_iter().enumerate() {
        let at = if i == 0 {
            400
        } else {
            460 + (i as u64 - 1) * 26
        };
        args.extend([
            "--command".into(),
            format!("KQMonster::inventory Connor INVITEM_{item} 1,{at}"),
        ]);
        if i == 0 {
            args.extend([
                "--command".into(),
                format!("KQConner::setDoMode 1,{}", at + 10),
            ]);
        }
        let from = if i == 0 { at + 30 } else { at + 5 };
        let until = if i == 0 { at + 55 } else { at + 25 };
        args.extend(["--hold".into(), format!("1,0,0,{from},{until}")]);
        checks.push((
            item.clone(),
            bank.clone(),
            if i == 0 { from } else { at },
            if i == 0 { at + 60 } else { at + 26 },
        ));
    }
    args.extend([
        "--updates".into(),
        "720".into(),
        "--frames".into(),
        "12000".into(),
    ]);
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .env("KQ8_SAVE_DIR", dir.join("slots"))
        .arg("--game")
        .arg(&game)
        .args(&args)
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    for (item, bank, from, until) in checks {
        let expected = format!("drawn movement {bank}/walk ");
        let selected = connor.lines().any(|line| {
            let Some(rest) = line.strip_prefix("connor update ") else {
                return false;
            };
            let Some((update, detail)) = rest.split_once(": ") else {
                return false;
            };
            update
                .parse::<u64>()
                .ok()
                .is_some_and(|n| n >= from && n < until)
                && detail
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
        });
        assert!(
            selected,
            "{item}: {bank}/walk was not drawn during updates {from}..{until}; inspect {}",
            dir.display()
        );
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
