//! The installed Gnome Dragon's light prerequisite, live combat and reward.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn gnome_dragon_fights_dies_and_leaves_the_black_diamond() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/gnome-dragon-boss/{nonce}"));
    let dir = root.join("fight");
    let slots = root.join("slots");
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
            "--world",
            "Gnome",
            "--at",
            "100117,136500,0",
            // Connor starts on the authored chamber floor. The original
            // Dragon stays at its placed Gnome position throughout.
            "--command",
            "move Connor 0 100117 136500 14901 0 0 0,4",
            // A strong installed weapon keeps this boss test bounded. The
            // ordinary attack, damage roll, death and drop paths still run.
            "--command",
            "KQMonster::inventory Connor INVITEM_TempleSword 1,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_Invulnerable 1,20",
            "--key",
            "5,25",
            "--command",
            "KQConner::setDoMode 0,35",
            // Fusebox Init first sets darkness. Its authored Cue then
            // restores the Dragon's attack collision before Connor fights.
            "--command",
            "sendEvent 0 fusebox KQCueEvent 10 0 on,65",
            "--swing",
            "Dragon,100",
            "--swing",
            "Dragon,130",
            "--swing",
            "Dragon,160",
            "--swing",
            "Dragon,190",
            "--swing",
            "Dragon,220",
            "--swing",
            "Dragon,250",
            "--swing",
            "Dragon,280",
            "--key",
            "menu:down:down:enter:release:enter:release:escape:type=DragonVictory:enter:release:escape,375",
            "--updates",
            "440",
            "--frames",
            "5500",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("collisionType Dragon -64: 0x4002 -> 0x4002")
            && doors.contains("collisionType Dragon +64: 0x4002 -> 0x4042"),
        "the original fuse-box lighting prerequisite failed; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("Dragon: attack (rule 7, standing)")
            && fight.contains("Connor: hit by Dragon, Normal: D 85")
            && !fight.contains("god mode is on"),
        "the Dragon did not attack through ordinary AI; inspect {}",
        dir.display()
    );
    assert!(
        items.contains("slot 5, INVITEM_Invulnerable, HealthItem")
            && items.contains("the shield: protection 32 -> 50"),
        "the original shield did not protect Connor; inspect {}",
        dir.display()
    );
    assert!(
        talk.contains("I cannot see my opponent!")
            && doors.contains("script dragworm.cs Dragon EndLoop Attack"),
        "the dark-chamber attack callback did not speak; inspect {}",
        dir.display()
    );
    assert!(
        fight.matches("Dragon: hit by Connor, Normal:").count() >= 4
            && fight.contains("Dragon: dies (rule 30): kind Normal"),
        "ordinary sword hits did not defeat the Dragon; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("Dragon: its death ended (rule 32): Connor gets its experience 25000:")
            .count(),
        1,
        "the Dragon's death or experience completed more than once; inspect {}",
        dir.display()
    );
    assert!(
        items.contains("Dragon: kept weapon (rule 31a):")
            && items.contains("(BlackDiamond x1) laid at"),
        "the Black Diamond did not leave the Die animation; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("script dragworm.cs Dragon Die Normal")
            && doors.contains("addMapIcon gn_dworm.bmp 235 91")
            && doors.contains("script Some(\"blackdmd.cs\")")
            && doors.contains("deleteObject dragblck: placed object deleted"),
        "the authored defeat script did not finish; inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("Save screen -> Save(1, \"DragonVictory\")"),
        "the victory was not saved; inspect {}",
        dir.display()
    );
    let slot: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let gone = slot["kept"]["gnome"]["gone"].as_array().unwrap();
    for name in ["dragon", "dragblck"] {
        assert!(
            gone.iter()
                .any(|entry| entry.as_str().is_some_and(|s| s.eq_ignore_ascii_case(name))),
            "{name} was not saved as gone; inspect {}",
            root.display()
        );
    }
    let diamonds: Vec<_> = slot["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["kind"] == "INVITEM_BlackDiamond")
        .collect();
    assert_eq!(
        diamonds.len(),
        1,
        "the Black Diamond was not saved once; inspect {}",
        root.display()
    );
    assert_eq!(diamonds[0]["script"], "blackdmd.cs");
    let saved_experience = slot["journey"]["stats"]["stats"][4].as_i64().unwrap();
    assert!(saved_experience >= 25000);
    assert!(slot["kept"]["gnome"]["vars"]
        .as_array()
        .unwrap()
        .iter()
        .any(|var| var[0] == "Gnome::Dragon::lights" && var[1] == "on"));
    let gnome_map = &slot["journey"]["other_maps"]["3"][1];
    assert!(
        gnome_map["icons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|icon| { icon["file"] == "gn_dworm.bmp" && icon["x"] == 235 && icon["y"] == 91 }),
        "the Dragon map icon was not saved; inspect {}",
        root.display()
    );

    let restored_dir = root.join("restore");
    fs::create_dir_all(&restored_dir).unwrap();
    let restored = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&restored_dir)
        .env("KQ8_SAVE_DIR", &slots)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed", "--renderer", "dx12", "--sound", "off",
            "--menu-input", "down,enter,release,enter,release",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=DragonRestored:enter:release:escape,30",
            "--updates", "65", "--frames", "2500",
        ])
        .output()
        .unwrap();
    fs::write(restored_dir.join("stdout.log"), &restored.stdout).unwrap();
    fs::write(restored_dir.join("stderr.log"), &restored.stderr).unwrap();
    let restored_logs = restored_dir.join("kq8-artifacts/logs");
    let restored_viewer = fs::read_to_string(restored_logs.join("viewer.log")).unwrap();
    let restored_fight = fs::read_to_string(restored_logs.join("fight.log")).unwrap();
    assert!(
        restored.status.success()
            && restored_viewer.contains("RESULT: PASS")
            && restored_viewer.contains("save restored in Gnome without replaying Begin")
            && restored_viewer.contains("Save screen -> Save(2, \"DragonRestored\")")
            && !restored_fight.contains("Dragon: its death ended"),
        "the Dragon victory did not resume in a fresh process; inspect {}",
        restored_dir.display()
    );
    let second: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-2.json")).unwrap()).unwrap();
    assert_eq!(second["journey"]["stats"]["stats"][4], saved_experience);
    assert_eq!(second["items"].as_array().unwrap().iter()
        .filter(|item| item["kind"] == "INVITEM_BlackDiamond" && item["script"] == "blackdmd.cs")
        .count(), 1);
    for name in ["dragon", "dragblck"] {
        assert!(second["kept"]["gnome"]["gone"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry.as_str().is_some_and(|s| s.eq_ignore_ascii_case(name))));
    }
    assert!(second["kept"]["gnome"]["vars"]
        .as_array()
        .unwrap()
        .iter()
        .any(|var| var[0] == "Gnome::Dragon::lights" && var[1] == "on"));
    assert_eq!(
        second["journey"]["other_maps"]["3"][1]["icons"],
        gnome_map["icons"]
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
