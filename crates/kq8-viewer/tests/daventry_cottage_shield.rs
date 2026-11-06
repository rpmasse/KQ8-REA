//! Earn the village shield with ordinary controls, save, then use it after a fresh launch.
#[path = "support/pickup_help.rs"]
mod pickup_help;
#[path = "support/windmill.rs"]
mod windmill;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Play {
    game: PathBuf,
    options: Vec<u8>,
    root: PathBuf,
    slots: PathBuf,
}

impl Play {
    fn new(case: &str) -> Self {
        let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
        let options = fs::read(game.join("Options.cs")).unwrap();
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = manifest.join(format!("../../kq8-artifacts/{case}/{nonce}"));
        let slots = root.join("slots");
        Self {
            game,
            options,
            root,
            slots,
        }
    }

    fn run(&self, name: &str, inputs: &[&str]) -> (String, String) {
        let dir = self.root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &self.slots)
            .arg("--game")
            .arg(&self.game)
            .args(["--sound", "off"])
            .args(inputs)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (viewer, fs::read_to_string(logs.join("items.log")).unwrap())
    }

    fn slot(&self, number: u32) -> Value {
        let path = self.slots.join(format!("slot-{number}.json"));
        serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
        )
        .unwrap()
    }

    fn earn_shield(&self) {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let route = manifest.join("tests/routes/daventry_cottage_shield.json");
        let (walk, items) = self.run("new-game", &[
        "--menu-input", "enter,release,enter,release",
        "--walk-route", route.to_str().unwrap(),
        "--key", "popup:help=144,0", "--key", "popup:help=141,0",
        "--key", "popup:help=127,0", "--key", "popup:help=131,0",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=VillageShield:enter:release:escape,920",
        "--key", "menu:up:enter:release:left:enter:release,934", "--frames", "4500",
    ]);
        for name in ["saradoor", "NoName20"] {
            assert!(
                walk.lines()
                    .any(|line| line.contains(&format!("test --aim {name} (camera"))
                        && line.contains(&format!("hits {name} first"))),
                "{name}: inspect {}",
                self.root.display()
            );
        }
        assert!(walk.contains("walk-route at Shield beside the east wall"));
        assert!(items.contains("IN REACH: picked up; Connor's INVITEM_Invulnerable 0 -> 1"));
        pickup_help::check(
            &items,
            &walk,
            "INVITEM_Invulnerable",
            131,
            45,
            "HEALTH ITEMS:",
        );
        self.check_shield();
    }

    fn check_shield(&self) {
        let earned = self.slot(1);
        assert_eq!(earned["label"], "VillageShield");
        assert_eq!(earned["journey"]["ai_off"], false);
        assert_eq!(earned["journey"]["god_mode"], false);
        let shield = earned["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["kind"] == "INVITEM_Invulnerable")
            .unwrap();
        assert_eq!(shield["name"], "NoName20");
        assert_eq!(shield["quantity"], 1);
        assert_eq!(shield["place"]["Carried"], "Connor");
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn cottage_shield_is_visible_earned_saved_and_used_on_normal() {
    let play = Play::new("cottage-shield");
    play.earn_shield();
    let (reload, timer) = play.run("dose", &[
        "--menu-input", "down,enter,release,enter,release", "--key", "5,12",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ShieldActive:enter:release:escape,45",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ShieldEnded:enter:release:escape,340",
        "--key", "menu:up:enter:release:left:enter:release,355", "--frames", "2200",
    ]);
    assert!(reload.contains("save restored in Daventry without replaying Begin"));
    assert!(timer.contains("update 12: bar click left on NoName20 (slot 5, INVITEM_Invulnerable"));
    assert!(timer.contains("the shield: protection 0 -> 50"));
    assert!(timer.contains("update 312: NoName20: its time is up"));
    assert!(timer.contains("the shield ends: protection 50 -> 0"));
    let active = play.slot(2);
    assert_eq!(active["label"], "ShieldActive");
    assert_eq!(active["journey"]["stats"]["stats"][3], 50);
    assert_eq!(
        active["journey"]["potions"][0][0]["kind"],
        "INVITEM_Invulnerable"
    );
    assert_eq!(active["journey"]["potions"][0][0]["duration"], 20);
    let ended = play.slot(3);
    assert_eq!(ended["label"], "ShieldEnded");
    assert_eq!(ended["journey"]["stats"]["stats"][3], 0);
    assert!(ended["journey"]["potions"].as_array().unwrap().is_empty());
    assert!(ended["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["kind"] != "INVITEM_Invulnerable"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn cottage_basket_and_wizard_keep_the_earned_supplies() {
    let play = Play::new("prepared-wizard");
    earn_prepared_wizard(&play);
}

fn start_at_earned_shield(play: &Play) {
    if let Some(slot) = std::env::var_os("KQ8_SHIELD_SLOT") {
        fs::create_dir_all(&play.slots).unwrap();
        fs::copy(slot, play.slots.join("slot-1.json")).unwrap();
        play.check_shield();
    } else {
        play.earn_shield();
    }
}

fn earn_prepared_wizard(play: &Play) {
    start_at_earned_shield(play);
    let mut wizard: Value =
        serde_json::from_str(include_str!("routes/daventry_wizard.json")).unwrap();
    let mut landmarks: Vec<Value> =
        serde_json::from_str(include_str!("routes/daventry_cottage_exit.json")).unwrap();
    landmarks.append(wizard["landmarks"].as_array_mut().unwrap());
    wizard["start"] = 3.into();
    wizard["landmarks"] = Value::Array(landmarks);
    let route = play.root.join("route.json");
    fs::write(&route, serde_json::to_vec(&wizard).unwrap()).unwrap();
    let (walk, items) = play.run("wizard", &[
        "--menu-input","down,enter,release,enter,release","--walk-route",route.to_str().unwrap(),
        "--key","popup:help=144,0","--key","popup:help=141,0","--key","popup:help=127,0",
        "--key","popup:help=131,0","--key","popup:help=132,0","--key","popup:help=153,0",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=PreparedWizard:enter:release:escape,3300",
        "--key","menu:up:enter:release:left:enter:release,3315","--frames","12000",
    ]);
    assert!(walk
        .lines()
        .any(|l| l.contains("test --aim basket2 (camera") && l.contains("hits basket2 first")));
    assert!(items
        .lines()
        .any(|l| l.contains("basket2.cs") && l.contains("INVITEM_Mushroom 0 -> 5")));
    assert!(walk.contains("walk-route at Wizard on update"));
    pickup_help::check(&items, &walk, "INVITEM_Piece1", 132, 45, "INVENTORY:");
    assert!(walk.contains("starting help: acknowledged case 153"));
    let save = play.slot(2);
    assert_eq!(save["label"], "PreparedWizard");
    assert_eq!(save["journey"]["ai_off"], false);
    assert_eq!(save["journey"]["god_mode"], false);
    let inventory = save["journey"]["items"].as_array().unwrap();
    for (kind, count) in [
        ("INVITEM_MagicMap", 1),
        ("INVITEM_Piece1", 1),
        ("INVITEM_Mask1", 1),
        ("INVITEM_Mushroom", 5),
        ("INVITEM_Invulnerable", 1),
        ("INVITEM_Piece1A", 0),
    ] {
        let total: i64 = inventory
            .iter()
            .filter(|item| item["kind"] == kind)
            .map(|item| item["quantity"].as_i64().unwrap())
            .sum();
        assert_eq!(total, count, "{kind}: inspect {}", play.root.display());
    }
    let vars = save["kept"]["daventry"]["vars"].as_array().unwrap();
    for (key, expected) in [
        ("Daventry::Wizard::metConnor", "1"),
        ("Daventry::basket2::full", "false"),
    ] {
        assert!(
            vars.iter().any(
                |v| v[0].as_str().is_some_and(|k| k.eq_ignore_ascii_case(key)) && v[1] == expected
            ),
            "{key}"
        );
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; KQ8_SHIELD_SLOT can repeat from the cottage"]
fn garden_mushrooms_survive_the_trip_to_the_wizard() {
    let play = Play::new("garden-wizard");
    start_at_earned_shield(&play);
    let mut landmarks: Vec<Value> =
        serde_json::from_str(include_str!("routes/daventry_cottage_exit.json")).unwrap();
    landmarks.insert(0, serde_json::json!({
        "name": "Look toward the Crystal from the east room",
        "xy": [292175, 196971],
        "radius": 130,
        "aim": "NoName19",
    }));
    let garden: Vec<Value> =
        serde_json::from_str(include_str!("routes/daventry_cottage_garden.json")).unwrap();
    landmarks.extend(garden);
    let route = play.root.join("garden.json");
    fs::write(
        &route,
        serde_json::to_vec(&serde_json::json!({"start":3,"landmarks":landmarks})).unwrap(),
    )
    .unwrap();
    let (walk, items) = play.run("garden", &[
        "--menu-input", "down,enter,release,enter,release", "--walk-route", route.to_str().unwrap(),
        "--key","popup:help=131,0", "--key","popup:help=132,0", "--key","popup:help=133,0",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=GardenMushrooms:enter:release:escape,610",
        "--key","menu:up:enter:release:left:enter:release,625", "--frames","3500",
    ]);
    let garden_names = ["NoName18", "NoName17", "NoName16"];
    assert!(walk.lines().any(|l| l.contains("test --aim NoName19 (camera") && l.contains("hits NoName19 first")));
    assert!(items.lines().any(|l| l.contains("click NoName19 (") && l.contains("INVITEM_Crystal 0 -> 1")));
    for (i, name) in garden_names.iter().enumerate() {
        assert!(walk
            .lines()
            .any(|l| l.contains(&format!("test --aim {name} (camera"))
                && l.contains(&format!("hits {name} first"))));
        let gain = format!("INVITEM_Mushroom {} -> {}", 5 + i, 6 + i);
        assert!(items
            .lines()
            .any(|l| l.contains(&format!("click {name} (")) && l.contains(&gain)));
    }
    assert!(walk.contains("starting help: acknowledged case 133"));
    let check_supplies = |save: &Value| {
        assert_eq!(save["journey"]["ai_off"], false);
        assert_eq!(save["journey"]["god_mode"], false);
        let carried = save["journey"]["items"].as_array().unwrap();
        for (kind, count) in [("INVITEM_Mushroom", 8), ("INVITEM_Invulnerable", 1), ("INVITEM_Crystal", 1)] {
            assert!(carried.iter().any(|i| i["kind"] == kind
                && i["quantity"] == count
                && i["place"]["Carried"] == "Connor"));
        }
        for name in garden_names {
            assert!(save["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["name"] == name && i["place"] == "Gone"));
        }
    };
    let garden = play.slot(2);
    assert_eq!(garden["label"], "GardenMushrooms");
    check_supplies(&garden);

    let mut wizard: Value =
        serde_json::from_str(include_str!("routes/daventry_wizard.json")).unwrap();
    wizard["start"] = 3.into();
    let route = play.root.join("wizard.json");
    fs::write(&route, serde_json::to_vec(&wizard).unwrap()).unwrap();
    let (walk, items) = play.run("wizard", &[
        "--menu-input","down,enter,release,down,enter,release", "--walk-route",route.to_str().unwrap(),
        "--key","popup:help=141,0", "--key","popup:help=127,0", "--key","popup:help=132,0", "--key","popup:help=153,0",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=GardenWizard:enter:release:escape,3200",
        "--key","menu:up:enter:release:left:enter:release,3215","--frames","12000",
    ]);
    assert!(walk.contains("save restored in Daventry without replaying Begin"));
    assert!(walk.contains("walk-route at Wizard on update"));
    assert!(items.contains("INVITEM_MagicMap 0 -> 1"));
    let wizard = play.slot(3);
    assert_eq!(wizard["label"], "GardenWizard");
    check_supplies(&wizard);
    for kind in ["INVITEM_MagicMap", "INVITEM_Piece1", "INVITEM_Mask1"] {
        assert!(wizard["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == kind && i["quantity"] == 1));
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; KQ8_PREPARED_AXE_SLOT can repeat the duel"]
fn earned_supplies_defeat_the_henchman_and_reopen_the_west_yard() {
    let play = Play::new("prepared-henchman");
    fs::create_dir_all(&play.slots).unwrap();
    let earned = if let Some(existing) = std::env::var_os("KQ8_PREPARED_AXE_SLOT") {
        PathBuf::from(existing)
    } else {
        let wizard = Play::new("guard-wizard");
        earn_prepared_wizard(&wizard);
        let mill = Play::new("guard-windmill");
        fs::create_dir_all(&mill.slots).unwrap();
        fs::copy(
            wizard.slots.join("slot-2.json"),
            mill.slots.join("slot-1.json"),
        )
        .unwrap();
        windmill::earn_axe(&play.game, &mill.root)
    };
    fs::copy(earned, play.slots.join("slot-1.json")).unwrap();
    let start = play.slot(1);
    assert_eq!(start["label"], "Windmill");
    assert_eq!(start["journey"]["god_mode"], false);
    assert_eq!(start["journey"]["ai_off"], false);
    let carried = start["journey"]["items"].as_array().unwrap();
    for name in [
        "INVITEM_MagicMap",
        "INVITEM_SmallAx",
        "INVITEM_Invulnerable",
        "INVITEM_Mushroom",
    ] {
        assert!(
            carried
                .iter()
                .any(|item| item["kind"] == name && item["quantity"].as_i64().unwrap() > 0),
            "missing earned {name}"
        );
    }
    let mut route: Value =
        serde_json::from_str(include_str!("routes/daventry_hero_ring.json")).unwrap();
    let landmarks = route["landmarks"].as_array_mut().unwrap();
    let guard_index = landmarks
        .iter()
        .position(|p| p["fight"] == "Henchman")
        .unwrap();
    landmarks.truncate(guard_index + 1);
    let exit = landmarks.iter_mut().find(|p| p["name"] == "Windmill exit").unwrap();
    exit["inventory_slot"] = 5.into();
    let guard = &mut landmarks[guard_index];
    // The guard stands here after the original scene. Start attacking
    // when he is in Connor's real hand reach on this level floor.
    guard["xy"] = serde_json::json!([137200, 250000]);
    guard["radius"] = 2350.into();
    guard["retreat"] = false.into();
    guard["big_swing"] = true.into();
    let input = play.root.join("duel.json");
    fs::write(&input, serde_json::to_vec(&route).unwrap()).unwrap();
    let (viewer, items) = play.run("duel", &[
        "--menu-input","down,enter,release,enter,release", "--walk-route",input.to_str().unwrap(),
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=GuardDuel:enter:release:escape,1080",
        "--key","menu:up:enter:release:left:enter:release,1095", "--frames","4000",
    ]);
    assert!(viewer.contains("walk-route scene complete at Henchman's approach trigger"));
    assert!(viewer.contains("walk-route at Windmill exit on update"));
    assert!(viewer.find("walk-route scene complete at Henchman's approach trigger").unwrap()
        < viewer.find("walk-route pressed inventory key 5 at Windmill exit").unwrap());
    assert!(viewer.contains("walk-route at Henchman on update"));
    assert!(items.contains("the shield: protection"));
    let logs = play.root.join("duel/kq8-artifacts/logs");
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(fight
        .lines()
        .any(|l| l.contains("Do (conaxe.anm/attack2)") && l.contains("hits Henchman")));
    assert_eq!(fight.matches("Henchman: dies (rule 30)").count(), 1);
    assert_eq!(
        fight
            .matches("Henchman: its death ended (rule 32): Connor gets its experience 30")
            .count(),
        1
    );
    assert!(viewer.contains("corpse of Henchman: hm_dead.kqs drawn"));
    let events = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert_eq!(events.matches("script henchman.cs Henchman Die").count(), 1);
    assert!(events.contains("henchman.cs line 14: handsOff 0"));
    assert!(events.contains("henchman.cs line 16: deleteObject clicker2"));
    let victory = play.slot(2);
    assert_eq!(victory["label"], "GuardDuel");
    let gone = victory["kept"]["daventry"]["gone"].as_array().unwrap();
    for name in ["henchman", "clicker2"] {
        assert!(
            gone.iter()
                .any(|v| v.as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))),
            "missing {name} removal in {}",
            play.root.display()
        );
    }
    assert!(victory["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    assert!(
        victory["journey"]["stats"]["level"].as_i64().unwrap()
            > start["journey"]["stats"]["level"].as_i64().unwrap()
    );

    let onward = play.root.join("yard.json");
    let walk = serde_json::json!({"start":3,"landmarks":[
        {"name":"West of the defeated guard","xy":[134133,250844],"radius":230}
    ]});
    fs::write(&onward, serde_json::to_vec(&walk).unwrap()).unwrap();
    let (reload,_) = play.run("yard",&[
        "--menu-input","down,enter,release,down,enter,release","--walk-route",onward.to_str().unwrap(),
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=WestYard:enter:release:escape,75",
        "--key","menu:up:enter:release:left:enter:release,88","--frames","1600",
    ]);
    assert!(reload.contains("save restored in Daventry without replaying Begin"));
    assert!(reload.contains("walk-route at West of the defeated guard"));
    let yard = play.slot(3);
    assert_eq!(yard["label"], "WestYard");
    let before = &victory["journey"]["stats"];
    let after = &yard["journey"]["stats"];
    assert_eq!(after["level"], before["level"]);
    assert_eq!(
        after["stats"].as_array().unwrap()[1..],
        before["stats"].as_array().unwrap()[1..]
    );
    let fight = fs::read_to_string(play.root.join("yard/kq8-artifacts/logs/fight.log")).unwrap();
    assert!(!fight.contains("Henchman: its death ended"));
    let mut health = vec![before["stats"][0].as_i64().unwrap()];
    for line in fight
        .lines()
        .filter(|line| line.contains(": Connor: hit by "))
    {
        if let Some((old, new)) = line
            .split_once("; health ")
            .and_then(|(_, hp)| hp.split_once(" -> "))
        {
            assert_eq!(old.parse::<i64>().unwrap(), *health.last().unwrap());
            health.push(
                new.split_whitespace()
                    .next()
                    .unwrap()
                    .parse::<i64>()
                    .unwrap(),
            );
        }
    }
    let saved_health = after["stats"][0].as_i64().unwrap();
    assert!(
        saved_health > 0 && health.contains(&saved_health),
        "inspect {}",
        play.root.display()
    );
    for name in ["henchman", "clicker2"] {
        assert!(yard["kept"]["daventry"]["gone"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry.as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))));
    }
    assert!(yard["position"][0].as_f64().unwrap() < 135000.);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}
