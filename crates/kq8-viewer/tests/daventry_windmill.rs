//! Resume at the first wizard meeting, enter the windmill and earn the Small Axe.
//! By default the New Game route earns the wizard checkpoint in a separate process.
#[path = "support/daventry.rs"]
mod daventry;

use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn quantity(slot: &Value, name: &str) -> i64 {
    slot["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| {
            i["kind"]
                .as_str()
                .is_some_and(|k| k.eq_ignore_ascii_case(name))
        })
        .map(|i| i["quantity"].as_i64().unwrap())
        .sum()
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn wizard_to_windmill_archer_bale_and_axe() {
    let game = daventry::game();
    let root = daventry::test_root("windmill-route");
    let slots = root.join("slots");
    fs::create_dir_all(&slots).unwrap();
    if let Some(existing) = std::env::var_os("KQ8_WIZARD_SLOT") {
        fs::copy(existing, slots.join("slot-1.json")).expect("copy the earned wizard slot");
    } else {
        daventry::earn_magic_map(&game, &root);
    }
    let read_slot = |file: &str| -> Value {
        serde_json::from_slice(&fs::read(slots.join(file)).unwrap()).unwrap()
    };
    let start = read_slot("slot-1.json");
    assert!(start["world"]
        .as_str()
        .is_some_and(|s| s.eq_ignore_ascii_case("daventry")));
    assert_eq!(start["journey"]["god_mode"], false);
    assert_eq!(start["journey"]["ai_off"], false);
    assert_eq!(quantity(&start, "INVITEM_MagicMap"), 1);
    assert_eq!(quantity(&start, "INVITEM_SmallAx"), 0);
    assert_eq!(quantity(&start, "INVITEM_Mushroom"), 0);

    let replay = root.join("windmill");
    fs::create_dir_all(&replay).unwrap();
    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_windmill.json");
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&replay).env("KQ8_SAVE_DIR", &slots)
        .arg("--game").arg(&game)
        .args(["--sound", "off", "--menu-input", "down,enter,release,enter,release"])
        .args(["--key", "popup:help=142,0"]) // pushing.cs, inside the windmill.
        .arg("--walk-route").arg(route)
        .args(["--key", "menu:down:down:down:enter:release:enter:release:escape:type=Hillside:enter:release:escape,838",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Windmill:enter:release:escape,1800",
            "--key", "menu:up:enter:release:left:enter:release,1810", "--frames", "6500"])
        .output().unwrap();
    fs::write(replay.join("stderr.log"), output.stderr).unwrap();
    let log =
        |name: &str| fs::read_to_string(replay.join("kq8-artifacts/logs").join(name)).unwrap();
    let viewer = log("viewer.log");
    assert!(
        output.status.success()
            && viewer.contains("RESULT: PASS")
            && viewer.contains("Save screen -> Save(3, \"Windmill\")"),
        "inspect {}",
        replay.display()
    );
    assert_eq!(quantity(&read_slot("slot-2.json"), "INVITEM_Mushroom"), 5);

    let fight = log("fight.log");
    assert!(viewer.contains("starting help: acknowledged case 142"));
    assert!(fight
        .lines()
        .any(|l| l.contains("Do (confists.anm/") && l.contains("hits sprigwnd1")));
    assert!(fight.contains("sprigwnd1: dies (rule 30)"));
    let connor = log("connor.log");
    assert!(connor
        .lines()
        .any(|l| l.contains("push touch haybail.01 (physical)")
            && l.contains(" -> Push;")
            && l.contains("moved (")));
    let doors = log("doors.log");
    let checks: Vec<_> = doors
        .lines()
        .filter(|l| l.contains("checkDistance Connor") && l.contains(" to ax ("))
        .collect();
    assert_eq!(checks.len(), 2, "inspect {}", replay.display());
    assert!(
        checks[0].contains("> 1700: 0") && checks[1].contains("<= 1700: 1"),
        "the original axe script must reject the floor and accept the hay bale: {checks:?}"
    );
    let second_tick = checks[1]
        .strip_prefix("update ")
        .unwrap()
        .split_once(':')
        .unwrap()
        .0;
    let feet = connor
        .lines()
        .find(|l| l.split_whitespace().next() == Some(second_tick))
        .unwrap();
    let fields: Vec<_> = feet.split_whitespace().collect();
    assert!(
        fields[4].parse::<f32>().unwrap() > 16_000.
            && fields[5] == "true"
            && feet.contains(" haybail.01 "),
        "the click must land with Connor standing on the bale: {feet}"
    );
    assert!(doors.contains("sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 25"));
    assert!(doors.contains("loadKQ henchtp.kq: Henchtp comes into the world"));
    let end = read_slot("slot-3.json");
    assert_eq!(quantity(&end, "INVITEM_SmallAx"), 1);
    assert_eq!(quantity(&end, "INVITEM_MagicMap"), 1);
    assert_eq!(end["journey"]["ai_off"], false);
    assert_eq!(end["journey"]["god_mode"], false);
    assert!(end["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
}
