//! Original Swamp sword kill keeps the Mandragor corpse across a Rust disk slot.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn sword_killed_tree_and_its_corpse_survive_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-melee-checkpoint/{nonce}"));
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
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fight, doors)
    };

    let (first, fight, doors) = run("save", &[
        "--world", "Swamp", "--at", "159904,140178,0",
        "--command", "KQConner::setDoMode 0,15",
        "--aim", "mand3t,45", "--aim", "mand3t,75",
        "--aim", "mand3t,105", "--aim", "mand3t,135",
        "--aim", "mand3t,185",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=TreeDead:enter:release:escape,235",
        "--updates", "250", "--frames", "1900",
    ]);
    assert!(first.contains("Save screen -> Save(1, \"TreeDead\")"), "inspect {}", root.display());
    assert!(doors.matches("swing pick meets character mand3t (type 0x4042)").count() >= 4);
    assert!((3..=4).contains(&fight.matches("hits mand3t:").count()),
        "ordinary damage rolls should kill the tree in three or four sword hits; inspect {}", root.display());
    assert!(fight.lines().any(|line| line.contains("mand3t: hit by Connor, Normal:")
            && line.contains("flinches: 'React' starts at once (rule 23)"))
        && fight.lines().any(|line| line.contains("mand3t: hit by Connor, Normal:")
            && line.contains("ouch sound manouch.aud (rule 24)")),
        "the tree did not use ordinary hit reactions; inspect {}", root.display());
    assert_eq!(fight.matches("mand3t: drops: nothing (rule 31)").count(), 1,
        "the shipped tree has no authored item drop; inspect {}", root.display());
    assert!(
        fight.contains("Connor: hit by mand3r, Normal:")
            && fight.contains("Connor: hit by mand3l, Normal:")
            && !fight.contains("god mode is on"),
        "both roots must deal ordinary damage; inspect {}",
        root.display()
    );
    assert!(fight.contains("mand3t: its death ended (rule 32): Connor gets its experience 20000")
        && fight.contains("a corpse mt_dead.kqs")
        && fight.contains("Mandragor tree finalizer has no ordinary Die callback")
        && fight.contains("mand3r: Mandragor tree mand3t enters Die: root link cleared")
        && fight.contains("mand3l: Mandragor tree mand3t enters Die: root link cleared"),
        "ordinary sword kill did not complete the group; inspect {}", root.display());

    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first_slot = read(1);
    let kept = &first_slot["kept"]["swamp"];
    assert_eq!(first_slot["journey"]["stats"]["stats"][4], 20010);
    assert_eq!(kept["corpses"].as_array().unwrap().len(), 1);
    assert_eq!(kept["corpses"][0]["name"], "mand3t");
    assert_eq!(kept["corpses"][0]["shape"], "mt_dead.kqs");
    for name in ["mand3t", "mand3r", "mand3l"] {
        assert!(kept["gone"].as_array().unwrap().iter().any(|v| v == name),
            "{name} was not saved as gone; inspect {}", root.display());
    }

    let (restored, second_fight, _) = run("restore", &[
        "--menu-input", "down,enter,release,enter,release",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=TreeStillDead:enter:release:escape,35",
        "--updates", "55", "--frames", "1300",
    ]);
    assert!(restored.contains("save restored in Swamp without replaying Begin")
        && restored.contains("corpse of mand3t: mt_dead.kqs drawn")
        && !second_fight.contains("mand3t: its death ended"),
        "restored tree corpse or kill state failed; inspect {}", root.display());
    let second_slot = read(2);
    assert_eq!(second_slot["journey"]["stats"]["stats"][4], 20010);
    assert_eq!(second_slot["kept"]["swamp"]["corpses"], kept["corpses"]);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
