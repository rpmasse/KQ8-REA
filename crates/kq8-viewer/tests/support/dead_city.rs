//! The skeleton archers on the road from the Dead City arrival to the first hall.
use super::{
    carries_one, earned_slot, hero_flag,
    shadow::{leg, log, prepare},
    wizard_ring::enter_dead_city,
    Play,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn check_first_hall(save: &Value) {
    assert_eq!(save["world"], "DeadCity");
    assert!(carries_one(save, "INVITEM_Paddle"));
    let removed = save["kept"]["deadcity"]["gone"].as_array().unwrap();
    for name in ["introtp", "skeleton14", "skeleton15", "dpaddle"] {
        assert!(
            removed
                .iter()
                .any(|v| v.as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))),
            "{name} was not removed"
        );
    }
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_DEAD_CITY_ARRIVAL_SLOT"]
fn entrance_archers_and_paddle() {
    earn_paddle();
}

fn earn_paddle() -> PathBuf {
    let play = Play::new("dead-city-first-hall");
    let source = earned_slot("KQ8_DEAD_CITY_ARRIVAL_SLOT").unwrap_or_else(enter_dead_city);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "DeadCityReload");
    assert_eq!(start["world"], "DeadCity");
    assert!(hero_flag(&start, "Daventry::BeenThere::DoneThat", "1"));
    assert!(carries_one(&start, "INVITEM_SmallAx") && carries_one(&start, "INVITEM_SmCrossbow"));
    let path = play.root.join("first-hall.json");
    let route = json!({"start":3,"landmarks":[
        {"name":"Enter the skeleton archers' approach","xy":[182262,53700],"radius":150,"scene":{"trigger":"introtp","controller":"introtp"}},
        {"name":"Fight the west skeleton archer","xy":[181530,64660],"radius":220,"run":true,"fight":"skeleton14","big_swing":true},
        {"name":"Fight the east skeleton archer","xy":[182680,65060],"radius":250,"run":true,"fight":"skeleton15","big_swing":true},
        {"name":"Go west of the first hall","xy":[177950,67400],"radius":240,"run":true},
        {"name":"Approach the paddle rack","xy":[173750,74689],"radius":145,"run":false,"aim":"dpaddle"}
    ]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (saved, viewer) = leg(
        &play,
        1,
        "FirstHallPaddle",
        1450,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route scene complete at Enter the skeleton archers' approach"));
    assert!(viewer.lines().any(|line| line
        .contains("test --aim dpaddle (camera through gameplay pixel")
        && line.contains("hits dpaddle first")));
    assert!(log(&play, "FirstHallPaddle", "doors.log")
        .contains("TalkComplete 2000 64 0 86 2 to introtp"));
    let fight = log(&play, "FirstHallPaddle", "fight.log");
    for enemy in ["skeleton14", "skeleton15"] {
        assert!(fight.contains(&format!("{enemy}: hit by Connor")));
        assert_eq!(
            fight
                .matches(&format!("{enemy}: its death ended (rule 32)"))
                .count(),
            1
        );
    }
    let items = log(&play, "FirstHallPaddle", "items.log");
    assert_eq!(items.matches("INVITEM_Paddle 0 -> 1").count(), 1);
    check_first_hall(&saved);
    let (again, _) = leg(&play, 2, "FirstHallReload", 55, &[]);
    check_first_hall(&again);
    assert_eq!(saved["journey"]["stats"], again["journey"]["stats"]);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

fn region_flag(save: &Value, name: &str, value: &str) -> bool {
    save["kept"]["deadcity"]["vars"]
        .as_array()
        .unwrap()
        .iter()
        .any(|pair| pair[0] == name && pair[1] == value)
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_DEAD_CITY_PADDLE_SLOT"]
fn four_paddle_pedestals_open_the_death_hall() {
    open_death_hall();
}

fn open_death_hall() -> PathBuf {
    let play = Play::new("dead-city-pedestals");
    let source = earned_slot("KQ8_DEAD_CITY_PADDLE_SLOT").unwrap_or_else(earn_paddle);
    let first = prepare(&play, &source);
    assert_eq!(first["label"], "FirstHallReload");
    check_first_hall(&first);
    let steps = [
        (
            "Scary3",
            "FirstPedestal",
            [173620, 65850],
            [175015, 65850],
            None,
            "pushccw",
            "1",
            620,
        ),
        (
            "Scary2",
            "SecondPedestal",
            [176400, 57860],
            [175015, 57860],
            None,
            "pushcw",
            "2",
            620,
        ),
        (
            "Scary4",
            "ThirdPedestal",
            [187920, 67100],
            [189395, 67100],
            Some(([186180, 67000], true)),
            "pushcw",
            "3",
            720,
        ),
        (
            "Scary5",
            "HallOpened",
            [190870, 59090],
            [189395, 59090],
            Some(([189120, 63360], false)),
            "pushoff",
            "3",
            1000,
        ),
    ];
    for (i, (name, label, at, push, approach, movement, count, tick)) in
        steps.into_iter().enumerate()
    {
        let mut points = Vec::new();
        if let Some((xy, run)) = approach {
            points.push(json!({"name":"Round the pedestal","xy":xy,"radius":145,"run":run}));
        }
        points.push(json!({"name":format!("Place the paddle on {name}"),"xy":at,"radius":145,"run":false,"aim":name}));
        points.push(
            json!({"name":format!("Push the paddle at {name}"),"xy":push,"radius":80,"run":false,
            "until_var":{"name":"DeadCity::Paddle::Active","value":name}}),
        );
        if i == 0 {
            points.push(json!({"name":"Defeat the skeleton approaching the pedestal","xy":[175440,65904],"radius":330,"fight":"skeleton20","big_swing":true}));
        }
        let path = play.root.join(format!("{name}.json"));
        fs::write(
            &path,
            serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
        )
        .unwrap();
        let (saved, viewer) = leg(
            &play,
            i as u32 + 1,
            label,
            tick,
            &[
                "--key",
                "left:IDCTL_INVBAR_QUEST2,8",
                "--walk-route",
                path.to_str().unwrap(),
            ],
        );
        assert!(viewer.lines().any(|line| line
            .contains(&format!("test --aim {name} (camera through gameplay pixel"))
            && line.contains(&format!("hits {name} first"))));
        assert!(region_flag(&saved, "DeadCity::Paddle::Active", name));
        assert!(region_flag(&saved, "DeadCity::Paddle::Used", count));
        let doors = log(&play, label, "doors.log");
        if i < 3 {
            assert_eq!(
                doors
                    .matches(&format!("movement '{movement}' ended: tells"))
                    .count(),
                1
            );
            assert!(carries_one(&saved, "INVITEM_Paddle"));
        } else {
            assert!(region_flag(&saved, "DeadCity::Paddle::Activate", name));
            assert!(doors.contains("laserctr.cs line 69: handsOff 0"));
            assert_eq!(
                saved["kept"]["deadcity"]["interactions"]["doors"]["ltddoor2"]["state"],
                "Opened"
            );
            assert_eq!(
                saved["kept"]["deadcity"]["interactions"]["doors"]["rtddoor2"]["state"],
                "Opened"
            );
        }
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-5.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_DEATH_HALL_SLOT"]
fn opened_doors_to_tile_floor() {
    reach_tiles();
}

fn reach_tiles() -> PathBuf {
    let play = Play::new("earned-tile-floor");
    let source = earned_slot("KQ8_DEATH_HALL_SLOT").unwrap_or_else(open_death_hall);
    let hall = prepare(&play, &source);
    assert_eq!(hall["label"], "HallOpened");
    assert_eq!(
        hall["kept"]["deadcity"]["interactions"]["doors"]["ltddoor2"]["state"],
        "Opened"
    );
    let path = play.root.join("enter-hall.json");
    let route = json!({"start":3,"landmarks":[
        {"name":"Approach the unlocked hall doors","xy":[182290,70000],"radius":155,"run":false},
        {"name":"Enter the hall of death","xy":[182290,74400],"radius":120,
            "until_var":{"name":"DeadCity::DeadHall::TrapOn","value":"false"}}
    ]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (entry, _) = leg(
        &play,
        1,
        "TileEntry",
        750,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(
        log(&play, "TileEntry", "connor.log").contains("enters the Tile state: 64 of 64 stones")
    );
    assert_eq!(entry["tile"]["stage"], "Keys");
    assert_eq!(entry["tile"]["expected"], 0);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-2.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_TILE_ENTRY_SLOT"]
fn saved_tile_jump_finishes_once_and_returns_to_the_strip() {
    let play = Play::new("earned-tile-jump");
    let source = earned_slot("KQ8_TILE_ENTRY_SLOT").unwrap_or_else(reach_tiles);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "TileEntry");
    assert_eq!(start["tile"]["stage"], "Keys");
    let (air, _) = leg(&play, 1, "TileJump", 31, &["--hold", "1,0,0,15,18"]);
    assert_eq!(
        air["tile"]["stage"]["Jumping"]["to"],
        json!({"row":0,"col":4})
    );
    assert_eq!(air["special"]["movement"], "tilejump1");
    let phase = air["special"]["phase"].as_f64().unwrap();
    assert!(phase > 0. && phase < 1.);
    let (ground, _) = leg(&play, 2, "TileLanding", 165, &[]);
    let log = log(&play, "TileLanding", "connor.log");
    assert!(
        log.contains("restored Tile: 64 of 64 stones rebuilt; completed 0 letters; stage Jumping")
    );
    assert_eq!(log.matches("expected D tile07: WRONG").count(), 1);
    assert_eq!(log.matches("Tile: teleportup ended:").count(), 1);
    assert_eq!(ground["tile"]["stage"], "Keys");
    assert_eq!(ground["tile"]["expected"], 0);
    assert_eq!(ground["tile"]["lines"]["teleported"], true);
    assert_eq!(ground["journey"]["stats"], air["journey"]["stats"]);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets and graphics; optional pre-checkpoint KQ8_LEGACY_TILE_ENTRY_SLOT"]
fn old_tile_entrance_recovers_the_original_first_jump() {
    let play = Play::new("legacy-tile-entrance");
    let old = earned_slot("KQ8_LEGACY_TILE_ENTRY_SLOT");
    let source = old
        .clone()
        .unwrap_or_else(|| earned_slot("KQ8_TILE_ENTRY_SLOT").unwrap_or_else(reach_tiles));
    let mut entrance = prepare(&play, &source);
    assert_eq!(entrance["label"], "TileEntry");
    if old.is_none() {
        // Isolate the prior on-disk representation without changing the
        // player's position, stats, camera, scripts or installed files.
        assert_eq!(entrance["tile"]["expected"], 0);
        entrance.as_object_mut().unwrap().remove("tile");
        fs::write(
            play.slots.join("slot-1.json"),
            serde_json::to_vec(&entrance).unwrap(),
        )
        .unwrap();
    }
    assert!(entrance.get("tile").is_none());
    let (saved, _) = leg(&play, 1, "RecoveredTile", 165, &["--hold", "1,0,0,15,18"]);
    let connor = log(&play, "RecoveredTile", "connor.log");
    assert!(connor.contains("restored Tile from the original hall entrance in an older save"));
    assert_eq!(connor.matches("expected D tile07: WRONG").count(), 1);
    assert_eq!(saved["tile"]["expected"], 0);
    assert_eq!(saved["tile"]["stage"], "Keys");
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

fn tile_route(play: &Play, file: &str, cells: &[(i32, i32)]) -> PathBuf {
    use kq8_core::tile_puzzle::{ORIGIN, SPACING};
    let points: Vec<_> = cells
        .iter()
        .map(|&(row, col)| {
            json!({
                "name":format!("Jump to row {row} column {col}"),
                "xy":[ORIGIN[0]+col as f32*SPACING,ORIGIN[1]+row as f32*SPACING],
                "radius":125,"tile_jump":true
            })
        })
        .collect();
    let path = play.root.join(file);
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    path
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_TILE_ENTRY_SLOT"]
fn tile_teleport_resumes_its_flight_and_speaks_once() {
    let play = Play::new("earned-tile-teleport");
    let source = earned_slot("KQ8_TILE_ENTRY_SLOT").unwrap_or_else(reach_tiles);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "TileEntry");
    let path = tile_route(&play, "d.json", &[(-1, 5), (-1, 6), (0, 7)]);
    let (down, _) = leg(
        &play,
        1,
        "TileDescending",
        132,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert_eq!(down["tile"]["stage"]["Down"]["right"], true);
    assert_eq!(down["tile"]["expected"], 1);
    assert_eq!(down["special"]["movement"], "teleportdown");
    let (up, _) = leg(&play, 2, "TileRising", 19, &[]);
    assert_eq!(up["tile"]["stage"]["Up"]["right"], true);
    assert_eq!(up["special"]["movement"], "teleportup");
    assert!(up["camera"]["flight"].is_object());
    let (idle, _) = leg(&play, 3, "TileTeleportReload", 155, &[]);
    assert_eq!(idle["tile"]["stage"], "Keys");
    assert_eq!(idle["tile"]["expected"], 1);
    assert_eq!(idle["tile"]["lines"]["had_right"], true);
    assert_eq!(idle["camera"]["cam"]["hold"], true);
    assert_eq!(
        log(&play, "TileRising", "connor.log")
            .matches("Tile: teleportdown ended: put at")
            .count(),
        1
    );
    let final_log = log(&play, "TileTeleportReload", "connor.log");
    assert_eq!(
        final_log
            .matches("Tile: teleportup ended: says 2000/2/0/3")
            .count(),
        1
    );
    assert!(!final_log.contains("Tile: teleportdown ended:"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_TILE_ENTRY_SLOT"]
fn restored_letter_sequence_earns_deaths_key() {
    earn_maze_key();
}

fn earn_maze_key() -> PathBuf {
    let play = Play::new("earned-death-maze-key");
    let source = earned_slot("KQ8_TILE_ENTRY_SLOT").unwrap_or_else(reach_tiles);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "TileEntry");
    let path = tile_route(&play, "d.json", &[(-1, 5), (-1, 6), (0, 7)]);
    let (letter, _) = leg(
        &play,
        1,
        "TileD",
        400,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert_eq!(letter["tile"]["expected"], 1);
    assert_eq!(letter["tile"]["stage"], "Keys");
    assert_eq!(letter["tile"]["lines"]["had_right"], true);
    assert_eq!(
        log(&play, "TileD", "connor.log")
            .matches("expected D tile07: RIGHT")
            .count(),
        1
    );
    let path = tile_route(&play, "eath.json", &[(6, 3), (2, 1), (5, 6), (1, 0)]);
    let (key, _) = leg(
        &play,
        2,
        "DeathMazeKey",
        2650,
        &["--walk-route", path.to_str().unwrap()],
    );
    let connor = log(&play, "DeathMazeKey", "connor.log");
    assert!(
        connor.contains("restored Tile: 64 of 64 stones rebuilt; completed 1 letters; stage Keys")
    );
    for right in [
        "expected E tile63: RIGHT",
        "expected A tile21: RIGHT",
        "expected T tile56: RIGHT",
        "expected H tile10: RIGHT",
    ] {
        assert_eq!(connor.matches(right).count(), 1);
    }
    assert!(connor.contains("SOLVED: leaves the Tile state"));
    assert!(key["tile"].is_null() && carries_one(&key, "INVITEM_KeyToDeathMaze"));
    let (restored, _) = leg(&play, 3, "MazeKeyReload", 55, &[]);
    assert!(restored["tile"].is_null() && carries_one(&restored, "INVITEM_KeyToDeathMaze"));
    assert_eq!(key["journey"]["stats"], restored["journey"]["stats"]);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-4.json")
}

fn gate_approach(play: &Play) -> PathBuf {
    let mut points = Vec::new();
    for xy in [
        [177500, 70250],
        [177500, 84000],
        [178750, 88250],
        [180500, 90000],
    ] {
        points.push(json!({"name":format!("Approach the gate {}",points.len()+1),"xy":xy,"radius":235,"run":true}));
    }
    points.push(json!({"name":"Use Death's key","xy":[181750,100700],"radius":150,"run":true,"aim":"spmove"}));
    let path = play.root.join("unlock.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    path
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_DEATH_KEY_SLOT"]
fn death_key_opens_the_spike_gate() {
    let play = Play::new("earned-spike-gate");
    let source = earned_slot("KQ8_DEATH_KEY_SLOT").unwrap_or_else(earn_maze_key);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "MazeKeyReload");
    assert!(carries_one(&start, "INVITEM_KeyToDeathMaze"));
    let item = start["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| {
            item["kind"] == "INVITEM_KeyToDeathMaze" && item["place"]["Carried"] == "Connor"
        })
        .unwrap();
    let key_name = item["name"].as_str().unwrap();
    let path = gate_approach(&play);
    let (opened, viewer) = leg(
        &play,
        1,
        "MazeGateOpen",
        397,
        &[
            "--key",
            "left:IDCTL_INVBAR_QUEST2,8",
            "--walk-route",
            path.to_str().unwrap(),
        ],
    );
    assert!(viewer.lines().any(
        |l| l.contains("test --aim spmove (camera through gameplay pixel")
            && l.contains("hits spmove first")
    ));
    let doors = log(&play, "MazeGateOpen", "doors.log");
    assert_eq!(
        doors
            .matches(&format!(
                "spmove.cs line 20: deleteObject {key_name}: item gone"
            ))
            .count(),
        1
    );
    assert_eq!(
        doors
            .matches("door spmove: movement 'open' finished; state opened")
            .count(),
        1
    );
    assert!(doors.contains("spmove.cs line 22: addMapIcon dc_gate1.bmp"));
    assert_eq!(
        opened["kept"]["deadcity"]["interactions"]["doors"]["spmove"]["state"],
        "Opened"
    );
    let path = play.root.join("cross.json");
    let route = json!({"start":3,"landmarks":[{"name":"Cross the lowered gate","xy":[181750,104750],"radius":240,"run":true}]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (crossed, viewer) = leg(
        &play,
        2,
        "AcrossMazeGate",
        54,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Cross the lowered gate"));
    assert!(crossed["position"][1].as_f64().unwrap() > 104000.0);
    assert_eq!(
        crossed["kept"]["deadcity"]["interactions"]["doors"]["spmove"]["state"],
        "Opened"
    );
    assert!(!crossed["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "INVITEM_KeyToDeathMaze"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

/// Identify the earned drop by kind and hall, independent of its generated name.
fn hall_drop<'a>(save: &'a Value, kind: &str) -> &'a Value {
    let mut drops = save["items"].as_array().unwrap().iter().filter(|item| {
        let pos = &item["place"]["World"]["position"];
        item["kind"] == kind
            && pos[0]
                .as_f64()
                .is_some_and(|x| (170000.0..190000.0).contains(&x))
            && pos[1]
                .as_f64()
                .is_some_and(|y| (58000.0..75000.0).contains(&y))
    });
    let drop = drops
        .next()
        .unwrap_or_else(|| panic!("missing {kind} in the first hall"));
    assert!(drops.next().is_none(), "ambiguous {kind} in the first hall");
    drop
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_DEATH_KEY_SLOT"]
fn hall_combat_drops_replace_the_axe_and_reload() {
    earn_hall_equipment();
}

fn earn_hall_equipment() -> PathBuf {
    let play = Play::new("earned-hall-equipment");
    let source = earned_slot("KQ8_DEATH_KEY_SLOT").unwrap_or_else(earn_maze_key);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "MazeKeyReload");
    assert!(carries_one(&start, "INVITEM_SmallAx"));
    let axe = start["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "INVITEM_SmallAx" && item["place"]["Carried"] == "Connor")
        .unwrap();
    let crystal = hall_drop(&start, "INVITEM_Crystal");
    let sword = hall_drop(&start, "INVITEM_BroadSword");
    let mut points = Vec::new();
    for (drop, dx, dy, radius) in [(crystal, 430.0, -320.0, 160), (sword, 490.0, -300.0, 150)] {
        let p = &drop["place"]["World"]["position"];
        points.push(
            json!({"name":format!("Pick up {}",drop["kind"]),"aim":drop["name"],
            "xy":[p[0].as_f64().unwrap()+dx,p[1].as_f64().unwrap()+dy],"radius":radius}),
        );
    }
    let path = play.root.join("pickups.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (taken, viewer) = leg(
        &play,
        1,
        "HallEquipment",
        170,
        &["--walk-route", path.to_str().unwrap()],
    );
    for item in [crystal, sword] {
        let name = item["name"].as_str().unwrap();
        assert!(viewer.lines().any(|line| line
            .contains(&format!("test --aim {name} (camera through gameplay pixel"))
            && line.contains(&format!("hits {name} first"))));
    }
    let (restored, _) = leg(&play, 2, "HallEquipmentReload", 45, &[]);
    let mut expected = start["journey"]["stats"].clone();
    let hand = expected["stats"][1].as_i64().unwrap();
    expected["stats"][1] = json!(hand + 2);
    for save in [&taken, &restored] {
        assert_eq!(save["journey"]["stats"], expected);
        assert!(carries_one(save, "INVITEM_KeyToDeathMaze"));
        for drop in [crystal, sword] {
            assert!(carries_one(save, drop["kind"].as_str().unwrap()));
            assert!(save["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["name"] == drop["name"] && i["place"]["Carried"] == "Connor"));
        }
        let exchanged = hall_drop(save, "INVITEM_SmallAx");
        assert_eq!(exchanged["name"], axe["name"]);
        assert_eq!(
            exchanged["place"]["World"]["position"],
            sword["place"]["World"]["position"]
        );
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-2.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_HALL_EQUIPMENT_SLOT"]
fn equipped_connor_defeats_the_three_maze_guards() {
    earn_gate_victory();
}

fn earn_gate_victory() -> PathBuf {
    use kq8_core::keyboard_menu::{keyword, Bindings};
    use kq8_data::GameDir;
    let play = Play::new("earned-maze-guards");
    let source = earned_slot("KQ8_HALL_EQUIPMENT_SLOT").unwrap_or_else(earn_hall_equipment);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "HallEquipment");
    for kind in [
        "INVITEM_KeyToDeathMaze",
        "INVITEM_Crystal",
        "INVITEM_BroadSword",
    ] {
        assert!(carries_one(&start, kind));
    }
    let path = gate_approach(&play);
    let bindings = GameDir::open(&play.game)
        .unwrap()
        .find("movekeys.kq")
        .unwrap();
    let bindings = Bindings::read(&fs::read(bindings).unwrap()).unwrap();
    let draw = format!("physical:{},348", keyword(bindings.0[6]).unwrap());
    let (armed, _) = leg(
        &play,
        1,
        "ArmedMazeGate",
        390,
        &[
            "--key",
            "left:IDCTL_INVBAR_QUEST2,8",
            "--key",
            &draw,
            "--walk-route",
            path.to_str().unwrap(),
        ],
    );
    assert_eq!(armed["weapon_state"]["set_mode"], "Hand");
    assert_eq!(armed["weapon_state"]["ready_item"], "INVITEM_BroadSword");
    assert_eq!(
        armed["kept"]["deadcity"]["interactions"]["doors"]["spmove"]["state"],
        "Opened"
    );
    let heal = json!({"slot":2,"at_health":46});
    let path = play.root.join("guards.json");
    let points = json!([
        {"name":"Fight the east guard","xy":[181970,103520],"radius":235,"run":true,
            "fight":"skeleton5","keep_weapon":true,"heal":heal},
        {"name":"Fight the west guard","xy":[181970,103520],"radius":435,
            "fight":"skeleton4","keep_weapon":true,"heal":heal},
        {"name":"Fight the archer","xy":[182210,110250],"radius":250,"run":true,
            "fight":"skeleton6","heal":heal}
    ]);
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (cleared, _) = leg(
        &play,
        2,
        "MazeSentries",
        550,
        &["--walk-route", path.to_str().unwrap()],
    );
    let combat = log(&play, "MazeSentries", "fight.log");
    let items = log(&play, "MazeSentries", "items.log");
    assert_eq!(items.matches("INVITEM_Crystal, slot used").count(), 1);
    for name in ["skeleton5", "skeleton4", "skeleton6"] {
        assert!(combat.contains(&format!("{name}: hit by Connor")));
        assert_eq!(
            combat
                .matches(&format!("{name}: its death ended (rule 32)"))
                .count(),
            1
        );
    }
    let (restored, _) = leg(&play, 3, "MazeGuardsReload", 45, &[]);
    for save in [&cleared, &restored] {
        assert!(carries_one(save, "INVITEM_BroadSword"));
        assert!(!carries_one(save, "INVITEM_Crystal"));
        let gone = save["kept"]["deadcity"]["gone"].as_array().unwrap();
        for name in ["skeleton5", "skeleton4", "skeleton6"] {
            assert!(gone
                .iter()
                .any(|v| v.as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))));
        }
    }
    assert_eq!(restored["journey"]["stats"], cleared["journey"]["stats"]);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-4.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_MAZE_GUARDS_SLOT"]
fn gate_drops_and_northwest_passage_reload() {
    let play = Play::new("earned-north-maze-corner");
    let source = earned_slot("KQ8_MAZE_GUARDS_SLOT").unwrap_or_else(earn_gate_victory);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "MazeGuardsReload");
    let mut herbs: Vec<_> = start["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            let pos = &item["place"]["World"]["position"];
            item["kind"] == "INVITEM_Mushroom"
                && pos[0]
                    .as_f64()
                    .is_some_and(|x| (180000.0..186000.0).contains(&x))
                && pos[1]
                    .as_f64()
                    .is_some_and(|y| (100000.0..108000.0).contains(&y))
        })
        .collect();
    assert_eq!(herbs.len(), 2);
    herbs.sort_by(|a, b| {
        let y = |v: &Value| v["place"]["World"]["position"][1].as_f64().unwrap();
        y(b).total_cmp(&y(a))
    });
    let mut points = Vec::new();
    for item in &herbs {
        let p = &item["place"]["World"]["position"];
        points.push(json!({"name":format!("Collect {}",item["name"]),"aim":item["name"],
            "xy":[p[0].as_f64().unwrap()-180.0,p[1].as_f64().unwrap()+280.0],"radius":150,"run":true}));
    }
    for xy in [
        [182185, 110160],
        [178000, 110250],
        [168000, 112250],
        [165750, 112250],
        [162250, 116000],
    ] {
        points.push(json!({"name":format!("Northwest passage {}",points.len()),"xy":xy,"radius":180,"run":true}));
    }
    let path = play.root.join("corner.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (corner, viewer) = leg(
        &play,
        1,
        "NorthMazeCorner",
        480,
        &["--walk-route", path.to_str().unwrap()],
    );
    for item in &herbs {
        let name = item["name"].as_str().unwrap();
        assert!(viewer.lines().any(|l| l
            .contains(&format!("test --aim {name} (camera through gameplay pixel"))
            && l.contains(&format!("hits {name} first"))));
    }
    let (restored, _) = leg(&play, 2, "NorthCornerReload", 45, &[]);
    for save in [&corner, &restored] {
        assert!((161500.0..163000.0).contains(&save["position"][0].as_f64().unwrap()));
        assert!((115500.0..117000.0).contains(&save["position"][1].as_f64().unwrap()));
        assert_eq!(save["journey"]["stats"], start["journey"]["stats"]);
        assert!(save["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["kind"] == "INVITEM_Mushroom"
                && item["place"]["Carried"] == "Connor"
                && item["quantity"] == 2));
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}
