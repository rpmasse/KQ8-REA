//! A real Flame Sword Dragon fight continues across a disk checkpoint.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn first_live_dragon_head_defeat_survives_reload_and_second_head_falls() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/dragon-live-checkpoint/{nonce}"
    ));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--windowed", "--sound", "off", "--combat-level", "1"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, fight, doors)
    };
    let (first, first_fight, first_doors) = run(
        "save",
        &[
            "--world", "IceWorld", "--at", "182334,490176,0",
            "--command", "KQMonster::inventory Connor INVITEM_FlameSword 1,20",
            "--command", "KQConner::setDoMode 0,25",
            "--aim", "draglhed,40", "--aim", "draglhed,75",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=DragonLive:enter:release:escape,110",
            "--updates", "120", "--frames", "1050",
        ],
    );
    assert!(
        first.contains("Save screen -> Save(1, \"DragonLive\")")
            && first_doors.contains("swing pick meets character draglhed (type 0x4042)")
            && first_fight.matches("hits draglhed:").count() == 2
            && first_fight.contains("draglhed: Dragon head Die: Fire, health 0, non-solid; synge starts; permanently defeated")
            && first_fight.contains("Connor: hit by draglhed,")
            && !first_fight.contains("dragrhed: Dragon head Die:")
            && !first_fight.contains("god mode is on"),
        "first live head defeat was not saved; inspect {}",
        root.display()
    );
    let slot: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let actors = slot["kept"]["iceworld"]["characters"].as_array().unwrap();
    let actor = |name: &str| {
        actors
            .iter()
            .find(|a| a["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing from slot; inspect {}", root.display()))
    };
    assert_eq!(
        actor("draglhed")["character"]["dragon_head"]["phase"],
        "Permanent"
    );
    assert_eq!(
        actor("dragrhed")["character"]["dragon_head"]["phase"],
        "Alive"
    );
    assert!(actor("dragrhed")["character"]["fight"]["stats"][0]
        .as_i64()
        .is_some_and(|health| health > 0));
    assert_eq!(actor("dragbody")["character"]["state"], "Wait");
    assert!(slot["journey"]["stats"]["stats"][0]
        .as_i64()
        .is_some_and(|h| h > 0));
    assert_eq!(slot["journey"]["weapon_ready"], "INVITEM_FlameSword");

    let (restored, fight, doors) = run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--aim",
            "dragrhed,35",
            "--aim",
            "dragrhed,70",
            "--updates",
            "200",
            "--frames",
            "1450",
        ],
    );
    assert!(
        restored.contains("save restored in IceWorld without replaying Begin")
            && doors.contains("swing pick meets character dragrhed (type 0x4042)")
            && fight.matches("hits dragrhed:").count() == 2
            && fight.contains("dragrhed: Dragon head Die: Fire, health 0, non-solid; synge starts; permanently defeated")
            && fight.contains("dragbody: both heads permanently defeated: body enters Die")
            && fight.contains("dragbody: Dragon body 'die' movement ended")
            && fight.contains("Connor: hit by dragrhed,")
            && !fight.contains("draglhed: Dragon head rebirth starts")
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "the restored live Dragon fight did not finish; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
