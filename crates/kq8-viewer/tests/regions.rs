//! Windows world loading and quest-state checks against the installed GOG data.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn run_world(case: &str, args: &[&str]) -> PathBuf {
    let game = fs::canonicalize(env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../kq8-artifacts/region-tests")
        .join(format!("{}-{case}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game").arg(game)
        .args(["--sound", "off"]).args(args)
        .stdout(Stdio::null())
        .stderr(fs::File::create(dir.join("stderr.log")).unwrap())
        .spawn().unwrap();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{case} failed: see {}", dir.display());
            break;
        }
        if start.elapsed() > Duration::from_secs(180) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("{case} timed out: see {}", dir.display());
        }
        thread::sleep(Duration::from_millis(200));
    }
    dir.join("kq8-artifacts/logs")
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn temple4_connors_vortex_defeat_opens_the_death_menu_after_45_updates() {
    let logs = run_world("temple4-defeat", &[
        "--world", "temple4", "--no-ai", "--god",
        "--command", "loadKQ vortex.kq,3", "--command", "loadKQ vortrap.kq,4",
        // A regular gravity step after this placement enters the real trap.
        "--command", "move Connor 0 100011 93054 6600 0 0 0,8",
        "--frames", "1000",
    ]);
    let log = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let entry = log.lines().find(|l| l.contains("Connor Die Now, hold until update")).expect("Die Now entry");
    let tick = |s: &str| s.split_once("update ").unwrap().1.split(':').next().unwrap().parse::<u64>().unwrap();
    let due = entry.split("hold until update ").nth(1).unwrap().split(';').next().unwrap().parse::<u64>().unwrap();
    assert_eq!(due, tick(entry) + 45);
    let end = log.lines().find(|l| l.contains("scripted Die hold finished")).expect("game over");
    assert_eq!(tick(end), due);
    let shot = entry.split_once("; camera ").unwrap().1;
    assert!(end.contains(shot), "entry {entry}; deadline {end}");
    assert!(shot.starts_with("Some([99990.68, 94458.15, 7234.8955])"), "{shot}");
    assert!(log.contains("copy csuck2 of cg_suck2.kqs: drawn"));
    assert!(log.contains("menu: shown Choice { game: false, row: 0"));
    assert!(!log.contains("movie: playing") && !log.contains("credits: ending"));
    assert!(log.contains("RESULT: PASS"));
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn temple4_renders_each_scripted_vortex_animation() {
    // Start at the first winning cutscene rather than waiting for several
    // combats. Its real end listener drives the remaining animations.
    let logs = run_world("temple4-vortex", &[
        "--world", "temple4", "--no-ai", "--god",
        "--command", "loadKQ vortex.kq,3", "--command", "loadKQ vortrap.kq,4",
        "--command", "setThirdPerson,5", "--command", "KQCamera::follow none,6",
        "--command", "move KQCamera 0 98540.68 98608.15 7284.90 -0.199232 0 -2.849947,7",
        "--command", "loadAnim lsuck1 lu_suck1.kqs 0 1 0 0 99832 94535 5938 0 0 3.14,8",
        "--command", "setDeleteWhenDone lsuck1 1,9", "--command", "getEndLoop lsuck1 vortexTrap 1,9",
        "--frames", "1600",
    ]);
    let log = fs::read_to_string(logs.join("viewer.log")).unwrap();
    for (name, file) in [("lsuck1", "lu_suck1.kqs"), ("lsuck2", "lu_suck2.kqs"), ("vortexbye", "vrtexbye.kqs"), ("imdone", "c_imdone.kqs")] {
        let caption = format!("copy {name} of {file}: drawn, 1 movement(s), ");
        let count = log.lines().find_map(|l| l.split_once(&caption)?.1.split_whitespace().next()?.parse::<usize>().ok());
        assert!(count.is_some_and(|n| n > 0), "{name} did not acquire renderable parts; see {}", logs.display());
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn every_region_builds_and_presents_frames() {
    for region in ["Daventry", "CastleD", "DeadCity", "Swamp", "Gnome", "Barren", "IceWorld",
                   "Snowexit", "temple1", "temple2", "temple3", "temple4"] {
        let logs = run_world(region, &["--world", region, "--frames", "30"]);
        let log = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(log.lines().any(|l| l.starts_with(&format!("world built: {region}:"))),
            "{region} never finished building: see {}", logs.display());
        assert!(log.contains("window closed: frames=30 ") && log.contains("RESULT: PASS"),
            "{region} failed to render 30 frames: see {}", logs.display());
        println!("{region}: loaded and rendered 30 frames");
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn dead_city_script_created_monster_types_are_live_and_drawn() {
    let logs = run_world("deadcity-added-types", &[
        "--world", "DeadCity", "--no-ai", "--god",
        "--command", "KQMonster::add testRat rat,10",
        "--command", "KQMonster::add testRat2 rat2,11",
        "--command", "KQMonster::add testSnake snake,12",
        "--command", "KQMonster::add testSpirit spirit,13",
        "--command", "KQMonster::add testSkeleton skeleton,14",
        "--command", "KQMonster::add testSkelbow skelbow,15",
        "--frames", "90",
    ]);
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    for (name, kind, file) in [("testRat", "rat", "rat.kq"), ("testRat2", "rat2", "rat2.kq"),
        ("testSnake", "snake", "snake.kq"), ("testSpirit", "spirit", "spirit.kq"),
        ("testSkeleton", "skeleton", "skelbs.kq"), ("testSkelbow", "skelbow", "skelbow.kq")] {
        assert!(doors.contains(&format!("KQMonster::add {name} type {kind}")), "{name} missing from live world; see {}", logs.display());
        assert!(viewer.contains(&format!("copy {name} of {file}: drawn")), "{name} has no rendered model; see {}", logs.display());
    }
    assert!(viewer.contains("RESULT: PASS"));
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn daventry_girl_uses_carried_quest_flag_during_init() {
    let fresh = run_world("girl-unmet", &["--world", "Daventry", "--frames", "30"]);
    let before = fs::read_to_string(fresh.join("doors.log")).unwrap();
    assert!(before.contains("echo Has not met girl in DC - hide"), "see {}", fresh.display());
    assert!(!before.contains("echo Met Girl in DC - move her home"));

    // Start at the point after Dead City's conversation sets its flag. Ordinary
    // world travel must make it visible to Daventry's own girl.cs during Init.
    let travel = run_world("girl-met", &[
        "--world", "DeadCity", "--command", "setConnorFlag MetGirlInDC 1,2",
        "--console", "goworld Daventry,6", "--frames", "500",
    ]);
    let after = fs::read_to_string(travel.join("doors.log")).unwrap();
    assert!(after.contains("echo Met Girl in DC - move her home"), "see {}", travel.display());
    assert!(after.contains("setHome: home (135240, 279172, 11200)"));
    assert!(!after.contains("echo Has not met girl in DC - hide"));
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn gnome_rolling_rock_hits_connor_through_its_own_script() {
    // Wait in the moving rock's path. Its original movement supplies the
    // contact, and its original rollside.cs supplies the attack event.
    let logs = run_world("rolling-rock", &[
        "--world", "Gnome", "--at", "57500,104693,0",
        "--command", "move Connor 0 57500 104693 17650 0 0 0,4",
        "--frames", "400",
    ]);
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(doors.contains("rollside.cs rrock1 ConnorCollision"), "see {}", logs.display());
    assert!(fight.lines().any(|l| l.contains("hit by rrock1") && l.contains("health ")), "see {}", logs.display());
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn scales_cue_lowers_rendered_river_and_keeps_it_on_reload() {
    let logs = run_world("scales-drain", &[
        "--world", "DeadCity", "--command", "sendEvent 0 scale KQCueEvent 1 0,4",
        "--console", "reload,18", "--frames", "600",
    ]);
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let changed: Vec<usize> = viewer.lines().filter_map(|line| {
        let rest = line.strip_prefix("terrain: 5 height matches, ")?;
        rest.split_whitespace().next()?.parse().ok()
    }).filter(|&n| n > 0).collect();
    assert_eq!(changed.len(), 2, "initial change and restored meshes: see {}", logs.display());
    assert_eq!(changed[0], changed[1], "same rendered vertices after return");
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn connor_steps_onto_and_off_the_original_scales_dish() {
    let logs = run_world("scales-steps", &[
        "--world", "DeadCity", "--no-ai",
        "--command", "sendEvent 0 portrap3 KQCueEvent 7 0,4",
        "--hold", "0,-1,0,9,14", "--hold", "1,0,0,15,23",
        "--hold", "1,0,0,113,121",
        "--hold", "1,0,0,150,158", "--updates", "305",
    ]);
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(connor.contains("Scales boarding") && connor.contains("Scales stepping off"), "see {}", logs.display());
    assert_eq!(connor.matches("Scales ready at").count(), 2, "see {}", logs.display());
    assert!(doors.contains("scalert.cs scalert ConnorCollision 1"), "see {}", logs.display());
    assert!(connor.contains("leaves Scales at"), "see {}", logs.display());
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn a_feather_and_connors_weight_solve_the_scales() {
    let logs = run_world("scales-feather", &[
        "--world", "DeadCity", "--no-ai", "--item", "INVITEM_Feather",
        "--command", "sendEvent 0 portrap3 KQCueEvent 7 0,4",
        "--click", "scalert,9",
        "--hold", "0,1,0,14,19", "--hold", "1,0,0,20,28",
        "--updates", "115",
    ]);
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(doors.contains("scalert ConnorAction TestItem INVITEM_Feather"), "item use: see {}", logs.display());
    assert!(doors.contains("set DeadCity::Scales::Feather = 'right'"), "see {}", logs.display());
    assert!(doors.contains("scalelt.cs scalelt ConnorCollision 1"), "dish contact: see {}", logs.display());
    assert!(doors.contains("echo scales complete") && doors.contains("script scale.cs scale Cue 1 0"), "win callback: see {}", logs.display());
    assert!(doors.lines().any(|s| s.contains("loadKQ sclfthr.kq: a new copy") && s.contains("$ObjectName =")), "the placed feather must exist");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(viewer.contains("terrain: 5 height matches,"), "see {}", logs.display());
}
