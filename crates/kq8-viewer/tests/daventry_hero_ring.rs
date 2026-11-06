//! Continue the earned Small Axe route through the hero's tomb, without debug state.
#[path = "support/daventry.rs"]
mod daventry;
use serde_json::Value;
use std::{fs, path::{Path, PathBuf}, process::Command};

fn quantity(slot: &Value, kind: &str) -> i64 {
    slot["journey"]["items"].as_array().unwrap().iter()
        .filter(|item| item["kind"].as_str().is_some_and(|name| name.eq_ignore_ascii_case(kind)))
        .map(|item| item["quantity"].as_i64().unwrap()).sum()
}

fn read(path: impl AsRef<Path>) -> Value { serde_json::from_slice(&fs::read(path).unwrap()).unwrap() }

fn windmill_exit_evidence(logs: &Path) -> (u64, u64, u64, u64) {
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    let at = |text: &str, needle: &str| -> u64 {
        let line = text.lines().find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("missing {needle}, inspect {}", logs.display()));
        line.split_once("update ").unwrap().1.split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap()
    };
    let triggered = at(&viewer, "trap Henchtp: On Connor");
    let released = at(&viewer, "walk-route scene complete at Henchman's approach trigger");
    let outward = at(&viewer, "walk-route at Windmill exit");
    let attack = at(&fight, "Henchman: attack (rule 7, standing)");
    assert!(triggered < released && released < outward && outward <= attack, "inspect {}", logs.display());
    assert!(at(&viewer, "ax.cs line 82: applied: ax deleted:") <= released
        && at(&viewer, "ax.cs line 81: applied: handsOff false:") <= released);
    let x_at = |tick: u64| -> f32 {
        let prefix = format!("{tick} ");
        connor.lines().find(|l| l.starts_with(&prefix)).unwrap()
            .split_whitespace().nth(2).unwrap().parse().unwrap()
    };
    assert!(x_at(outward) < 141100. && x_at(outward) < x_at(outward - 1),
        "Connor must be outside and moving west, inspect {}", logs.display());
    (triggered, released, outward, attack)
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; KQ8_AXE_SLOT can repeat the earned continuation"]
fn earned_windmill_exit_reaches_guard_in_two_processes() {
    let game = daventry::game();
    let root = daventry::test_root("windmill-exit");
    fs::create_dir_all(&root).unwrap();
    let earned = std::env::var_os("KQ8_AXE_SLOT").map(PathBuf::from).unwrap_or_else(|| earn_axe(&game, &root));
    let slot = read(&earned);
    assert_eq!(quantity(&slot, "INVITEM_SmallAx"), 1);
    assert_eq!(quantity(&slot, "INVITEM_MagicMap"), 1);
    assert_eq!(quantity(&slot, "INVITEM_RingOfDeadHero"), 0);
    assert_eq!(slot["journey"]["ai_off"], false);
    assert_eq!(slot["journey"]["god_mode"], false);
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_hero_ring.json");
    let mut replay = read(source);
    // The campaign inputs are identical. End this diagnostic at the first
    // live attack, before the separate duel checkpoint.
    replay["stop_on_attack"] = "Henchman".into();
    let route = root.join("input.json");
    fs::write(&route, serde_json::to_vec(&replay).unwrap()).unwrap();
    for run in 1..=2 {
        let dir = root.join(format!("process-{run}"));
        let slots = dir.join("slots");
        fs::create_dir_all(&slots).unwrap();
        fs::copy(&earned, slots.join("slot-1.json")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--menu-input", "down,enter,release,enter,release", "--frames", "3200"])
            .arg("--walk-route").arg(&route).output().unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS") &&
            viewer.contains("walk-route observed Henchman attacking"), "inspect {}", dir.display());
        let (trigger, scene, exit, attack) = windmill_exit_evidence(&logs);
        println!("process {run}: authored trigger {trigger}, scene complete {scene}, outward crossing {exit}, guard attack {attack}; {}", dir.display());
    }
}

fn earn_axe(game: &Path, root: &Path) -> PathBuf {
    let earned = root.join("earned");
    daventry::earn_magic_map(game, &earned);
    let dir = earned.join("windmill"); fs::create_dir_all(&dir).unwrap();
    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_windmill.json");
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer")).current_dir(&dir)
        .env("KQ8_SAVE_DIR", earned.join("slots")).arg("--game").arg(game)
        .args(["--sound", "off", "--menu-input", "down,enter,release,enter,release", "--key", "popup:help=142,0"])
        .arg("--walk-route").arg(route)
        .args(["--key", "menu:down:down:down:enter:release:enter:release:escape:type=Hillside:enter:release:escape,838",
            "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Windmill:enter:release:escape,1800",
            "--key", "menu:up:enter:release:left:enter:release,1810", "--frames", "6500"]).output().unwrap();
    fs::write(dir.join("stderr.log"), output.stderr).unwrap();
    assert!(output.status.success(), "inspect {}", dir.display());
    earned.join("slots/slot-3.json")
}

