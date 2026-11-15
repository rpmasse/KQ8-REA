//! A settled weapon remains ready with its original movement bank after a disk reload.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn every_authored_weapon_keeps_its_ready_bank_after_disk_reload() {
    use kq8_data::characters::ConnorAnimLists;

    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let file = kq8_data::kq_text::parse(&fs::read(game.join("GAME/Kq/Connor.kq")).unwrap());
    let third = ConnorAnimLists::from_file(&file, &file[0], false);
    let first = ConnorAnimLists::from_file(&file, &file[0], true);
    let mut weapons = Vec::new();
    for (mode, items, banks) in [
        ("0", &first.hand, &third.hand),
        ("1", &first.ranged, &third.ranged),
    ] {
        for (item, _) in items {
            if item.eq_ignore_ascii_case("NoRanged") {
                continue;
            }
            let bank = ConnorAnimLists::equipped(banks, item)
                .unwrap_or_else(|| panic!("Connor.kq has no third-person bank for {item}"));
            weapons.push((item, mode, bank));
        }
    }
    assert_eq!(
        weapons.len(),
        22,
        "unexpected combat weapon count in Connor.kq"
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/weapon-ready-matrix/{nonce}"));

    for (item, mode, bank) in weapons {
        let base = root.join(item);
        let slots = base.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = base.join(part);
            fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir)
                .env("KQ8_SAVE_DIR", &slots)
                .arg("--game")
                .arg(&game)
                .args([
                    "--windowed",
                    "--renderer",
                    "dx12",
                    "--sound",
                    "off",
                    "--no-ai",
                    "--god",
                ])
                .args(args)
                .output()
                .unwrap();
            fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
            fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
            let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
            assert!(
                output.status.success() && viewer.contains("RESULT: PASS"),
                "{item} {part}: inspect {}",
                dir.display()
            );
            viewer
        };
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let ready = format!("KQConner::setDoMode {mode},35");
        let before = run("save", &["--world", "DeadCity", "--command", &give, "--command", &ready,
            "--key", "menu:down:down:enter:release:enter:release:escape:type=Ready:enter:release:escape,80",
            "--updates", "105"]);
        assert!(
            before.contains("Save screen -> Save(1, \"Ready\")"),
            "{item}: no first save; inspect {}",
            base.display()
        );
        let read = |slot| -> serde_json::Value {
            serde_json::from_slice(&fs::read(slots.join(format!("slot-{slot}.json"))).unwrap())
                .unwrap()
        };
        let check = |slot: u16| {
            let saved = read(slot);
            let ready = saved["journey"]["weapon_ready"].as_str().unwrap_or("");
            assert!(
                ready.eq_ignore_ascii_case(&format!("INVITEM_{item}")),
                "{item}: slot {slot} has wrong ready item {ready:?}; inspect {}",
                base.display()
            );
            let weapon = &saved["weapon_state"];
            for key in ["weapon_set", "weapon_target"] {
                let actual = weapon[key].as_str().unwrap_or("");
                assert!(
                    actual.eq_ignore_ascii_case(bank),
                    "{item}: slot {slot} {key} {actual:?} differs from GOG {bank:?}; inspect {}",
                    base.display()
                );
            }
            let expected_mode = if mode == "0" { "Hand" } else { "Ranged" };
            assert_eq!(
                weapon["set_mode"],
                expected_mode,
                "{item}: slot {slot} mode; inspect {}",
                base.display()
            );
            assert_eq!(
                weapon["target_mode"],
                expected_mode,
                "{item}: slot {slot} target mode; inspect {}",
                base.display()
            );
            assert_eq!(
                weapon["readying"], false,
                "{item}: slot {slot} did not finish the draw"
            );
            assert!(
                weapon["pending_set"].is_null(),
                "{item}: slot {slot} still has a pending switch"
            );
        };
        check(1);

        let after = run("restore", &["--menu-input", "down,enter,release,enter,release",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterReady:enter:release:escape,70",
            "--updates", "105"]);
        assert!(
            after.contains("save restored in DeadCity without replaying Begin")
                && after.contains("Save screen -> Save(2, \"AfterReady\")"),
            "{item}: no fresh-process reload/save; inspect {}",
            base.display()
        );
        let connor =
            fs::read_to_string(base.join("restore/kq8-artifacts/logs/connor.log")).unwrap();
        assert!(
            !connor.contains("weapon: the set switches now"),
            "{item}: ready bank switched again after reload; inspect {}",
            base.display()
        );
        check(2);
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
