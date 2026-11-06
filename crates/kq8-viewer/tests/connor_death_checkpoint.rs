//! An unfinished combat death keeps its pose and immunity to later hits after disk load.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; isolated fatal-hit setup"]
fn combat_deaths_resume_once_and_ignore_further_hits_after_loading() {
    check_deaths(&[
        ("fire-final", 2, 25, "firedie", None), ("fire-drop", 2, 13, "dropweapon", None),
        ("ice-final", 3, 25, "icedie", None), ("ice-drop", 3, 13, "dropweapon", None),
        ("normal", 0, 13, "die", None), ("projectile", 1, 13, "die", None),
    ], false);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; fatal hits during equipped attacks"]
fn death_waits_for_equipped_attacks_across_a_disk_save() {
    check_deaths(&[
        ("axe-attack", 0, 43, "die", Some(("INVITEM_SmallAx", "--swing", "skeleton1,38", "melee"))),
        ("crossbow-turn", 0, 43, "die", Some(("INVITEM_SmCrossbow", "--aim", "skeleton1,38", "ranged"))),
        ("crossbow-attack", 0, 43, "die", Some(("INVITEM_SmCrossbow", "--aim", "skeleton1,38", "ranged"))),
    ], false);
}

#[test]
#[ignore = "requires GOG assets and graphics; isolated fatal-hit setup"]
fn first_person_death_selects_the_third_person_pose_and_resumes_once() {
    check_deaths(&[("first-person", 0, 13, "die", None)], true);
}

type DeathCase = (&'static str, u8, u64, &'static str, Option<(&'static str, &'static str, &'static str, &'static str)>);

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn weapon_swap_retains_pending_die_and_reincarnation_is_sampled_at_final_end() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/swap-death/{nonce}"));
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part); fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer")).current_dir(&dir)
            .env("KQ8_SAVE_DIR", root.join("slots")).arg("--game").arg(&game)
            .args(["--sound","off","--no-ai"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let first = run("swap", &["--world","DeadCity","--item","INVITEM_SmCrossbow","--frames","1300",
        "--command","KQConner::setDoMode 1,25",
        "--command","sendEvent 0 Connor KQAttackEvent first 2 10000 1,26",
        "--key","menu:down:down:enter:release:enter:release:escape:type=SwapDie:enter:release:down:down:down:enter:release:left:enter:release,28"]);
    assert!(first.contains("Save screen -> Save(1, \"SwapDie\")"));
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(root.join(format!("slots/slot-{n}.json"))).unwrap()).unwrap()
    };
    let pending = read(1);
    assert_eq!(pending["pending_combat_death"], "Fire");
    assert_eq!(pending["connor_dying"], false);
    assert_eq!(pending["weapon_state"]["readying"], true);
    assert!(pending["special"].is_object() && pending["combat_death"].is_null());
    let next = run("ice", &["--menu-input","down,enter,release,enter,release","--frames","1700",
        "--command","sendEvent 0 Connor KQAttackEvent second 3 10000 1,2",
        "--console","set KQConner::reincarnate 1,22",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=Recovered:enter:release:down:down:down:enter:release:left:enter:release,110"]);
    let connor = fs::read_to_string(root.join("ice/kq8-artifacts/logs/connor.log")).unwrap();
    assert_eq!(connor.matches("dies (Ice): plays [\"dropweapon\", \"icedie\"]").count(), 1);
    let end = connor.lines().find(|line| line.contains("out end admitted pending Die")).unwrap();
    let tick = end.split_whitespace().nth(1).unwrap().trim_end_matches(':');
    assert!(connor.contains(&format!("connor update {tick}: dies (Ice)")), "Die must enter on the swap's final end");
    let variable = fs::read_to_string(root.join("ice/kq8-artifacts/logs/console.log")).unwrap();
    assert!(variable.contains("KQConner::reincarnate = '1'"));
    assert!(next.contains("icedie finished: reincarnated into Wait"), "inspect {}", root.display());
    assert!(!next.contains("Die state holds until") && !next.contains("scripted Die hold finished"));
    let recovered = read(2);
    assert_eq!(recovered["connor_dying"], false);
    assert!(recovered["pending_combat_death"].is_null() && recovered["combat_death"].is_null());
    assert_eq!(recovered["journey"]["stats"]["stats"][0].as_i64(), recovered["journey"]["stats"]["level"].as_i64().map(|lvl| 10*lvl));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

fn check_deaths(cases: &[DeathCase], first_person: bool) {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-death-checkpoint/{nonce}"));
    for &(case, kind, save_at, movement, attack) in cases {
        let case_root = root.join(case);
        let slots = case_root.join("slots");
        let run = |part: &str, args: &[&str]| {
            let dir = case_root.join(part); fs::create_dir_all(&dir).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
                .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
                .args(["--sound", "off", "--no-ai"]).args(args).output().unwrap();
            fs::write(dir.join("stderr.log"), output.stderr).unwrap();
            let logs = dir.join("kq8-artifacts/logs");
            let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
            assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
            (viewer, fs::read_to_string(logs.join("connor.log")).unwrap(), fs::read_to_string(logs.join("fight.log")).unwrap())
        };
        let hit_at = if attack.is_some() { 40 } else { 10 };
        let hit = format!("sendEvent 0 Connor KQAttackEvent trap {kind} 100000 1,{hit_at}");
        let key = format!("menu:down:down:enter:release:enter:release:escape:type=Dying:enter:release:down:down:down:enter:release:left:enter:release,{save_at}");
        let mut args = vec!["--world", "DeadCity", "--frames", "600", "--command", &hit, "--key", &key];
        if first_person { args.extend(["--key", "physical:f2,3"]); }
        if let Some((item, flag, action, kind)) = attack {
            let ready = if kind == "melee" { "KQConner::setDoMode 0,5" } else { "KQConner::setDoMode 1,5" };
            // The original side target leaves the ranged shot turning at
            // the losing hit. A target in front starts its attack at once.
            let target = if case == "crossbow-attack" { "move skeleton1 0 182274 46200 864 0 0 0,5" }
                else { "move skeleton1 0 183300 45000 864 0 0 0,5" };
            args.extend(["--item", item, "--command", ready, "--command", target, flag, action]);
        }
        let before = run("save", &args);
        assert!(slots.join("slot-1.json").exists(), "death could not be saved; inspect {}", case_root.display());
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
        if first_person {
            assert!(before.0.contains("CameraFirstPerson: first person true"));
            let entered = before.2.find("first person true: bank").unwrap();
            let returned = before.2.find("first person false: bank").unwrap();
            let selected = before.2.find("dies (Normal): plays").unwrap();
            assert!(entered < returned && returned < selected);
            assert_eq!(saved["view_flags"]["active"], false);
            assert_eq!(saved["weapon_state"]["actor_first_person"], false);
        }
        let pending = matches!(case, "axe-attack" | "crossbow-attack");
        if case == "crossbow-turn" { assert!(before.1.contains("Die entered while the ranged aim was still turning")); }
        assert_eq!(before.1.matches("dies (").count(), usize::from(!pending), "inspect {}", case_root.display());
        assert_eq!(saved["connor_dying"], !pending);
        let pose = if pending {
            assert_eq!(saved["pending_combat_death"], "Normal");
            assert!(saved["journey"]["stats"]["stats"][0].as_i64().unwrap() < 0);
            assert!(saved["combat_death"].is_null());
            let key = if case == "axe-attack" { "swing" } else { "shot" };
            assert!(saved[key].is_object(), "the attack still owns the pose; inspect {}", case_root.display());
            &saved[key]
        } else {
            assert_eq!(saved["journey"]["stats"]["stats"][0], 0);
            assert!(saved["pending_combat_death"].is_null());
            assert!(saved["swing"].is_null() && saved["shot"].is_null() && saved["special"].is_null());
            let pose = &saved["combat_death"]["pose"];
            assert_eq!(pose["animation_input_lock"], true);
            assert_eq!(pose["hands_off"], false);
            assert_eq!(pose["moving"]["use_gravity"], true);
            let name = pose["movement"].as_str().unwrap();
            assert!(name.eq_ignore_ascii_case(movement) || name.to_ascii_lowercase().ends_with(&format!("/{movement}")));
            pose
        };
        let phase = pose["phase"].as_f64().unwrap();
        assert!(phase > 0. && phase < 1., "the slot must be inside the requested movement");
        if case == "fire-drop" {
            // Convert the captured pose to the earlier Special/Kill layout.
            let mut legacy = saved.clone();
            legacy.as_object_mut().unwrap().remove("connor_dying");
            legacy.as_object_mut().unwrap().remove("combat_death");
            legacy["special"] = serde_json::json!({
                "movement":pose["movement"], "phase":phase, "bank":pose["bank"], "fresh":pose["fresh"],
                "steps":[{"Play":[pose["movement"],"dropweapon"]},{"Play":["firedie","firedie"]},"Kill"],
                "at":0,"auto":null,"wanted":null,"transition_waiting":null,"transition_playing":null,"death":"firedie"
            });
            fs::write(slots.join("slot-1.json"), serde_json::to_vec(&legacy).unwrap()).unwrap();
        }
        let after = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1100",
            "--command", "sendEvent 0 Connor KQAttackEvent later 0 1000 1,2"]);
        assert!(after.0.contains("save restored in DeadCity without replaying Begin"));
        assert_eq!(after.1.matches("dies (").count(), usize::from(pending), "inspect {}", case_root.display());
        assert_eq!(after.2.matches("hit by later: ignored, he is dying (rule 29)").count(), usize::from(!pending));
        if !pending {
            let first: Vec<_> = after.1.lines().find(|line| line.starts_with("1 ")).unwrap().split_whitespace().collect();
            let mode = first.iter().position(|field| *field == "Die").unwrap();
            assert_eq!(first[mode + 1], pose["movement"].as_str().unwrap());
            let next_phase: f64 = first[mode + 3].parse().unwrap();
            assert!(next_phase > phase && next_phase <= phase + 0.201, "the saved pose must advance; inspect {}", case_root.display());
        }
        assert!(!after.1.contains("cue 'playsound c_dropit.aud'"), "the earlier drop cue must not replay");
        let (_, tail) = after.0.split_once("save restored in DeadCity without replaying Begin").unwrap();
        let tick = |line: &str| line.strip_prefix("update ").unwrap().split(':').next().unwrap().parse::<u64>().unwrap();
        let hold = tail.lines().find(|l| l.contains("finished: Die state holds until update")).unwrap();
        let menu = tail.lines().find(|l| l.contains("scripted Die hold finished")).unwrap();
        assert_eq!(tick(menu), tick(hold) + 45, "the deadline starts at the real final movement end");
        assert_eq!(tail.matches("menu: shown Choice { game: false, row: 0").count(), 1, "inspect {}", case_root.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
