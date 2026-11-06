//! M6 acceptance route through the shipped cottage, basket and two castle
//! portals. Move Connor between distant landmarks to avoid a timed walk.
//! The interactions, item use, save menu and portal prompts use game input.
use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn mushrooms(slot: &Value) -> i64 {
    slot["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            item["kind"]
                .as_str()
                .is_some_and(|k| k.eq_ignore_ascii_case("INVITEM_Mushroom"))
        })
        .map(|item| item["quantity"].as_i64().unwrap())
        .sum()
}

fn world_var<'a>(slot: &'a Value, name: &str) -> Option<&'a str> {
    slot["kept"]["daventry"]["vars"]
        .as_array()?
        .iter()
        .find(|v| v[0].as_str().is_some_and(|k| k.eq_ignore_ascii_case(name)))?[1]
        .as_str()
}

fn trap<'a>(record: &'a Value, name: &str) -> &'a Value {
    record["states"].as_array().expect("trap records").iter()
        .find(|state| state["name"] == name).unwrap_or_else(|| panic!("missing {name}"))
}

/// Sum all mushroom stacks for each read-only inventory query.
fn observed_mushrooms(log: &str) -> Vec<i64> {
    let mut counts = Vec::new();
    let mut reading = false;
    for line in log.lines() {
        if let Some(command) = line.strip_prefix("> ") {
            reading = command.split_whitespace().next() == Some("inv");
            if reading {
                counts.push(0);
            }
        } else if reading {
            let mut words = line.split_whitespace();
            if words
                .next()
                .is_some_and(|item| item.eq_ignore_ascii_case("Mushroom"))
            {
                let amount = words
                    .next()
                    .unwrap()
                    .trim_start_matches('x')
                    .parse::<i64>()
                    .unwrap();
                *counts.last_mut().unwrap() += amount;
            }
        }
    }
    counts
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn cottage_door_mushrooms_and_castle_round_trip() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/cottage-tests/{nonce}"));
    let slots = root.join("saves");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off", "--no-ai"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &out.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(
            out.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        viewer
    };
    let checkpoint = |n: u16| -> Value {
        let path = slots.join(format!("slot-{n}.json"));
        serde_json::from_slice(
            &fs::read(&path)
                .unwrap_or_else(|e| panic!("{}: {e}; inspect {}", path.display(), root.display())),
        )
        .unwrap()
    };

    // New Game -> Normal runs world.cs Begin and completes the original
    // stngrl introduction before the fixture positions Connor at the door.
    let first = run("cottage", &[
        "--menu-input", "enter,release,enter,release", "--frames", "2400",
        "--key", "popup:help=144,0",
        "--console", "inv,300",
        "--console", "move Connor 0 267900 225300 13981 0 0 1.57,300",
        "--click", "Condoor,308",
        "--console", "move Connor 0 286830 194873 14856 0 0 1.57,348",
        "--click", "basket2,356",
        "--console", "inv,370",
        // Give a five-health mushroom room to heal.
        "--console", "sendEvent 0 Connor KQStatsChangeRequestEvent -5 0 0 0 0,371",
        "--key", "1,380",
        "--console", "inv,387",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Cottage:enter:release:escape,393",
        "--key", "menu:up:enter:release:left:enter:release,403",
    ]);
    assert!(
        first.contains("Save screen -> Save(1, \"Cottage\")"),
        "inspect {}",
        root.display()
    );
    assert!(first.contains("Quit, cue="), "quit via the real menu");
    let inventory =
        fs::read_to_string(root.join("cottage/kq8-artifacts/logs/console.log")).unwrap();
    assert_eq!(
        observed_mushrooms(&inventory),
        [0, 5, 4],
        "inspect {}",
        root.display()
    );
    let saved = checkpoint(1);
    let verify_cottage = |slot: &Value| {
        assert_eq!(world_var(slot, "Daventry::girlhut::FirstTime"), Some("0"));
        assert!(slot["world"]
            .as_str()
            .unwrap()
            .eq_ignore_ascii_case("Daventry"));
        assert_eq!(mushrooms(slot), 4, "inspect {}", root.display());
        assert_eq!(
            slot["kept"]["daventry"]["interactions"]["doors"]["condoor"]["state"],
            "Opened"
        );
        assert_eq!(
            world_var(slot, "Daventry::ConnorsHouse::Entered"),
            Some("1")
        );
        assert_eq!(world_var(slot, "Daventry::basket2::full"), Some("false"));
    };
    verify_cottage(&saved);

    // Independent process. Clicking the collected basket cannot grant
    // five more. Opening a fresh chest proves interaction still works.
    // Travel via portaltp/portalt2 and accept each region prompt.
    let after = run("castle", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "2400",
        "--console", "inv,2",
        "--click", "basket2,4",
        "--console", "inv,17",
        "--console", "move Connor 0 201770 263155 11249 0 0 1.57,22",
        "--click", "smlchst3,31",
        "--console", "KQObject::collision portaltp +1,44",
        "--console", "move Connor 0 158400 308505 9720 0 0 3.14,65",
        "--key", "popup:yes,65",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Castle:enter:release:escape,100",
        "--key", "menu:up:enter:release:left:enter:release,110",
    ]);
    assert!(after.contains("save restored in Daventry without replaying Begin"));
    assert!(after.contains("popup: Yes (test --key popup:yes): portal1 gets Accept"));
    assert!(after.contains("removed Connor from departure trap: true"), "inspect {}", root.display());
    assert_eq!(after.matches(": trap portaltp: On Connor at").count(), 1, "a queued handover must not trigger a second On");
    let away = checkpoint(2);
    assert_eq!(away["world"], "CastleD");
    let departure = trap(&away["kept"]["daventry"]["traps"], "portaltp");
    assert_eq!(departure["mask"], 5);
    assert_eq!(departure["inside"], serde_json::json!([]));
    // Reload the castle slot in a fresh process while Daventry is absent.
    let returned = run("home", &[
        "--menu-input", "down,enter,release,down,enter,release", "--frames", "2400",
        "--console", "move Connor 0 6005 111600 195 0 0 0,22",
        "--key", "popup:yes,22",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Home:enter:release:escape,60",
        "--key", "menu:up:enter:release:left:enter:release,70",
    ]);
    assert!(returned.contains("save restored in CastleD without replaying Begin"));
    assert!(returned.contains("popup: Yes (test --key popup:yes): portal2 gets Accept"));
    assert!(returned.contains("restored persistent traps in Daventry before Init"));
    let inventory = fs::read_to_string(root.join("castle/kq8-artifacts/logs/console.log")).unwrap();
    assert_eq!(observed_mushrooms(&inventory), [4, 4]);
    let built: Vec<_> = after.lines().chain(returned.lines())
        .filter_map(|l| l.strip_prefix("world built: ")?.split(':').next())
        .map(str::to_ascii_lowercase)
        .collect();
    assert_eq!(
        built,
        ["daventry", "castled", "castled", "daventry"],
        "inspect {}",
        root.display()
    );
    assert!(returned.contains("Save screen -> Save(3, \"Home\")") && returned.contains("Quit, cue="));
    let home = checkpoint(3);
    assert_eq!(trap(&home["traps"], "portaltp")["mask"], 5);
    assert_eq!(trap(&home["kept"]["castled"]["traps"], "portalt2")["inside"], serde_json::json!([]));
    verify_cottage(&home);
    assert!(home["kept"]["castled"].is_object());
    assert_eq!(world_var(&home, "Daventry::Castle::foundPortal"), Some("1"));
    let chest = &home["kept"]["daventry"]["interactions"]["chests"]["smlchst3"];
    assert_eq!(chest["state"], "Open");
    assert_eq!(chest["loot"]["quantity"], 3);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
