//! Stored loose loot and collected items survive an intervening Castle save.
use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn items(slot: &Value) -> &[Value] {
    slot["items"].as_array().unwrap()
}

fn purse(slot: &Value) -> i64 {
    slot["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["kind"] == "INVITEM_SilverCoins")
        .map(|o| o["quantity"].as_i64().unwrap())
        .sum()
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn region_keeps_named_loot_and_omits_collected_start_items() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/regional-items/{nonce}"));
    let slots = root.join("slots");
    let run = |label: &str, args: &[&str]| {
        let dir = root.join(label);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off", "--no-ai", "--frames", "2400"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        viewer
    };
    let slot = |n| -> Value {
        serde_json::from_slice(
            &fs::read(slots.join(format!("slot-{n}.json")))
                .unwrap_or_else(|e| panic!("{e}; inspect {}", root.display())),
        )
        .unwrap()
    };
    // WorldInv.cs creates the six coins at this position and registers
    // ring.cs on NoName3. No quest or combat outcome is forced here.
    let outward = run(
        "outward",
        &[
            "--menu-input",
            "enter,release,enter,release",
            "--key",
            "popup:help=144,0",
            "--console",
            "move Connor 0 288939 194074 13970 0 0 0,300",
            "--click",
            "NoName4,306",
            "--console",
            "KQMonster::inventory world SilverCoins 7 158281 305600 9742 0 0 0,312",
            "--console",
            "KQObject::setHidden NoName3 1,314",
            "--console",
            "move Connor 0 158400 308505 9720 0 0 3.14,335",
            "--key",
            "popup:yes,335",
            "--key",
            "menu:down:down:enter:release:enter:release:escape:type=Away:enter:release:escape,388",
            "--key",
            "menu:up:enter:release:left:enter:release,398",
        ],
    );
    assert!(outward.contains("portal1 gets Accept"));
    let away = slot(1);
    assert_eq!(away["world"], "CastleD");
    assert!(away["kept"]["daventry"]["vars"].as_array().unwrap().iter()
        .any(|v| v[0] == "Daventry::girlhut::FirstTime" && v[1] == "0"),
        "the original New Game opening must have completed on the first visit");
    let stored = away["kept"]["daventry"]["items"]["objects"]
        .as_array()
        .unwrap();
    let ring = stored.iter().find(|o| o["name"] == "NoName3").unwrap();
    assert_eq!(ring["script"], "ring.cs");
    assert_eq!(ring["hidden"], true);
    assert!(
        !stored.iter().any(|o| o["name"] == "NoName4"),
        "the carried starting pile must leave Daventry"
    );
    let console = fs::read_to_string(root.join("outward/kq8-artifacts/logs/console.log")).unwrap();
    let made = console
        .lines()
        .find(|line| {
            line.contains("console: made ")
                && line.contains("INVITEM_SilverCoins x7 in the world at (158281, 305600, 9742)")
        })
        .unwrap();
    let name = made.split_once("NewInvItem = ").unwrap().1.trim();
    let loot = stored.iter().find(|o| o["name"] == name).unwrap();
    assert_eq!(
        loot["place"]["World"]["position"],
        serde_json::json!([158281.0, 305600.0, 9742.0])
    );

    let returned = run("return", &["--menu-input", "down,enter,release,enter,release",
        "--console", "move Connor 0 6005 111600 195 0 0 0,22",
        "--key", "popup:yes,22",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Before:enter:release:escape,60",
        "--click", &format!("{name},73"), "--click", &format!("{name},80"),
        "--console", "KQMonster::inventory world SilverCoins 2 158281 305600 9742 0 0 0,86",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=After:enter:release:escape,98",
        "--key", "menu:up:enter:release:left:enter:release,108"]);
    assert!(
        returned.contains("save restored in CastleD without replaying Begin")
            && returned.contains("portal2 gets Accept")
    );
    let before = slot(2);
    assert_eq!(before["world"], "Daventry");
    assert_eq!(
        items(&before)
            .iter()
            .filter(|o| o["name"] == name)
            .collect::<Vec<_>>(),
        vec![loot]
    );
    assert_eq!(
        items(&before)
            .iter()
            .filter(|o| o["kind"] == "INVITEM_RingOfDeadHero")
            .collect::<Vec<_>>(),
        vec![ring]
    );
    assert!(!items(&before)
        .iter()
        .any(|o| o["name"] == "NoName4" && o["place"]["World"].is_object()));
    assert_eq!(purse(&before), purse(&away));
    let after = slot(3);
    assert_eq!(
        purse(&after),
        purse(&before) + 7,
        "a second click cannot collect it again"
    );
    let next = before["journey"]["next_no_name"].as_u64().unwrap();
    let new_name = format!("NoName{next}");
    assert!(items(&after)
        .iter()
        .any(|o| o["name"] == new_name && o["quantity"] == 2));
    assert_eq!(after["journey"]["next_no_name"], next + 1);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}
