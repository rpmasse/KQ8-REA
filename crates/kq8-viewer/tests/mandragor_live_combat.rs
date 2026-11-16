//! GOG-data Swamp fight: root damage, temporary death and disk-load rebirth.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn sword_defeats_a_live_root_and_it_rebirths_under_normal_damage() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-live-combat/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--windowed", "--sound", "off"])
            .args(args)
            .output()
            .unwrap();
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
    let (viewer, fight, doors) = run("save", &[
            "--world",
            "Swamp",
            "--at",
            "159904,140178,0",
            "--command",
            "KQConner::setDoMode 0,15",
            "--aim",
            "mand3r,45",
            "--aim",
            "mand3r,75",
            "--aim",
            "mand3r,105",
            "--aim",
            "mand3r,135",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=RootWaiting:enter:release:escape,190",
            "--updates",
            "205", "--frames", "1600",
    ]);
    assert!(
        viewer.contains("weapon set conKng.anm (SkelKingSword): named in conStart.cs")
            && fight.contains("hand (Some(\"INVITEM_SkelKingSword\"))"),
        "original Swamp sword was not readied; inspect {}",
        root.display()
    );
    assert!(
        doors.contains("test --aim mand3r (camera to the middle of its box):")
            && doors.contains("swing pick meets character mand3l (type 0x4042)"),
        "the nearer root was not selected through the camera ray; inspect {}",
        root.display()
    );
    assert!(
        doors.contains("'conkng.anm/react' can't be cut short (mode free): no swing"),
        "the first aim did not respect Connor's ordinary hit reaction; inspect {}",
        root.display()
    );
    assert_eq!(
        fight.matches("Connor: Do (conkng.anm/").count(),
        3,
        "expected three sword hit cues; inspect {}",
        root.display()
    );
    assert_eq!(
        fight
            .matches("mand3l: hit by Connor, Normal: D 110")
            .count(),
        3,
        "the left root did not take all three ordinary hits; inspect {}",
        root.display()
    );
    assert!(
        fight.contains("mand3l: Mandragor root Die: Normal, non-solid, linked tree mand3t holds 1 down root(s)")
            && fight.contains("mand3l: Mandragor root die ended: rebirth due at")
            && !fight.contains("mand3l: Mandragor root rebirth due:"),
        "root did not enter its saved waiting phase; inspect {}",
        root.display()
    );
    let connor_hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by mand3") && line.contains("; health "))
        .collect();
    assert!(
        connor_hits
            .iter()
            .any(|line| line.contains("hit by mand3r"))
            && connor_hits
                .iter()
                .any(|line| line.contains("hit by mand3l"))
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "normal root attacks did not damage a surviving Connor; inspect {}",
        root.display()
    );
    let final_health = connor_hits
        .last()
        .and_then(|line| line.split("; health ").nth(1))
        .and_then(|health| health.split(" -> ").nth(1))
        .and_then(|health| health.parse::<i32>().ok())
        .expect("a root hit must log Connor's final health");
    assert!(
        final_health > 0,
        "Connor did not survive; inspect {}",
        root.display()
    );
    assert!(
        viewer.contains("Save screen -> Save(1, \"RootWaiting\")"),
        "the waiting root was not saved; inspect {}",
        root.display()
    );
    let slot: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    let actors = slot["kept"]["swamp"]["characters"].as_array().unwrap();
    let actor = |name: &str| {
        actors
            .iter()
            .find(|a| a["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing from slot; inspect {}", root.display()))
    };
    let remaining = actor("mand3l")["character"]["mandragor_root"]["phase"]["Waiting"]
        .as_f64()
        .expect("root must save its remaining rebirth time");
    assert!(
        (0.0..5.0).contains(&remaining),
        "remaining rebirth time {remaining}"
    );
    assert_eq!(
        actor("mand3l")["character"]["mandragor_root"]["linked"],
        true
    );
    assert_eq!(actor("mand3l")["character"]["state"], "Die");
    assert_eq!(actor("mand3l")["character"]["fight"]["stats"][0], 0);
    assert_eq!(actor("mand3l")["character"]["collision_type"], 0);
    assert_eq!(actor("mand3l")["character"]["collision_mask"], 0);
    assert_eq!(actor("mand3t")["character"]["mandragor_down_roots"], 1);
    assert_eq!(actor("mand3t")["character"]["no_turn"], true);

    let (restored, resumed_fight, _) = run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--key",
            "menu:down:down:down:enter:release:enter:release:escape:type=RootStillDown:enter:release:escape,25",
            "--updates",
            "110",
            "--frames",
            "1800",
        ],
    );
    assert!(
        restored.contains("save restored in Swamp without replaying Begin")
            && restored.contains("Save screen -> Save(2, \"RootStillDown\")")
            && resumed_fight
                .contains("mand3l: Mandragor root rebirth due: restores health 250, starts birth")
            && resumed_fight
                .contains("mand3l: Mandragor root birth ended: collision 0x4042/0x201f")
            && !resumed_fight.contains("mand3l: Mandragor root Die:"),
        "fresh-process rebirth or root link failed; inspect {}",
        root.display()
    );
    let still_down: serde_json::Value =
        serde_json::from_slice(&fs::read(slots.join("slot-2.json")).unwrap()).unwrap();
    let actors = still_down["kept"]["swamp"]["characters"]
        .as_array()
        .unwrap();
    let actor = |name: &str| {
        actors
            .iter()
            .find(|a| a["name"] == name)
            .unwrap_or_else(|| panic!("{name} missing from second slot; inspect {}", root.display()))
    };
    let resumed_remaining = actor("mand3l")["character"]["mandragor_root"]["phase"]
        ["Waiting"]
        .as_f64()
        .expect("the root must still be waiting at the second save");
    assert!((0.0..remaining).contains(&resumed_remaining));
    assert_eq!(actor("mand3l")["character"]["mandragor_root"]["linked"], true);
    assert_eq!(actor("mand3l")["character"]["state"], "Die");
    assert_eq!(actor("mand3l")["character"]["fight"]["stats"][0], 0);
    assert_eq!(actor("mand3l")["character"]["collision_type"], 0);
    assert_eq!(actor("mand3l")["character"]["collision_mask"], 0);
    assert_eq!(actor("mand3t")["character"]["mandragor_down_roots"], 1);
    assert_eq!(actor("mand3t")["character"]["no_turn"], true);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
