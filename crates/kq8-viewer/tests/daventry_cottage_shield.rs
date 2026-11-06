//! Earn the village shield with ordinary controls, save, then use it after a fresh launch.
#[path = "support/church.rs"]
mod church;
#[path = "support/shadow.rs"]
mod shadow;
#[path = "support/castle.rs"]
mod castle;
#[path = "support/castle_interior.rs"]
mod castle_interior;
#[path = "support/pickup_help.rs"]
mod pickup_help;
#[path = "support/first_person.rs"]
mod first_person;
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

fn earned_slot(variable: &str) -> Option<PathBuf> {
    std::env::var_os(variable).filter(|path| !path.is_empty()).map(PathBuf::from)
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
    if let Some(slot) = earned_slot("KQ8_SHIELD_SLOT") {
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
    earn_garden_wizard(&play);
}

fn earn_garden_wizard(play: &Play) {
    start_at_earned_shield(play);
    let mut landmarks: Vec<Value> =
        serde_json::from_str(include_str!("routes/daventry_cottage_exit.json")).unwrap();
    // The garden path has already left the shelf by the time the Crystal is clicked.
    landmarks.splice(0..1, [
        serde_json::json!({"name":"Look toward the Crystal from open floor",
            "xy":[291725,197646],"radius":140,"aim":"NoName19"}),
        serde_json::json!({"name":"Cross the east room","xy":[290950,197330],"radius":170}),
    ]);
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
    let earned = if let Some(existing) = earned_slot("KQ8_PREPARED_AXE_SLOT") {
        existing
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
    defeat_henchman(&play, &earned);
}

fn defeat_henchman(play: &Play, earned: &Path) {
    fs::create_dir_all(&play.slots).unwrap();
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

fn carries_one(save: &Value, item: &str) -> bool {
    save["journey"]["items"].as_array().unwrap().iter()
        .any(|i| i["kind"] == item && i["quantity"] == 1 && i["place"]["Carried"] == "Connor")
}

fn hero_flag(save: &Value, name: &str, value: &str) -> bool {
    save["kept"]["daventry"]["vars"].as_array().unwrap().iter()
        .any(|entry| entry[0] == name && entry[1] == value)
}

fn earn_hero_door() -> PathBuf {
    let hero = Play::new("crystal-hero-ring");
    let yard = if let Some(path) = earned_slot("KQ8_CRYSTAL_YARD_SLOT") {
        path
    } else {
        let garden = Play::new("hero-garden");
        earn_garden_wizard(&garden);
        let mill = Play::new("hero-windmill");
        fs::create_dir_all(&mill.slots).unwrap();
        fs::copy(
            garden.slots.join("slot-3.json"),
            mill.slots.join("slot-1.json"),
        )
        .unwrap();
        let axe = windmill::earn_axe(&mill.game, &mill.root);
        let guard = Play::new("hero-guard");
        defeat_henchman(&guard, &axe);
        guard.slots.join("slot-3.json")
    };
    fs::create_dir_all(&hero.slots).unwrap();
    fs::copy(yard, hero.slots.join("slot-1.json")).unwrap();
    let start = hero.slot(1);
    assert_eq!(start["label"], "WestYard");
    assert_eq!(start["journey"]["ai_off"], false);
    assert_eq!(start["journey"]["god_mode"], false);
    assert!(carries_one(&start, "INVITEM_Crystal") && carries_one(&start, "INVITEM_SmallAx"));
    assert!(!carries_one(&start, "INVITEM_RingOfDeadHero"));
    let route =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_crystal_tomb.json");
    let (viewer, items) = hero.run("door", &[
        "--menu-input","down,enter,release,enter,release","--walk-route",route.to_str().unwrap(),
        "--key","popup:help=132,0",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=HeroDoor:enter:release:escape,1840",
        "--key","menu:up:enter:release:left:enter:release,1855","--frames","9500",
    ]);
    assert!(items
        .lines()
        .any(|l| l.contains("slot 2, INVITEM_Crystal") && l.contains("x1 -> x0")));
    assert!(viewer.contains("walk-route at Inside the open tomb on update"));
    let fight = fs::read_to_string(hero.root.join("door/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("sprigan: its death ended (rule 32)").count(), 1);
    assert_eq!(fs::read(hero.game.join("Options.cs")).unwrap(), hero.options);
    hero.slots.join("slot-2.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; KQ8_HERO_DOOR_SLOT or KQ8_CRYSTAL_YARD_SLOT can repeat earned legs"]
fn earned_crystal_to_hero_ring_and_fresh_reload() {
    earn_hero_ring();
}

fn earn_hero_ring() -> PathBuf {
    let hero = Play::new("hero-coffin-reload");
    let source = earned_slot("KQ8_HERO_DOOR_SLOT").unwrap_or_else(earn_hero_door);
    fs::create_dir_all(&hero.slots).unwrap();
    fs::copy(source, hero.slots.join("slot-1.json")).unwrap();
    let start = hero.slot(1);
    assert_eq!(start["label"], "HeroDoor");
    assert_eq!(start["journey"]["god_mode"], false);
    assert_eq!(start["journey"]["ai_off"], false);
    assert!(start["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    assert!(start["kept"]["daventry"]["gone"].as_array().unwrap().iter().any(|n| n == "sprigan"));
    assert!(hero_flag(&start, "Daventry::HeroDoor::Locked", "0"));
    assert!(hero_flag(&start, "Daventry::HeroCoffin::Open", "0"));
    assert!(!carries_one(&start, "INVITEM_Crystal") && !carries_one(&start, "INVITEM_RingOfDeadHero"));
    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_hero_coffin.json");
    let (viewer, items) = hero.run("tomb", &[
        "--menu-input","down,enter,release,enter,release","--walk-route",route.to_str().unwrap(),
        "--key","popup:help=132,0",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=HeroRing:enter:release:escape,600",
        "--key","menu:up:enter:release:left:enter:release,615","--frames","4300",
    ]);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    let open = viewer
        .find("walk-route observed Daventry::HeroCoffin::Open = 1")
        .unwrap();
    let ring = viewer
        .lines()
        .find(|l| l.contains("test --aim NoName3 (camera") && l.contains("hits NoName3 first"))
        .unwrap();
    assert!(open < viewer.find(ring).unwrap());
    assert!(items
        .lines()
        .any(|l| l.contains("click NoName3 (") && l.contains("INVITEM_RingOfDeadHero 0 -> 1")));
    let logs = hero.root.join("tomb/kq8-artifacts/logs");
    let motion = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert!(motion.lines().any(|line| line.contains("push touch coffnlid (scripted)")
        && line.contains("-> Push")));
    let events = fs::read_to_string(logs.join("doors.log")).unwrap();
    assert!(events.contains("script coffnlid.cs coffnlid Push"));
    let check_ring = |s: &Value| {
        assert_eq!(s["journey"]["ai_off"], false);
        assert_eq!(s["journey"]["god_mode"], false);
        assert!(s["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
        assert!(carries_one(s, "INVITEM_RingOfDeadHero") && !carries_one(s, "INVITEM_Crystal"));
        let kept = &s["kept"]["daventry"];
        for (flag, value) in [
            ("Daventry::HeroDoor::Locked", "0"),
            ("Daventry::HeroCoffin::Open", "1"),
        ] {
            assert!(hero_flag(s, flag, value));
        }
        assert!(kept["gone"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "sprigan"));
    };
    let tomb = hero.slot(2);
    assert_eq!(tomb["label"], "HeroRing");
    check_ring(&tomb);
    let (reload, items) = hero.run("ring-reload", &[
        "--menu-input","down,enter,release,down,enter,release",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=HeroRingReload:enter:release:escape,45",
        "--key","menu:up:enter:release:left:enter:release,60","--frames","1300",
    ]);
    assert!(reload.contains("save restored in Daventry without replaying Begin"));
    assert!(!items.contains("INVITEM_RingOfDeadHero 0 -> 1"));
    let restored = hero.slot(3);
    assert_eq!(restored["label"], "HeroRingReload");
    check_ring(&restored);
    assert_eq!(
        fs::read(hero.game.join("Options.cs")).unwrap(),
        hero.options
    );
    hero.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; KQ8_HERO_RING_SLOT repeats the watermill leg"]
fn earned_hero_ring_to_watermill_tree() {
    let ring = earned_slot("KQ8_HERO_RING_SLOT").unwrap_or_else(earn_hero_ring);
    earn_mill_tree(&ring);
}

fn earn_mill_tree(ring: &Path) -> PathBuf {
    let mill = Play::new("hero-ring-watermill");
    fs::create_dir_all(&mill.slots).unwrap();
    fs::copy(ring, mill.slots.join("slot-1.json")).unwrap();
    let start = mill.slot(1);
    assert_eq!(start["label"], "HeroRingReload");
    assert_eq!(start["journey"]["ai_off"], false);
    assert_eq!(start["journey"]["god_mode"], false);
    assert!(carries_one(&start, "INVITEM_RingOfDeadHero") && carries_one(&start, "INVITEM_SmallAx"));
    assert!(hero_flag(&start, "Daventry::MillTree::down", "0"));

    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_ring_watermill.json");
    let (viewer, _) = mill.run("tree", &[
        "--menu-input", "down,enter,release,enter,release",
        "--walk-route", route.to_str().unwrap(), "--key", "popup:help=132,0",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=MillTree:enter:release:escape,1550",
        "--key", "menu:up:enter:release:left:enter:release,1565", "--frames", "8800",
    ]);
    assert!(viewer.contains("walk-route at Water mill tree on update"));
    let logs = mill.root.join("tree/kq8-artifacts/logs");
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(fight.contains("Do (conaxe.anm/attack2): hits object MillTree"));
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    for prop in ["MillTre2", "MillTre3", "Dock3"] {
        assert!(doors.contains(&format!("{prop} comes into the world")), "{prop}");
        let shape = if prop == "Dock3" { "interior BSP" } else { "object box" };
        let registered = viewer.lines().filter(|line| line.contains(&format!("loadKQ: {prop} came in:"))
            && line.contains(&format!("registered collision [\"{shape}\"]"))).count();
        assert_eq!(registered, 1, "{prop} should have exactly one live registration");
    }
    let saved = mill.slot(2);
    assert_eq!(saved["label"], "MillTree");
    assert!(saved["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    assert!(hero_flag(&saved, "Daventry::MillTree::down", "1"));
    assert!(carries_one(&saved, "INVITEM_RingOfDeadHero") && carries_one(&saved, "INVITEM_SmallAx"));
    assert_eq!(saved["journey"]["ai_off"], false);
    assert_eq!(saved["journey"]["god_mode"], false);
    assert_eq!(fs::read(mill.game.join("Options.cs")).unwrap(), mill.options);
    mill.slots.join("slot-2.json")
}

fn take_mill_rope(play: &Play, source: &Path, from_ring: bool) {
    fs::create_dir_all(&play.slots).unwrap();
    fs::copy(source, play.slots.join("slot-1.json")).unwrap();
    let start = play.slot(1);
    assert_eq!(start["label"], if from_ring { "HeroRingReload" } else { "MillTree" });
    assert!(carries_one(&start, "INVITEM_RingOfDeadHero") && carries_one(&start, "INVITEM_SmallAx"));
    assert!(!carries_one(&start, "INVITEM_RopeAndHook"));
    assert_eq!(start["journey"]["ai_off"], false);
    assert_eq!(start["journey"]["god_mode"], false);
    assert!(hero_flag(&start, "Daventry::MillTree::down", if from_ring { "0" } else { "1" }));

    let routes = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes");
    let climb: Value = serde_json::from_slice(&fs::read(routes.join("daventry_mill_rope.json")).unwrap()).unwrap();
    let route = if from_ring {
        let mut trip: Value = serde_json::from_slice(&fs::read(routes.join("daventry_ring_watermill.json")).unwrap()).unwrap();
        trip["landmarks"].as_array_mut().unwrap().extend_from_slice(climb["landmarks"].as_array().unwrap());
        trip
    } else { climb };
    let path = play.root.join("route.json");
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let save_at = if from_ring { 2200 } else { 900 };
    let save = format!("menu:down:down:down:enter:release:enter:release:escape:type=MillRope:enter:release:escape,{save_at}");
    let quit = format!("menu:up:enter:release:left:enter:release,{}", save_at + 15);
    let (viewer, items) = play.run("climb", &[
        "--menu-input", "down,enter,release,enter,release", "--walk-route", path.to_str().unwrap(),
        "--key", "popup:help=132,0", "--key", "popup:help=139,0", "--key", "popup:help=128,0",
        "--key", &save, "--key", &quit, "--frames", "11500",
    ]);
    assert_eq!(viewer.matches("walk-route landed on update").count(), 2);
    for target in ["Watmdoor", "ropenhok"] {
        assert!(viewer.lines().any(|line| line.contains(&format!("test --aim {target} (camera through gameplay pixel"))
            && line.contains(&format!("hits {target} first"))), "{target}");
    }
    if from_ring {
        assert!(viewer.lines().any(|line| line.contains("loadKQ: Dock3 came in:")
            && line.contains("registered collision [\"interior BSP\"]")));
    }
    let logs = play.root.join("climb/kq8-artifacts/logs");
    let traps = fs::read_to_string(logs.join("traps.log")).unwrap();
    assert!(traps.contains("trap miltrap2: On Connor"));
    assert!(items.contains("ropenhok.cs line 4: give"));
    assert_eq!(items.matches("INVITEM_RopeAndHook 0 -> 1").count(), 1);
    let saved = play.slot(2);
    assert_eq!(saved["label"], "MillRope");
    check_mill_rope_save(&saved);
    let (reload, items) = play.run("reload", &[
        "--menu-input", "down,enter,release,down,enter,release",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=MillRopeReload:enter:release:escape,45",
        "--key", "menu:up:enter:release:left:enter:release,60", "--frames", "1500",
    ]);
    assert!(reload.contains("save restored in Daventry without replaying Begin"));
    assert!(!items.contains("INVITEM_RopeAndHook 0 -> 1"));
    let restored = play.slot(3);
    assert_eq!(restored["label"], "MillRopeReload");
    check_mill_rope_save(&restored);
    assert_eq!(restored["journey"]["stats"]["stats"][4], saved["journey"]["stats"]["stats"][4]);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

fn check_mill_rope_save(save: &Value) {
    assert!(carries_one(save, "INVITEM_RopeAndHook") && carries_one(save, "INVITEM_RingOfDeadHero"));
    assert!(hero_flag(save, "Daventry::MillTree::down", "1"));
    assert!(save["kept"]["daventry"]["gone"].as_array().unwrap().iter().any(|name| name == "ropenhok"));
    assert!(save["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    assert_eq!(save["journey"]["god_mode"], false);
    assert_eq!(save["journey"]["ai_off"], false);
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_HERO_RING_SLOT and KQ8_MILL_TREE_SLOT"]
fn earned_mill_rope_on_live_and_reloaded_scenery() {
    let ring = earned_slot("KQ8_HERO_RING_SLOT").unwrap_or_else(earn_hero_ring);
    let tree = earned_slot("KQ8_MILL_TREE_SLOT").unwrap_or_else(|| earn_mill_tree(&ring));
    take_mill_rope(&Play::new("mill-rope-reloaded"), &tree, false);
    take_mill_rope(&Play::new("mill-rope-live"), &ring, true);
}

fn church_yard(play: &Play) {
    let source = earned_slot("KQ8_MILL_ROPE_SLOT").unwrap_or_else(|| {
        let mill = Play::new("church-mill-prefix");
        take_mill_rope(&mill, &earn_hero_ring(), true);
        mill.slots.join("slot-3.json")
    });
    fs::create_dir_all(&play.slots).unwrap();
    fs::copy(source, play.slots.join("slot-1.json")).unwrap();
    let start = play.slot(1);
    assert_eq!(start["label"], "MillRopeReload");
    check_mill_rope_save(&start);
    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_mill_church.json");
    let (viewer, _) = play.run("walk", &[
        "--menu-input", "down,enter,release,enter,release", "--walk-route", route.to_str().unwrap(),
        "--key", "popup:help=128,0", "--key", "popup:help=139,0", "--key", "popup:help=132,0",
        "--key", "popup:help=145,0",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchYard:enter:release:escape,860",
        "--key", "menu:up:enter:release:left:enter:release,875", "--frames", "5000",
    ]);
    assert!(viewer.contains("walk-route landed on update") && viewer.contains("walk-route at Reached the church rope trap"));
    let logs = play.root.join("walk/kq8-artifacts/logs");
    let traps = fs::read_to_string(logs.join("traps.log")).unwrap();
    assert_eq!(traps.matches("trap churchr1: On Connor").count(), 1);
    let gui = fs::read_to_string(logs.join("gui.log")).unwrap();
    assert!(gui.contains("hook icon HookUp") && gui.contains("acknowledged case 145"));
    let saved = play.slot(2);
    assert_eq!(saved["label"], "ChurchYard");
    check_mill_rope_save(&saved);
    assert_eq!(saved["rope_place"]["settings"]["status"], "Bottom");
    assert_eq!(saved["rope_place"]["settings"]["place"], serde_json::json!([206337.0,186373.0,8304.0]));
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_MILL_ROPE_SLOT"]
fn church_rope_is_usable_after_a_fresh_launch_inside_its_trap() {
    let play = Play::new("church-rope-place");
    church_yard(&play);
    let (viewer, items) = play.run("reload", &[
        "--menu-input", "down,enter,release,down,enter,release", "--key", "left:IDCTL_INVBAR_ROPE,15",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchHeld:enter:release:escape,45",
        "--key", "menu:up:enter:release:left:enter:release,60", "--frames", "800",
    ]);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    let logs = play.root.join("reload/kq8-artifacts/logs");
    assert!(!fs::read_to_string(logs.join("traps.log")).unwrap().contains("trap churchr1: On Connor"));
    assert!(fs::read_to_string(logs.join("gui.log")).unwrap().contains("hook icon HookUp"));
    assert!(items.contains("at the foot of a rope"));
    let saved = play.slot(3);
    assert_eq!(saved["label"], "ChurchHeld");
    assert_eq!(saved["journey"]["hand"], "NoName31");
    check_mill_rope_save(&saved);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_CHURCH_YARD_SLOT"]
fn church_climb_resumes_from_its_saved_frame_on_the_wall() {
    check_saved_church_climb(false);
}

fn check_saved_church_climb(first_person: bool) -> PathBuf {
    let play = Play::new(if first_person { "church-climb-first-person" } else { "church-climb" });
    fs::create_dir_all(&play.slots).unwrap();
    let yard = earned_slot("KQ8_CHURCH_YARD_SLOT").unwrap_or_else(|| {
        let yard = Play::new("church-climb-prefix");
        church_yard(&yard);
        yard.slots.join("slot-2.json")
    });
    fs::copy(yard, play.slots.join("slot-1.json")).unwrap();
    let start = play.slot(1);
    assert_eq!(start["label"], "ChurchYard");
    check_mill_rope_save(&start);
    let mut inputs = vec![
        "--menu-input", "down,enter,release,enter,release", "--key", "left:IDCTL_INVBAR_ROPE,8",
        "--aim-point", "206337,186373,11000,12", "--hold", "1,0,0,180,202",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=HookInFlight:enter:release:escape,73",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=MidClimb:enter:release:escape,210",
        "--key", "menu:up:enter:release:left:enter:release,230", "--frames", "1900",
    ];
    if first_person { inputs.extend(["--key", "physical:f2,4"]); }
    let (viewer, _) = play.run("original", &inputs);
    // The throw disables player controls. Its menu request must not create a slot.
    assert!(viewer.contains("gameplay menu request blocked by handsOff at Connor update 73"));
    let mid = play.slot(2);
    assert_eq!(mid["label"], "MidClimb");
    assert_eq!(mid["rope_climb"]["state"]["stage"], "Hanging");
    assert_eq!(mid["view_flags"]["active"], false);
    assert_eq!(mid["view_flags"]["rope_locked"], true);
    assert_eq!(mid["view_flags"]["resume_after_rope"], first_person);
    assert_eq!(mid["rope_climb"]["state"]["first_to_third"], first_person);
    let phase = mid["rope_climb"]["pose"]["phase"].as_f64().unwrap();
    assert!((0.20..0.40).contains(&phase));
    assert_eq!(mid["rope_climb"]["steps"], serde_json::json!([0.0,0.0]));
    assert_eq!(mid["trap_checked_at"], mid["position"]);
    check_mill_rope_save(&mid);
    let rope_log = |run: &str| fs::read_to_string(play.root.join(run).join("kq8-artifacts/logs/rope.log")).unwrap();
    let before = rope_log("original");
    assert_eq!(before.matches("the rope model ended: Connor grabs").count(), 1);

    play.run("from-wall", &[
        "--menu-input", "down,enter,release,down,enter,release", "--hold", "1,0,0,5,65",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchRoof:enter:release:escape,200",
        "--key", "menu:up:enter:release:left:enter:release,215", "--frames", "1700",
    ]);
    let finished = rope_log("from-wall");
    let traps = fs::read_to_string(play.root.join("from-wall/kq8-artifacts/logs/traps.log")).unwrap();
    assert!(!traps.contains("update 1: trap churchr2: On Connor"));
    assert_eq!(traps.matches("trap churchr2: On Connor").count(), 1);
    assert!(finished.contains("up (forwards), frame 23"));
    assert_eq!(finished.matches("climb: reached the top end").count(), 1);
    assert!(finished.contains("step heights back to up 325, down 325"));
    assert_eq!(finished.matches("rope state: over (rule 26)").count(), 1);
    assert!(!finished.contains("the rope model ended: Connor grabs"));
    let roof = play.slot(3);
    assert_eq!(roof["label"], "ChurchRoof");
    assert!(roof["rope_climb"].is_null());
    assert_eq!(roof["rope_camera_locked"], false);
    assert_eq!(roof["view_flags"]["active"], first_person);
    assert_eq!(roof["view_flags"]["resume_after_rope"], false);
    assert_eq!(roof["weapon_state"]["actor_first_person"], first_person);
    assert!(roof["position"][2].as_f64().unwrap() > start["position"][2].as_f64().unwrap());
    check_mill_rope_save(&roof);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
    play.slots.join("slot-3.json")
}
