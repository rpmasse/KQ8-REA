//! An authored Rock arc saved in flight resumes through one landing script.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn rock_in_flight_recovers_once_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/rock-flight-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, save_dir: &Path, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", save_dir)
            .arg("--game").arg(&game)
            .args(["--windowed", "--sound", "off", "--no-ai", "--god"])
            .args(args).output().unwrap();
        fs::write(dir.join("stdout.log"), output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let start = [
        "--world", "Daventry", "--at", "277744,199295,0", "--item", "Rock",
        "--key", "left:IDCTL_INVBAR_ROCKS,10",
        "--key", "left:IDCTL_INVBAR_ROCKS,14",
        "--aim-point", "277744,198000,13580,24",
    ];
    let before = run("save", &slots, &[
        &start[..],
        &["--key", "menu:down:down:enter:release:enter:release:escape:type=Flight:enter:release:escape,61",
            "--frames", "200"],
    ].concat());
    assert!(before.contains("Save screen -> Save(1, \"Flight\")"), "inspect {}", root.display());
    let read = |dir: &Path, slot: usize| -> serde_json::Value {
        serde_json::from_slice(&fs::read(dir.join(format!("slot-{slot}.json"))).unwrap()).unwrap()
    };
    let first = read(&slots, 1);
    let bolts = first["bolts"]["bolts"].as_array().unwrap();
    assert_eq!(bolts.len(), 1, "the saved Rock must still be in flight; inspect {}", root.display());
    let rock = &bolts[0];
    assert_eq!((rock["name"].as_str(), rock["updates"].as_u64(), rock["script"].as_str()),
        (Some("rock1"), Some(2), Some("rockHit.cs")));
    assert_eq!(rock["arc"]["authored_curve_v1"], true);
    assert_eq!(rock["arc"]["falling"], true);
    assert!(rock["arc"]["progress"].as_f64().is_some_and(|n| (n - 333.333).abs() < 1.0));
    assert!(first["journey"]["items"].as_array().unwrap().iter().any(|i| i["name"] == "TestItem"
        && i["kind"] == "INVITEM_Rock"), "launch must not deduct the carried Rock");

    let after = run("restore", &slots, &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "180",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterFlight:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in Daventry without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterFlight\")"), "inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    let items = fs::read_to_string(root.join("restore/kq8-artifacts/logs/items.log")).unwrap();
    assert!(fight.contains("rock1 update 3: at (") && fight.contains("rock1 update 6: at ("),
        "the saved arc did not finish; inspect {}", root.display());
    assert!(!fight.contains("shoots rock1"), "the launch cue replayed after loading");
    assert_eq!(items.matches("rockHit.cs line 22: dropItem: fresh landing").count(), 1);
    let second = read(&slots, 2);
    assert!(second["bolts"]["bolts"].as_array().unwrap().is_empty());
    assert_eq!(second["bolts"]["bolts_shot"], 1);
    assert!(!second["journey"]["items"].as_array().unwrap().iter().any(|i| i["kind"] == "INVITEM_Rock"));

    // The same input route without a midflight save supplies the control.
    let control_slots = root.join("uninterrupted-slots");
    let control = run("uninterrupted", &control_slots, &[
        &start[..],
        &["--key", "menu:down:down:enter:release:enter:release:escape:type=Complete:enter:release:escape,80",
            "--frames", "200"],
    ].concat());
    assert!(control.contains("Save screen -> Save(1, \"Complete\")"), "inspect {}", root.display());
    let uninterrupted = read(&control_slots, 1);
    let rocks = |slot: &serde_json::Value| -> Vec<serde_json::Value> {
        slot["items"].as_array().unwrap().iter().filter(|i| i["kind"] == "INVITEM_Rock").cloned().collect()
    };
    let final_rocks = rocks(&second);
    assert_eq!(final_rocks.len(), 2, "Q203 keeps the old Rock identity and creates one landing Rock");
    assert_eq!(final_rocks[0]["name"], "TestItem");
    assert_eq!(final_rocks[1]["name"], "NoName22");
    assert_eq!(final_rocks, rocks(&uninterrupted), "reload changed the final Rock records");
    assert_eq!(second["bolts"], uninterrupted["bolts"]);
    assert_eq!(second["journey"]["next_no_name"], uninterrupted["journey"]["next_no_name"]);
}
