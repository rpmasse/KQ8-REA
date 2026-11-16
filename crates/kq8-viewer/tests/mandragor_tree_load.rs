//! Unattended original-data check for Mandragor tree and linked-root startup.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn swamp_mandragor_trees_and_roots_load_as_animated_characters() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-load/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "157904,140178,0", "--god", "--sound", "off",
            "--windowed", "--command", "move mand3t 0 160000 140000 6120 0 0 1.5707963,10",
            "--updates", "20",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert_eq!(
        characters.matches("KQMandragorTree: from").count(),
        11,
        "expected all Swamp trees; inspect {}",
        dir.display()
    );
    assert_eq!(
        characters.matches("KQMandragorRoot: from").count(),
        22,
        "expected all Swamp roots; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("[mand3r] KQMandragorRoot: from")
            && characters.contains("at (158504, 139178, 6120)")
            && characters.contains("[mand3l] KQMandragorRoot: from")
            && characters.contains("at (157304, 139178, 6120)"),
        "root startup offsets not applied; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("has Birth: NOT DRAWN until born"),
        "root Unborn startup missing; inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("mand3r: Mandragor root follows mand3t at (161000, 140600, 6120)")
            && characters.contains("mand3l: Mandragor root follows mand3t at (161000, 139400, 6120)"),
        "roots did not follow the moved, turned tree; inspect {}",
        dir.display()
    );
    assert!(
        viewer.contains("object [mand3t] class=KQMandragorTree shape=mt_stop.kqs"),
        "expected an animated tree model; inspect {}",
        dir.display()
    );
    assert!(
        !viewer.contains("marker [mand3t] class=KQMandragorTree")
            && !viewer.contains("marker [mand3r] class=KQMandragorRoot")
            && !viewer.contains("marker [mand3l] class=KQMandragorRoot"),
        "Mandragor actors must use animated models; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_swamp_tree_uses_standing_alarm_and_attack_without_a_hit_cue() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-alarm/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "157904,140178,0", "--god", "--sound", "off",
            "--windowed", "--command", "move Connor 0 158404 140178 6120 0 0 0,5",
            "--updates", "90",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("mand3t: joins the active list"), "tree AI did not activate; inspect {}", dir.display());
    assert!(characters.lines().any(|line| line.contains("mand3t: alarmed:") && line.contains("no random reaction deadline (Q209); target position copied")), "tree alarm missed its special target rule; inspect {}", dir.display());
    assert!(fight.contains("mand3t: attack (rule 7, standing)"), "tree did not request its authored attack; inspect {}", dir.display());
    assert!(characters.contains("mand3t: turning to")
        && characters.contains("mand3r: Mandragor root follows mand3t")
        && characters.contains("mand3l: Mandragor root follows mand3t"),
        "tree turn did not carry both linked roots; inspect {}", dir.display());
    assert!(!fight.contains("mand3t: Do (Attack") && !fight.contains("mand3t: direct Normal hit on Connor"), "tree produced an un-authored hit; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn an_original_swamp_root_pushes_nearby_connor_after_tree_move() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-root-proximity/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args([
            "--world", "Swamp", "--at", "161000,140100,0", "--sound", "off",
            "--windowed", "--command", "move mand3t 0 160000 140000 6120 0 0 1.5707963,10",
            "--updates", "25",
        ])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("mand3r: Mandragor proximity:"), "right root did not respond; inspect {}", dir.display());
    assert!(connor.contains("moved Connor from (161000, 140100")
        && connor.contains("to (160204, 140520") && connor.contains("KQMoveEvent"),
        "Connor's mover did not receive the root destination; inspect {}", dir.display());
    assert!(viewer.contains("connor: movement reactback = movement"), "shipped Connor reactback was not loaded; inspect {}", dir.display());
    assert!(connor.contains("special: plays reactback (") && !connor.contains("movement reactback not loaded"),
        "Connor did not play his active-list reaction; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn a_tree_team_alarm_schedules_root_rise() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-root-rise/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "159904,140178,0", "--god", "--sound", "off",
            "--windowed", "--updates", "40",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(characters.contains("mand3r: made active by its team member mand3t's alarm")
        && characters.contains("mand3r: alarmed: told by its team member mand3t")
        && characters.contains("mand3r: Unborn alarm: Wait due in 0 s")
        && characters.contains("mand3r: rises (rule 30):")
        && characters.contains("mand3r: solid for Connor:"),
        "team-alarmed Unborn root did not rise; inspect {}", dir.display());
    assert!(characters.contains("mand3t: turning to")
        && characters.contains("mand3r: Mandragor root follows mand3t"),
        "the active tree's turn should update its root; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn an_original_swamp_tree_dies_with_its_authored_corpse_after_fire() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-fire/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "157904,140178,0", "--god", "--sound", "off",
            "--no-ai", "--windowed", "--command",
            "sendEvent 0 mand3t KQAttackEvent Connor 2 10000 0,10", "--updates", "85",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(fight.contains("mand3t: dies (rule 30): kind Fire, 'firedie' starts"), "inspect {}", dir.display());
    assert!(fight.contains("mand3t: its death ended (rule 32): Connor gets its experience 20000"), "inspect {}", dir.display());
    assert!(fight.contains("a corpse mt_dead.kqs") && fight.contains("Mandragor tree finalizer has no ordinary Die callback"), "inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_swamp_roots_die_and_rebirth_on_their_own_timers() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-root-rebirth/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world", "Swamp", "--at", "159904,140178,0", "--god", "--sound", "off",
            // The tree's team alarm starts both roots' authored birth. Let
            // their first Attack reach Do before these scripted lethal hits.
            "--windowed", "--command", "sendEvent 0 mand3r KQAttackEvent Connor 0 10000 0,55",
            "--command", "sendEvent 0 mand3l KQAttackEvent Connor 0 10000 0,57", "--updates", "215",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(characters.contains("mand3r: rises (rule 30):")
        && characters.contains("mand3r: solid for Connor:"),
        "controlled root did not emerge before combat; inspect {}", dir.display());
    let at_update = |needle: &str| -> u64 {
        fight.lines().find(|line| line.contains(needle))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|word| word.trim_end_matches(':').parse().ok())
            .unwrap_or_else(|| panic!("missing {needle}; inspect {}", dir.display()))
    };
    assert!(fight.lines().any(|line| line.contains("mand3r: Do (Attack") && line.contains("hits Connor: distance")), "root shared Do attack missing; inspect {}", dir.display());
    for (root, expected) in [("mand3r", 60), ("mand3l", 75)] {
        assert!(fight.contains(&format!("{root}: Mandragor root Die:")), "{root} did not enter timed Die; inspect {}", dir.display());
        assert!(fight.contains(&format!("{root}: Mandragor root die ended: rebirth due")), "{root} did not start timer; inspect {}", dir.display());
        assert!(fight.contains(&format!("{root}: Mandragor root rebirth due: restores health 250, starts birth")), "{root} did not rebirth; inspect {}", dir.display());
        assert!(fight.contains(&format!("{root}: Mandragor root birth ended: collision 0x4042/0x201f")), "{root} did not finish birth; inspect {}", dir.display());
        let elapsed = at_update(&format!("{root}: Mandragor root rebirth due")) - at_update(&format!("{root}: Mandragor root die ended"));
        assert!((expected..=expected + 2).contains(&elapsed), "{root} waited {elapsed} updates; inspect {}", dir.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_swamp_root_rebirth_resumes_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-root-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--god", "--windowed"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "Swamp", "--at", "159904,140178,0",
        "--command", "sendEvent 0 mand3r KQAttackEvent Connor 0 10000 0,40",
        "--frames", "380", "--key",
        "menu:down:down:enter:release:enter:release:escape:type=RootWaiting:enter:release:escape,75",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"RootWaiting\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let actors = first["kept"]["swamp"]["characters"].as_array().unwrap();
    let root_at = |actors: &[serde_json::Value], name: &str| -> serde_json::Value {
        actors.iter().find(|a| a["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(name)))
            .unwrap_or_else(|| panic!("missing {name}; inspect {}", root.display())).clone()
    };
    let waiting = root_at(actors, "mand3r");
    assert!(waiting["character"]["mandragor_root"]["phase"].get("Waiting").is_some(), "root timer not saved; inspect {}", root.display());
    assert_eq!(root_at(actors, "mand3t")["character"]["mandragor_down_roots"], 1);

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "950",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=RootReborn:enter:release:escape,180",
    ]);
    assert!(after.contains("save restored in Swamp without replaying Begin"), "inspect {}", root.display());
    let second = read(2);
    let actors = second["kept"]["swamp"]["characters"].as_array().unwrap();
    let reborn = root_at(actors, "mand3r");
    assert_eq!(reborn["character"]["mandragor_root"]["phase"], "Alive");
    assert_eq!(reborn["character"]["fight"]["stats"][0], 250);
    assert_eq!(reborn["character"]["collision_type"], 0x4042);
    assert_eq!(reborn["character"]["collision_mask"], 0x201f);
    assert_eq!(root_at(actors, "mand3t")["character"]["mandragor_down_roots"], 0);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn tree_death_ends_both_original_swamp_roots_even_when_one_is_down() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/mandragor-tree-overlap/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args([
            "--world", "Swamp", "--at", "159904,140178,0", "--god", "--sound", "off",
            "--windowed", "--command", "sendEvent 0 mand3r KQAttackEvent Connor 0 10000 0,40",
            "--command", "sendEvent 0 mand3t KQAttackEvent Connor 2 10000 0,75",
            "--updates", "205",
        ])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    assert!(fight.contains("mand3r: Mandragor root die ended: rebirth due"), "right root not down; inspect {}", dir.display());
    assert!(fight.contains("mand3t: dies (rule 30): kind Fire, 'firedie' starts"), "tree not killed; inspect {}", dir.display());
    for root in ["mand3r", "mand3l"] {
        assert!(fight.contains(&format!("{root}: Mandragor tree mand3t enters Die: root link cleared")), "{root} stayed linked; inspect {}", dir.display());
        assert!(fight.contains(&format!("{root}: dies (rule 30): kind Normal")), "{root} never entered permanent Die; inspect {}", dir.display());
        assert!(fight.contains(&format!("{root}: its death ended (rule 32):")), "{root} survived; inspect {}", dir.display());
    }
    assert!(fight.contains("mand3r: Mandragor root birth ended without tree: enters permanent Die"), "down root did not finish birth before permanent death; inspect {}", dir.display());
    assert!(fight.contains("mand3t: its death ended (rule 32): Connor gets its experience 20000")
        && fight.contains("a corpse mt_dead.kqs"), "tree outcome missing; inspect {}", dir.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