#[test]
#[ignore = "unfinished campaign: stops at Henchman (see STATUS); requires KQ8_GAME_DIR/graphics; KQ8_AXE_SLOT repeats the leg"]
fn earned_small_axe_to_hero_ring_and_disk_reload() {
    let game = daventry::game();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let root = daventry::test_root("hero-ring-route"); fs::create_dir_all(&root).unwrap();
    println!("hero-ring-route artifacts: {}", root.display());
    let earned = std::env::var_os("KQ8_AXE_SLOT").map(PathBuf::from).unwrap_or_else(|| earn_axe(&game, &root));
    let start = read(&earned);
    assert_eq!(quantity(&start, "INVITEM_SmallAx"), 1);
    assert_eq!(quantity(&start, "INVITEM_MagicMap"), 1);
    assert_eq!(quantity(&start, "INVITEM_RingOfDeadHero"), 0);
    assert_eq!(start["journey"]["god_mode"], false); assert_eq!(start["journey"]["ai_off"], false);
    let slots = root.join("slots"); fs::create_dir_all(&slots).unwrap();
    fs::copy(earned, slots.join("slot-1.json")).unwrap();
    let run = |part: &str, args: &[&str], route: bool| {
        let dir = root.join(part); fs::create_dir_all(&dir).unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        cmd.current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game).args(["--sound", "off"]).args(args);
        if route { cmd.arg("--walk-route").arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_hero_ring.json")); }
        let output = cmd.output().unwrap(); fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        if route {
            windmill_exit_evidence(&logs);
            let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
            let mut events = fight.lines();
            let miss = events.find(|line| line.contains("Henchman: Do (Attack") && line.contains("miss, out of reach"))
                .unwrap_or_else(|| panic!("no ordinary missed spear thrust; inspect {}", dir.display()));
            let reply = events.find(|line| line.contains("Connor: Do (conaxe.anm/attack") && line.contains("hits Henchman"))
                .unwrap_or_else(|| panic!("no return axe hit after {miss}; inspect {}", dir.display()));
            let loss = events.find(|line| line.contains("Henchman: hit by Connor") && line.contains("loss "))
                .expect("return attack must apply combat damage");
            println!("Henchman exchange:\n{miss}\n{reply}\n{loss}");
        }
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, logs)
    };
    let (viewer, logs) = run("tomb", &["--menu-input", "down,enter,release,enter,release", "--frames", "11000",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=HeroRing:enter:release:down:down:down:enter:release:left:enter:release,2600"], true);
    assert!(viewer.contains("walk-route at Dead hero ring on update"), "inspect {}", root.display());
    let end = read(slots.join("slot-2.json"));
    assert_eq!(quantity(&end, "INVITEM_RingOfDeadHero"), 1);
    assert_eq!(end["journey"]["god_mode"], false); assert_eq!(end["journey"]["ai_off"], false);
    assert!(end["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    let vars = end["kept"]["daventry"]["vars"].as_array().unwrap();
    assert!(vars.iter().any(|entry| entry[0] == "Daventry::HeroDoor::Locked" && entry[1] == "0"));
    assert!(vars.iter().any(|entry| entry[0] == "Daventry::HeroCoffin::Open" && entry[1] == "1"));
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(fight.contains("hits object tombdor2") && fight.contains("sprigan: dies"));
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert_eq!(doors.matches("script tombdor2.cs tombdor2 Die Normal").count(), 1);
    assert!(doors.contains("script coffnlid.cs coffnlid Push") && doors.contains("script ring.cs NoName3 ConnorAction"));
    run("reload", &["--menu-input", "down,enter,release,down,enter,release", "--frames", "1900",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=KeptHeroRing:enter:release:down:down:down:enter:release:left:enter:release,25"], false);
    let restored = read(slots.join("slot-3.json"));
    assert_eq!(quantity(&restored, "INVITEM_RingOfDeadHero"), 1);
    assert_eq!(restored["journey"]["stats"], end["journey"]["stats"]);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; isolated fixture, not campaign acceptance"]
fn breakable_route_dispatches_normal_weapon_hits_to_the_original_tomb_door() {
    let game = daventry::game();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let root = daventry::test_root("tomb-door-replay"); fs::create_dir_all(&root).unwrap();
    let route = root.join("route.json");
    fs::write(&route, serde_json::to_vec(&serde_json::json!({"start":10,"landmarks":[
        {"name":"Break original tomb door","xy":[225804,308950],"radius":200,"break_object":"tombdor2","big_swing":false}
    ]})).unwrap()).unwrap();
    let slots = root.join("slots");
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer")).current_dir(&root)
        .env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
        .args(["--world", "Daventry", "--sound", "off", "--no-ai", "--god", "--at", "225804,308950,0",
            "--command", "give SmallAx 1,5", "--frames", "1800",
            "--key", "menu:down:down:enter:release:enter:release:escape:type=DoorReplay:enter:release:down:down:down:enter:release:left:enter:release,100"])
        .arg("--walk-route").arg(route).output().unwrap();
    fs::write(root.join("stderr.log"), output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS") && viewer.contains("walk-route at Break original tomb door"), "inspect {}", root.display());
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(fight.contains("Do (conaxe.anm/attack") && fight.contains("hits object tombdor2"), "inspect {}", root.display());
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert_eq!(doors.matches("script tombdor2.cs tombdor2 Die Normal").count(), 1);
    let save = read(slots.join("slot-1.json"));
    let interactions = &save["kept"]["daventry"]["interactions"];
    assert_eq!(interactions["doors"]["tombdor2"]["state"], "Opened");
    assert_eq!(interactions["doors"]["tombdor2"]["collision_type"].as_u64().unwrap() & 32, 0);
    assert_eq!(interactions["object_stats"]["tombdor2"]["last_hit"], "Normal");
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
