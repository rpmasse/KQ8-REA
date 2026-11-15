//! An original-data Weeper bolt saved before impact must finish one drain after reload.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn dead_city_weeper_flight_becomes_one_drain_after_disk_reload() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/weeper-flight-checkpoint/{nonce}"
    ));
    let slots = root.join("slots");
    let run = |part: &str, save_dir: &Path, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", save_dir)
            .arg("--game")
            .arg(&game)
            .args(["--windowed", "--sound", "off"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        viewer
    };
    // --at crosses two intro traps on the way from the default spawn. Remove
    // only their Connor collision bit so this fixture exercises the Weeper.
    let start = [
        "--world",
        "DeadCity",
        "--at",
        "170920,149369,0",
        "--command",
        "KQObject::collision dhalltrp -4,1",
        "--command",
        "KQObject::collision introtp -4,1",
        "--command",
        "KQConner::setLevel 8,1",
        "--command",
        "sendEvent 0 Connor KQStatsChangeRequestEvent 80 0 0 0 0,2",
    ];
    let before = run("save", &slots, &[
        &start[..],
        &["--key", "menu:down:down:enter:release:enter:release:escape:type=Flight:enter:release:escape,31",
            "--frames", "120"],
    ].concat());
    assert!(
        before.contains("Save screen -> Save(1, \"Flight\")"),
        "inspect {}",
        root.display()
    );
    let read = |dir: &Path, slot: usize| -> serde_json::Value {
        serde_json::from_slice(&fs::read(dir.join(format!("slot-{slot}.json"))).unwrap()).unwrap()
    };
    let first = read(&slots, 1);
    let bolts = first["bolts"]["bolts"].as_array().unwrap();
    assert_eq!(
        bolts.len(),
        1,
        "save must precede impact; inspect {}",
        root.display()
    );
    assert_eq!(bolts[0]["name"], "bolt1");
    assert_eq!(bolts[0]["updates"], 2);
    assert_eq!(bolts[0]["drain"]["duration"].as_f64(), Some(3.0));
    assert_eq!(bolts[0]["drain"]["segment"], "lifesuck.kq");
    assert!(first["drains"].as_array().unwrap().is_empty());
    assert_eq!(first["journey"]["stats"]["stats"][0], 80);

    let after = run("restore", &slots, &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "220",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterFlight:enter:release:escape,80",
    ]);
    assert!(
        after.contains("save restored in DeadCity without replaying Begin")
            && after.contains("Save screen -> Save(2, \"AfterFlight\")"),
        "inspect {}",
        root.display()
    );
    let logs = root.join("restore/kq8-artifacts/logs");
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert_eq!(
        fight
            .matches("bolt1 starts drain on Connor: 8 links")
            .count(),
        1
    );
    for pulse in 1..=12 {
        assert_eq!(
            fight
                .matches(&format!("bolt1 drain pulse {pulse} on Connor:"))
                .count(),
            1,
            "pulse {pulse} missing or duplicated; inspect {}",
            logs.display()
        );
    }
    assert_eq!(fight.matches("bolt1 ends drain after 12 pulses").count(), 1);
    assert!(
        !fight.contains("shoots bolt1"),
        "the Weeper replayed its launch after loading"
    );
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert_eq!(doors.matches("copy WeepLink.weeper14.").count(), 7);
    assert_eq!(doors.matches("copy WeepEnd.weeper14 ").count(), 1);
    let second = read(&slots, 2);
    assert_eq!(second["journey"]["stats"]["stats"][0], 68);
    assert!(second["bolts"]["bolts"].as_array().unwrap().is_empty());
    assert_eq!(second["bolts"]["bolts_shot"], 1);
    assert!(second["drains"].as_array().unwrap().is_empty());

    // The uninterrupted route reaches the same world update (31 + 80).
    let control_slots = root.join("uninterrupted-slots");
    let control = run("uninterrupted", &control_slots, &[
        &start[..],
        &["--key", "menu:down:down:enter:release:enter:release:escape:type=Complete:enter:release:escape,111",
            "--frames", "240"],
    ].concat());
    assert!(
        control.contains("Save screen -> Save(1, \"Complete\")"),
        "inspect {}",
        root.display()
    );
    let uninterrupted = read(&control_slots, 1);
    for key in [
        "bolts",
        "drains",
        "shot_timing",
        "hit_timing",
        "fight_rng",
        "position",
    ] {
        assert_eq!(
            second[key], uninterrupted[key],
            "{key} changed across reload"
        );
    }
    assert_eq!(
        second["journey"]["stats"],
        uninterrupted["journey"]["stats"]
    );
}
