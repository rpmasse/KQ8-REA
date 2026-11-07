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

fn draw_key(play: &Play, tick: u32) -> String {
    use kq8_core::keyboard_menu::{keyword, Bindings};
    use kq8_data::GameDir;
    let path = GameDir::open(&play.game)
        .unwrap()
        .find("movekeys.kq")
        .unwrap();
    let bindings = Bindings::read(&fs::read(path).unwrap()).unwrap();
    format!("physical:{},{tick}", keyword(bindings.0[6]).unwrap())
}

fn earn_gate_victory() -> PathBuf {
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
    let draw = draw_key(&play, 348);
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
    reach_north_corner();
}

fn reach_north_corner() -> PathBuf {
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
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_NORTH_MAZE_CORNER_SLOT"]
fn northern_mace_guard_and_archers_reload() {
    earn_north_junction();
}

fn earn_north_junction() -> PathBuf {
    let play = Play::new("earned-north-maze-guards");
    let source = earned_slot("KQ8_NORTH_MAZE_CORNER_SLOT").unwrap_or_else(reach_north_corner);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "NorthCornerReload");
    assert!(carries_one(&start, "INVITEM_BroadSword"));
    let draw = draw_key(&play, 8);
    let path = play.root.join("junction.json");
    let points = json!([
        {"name":"Fight the mace guard","xy":[162250,122500],"radius":400,"run":true,
            "fight":"skeleton29","keep_weapon":true},
        {"name":"Fight the east junction archer","xy":[163480,126700],"radius":400,"run":true,
            "fight":"skeleton28","keep_weapon":true},
        {"name":"Fight the west junction archer","xy":[162240,126800],"radius":400,"run":true,
            "fight":"skeleton27"}
    ]);
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (cleared, _) = leg(
        &play,
        1,
        "NorthMazeGuards",
        760,
        &["--key", &draw, "--walk-route", path.to_str().unwrap()],
    );
    let combat = log(&play, "NorthMazeGuards", "fight.log");
    for name in ["skeleton29", "skeleton28", "skeleton27"] {
        assert!(combat.contains(&format!("{name}: hit by Connor")));
        assert_eq!(
            combat
                .matches(&format!("{name}: its death ended (rule 32)"))
                .count(),
            1
        );
    }
    assert_eq!(
        log(&play, "NorthMazeGuards", "items.log")
            .matches("INVITEM_Mushroom, slot used")
            .count(),
        2
    );
    assert_eq!(
        cleared["journey"]["stats"]["level"],
        start["journey"]["stats"]["level"]
    );
    assert_eq!(
        cleared["journey"]["stats"]["stats"][4].as_i64().unwrap(),
        start["journey"]["stats"]["stats"][4].as_i64().unwrap() + 650
    );
    let (restored, _) = leg(&play, 2, "NorthJunctionReload", 45, &[]);
    assert_eq!(restored["journey"]["stats"], cleared["journey"]["stats"]);
    for name in ["skeleton29", "skeleton28", "skeleton27"] {
        assert!(restored["kept"]["deadcity"]["gone"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == name));
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_NORTH_JUNCTION_SLOT"]
fn northern_drops_equip_mace_and_heal_before_the_cell_gate() {
    earn_north_supplies();
}

fn earn_north_supplies() -> PathBuf {
    let play = Play::new("earned-north-road-supplies");
    let source = earned_slot("KQ8_NORTH_JUNCTION_SLOT").unwrap_or_else(earn_north_junction);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "NorthJunctionReload");
    let drops: Vec<_> = start["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            let pos = &item["place"]["World"]["position"];
            matches!(
                item["kind"].as_str(),
                Some("INVITEM_Mace" | "INVITEM_Mushroom")
            ) && pos[0]
                .as_f64()
                .is_some_and(|x| (160000.0..165000.0).contains(&x))
                && pos[1]
                    .as_f64()
                    .is_some_and(|y| (122000.0..131000.0).contains(&y))
        })
        .collect();
    let mace = drops
        .iter()
        .find(|item| item["kind"] == "INVITEM_Mace")
        .unwrap();
    let mut herbs: Vec<_> = drops
        .iter()
        .filter(|item| item["kind"] == "INVITEM_Mushroom")
        .collect();
    assert_eq!(drops.len(), 4);
    assert_eq!(herbs.len(), 3);
    herbs.sort_by(|a, b| {
        let y = |v: &Value| v["place"]["World"]["position"][1].as_f64().unwrap();
        y(a).total_cmp(&y(b))
    });
    let mut points = Vec::new();
    for item in std::iter::once(mace).chain(herbs.into_iter()) {
        let p = &item["place"]["World"]["position"];
        points.push(json!({"name":format!("Take {}",item["name"]),"aim":item["name"],
            "xy":[p[0].as_f64().unwrap()+360.0,p[1].as_f64().unwrap()-250.0],"radius":400,"run":true}));
    }
    for y in [132000, 134000, 136000] {
        points.push(
            json!({"name":format!("Use a Mushroom at {y}"),"xy":[162250,y],
            "radius":350,"run":true,"inventory_slot":1}),
        );
    }
    points.push(json!({"name":"North road before the cell gates","xy":[162000,140750],"radius":300,"run":true}));
    let path = play.root.join("supplies.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let draw = draw_key(&play, 210);
    let (supplied, viewer) = leg(
        &play,
        1,
        "NorthRoadSupplies",
        300,
        &["--key", &draw, "--walk-route", path.to_str().unwrap()],
    );
    for item in &drops {
        let name = item["name"].as_str().unwrap();
        assert!(viewer.lines().any(|l| l
            .contains(&format!("test --aim {name} (camera through gameplay pixel"))
            && l.contains(&format!("hits {name} first"))));
    }
    assert_eq!(
        log(&play, "NorthRoadSupplies", "items.log")
            .matches("INVITEM_Mushroom, slot used")
            .count(),
        3
    );
    let (restored, _) = leg(&play, 2, "NorthSuppliesReload", 45, &[]);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = json!(expected["stats"][0].as_i64().unwrap() + 15);
    expected["stats"][1] = json!(expected["stats"][1].as_i64().unwrap() + 3);
    for save in [&supplied, &restored] {
        assert_eq!(save["journey"]["stats"], expected);
        assert!(carries_one(save, "INVITEM_Mace"));
        assert!(!carries_one(save, "INVITEM_Mushroom"));
        assert_eq!(save["weapon_state"]["ready_item"], "INVITEM_Mace");
        assert_eq!(
            save["kept"]["deadcity"]["interactions"]["doors"]["cell04"]["state"],
            "Closed"
        );
        assert!(save["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["kind"] == "INVITEM_BroadSword"
                && item["place"]["World"]["position"] == mace["place"]["World"]["position"]));
        assert!((161600.0..162800.0).contains(&save["position"][0].as_f64().unwrap()));
        assert!((140000.0..143000.0).contains(&save["position"][1].as_f64().unwrap()));
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_NORTH_SUPPLIES_SLOT"]
fn outer_mace_guard_retreats_after_running_and_reloads() {
    earn_outer_guard();
}

fn earn_outer_guard() -> PathBuf {
    let play = Play::new("earned-outer-mace-guard");
    let source = earned_slot("KQ8_NORTH_SUPPLIES_SLOT").unwrap_or_else(earn_north_supplies);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "NorthSuppliesReload");
    assert!(carries_one(&start, "INVITEM_Mace"));
    let path = play.root.join("patrol.json");
    let route = json!({"start":3,"landmarks":[
        {"name":"East side of the cells","xy":[162250,149500],"radius":250,"run":true},
        {"name":"North side of the cells","xy":[159750,152000],"radius":250,"run":true},
        {"name":"Counterattack the outer mace guard","xy":[158600,152100],"radius":500,"run":true,
            "fight":"skeleton56","retreat":true}
    ]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (won, _) = leg(
        &play,
        1,
        "OuterMaceGuard",
        760,
        &["--walk-route", path.to_str().unwrap()],
    );
    let combat = log(&play, "OuterMaceGuard", "fight.log");
    assert!(combat.contains("skeleton56 missing Connor, too far away"));
    assert!(combat.contains("skeleton56: hit by Connor"));
    assert_eq!(
        combat
            .matches("skeleton56: its death ended (rule 32)")
            .count(),
        1
    );
    assert_eq!(
        won["journey"]["stats"]["level"],
        start["journey"]["stats"]["level"]
    );
    assert_eq!(
        won["journey"]["stats"]["stats"][4].as_i64().unwrap(),
        start["journey"]["stats"]["stats"][4].as_i64().unwrap() + 450
    );
    let (restored, _) = leg(&play, 2, "OuterGuardReload", 45, &[]);
    assert_eq!(restored["journey"]["stats"], won["journey"]["stats"]);
    assert!(restored["kept"]["deadcity"]["gone"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "skeleton56"));
    assert!(carries_one(&restored, "INVITEM_Mace"));
    assert_eq!(
        restored["kept"]["deadcity"]["interactions"]["doors"]["cell04"]["state"],
        "Closed"
    );
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_OUTER_GUARD_SLOT"]
fn first_hall_supplies_heal_the_return_to_the_outer_corridor() {
    earn_corridor_supplies();
}

fn earn_corridor_supplies() -> PathBuf {
    let play = Play::new("earned-corridor-supplies");
    let source = earned_slot("KQ8_OUTER_GUARD_SLOT").unwrap_or_else(earn_outer_guard);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "OuterGuardReload");
    let mut herbs: Vec<_> = start["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            item["kind"] == "INVITEM_Mushroom"
                && item["place"]["World"]["position"][1]
                    .as_f64()
                    .is_some_and(|y| y < 80000.0)
        })
        .collect();
    assert_eq!(herbs.len(), 4);
    herbs.sort_by(|a, b| {
        a["place"]["World"]["position"][0]
            .as_f64()
            .unwrap()
            .total_cmp(&b["place"]["World"]["position"][0].as_f64().unwrap())
    });
    let mut points: Vec<Value> = [
        [159750, 152000],
        [162250, 149500],
        [161885, 144000],
        [161885, 140750],
        [162250, 129750],
        [162250, 116000],
        [165750, 112250],
        [168750, 112250],
        [178000, 108000],
        [180000, 107250],
        [181750, 104750],
        [181750, 101500],
        [180500, 90000],
        [178750, 88250],
        [177500, 84000],
        [177500, 70250],
    ]
    .into_iter()
    .enumerate()
    .map(|(i, xy)| {
        let narrow = i == 2 || i == 3;
        json!({"name":format!("Return to the first hall {}",i+1),"xy":xy,
            "radius":if narrow {90} else {280},"run":!narrow})
    })
    .collect();
    for herb in &herbs {
        let p = &herb["place"]["World"]["position"];
        points.push(
            json!({"name":format!("Take first hall Mushroom {}",herb["name"]),
            "xy":[p[0].as_f64().unwrap()+360.0,p[1].as_f64().unwrap()-250.0],
            "radius":300,"run":true,"aim":herb["name"]}),
        );
    }
    let pickup_path = play.root.join("hall-supplies.json");
    fs::write(
        &pickup_path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (picked, viewer) = leg(
        &play,
        1,
        "HallHerbs",
        1120,
        &["--walk-route", pickup_path.to_str().unwrap()],
    );
    assert_eq!(picked["journey"]["stats"], start["journey"]["stats"]);
    assert!(picked["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["kind"] == "INVITEM_Mushroom"
            && item["quantity"] == 4
            && item["place"]["Carried"] == "Connor"));
    for herb in &herbs {
        let name = herb["name"].as_str().unwrap();
        assert!(viewer.lines().any(|line| line
            .contains(&format!("test --aim {name} (camera through gameplay pixel"))
            && line.contains(&format!("hits {name} first"))));
    }
    let points: Vec<Value> = [
        [177500, 70250],
        [177500, 84000],
        [178750, 88250],
        [180500, 90000],
        [181750, 101500],
        [181750, 104750],
        [180000, 107250],
        [178000, 108000],
        [168750, 112250],
        [165750, 112250],
        [162250, 116000],
        [162250, 129750],
        [161885, 140750],
        [161885, 144000],
        [162250, 149500],
        [159750, 152000],
    ]
    .into_iter()
    .enumerate()
    .map(|(i, xy)| {
        let narrow = i == 12 || i == 13;
        let mut point = json!({"name":format!("Return with supplies {}",i+1),"xy":xy,
            "radius":if narrow {90} else {280},"run":!narrow});
        if i < 4 {
            point["inventory_slot"] = json!(1);
        }
        point
    })
    .collect();
    let return_path = play.root.join("supplied-return.json");
    fs::write(
        &return_path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (healed, viewer) = leg(
        &play,
        2,
        "RestockedCorridor",
        1040,
        &["--walk-route", return_path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Return with supplies 16"));
    assert_eq!(
        log(&play, "RestockedCorridor", "items.log")
            .matches("INVITEM_Mushroom, slot used")
            .count(),
        4
    );
    let (restored, _) = leg(&play, 3, "CorridorSuppliesReload", 45, &[]);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = json!(expected["stats"][0].as_i64().unwrap() + 20);
    for saved in [&healed, &restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert!(
            !saved["journey"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["kind"] == "INVITEM_Mushroom"
                    && item["place"]["Carried"] == "Connor")
        );
        assert!(carries_one(saved, "INVITEM_Mace"));
        assert!(carries_one(saved, "INVITEM_SmCrossbow"));
        assert!((159000.0..160500.0).contains(&saved["position"][0].as_f64().unwrap()));
        assert!((151500.0..152500.0).contains(&saved["position"][1].as_f64().unwrap()));
        assert_eq!(
            saved["kept"]["deadcity"]["interactions"]["doors"]["cell04"]["state"],
            "Closed"
        );
        for herb in &herbs {
            assert!(!saved["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["name"] == herb["name"] && item["place"].get("World").is_some()));
        }
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-4.json")
}

#[test]
#[ignore = "known failing combat route; requires GOG assets and graphics; optional earned KQ8_CORRIDOR_SUPPLIES_SLOT"]
fn supplied_western_patrol_victory_and_award_reload() {
    earn_western_patrol();
}

fn earn_western_patrol() -> PathBuf {
    let play = Play::new("earned-western-patrol");
    let source = earned_slot("KQ8_CORRIDOR_SUPPLIES_SLOT").unwrap_or_else(earn_corridor_supplies);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "CorridorSuppliesReload");
    assert!(carries_one(&start, "INVITEM_Mace"));
    assert!(carries_one(&start, "INVITEM_SmCrossbow"));
    let path = play.root.join("western-patrol.json");
    let route = json!({"start":3,"landmarks":[
        {"name":"Follow the northern corridor 1","xy":[146000,152000],"radius":300,"run":true},
        {"name":"Follow the northern corridor 2","xy":[132250,152000],"radius":300,"run":true},
        {"name":"Follow the northern corridor 3","xy":[118500,153250],"radius":300,"run":true},
        {"name":"Follow the northern corridor 4","xy":[104750,153250],"radius":300,"run":true},
        {"name":"Fight the western swordsman","xy":[101500,153250],"radius":500,"run":true,
            "fight":"skeleton55","big_swing":true}
    ]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (won, viewer) = leg(
        &play,
        1,
        "WesternPatrol",
        1050,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Fight the western swordsman"));
    let combat = log(&play, "WesternPatrol", "fight.log");
    assert!(combat.contains("skeleton55: hit by Connor"));
    assert!(combat.contains("conmace.anm/attack2"));
    assert_eq!(
        combat
            .matches("skeleton55: its death ended (rule 32)")
            .count(),
        1
    );
    let mut expected = start["journey"]["stats"].clone();
    // The damage rolls vary. Every other stat and the exact award must match.
    expected["stats"][0] = won["journey"]["stats"]["stats"][0].clone();
    expected["stats"][4] = json!(expected["stats"][4].as_i64().unwrap() + 500);
    assert_eq!(won["journey"]["stats"], expected);
    let (restored, _) = leg(&play, 2, "WesternPatrolReload", 45, &[]);
    assert_eq!(restored["journey"]["stats"], expected);
    assert!(!log(&play, "WesternPatrolReload", "fight.log")
        .contains("skeleton55: its death ended (rule 32)"));
    for saved in [&won, &restored] {
        let gone = saved["kept"]["deadcity"]["gone"].as_array().unwrap();
        assert!(gone.iter().any(|name| name == "skeleton55"));
        assert!(!gone.iter().any(|name| name == "weeper15"));
        assert!(carries_one(saved, "INVITEM_Mace"));
        assert!(carries_one(saved, "INVITEM_SmCrossbow"));
        assert_eq!(
            saved["kept"]["deadcity"]["interactions"]["doors"]["cell04"]["state"],
            "Closed"
        );
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_WESTERN_PATROL_SLOT victory"]
fn saved_western_patrol_victory_reloads_without_another_award() {
    let play = Play::new("western-patrol-victory-reload");
    let source = earned_slot("KQ8_WESTERN_PATROL_SLOT")
        .expect("set KQ8_WESTERN_PATROL_SLOT to an earned WesternPatrol save");
    let won = prepare(&play, &source);
    assert_eq!(won["label"], "WesternPatrol");
    assert_eq!(won["world"], "DeadCity");
    assert_eq!(won["journey"]["god_mode"], false);
    assert_eq!(won["journey"]["ai_off"], false);
    assert!(won["kept"]["deadcity"]["gone"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == "skeleton55"));
    let (restored, _) = leg(&play, 1, "WesternPatrolReload", 45, &[]);
    assert_eq!(restored["journey"]["stats"], won["journey"]["stats"]);
    assert_eq!(restored["journey"]["items"], won["journey"]["items"]);
    assert!(restored["kept"]["deadcity"]["gone"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == "skeleton55"));
    assert!(carries_one(&restored, "INVITEM_Mace"));
    assert!(carries_one(&restored, "INVITEM_SmCrossbow"));
    assert!(!log(&play, "WesternPatrolReload", "fight.log")
        .contains("skeleton55: its death ended (rule 32)"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_WESTERN_PATROL_RELOAD_SLOT"]
fn earned_crossbow_defeats_the_river_weeper_and_restores_its_award() {
    let play = Play::new("earned-river-weeper");
    let source = earned_slot("KQ8_WESTERN_PATROL_RELOAD_SLOT")
        .expect("set KQ8_WESTERN_PATROL_RELOAD_SLOT to an earned WesternPatrolReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "WesternPatrolReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["level"], 9);
    assert_eq!(start["journey"]["stats"]["stats"][4], 2317);
    assert!(carries_one(&start, "INVITEM_Mace"));
    assert!(carries_one(&start, "INVITEM_SmCrossbow"));
    let path = play.root.join("river-weeper.json");
    let route = json!({"start":25,"landmarks":[
        {"name":"Follow the cleared river approach 1","xy":[92000,153250],"radius":230,"run":true},
        {"name":"Follow the cleared river approach 2","xy":[88500,157000],"radius":230,"run":true},
        {"name":"Follow the cleared river approach 3","xy":[88250,158000],"radius":230,"run":true},
        {"name":"Follow the cleared river approach 4","xy":[82000,158750],"radius":230,"run":true},
        {"name":"Use the earned Small Crossbow against the river Weeper","xy":[74250,158750],
            "radius":180,"run":true,"fight":"weeper15","ranged":true}
    ]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (won, viewer) = leg(
        &play,
        1,
        "RiverWeeper",
        900,
        &[
            "--walk-route",
            path.to_str().unwrap(),
            "--key",
            "physical:F2,3",
        ],
    );
    assert!(viewer.contains("walk-route selects the ranged weapon slot for weeper15"));
    assert!(viewer.lines().any(|line| line
        .contains("test --aim weeper15 (camera through gameplay pixel")
        && line.contains("the aim pick meets character weeper15")));
    assert!(viewer.contains("walk-route at Use the earned Small Crossbow against the river Weeper"));
    let combat = log(&play, "RiverWeeper", "fight.log");
    assert!(combat.contains("weeper15: hit by Connor, Projectile"));
    assert_eq!(
        combat
            .matches("weeper15: its death ended (rule 32)")
            .count(),
        1
    );
    assert!(!combat.contains("test --swing weeper15"));
    let (restored, _) = leg(&play, 2, "RiverWeeperReload", 45, &[]);
    for saved in [&won, &restored] {
        assert_eq!(saved["journey"]["stats"]["level"], 10);
        assert_eq!(
            saved["journey"]["stats"]["stats"],
            json!([100, 75, 23, 9, 332])
        );
        assert!(saved["kept"]["deadcity"]["gone"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "weeper15"));
        assert!(carries_one(saved, "INVITEM_Mace"));
        assert!(carries_one(saved, "INVITEM_SmCrossbow"));
        assert_eq!(saved["view_flags"]["active"], true);
    }
    assert!(!log(&play, "RiverWeeperReload", "fight.log")
        .contains("weeper15: its death ended (rule 32)"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_RIVER_WEEPER_SLOT"]
fn earned_mace_clears_the_boathouse_guard_and_reloads() {
    let play = Play::new("earned-boathouse-guard");
    let source = earned_slot("KQ8_RIVER_WEEPER_SLOT")
        .expect("set KQ8_RIVER_WEEPER_SLOT to an earned RiverWeeperReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "RiverWeeperReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["level"], 10);
    assert_eq!(
        start["journey"]["stats"]["stats"],
        json!([100, 75, 23, 9, 332])
    );
    assert_eq!(start["view_flags"]["active"], true);
    assert!(carries_one(&start, "INVITEM_Mace"));
    assert!(carries_one(&start, "INVITEM_SmCrossbow"));
    let path = play.root.join("boathouse-guard.json");
    let mut points: Vec<Value> = [
        [72250, 158750],
        [68500, 155000],
        [67750, 153250],
        [63500, 153250],
        [63250, 153500],
        [61750, 167250],
    ]
    .into_iter()
    .enumerate()
    .map(|(i, xy)| {
        json!({"name":format!("Follow the cleared road to the boathouse {}",i+1),
            "xy":xy,"radius":230,"run":true})
    })
    .collect();
    points.push(json!({"name":"Fight the boathouse guard on his road",
        "xy":[59600,168500],"radius":300,"run":true,
        "fight":"skeleton38","big_swing":true}));
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":35,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (won, viewer) = leg(
        &play,
        1,
        "BoathouseGuard",
        850,
        &[
            "--walk-route",
            path.to_str().unwrap(),
            "--key",
            "physical:F2,3",
            "--key",
            "left:IDCTL_INVBAR_SHORT,8",
        ],
    );
    assert!(viewer.contains("walk-route at Fight the boathouse guard on his road"));
    let combat = log(&play, "BoathouseGuard", "fight.log");
    assert!(combat.contains("skeleton38: hit by Connor"));
    assert!(combat.contains("conmace.anm/attack2"));
    assert_eq!(
        combat
            .matches("skeleton38: its death ended (rule 32)")
            .count(),
        1
    );
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = won["journey"]["stats"]["stats"][0].clone();
    expected["stats"][4] = json!(632);
    let (restored, _) = leg(&play, 2, "BoathouseGuardReload", 45, &[]);
    for saved in [&won, &restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        for name in ["skeleton55", "weeper15", "skeleton38"] {
            assert!(saved["kept"]["deadcity"]["gone"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == name));
        }
        assert!(carries_one(saved, "INVITEM_Mace"));
        assert!(carries_one(saved, "INVITEM_SmCrossbow"));
        assert_eq!(saved["view_flags"]["active"], false);
    }
    assert!(!log(&play, "BoathouseGuardReload", "fight.log")
        .contains("skeleton38: its death ended (rule 32)"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_BOATHOUSE_GUARD_SLOT"]
fn four_ferryman_topics_earn_knowledge_and_reload() {
    let play = Play::new("earned-ferryman-topics");
    let source = earned_slot("KQ8_BOATHOUSE_GUARD_SLOT")
        .expect("set KQ8_BOATHOUSE_GUARD_SLOT to an earned BoathouseGuardReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "BoathouseGuardReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["level"], 10);
    assert_eq!(start["journey"]["stats"]["stats"][4], 632);
    assert!(region_flag(&start, "DeadCity::Reaper::metConnor", "0"));
    assert!(!carries_one(&start, "INVITEM_Feather"));
    let mut previous = start.clone();
    for topic in 1..=4 {
        let label = format!("FerrymanTopic{topic}");
        let path = play.root.join(format!("ferryman-topic-{topic}.json"));
        let mut points: Vec<Value> = Vec::new();
        if topic == 1 {
            for xy in [
                [57000, 172000],
                [57000, 179500],
                [55750, 182250],
                [55900, 185500],
            ] {
                points.push(
                    json!({"name":format!("Enter the cleared boathouse {}",points.len()+1),
                    "xy":xy,"radius":180,"run":true}),
                );
            }
            points.push(json!({"name":"Meet the Ferryman","xy":[57000,192460],
                "radius":180,"run":false,"aim":"Ferryman"}));
        } else {
            let at = &previous["position"];
            points.push(json!({"name":format!("Ask Ferryman topic {topic}"),
                "xy":[at[0],at[1]],"radius":250,"aim":"Ferryman"}));
        }
        fs::write(
            &path,
            serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
        )
        .unwrap();
        let (saved, viewer) = leg(
            &play,
            topic,
            &label,
            if topic == 1 { 1200 } else { 1000 },
            &["--walk-route", path.to_str().unwrap()],
        );
        assert!(
            viewer.lines().any(|line| line
                .contains("test --aim Ferryman (camera through gameplay pixel")
                && line.contains("hits Ferryman first")),
            "{label}: no visible Ferryman click"
        );
        assert!(
            region_flag(&saved, "DeadCity::Reaper::metConnor", &topic.to_string()),
            "{label}: the original final TalkComplete did not advance the topic"
        );
        let conversation = log(&play, &label, "conversation.log");
        assert!(conversation.contains("the conversation with Ferryman ends"));
        assert!(!conversation.contains("walked out of the stay area"));
        let doors = log(&play, &label, "doors.log");
        let last = if topic < 3 { 7 } else { 5 };
        assert_eq!(
            doors
                .matches(&format!(
                    "Ferryman: TalkComplete 2000 29 14 {} {last} (line #",
                    40 + topic
                ))
                .count(),
            1
        );
        assert_eq!(
            doors
                .matches("KQStatsChangeRequestEvent 0 0 0 0 2000 to Connor")
                .count(),
            usize::from(topic == 1)
        );
        assert_eq!(
            doors
                .matches("KQStatsChangeRequestEvent 0 0 0 0 3000 to Connor")
                .count(),
            usize::from(topic == 4)
        );
        assert!(region_flag(&saved, "DeadCity::Reaper::Experience", "true"));
        if topic < 4 {
            let mut expected = start["journey"]["stats"].clone();
            expected["stats"][4] = json!(2632);
            assert_eq!(saved["journey"]["stats"], expected);
        } else {
            assert_eq!(saved["journey"]["stats"]["level"], 11);
            assert_eq!(saved["journey"]["stats"]["stats"][0], 110);
            assert_eq!(saved["journey"]["stats"]["stats"][4], 512);
        }
        assert_eq!(
            saved["journey"]["flags"]["KnowAboutFeather"] == true,
            topic >= 3
        );
        if topic == 3 {
            assert!(
                log(&play, &label, "items.log").contains("invClick Feather: right-click message")
            );
            assert!(!doors.contains("invClick: no item type"));
        }
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert!(!carries_one(&saved, "INVITEM_Feather"));
        previous = saved;
    }
    let (restored, _) = leg(&play, 5, "FerrymanKnowledgeReload", 45, &[]);
    assert_eq!(restored["journey"]["stats"], previous["journey"]["stats"]);
    assert_eq!(restored["journey"]["items"], previous["journey"]["items"]);
    assert_eq!(restored["journey"]["flags"]["KnowAboutFeather"], true);
    assert!(region_flag(&restored, "DeadCity::Reaper::metConnor", "4"));
    let reload = log(&play, "FerrymanKnowledgeReload", "doors.log");
    assert!(!reload.contains("KQStatsChangeRequestEvent 0 0 0 0 2000 to Connor"));
    assert!(!reload.contains("KQStatsChangeRequestEvent 0 0 0 0 3000 to Connor"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_FERRYMAN_KNOWLEDGE_SLOT"]
fn western_archers_and_rock_survive_reload() {
    let play = Play::new("earned-western-rock");
    let source = earned_slot("KQ8_FERRYMAN_KNOWLEDGE_SLOT")
        .expect("set KQ8_FERRYMAN_KNOWLEDGE_SLOT to an earned FerrymanKnowledgeReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "FerrymanKnowledgeReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["level"], 11);
    assert_eq!(start["journey"]["stats"]["stats"][4], 512);
    assert!(region_flag(&start, "DeadCity::Reaper::metConnor", "4"));
    assert!(carries_one(&start, "INVITEM_Mace"));
    assert!(!carries_one(&start, "INVITEM_Rock"));
    let path = play.root.join("western-rock.json");
    let mut points: Vec<Value> = [
        [55900, 185500],
        [55750, 182250],
        [57000, 179500],
        [57000, 172000],
        [59623, 168567],
        [59750, 157750],
        [59250, 157250],
        [54000, 155500],
    ]
    .into_iter()
    .enumerate()
    .map(|(i, xy)| {
        json!({"name":format!("Leave the boathouse for the western road {}",i+1),
            "xy":xy,"radius":180,"run":true})
    })
    .collect();
    for (name, xy, radius) in [
        ("BowGuys16", [50250, 150250], 300),
        // These two marks are the actor centers. Stop outside their bodies
        // before beginning the ordinary attack input.
        ("BowGuys15", [47857, 150050], 900),
        ("BowGuys17", [48498, 149000], 900),
    ] {
        points.push(json!({"name":format!("Clear western archer {name}"),
            "xy":xy,"radius":radius,"run":true,"fight":name,"big_swing":true}));
    }
    for (i, xy) in [[50750, 147000], [50250, 146500], [50250, 133750]]
        .into_iter()
        .enumerate()
    {
        points.push(
            json!({"name":format!("Follow the western road to the rock {}",i+1),
            "xy":xy,"radius":220,"run":true}),
        );
    }
    points.push(
        json!({"name":"Pick up the western rock","xy":[56600,126200],
        "radius":150,"run":true,"aim":"NoName45"}),
    );
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":30,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (earned, viewer) = leg(
        &play,
        1,
        "WesternRock",
        2100,
        &[
            "--walk-route",
            path.to_str().unwrap(),
            "--key",
            "left:IDCTL_INVBAR_SHORT,8",
        ],
    );
    assert!(viewer.contains("walk-route at Pick up the western rock"));
    assert!(viewer.lines().any(|line| line
        .contains("test --aim NoName45 (camera through gameplay pixel")
        && line.contains("hits NoName45 first")));
    let fight = log(&play, "WesternRock", "fight.log").to_ascii_lowercase();
    for name in ["bowguys15", "bowguys16", "bowguys17"] {
        assert!(fight.contains(&format!("{name}: hit by Connor").to_ascii_lowercase()));
        assert_eq!(
            fight
                .matches(&format!("{name}: its death ended (rule 32)"))
                .count(),
            1
        );
    }
    assert!(log(&play, "WesternRock", "items.log").contains("INVITEM_Rock 0 -> 1"));
    let (restored, _) = leg(&play, 2, "WesternRockReload", 45, &[]);
    for saved in [&earned, &restored] {
        assert!(carries_one(saved, "INVITEM_Rock"));
        assert!(carries_one(saved, "INVITEM_Mace"));
        assert!(carries_one(saved, "INVITEM_SmCrossbow"));
        assert_eq!(saved["journey"]["stats"]["level"], 11);
        assert_eq!(saved["journey"]["stats"]["stats"][4], 1112);
        assert_eq!(saved["journey"]["flags"]["KnowAboutFeather"], true);
        assert!(!carries_one(saved, "INVITEM_Feather"));
        assert!(region_flag(saved, "DeadCity::Reaper::metConnor", "4"));
        for name in ["BowGuys15", "BowGuys16", "BowGuys17"] {
            assert!(saved["kept"]["deadcity"]["gone"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))));
        }
    }
    assert_eq!(earned["journey"]["stats"], restored["journey"]["stats"]);
    assert_eq!(earned["journey"]["items"], restored["journey"]["items"]);
    assert!(!log(&play, "WesternRockReload", "fight.log").contains("its death ended (rule 32)"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_WESTERN_ROCK_SLOT"]
fn first_sylph_block_lights_its_tower_and_reloads() {
    let play = Play::new("earned-sylph-first-block");
    let source = earned_slot("KQ8_WESTERN_ROCK_SLOT")
        .expect("set KQ8_WESTERN_ROCK_SLOT to an earned WesternRockReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "WesternRockReload");
    assert_eq!(start["world"], "DeadCity");
    assert!(region_flag(&start, "DeadCity::Sylph::B15", "down"));
    assert_eq!(start["weapon_state"]["set_mode"], "None");
    let path = play.root.join("block15.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":3,"landmarks":[
        {"name":"Stand north of the western block","xy":[55657,125300],"radius":120,"run":false},
        {"name":"Push block15 off its plate","xy":[55657,121500],"radius":150,"run":false,
            "until_var":{"name":"DeadCity::Sylph::B15","value":"up"}},
        {"name":"Leave the first raised plate clear","xy":[56800,121500],"radius":160,"run":false}
    ]})).unwrap()).unwrap();
    let (earned, viewer) = leg(
        &play,
        1,
        "FirstSylphBlock",
        550,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Push block15 off its plate"));
    assert!(viewer.contains("walk-route at Leave the first raised plate clear"));
    let motion = log(&play, "FirstSylphBlock", "connor.log");
    assert!(
        motion.contains("push touch block15 (physical)") && motion.contains("pushed block15 by ")
    );
    let doors = log(&play, "FirstSylphBlock", "doors.log");
    assert_eq!(
        doors.matches("KQAnimPlayEvent up to object trip15").count(),
        1
    );
    let (restored, _) = leg(&play, 2, "FirstSylphBlockReload", 45, &[]);
    for saved in [&earned, &restored] {
        assert!(region_flag(saved, "DeadCity::Sylph::B15", "up"));
        for name in ["B12", "B13", "B14"] {
            assert!(region_flag(
                saved,
                &format!("DeadCity::Sylph::{name}"),
                "down"
            ));
        }
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "false"));
        assert!(region_flag(saved, "DeadCity::Sylph::FirstTime", "false"));
        let position = &saved["kept"]["deadcity"]["prop_placements"]["block15"]["position"];
        assert!(position[1].as_f64().unwrap() < 122900.0);
        assert_eq!(position[2], 950.0);
        let trap = saved["traps"]["states"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == "trptrp15")
            .unwrap();
        assert!(trap["inside"].as_array().unwrap().is_empty());
        assert!(saved["kept"]["deadcity"]["object_copies"]
            .as_array()
            .unwrap()
            .iter()
            .any(|object| object["name"] == "flame.port501"));
        assert_eq!(saved["journey"]["stats"], start["journey"]["stats"]);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
    }
    assert_eq!(
        earned["kept"]["deadcity"]["prop_placements"],
        restored["kept"]["deadcity"]["prop_placements"]
    );
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_FIRST_SYLPH_BLOCK_SLOT"]
fn all_four_sylph_plates_summon_her_once_and_reload() {
    let play = Play::new("earned-sylph-summoned");
    let source = earned_slot("KQ8_FIRST_SYLPH_BLOCK_SLOT")
        .expect("set KQ8_FIRST_SYLPH_BLOCK_SLOT to an earned FirstSylphBlockReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "FirstSylphBlockReload");
    assert!(region_flag(&start, "DeadCity::Sylph::B15", "up"));
    assert!(region_flag(&start, "DeadCity::Sylph::FirstTime", "false"));
    let path = play.root.join("remaining-blocks.json");
    let mut points = Vec::<Value>::new();
    for (id, approach, end, clear) in [
        (
            14,
            vec![
                [70500, 121500],
                [78750, 123000],
                [82500, 125000],
                [83657, 125300],
            ],
            [83657, 121500],
            // Leave sideways once the plate is up; otherwise the next
            // northbound leg keeps pushing the block along the road.
            [85500, 123000],
        ),
        (
            13,
            vec![
                [86250, 112500],
                [86250, 98750],
                [86250, 85000],
                [83657, 79300],
            ],
            [83657, 75500],
            [85500, 77000],
        ),
        (
            12,
            vec![
                // Keep the crossing clear of the Y80540 torches. The
                // input driver accepts 0.13 rad of heading error, so a
                // long segment grazing those colliders is not a safe
                // route even when its ideal center line is walkable.
                [85500, 82500],
                [77500, 82500],
                [73500, 82500],
                [69500, 82500],
                [65500, 82500],
                [58250, 80500],
                [55657, 79300],
            ],
            [55657, 75500],
            [57500, 77000],
        ),
    ] {
        for (i, xy) in approach.iter().enumerate() {
            points.push(json!({"name":format!("Approach block{id} {}",i+1),"xy":xy,
                "radius":if i+1==approach.len(){120}else{220},"run":i+1<approach.len()}));
        }
        points.push(
            json!({"name":format!("Push block{id} clear"),"xy":end,"radius":150,"run":false,
            "until_var":{"name":format!("DeadCity::Sylph::B{id}"),"value":"up"}}),
        );
        points.push(
            json!({"name":format!("Leave plate {id} clear"),"xy":clear,"radius":160,"run":false}),
        );
    }
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (earned, viewer) = leg(
        &play,
        1,
        "SylphSummoned",
        1600,
        &["--walk-route", path.to_str().unwrap()],
    );
    let motion = log(&play, "SylphSummoned", "connor.log");
    for id in [14, 13, 12] {
        assert!(viewer.contains(&format!("walk-route at Leave plate {id} clear")));
        assert!(motion.contains(&format!("push touch block{id} (physical)")));
    }
    let doors = log(&play, "SylphSummoned", "doors.log");
    assert_eq!(
        doors
            .matches("KQStatsChangeRequestEvent 0 0 0 0 2000 to Connor")
            .count(),
        1
    );
    let mut stats = start["journey"]["stats"].clone();
    stats["stats"][4] = json!(3112);
    let (restored, _) = leg(&play, 2, "SylphSummonedReload", 45, &[]);
    for saved in [&earned, &restored] {
        for id in [12, 13, 14, 15] {
            assert!(region_flag(saved, &format!("DeadCity::Sylph::B{id}"), "up"));
            let trap = saved["traps"]["states"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["name"] == format!("trptrp{id}"))
                .unwrap();
            assert!(trap["inside"].as_array().unwrap().is_empty());
            assert!(saved["kept"]["deadcity"]["prop_placements"][format!("block{id}")].is_object());
        }
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert!(region_flag(saved, "DeadCity::Sylph::FirstTime", "true"));
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "0"));
        let sylph = saved["kept"]["deadcity"]["characters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "Sylph")
            .unwrap();
        assert_eq!(
            sylph["character"]["ai"]["home"],
            json!([70662.0, 105677.0, 1584.0])
        );
        assert_eq!(saved["journey"]["stats"], stats);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
    }
    assert_eq!(
        earned["kept"]["deadcity"]["prop_placements"],
        restored["kept"]["deadcity"]["prop_placements"]
    );
    assert!(!log(&play, "SylphSummonedReload", "doors.log")
        .contains("KQStatsChangeRequestEvent 0 0 0 0 2000 to Connor"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_SYLPH_SUMMONED_SLOT"]
fn three_sylph_topics_award_experience_once_and_reload() {
    let play = Play::new("earned-sylph-topics");
    let source = earned_slot("KQ8_SYLPH_SUMMONED_SLOT")
        .expect("set KQ8_SYLPH_SUMMONED_SLOT to an earned SylphSummonedReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "SylphSummonedReload");
    assert_eq!(start["world"], "DeadCity");
    assert!(region_flag(&start, "DeadCity::Sylph::metConnor", "0"));
    assert!(region_flag(&start, "DeadCity::Sylph::isIn", "true"));
    assert_eq!(start["journey"]["stats"]["level"], 11);
    assert_eq!(start["journey"]["stats"]["stats"][4], 3112);
    let mut previous = start.clone();
    for topic in 1..=3 {
        let label = format!("SylphTopic{topic}");
        let path = play.root.join(format!("sylph-topic-{topic}.json"));
        let mut points = Vec::<Value>::new();
        if topic == 1 {
            // Use the wide route around portsa2 and portsa5, then
            // enter SylphInfo's original conversation start area.
            for xy in [
                [58000, 83500],
                [57750, 90750],
                [57000, 98750],
                [57000, 104500],
                [57750, 113000],
                [59750, 115000],
                [62750, 115000],
                [66500, 111380],
            ] {
                points.push(json!({"name":format!("Approach Sylph {}",points.len()+1),
                    "xy":xy,"radius":180,"run":true}));
            }
            points.push(json!({"name":"Meet Sylph at the well","xy":[70000,108000],
                "radius":150,"run":false,"aim":"Sylph"}));
        } else {
            let at = &previous["position"];
            points.push(json!({"name":format!("Ask Sylph topic {topic}"),
                "xy":[at[0],at[1]],"radius":250,"aim":"Sylph"}));
        }
        fs::write(
            &path,
            serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
        )
        .unwrap();
        let (saved, viewer) = leg(
            &play,
            topic,
            &label,
            if topic == 1 { 1300 } else { 1000 },
            &["--walk-route", path.to_str().unwrap()],
        );
        assert!(viewer.lines().any(|line| line
            .contains("test --aim Sylph (camera through gameplay pixel")
            && line.contains("hits Sylph first")), "{label}: no visible Sylph click");
        assert!(region_flag(&saved, "DeadCity::Sylph::metConnor", &topic.to_string()),
            "{label}: the original final TalkComplete did not advance the topic");
        let conversation = log(&play, &label, "conversation.log");
        assert!(conversation.contains("the conversation with Sylph ends"));
        assert!(!conversation.contains("walked out of the stay area"));
        let doors = log(&play, &label, "doors.log");
        let last = [2, 5, 3][topic as usize - 1];
        assert_eq!(doors.matches(&format!(
            "Sylph: TalkComplete 2000 7 14 {} {last} (line #", 9 + topic
        )).count(), 1);
        assert_eq!(doors.matches("KQStatsChangeRequestEvent 0 0 0 0 3000 to Connor").count(),
            usize::from(topic == 2));
        assert!(!doors.contains("KQStatsChangeRequestEvent 0 0 0 0 2000 to Connor"));
        let mut expected = start["journey"]["stats"].clone();
        expected["stats"][4] = json!(if topic == 1 { 3112 } else { 6112 });
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        for id in [12, 13, 14, 15] {
            assert!(region_flag(&saved, &format!("DeadCity::Sylph::B{id}"), "up"));
        }
        assert!(region_flag(&saved, "DeadCity::Sylph::isIn", "true"));
        assert_eq!(saved["kept"]["deadcity"]["prop_placements"],
            start["kept"]["deadcity"]["prop_placements"]);
        previous = saved;
    }
    let (restored, _) = leg(&play, 4, "SylphKnowledgeReload", 45, &[]);
    assert!(region_flag(&restored, "DeadCity::Sylph::metConnor", "3"));
    assert!(region_flag(&restored, "DeadCity::Sylph::isIn", "true"));
    assert_eq!(restored["journey"]["stats"], previous["journey"]["stats"]);
    assert_eq!(restored["journey"]["items"], previous["journey"]["items"]);
    assert_eq!(restored["kept"]["deadcity"]["prop_placements"],
        start["kept"]["deadcity"]["prop_placements"]);
    for id in [12, 13, 14, 15] {
        assert!(region_flag(&restored, &format!("DeadCity::Sylph::B{id}"), "up"));
    }
    let reload = log(&play, "SylphKnowledgeReload", "doors.log");
    assert!(!reload.contains("KQStatsChangeRequestEvent 0 0 0 0 2000 to Connor"));
    assert!(!reload.contains("KQStatsChangeRequestEvent 0 0 0 0 3000 to Connor"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_SYLPH_KNOWLEDGE_SLOT"]
fn sylph_to_southern_passage_preserves_campaign_and_reloads() {
    let play = Play::new("earned-southern-passage");
    let source = earned_slot("KQ8_SYLPH_KNOWLEDGE_SLOT")
        .expect("set KQ8_SYLPH_KNOWLEDGE_SLOT to an earned SylphKnowledgeReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "SylphKnowledgeReload");
    assert_eq!(start["world"], "DeadCity");
    let mut points = vec![json!({"name":"Leave Sylph's talking area",
        "xy":[70000,108000],"radius":140,"run":false})];
    let mut from = [70000_f32, 108000_f32];
    for (to, run) in [
        ([75250_f32, 107250_f32], false),
        ([80250., 103000.], true),
        ([90000., 93250.], true),
        ([91750., 82000.], false),
        ([95500., 80250.], false),
        ([96250., 80250.], false),
        ([98750., 77750.], false),
        ([100000., 71500.], false),
    ] {
        let steps = ((to[0] - from[0]).hypot(to[1] - from[1]) / 3000.).ceil() as u32;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let xy = [from[0] + t * (to[0] - from[0]), from[1] + t * (to[1] - from[1])];
            points.push(json!({"name":format!("Southern approach {}",points.len()),
                "xy":xy,"radius":140,"run":run}));
        }
        from = to;
    }
    points.push(json!({"name":"Southern passage reached","xy":from,"radius":300,"run":false}));
    let path = play.root.join("approach.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap()).unwrap();
    let (arrived, viewer) = leg(&play, 1, "SouthernPassage", 1800,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains("walk-route at Southern passage reached"));
    let (restored, _) = leg(&play, 2, "SouthernPassageReload", 45, &[]);
    for saved in [&arrived, &restored] {
        assert!((saved["position"][0].as_f64().unwrap() - 100000.).abs() < 800.);
        assert!((saved["position"][1].as_f64().unwrap() - 71500.).abs() < 800.);
        assert_eq!(saved["journey"]["stats"], start["journey"]["stats"]);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        for id in [12,13,14,15] {
            assert!(region_flag(saved, &format!("DeadCity::Sylph::B{id}"), "up"));
        }
        assert_eq!(saved["kept"]["deadcity"]["prop_placements"],
            start["kept"]["deadcity"]["prop_placements"]);
    }
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_SOUTHERN_PASSAGE_SLOT"]
fn passage_weeper_with_earned_crossbow_survives_reload() {
    let play = Play::new("earned-passage-weeper");
    let source = earned_slot("KQ8_SOUTHERN_PASSAGE_SLOT")
        .expect("set KQ8_SOUTHERN_PASSAGE_SLOT to an earned SouthernPassageReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "SouthernPassageReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["view_flags"]["active"], false);
    assert_eq!(start["journey"]["stats"]["level"], 11);
    assert_eq!(start["journey"]["stats"]["stats"][4], 6112);
    assert!(carries_one(&start, "INVITEM_SmCrossbow"));
    let path = play.root.join("passage-weeper.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":[
        {"name":"Round the southern bend","xy":[102500,69250],"radius":150,"run":false},
        {"name":"Face the Weeper corridor","xy":[104750,71500],"radius":150,"run":false},
        {"name":"Shoot the passage Weeper with the earned crossbow","xy":[105000,74000],
            "radius":150,"run":false,"fight":"weeper16","ranged":true}
    ]})).unwrap()).unwrap();
    let (won, viewer) = leg(&play, 1, "PassageWeeper", 800, &[
        "--walk-route",path.to_str().unwrap(),"--key","physical:F2,3"
    ]);
    assert!(viewer.contains("walk-route at Shoot the passage Weeper with the earned crossbow"));
    assert!(viewer.contains("walk-route selects the ranged weapon slot for weeper16"));
    assert!(viewer.lines().any(|line| line
        .contains("test --aim weeper16 (camera through gameplay pixel")
        && line.contains("the aim pick meets character weeper16")));
    let combat = log(&play, "PassageWeeper", "fight.log");
    assert!(combat.contains("weeper16: hit by Connor, Projectile"));
    assert_eq!(combat.matches("weeper16: its death ended (rule 32)").count(), 1);
    assert!(!combat.contains("test --swing weeper16"));
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = won["journey"]["stats"]["stats"][0].clone();
    expected["stats"][4] = json!(6612);
    assert!(expected["stats"][0].as_i64().unwrap() <= start["journey"]["stats"]["stats"][0].as_i64().unwrap());
    let (restored, _) = leg(&play, 2, "PassageWeeperReload", 45, &[]);
    for saved in [&won, &restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name == "weeper16"));
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert_eq!(saved["view_flags"]["active"], true);
    }
    assert!(!log(&play, "PassageWeeperReload", "fight.log")
        .contains("weeper16: its death ended (rule 32)"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_PASSAGE_WEEPER_SLOT"]
fn second_placed_rock_is_picked_up_and_reloaded() {
    let play = Play::new("earned-second-rock");
    let source = earned_slot("KQ8_PASSAGE_WEEPER_SLOT")
        .expect("set KQ8_PASSAGE_WEEPER_SLOT to an earned PassageWeeperReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "PassageWeeperReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["view_flags"]["active"], true);
    assert!(carries_one(&start, "INVITEM_Rock"));
    let path = play.root.join("second-rock.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":[
        {"name":"Pass the defeated Weeper","xy":[106250,78000],"radius":140,"run":false},
        {"name":"Round the eastern bend","xy":[108500,80250],"radius":140,"run":false},
        {"name":"Enter the rock corridor","xy":[111000,77750],"radius":140,"run":false},
        {"name":"Approach the second Rock","xy":[111750,74500],"radius":140,"run":false},
        {"name":"Pick up the second placed Rock","xy":[112595,71500],
            "radius":120,"run":false,"aim":"NoName44"}
    ]})).unwrap()).unwrap();
    let (earned, viewer) = leg(&play, 1, "SecondRock", 700, &[
        "--walk-route",path.to_str().unwrap(),"--key","physical:F2,3"
    ]);
    assert!(viewer.lines().any(|line| line
        .contains("test --aim NoName44 (camera through gameplay pixel")
        && line.contains("hits NoName44 first")));
    let others = |saved: &Value| saved["journey"]["items"].as_array().unwrap().iter()
        .filter(|item| item["kind"] != "INVITEM_Rock").cloned().collect::<Vec<_>>();
    let (restored, _) = leg(&play, 2, "SecondRockReload", 45, &[]);
    for saved in [&earned, &restored] {
        let rocks: i64 = saved["journey"]["items"].as_array().unwrap().iter()
            .filter(|item| item["kind"] == "INVITEM_Rock")
            .map(|item| item["quantity"].as_i64().unwrap()).sum();
        assert_eq!(rocks, 2);
        assert_eq!(others(saved), others(&start));
        assert_eq!(saved["journey"]["stats"], start["journey"]["stats"]);
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name == "weeper16"));
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert_eq!(saved["view_flags"]["active"], false);
    }
    assert_eq!(earned["journey"]["items"], restored["journey"]["items"]);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_SYLPH_WELL_SLOT"]
fn central_passage_guards_with_earned_mace_survive_reload() {
    let play = Play::new("earned-prepared-central-guards");
    let source = earned_slot("KQ8_SYLPH_WELL_SLOT")
        .expect("set KQ8_SYLPH_WELL_SLOT to an earned SylphWellReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "SylphWellReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["view_flags"]["active"], false);
    assert_eq!(start["journey"]["stats"]["level"], 11);
    assert_eq!(start["journey"]["stats"]["stats"][0], 110);
    assert_eq!(start["journey"]["stats"]["stats"][4], 6612);
    assert!(carries_one(&start, "INVITEM_Mace"));
    let mut points = Vec::new();
    let mut from = [66725.86_f32, 104720.02_f32];
    for (to, run) in [
        ([67000.,105000.],false), ([70000.,108000.],false),
        ([75250.,107250.],false), ([80250.,103000.],true),
        ([90000.,93250.],true), ([91750.,82000.],false),
        ([95500.,80250.],false), ([96250.,80250.],false),
        ([98750.,77750.],false), ([100000.,71500.],false),
        ([102500.,69250.],false), ([104750.,71500.],false),
        ([105000.,74000.],false), ([106250.,78000.],false),
        ([108500.,80250.],false), ([107250.,81500.],false),
        ([106250.,81750.],false), ([106250.,87000.],false),
        ([107250.,87750.],false), ([121000.,87750.],false),
    ] {
        let steps = ((to[0] - from[0]).hypot(to[1] - from[1]) / 2500.).ceil() as u32;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let xy = [from[0] + t * (to[0] - from[0]), from[1] + t * (to[1] - from[1])];
            points.push(json!({"name":format!("Healed central approach {}",points.len()+1),
                "xy":xy,"radius":140,"run":run}));
        }
        from = to;
    }
    points.push(json!({"name":"Prepared before the central team","xy":from,"radius":900}));
    let approach = play.root.join("healed-approach.json");
    fs::write(&approach, serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap()).unwrap();
    let (prepared, walked) = leg(&play, 1, "CentralPrepared", 1650,
        &["--walk-route",approach.to_str().unwrap()]);
    assert!(walked.contains("walk-route at Prepared before the central team"));
    assert_eq!(prepared["journey"]["stats"], start["journey"]["stats"]);
    assert_eq!(prepared["journey"]["items"], start["journey"]["items"]);
    assert!((prepared["position"][0].as_f64().unwrap() - 121000.).abs() < 900.);
    assert!((prepared["position"][1].as_f64().unwrap() - 87750.).abs() < 900.);
    let mut points = Vec::new();
    for x in [123062.5,125125.,127187.5,129250.] {
        points.push(json!({"name":format!("Central exposed approach {x}"),
            "xy":[x,87750.],"radius":140,"run":true}));
    }
    for (index, (name, xy, radius)) in [
        ("skeleton1", [129250,88750], 180),
        ("skeleton2", [129379,98863], 1000),
        ("skeleton3", [130629,98515], 1000),
    ].into_iter().enumerate() {
        points.push(json!({"name":format!("Defeat central guard {name}"),
            "xy":xy,"radius":radius,"run":true,"fight":name,"big_swing":true,
            "keep_weapon":index<2}));
    }
    for y in [96000, 99000, 102000, 104000] {
        points.push(json!({"name":format!("Beyond the central archers {y}"),
            "xy":[129500,y],"radius":160,"run":false}));
    }
    let path = play.root.join("central-guards.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":points})).unwrap()).unwrap();
    let (won, viewer) = leg(&play, 2, "CentralGuards", 700, &[
        "--walk-route",path.to_str().unwrap(),"--key","left:IDCTL_INVBAR_SHORT,8"
    ]);
    assert!(viewer.contains("walk-route at Beyond the central archers 104000"));
    let combat = log(&play, "CentralGuards", "fight.log");
    for name in ["skeleton1", "skeleton2", "skeleton3"] {
        assert!(combat.contains(&format!("{name}: hit by Connor")));
        assert_eq!(combat.matches(&format!("{name}: its death ended (rule 32)")).count(), 1);
    }
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = won["journey"]["stats"]["stats"][0].clone();
    expected["stats"][4] = json!(7112);
    assert!(expected["stats"][0].as_i64().unwrap() <= start["journey"]["stats"]["stats"][0].as_i64().unwrap());
    let (restored, _) = leg(&play, 3, "CentralGuardsReload", 45, &[]);
    for saved in [&won, &restored] {
        assert!((saved["position"][0].as_f64().unwrap() - 129500.).abs() < 900.);
        assert!((saved["position"][1].as_f64().unwrap() - 104000.).abs() < 900.);
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        for name in ["skeleton1", "skeleton2", "skeleton3", "weeper16"] {
            assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
                .iter().any(|gone| gone == name));
        }
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert_eq!(saved["view_flags"]["active"], false);
    }
    let reload = log(&play, "CentralGuardsReload", "fight.log");
    for name in ["skeleton1", "skeleton2", "skeleton3"] {
        assert!(!reload.contains(&format!("{name}: its death ended (rule 32)")));
    }
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_SECOND_ROCK_SLOT"]
fn earned_return_to_sylph_well_heals_through_visible_clicks_and_reloads() {
    let play = Play::new("earned-sylph-well");
    let source = earned_slot("KQ8_SECOND_ROCK_SLOT")
        .expect("set KQ8_SECOND_ROCK_SLOT to an earned SecondRockReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "SecondRockReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["view_flags"]["active"], false);
    assert_eq!(start["journey"]["stats"]["level"], 11);
    assert_eq!(start["journey"]["stats"]["stats"][0], 61);
    let mut points = Vec::new();
    let mut from = [112862_f32, 70939_f32];
    for (to, run) in [
        ([112250.,71750.],false), ([111750.,74500.],false),
        ([111000.,77750.],false), ([108500.,80250.],false),
        ([106250.,78000.],false), ([105000.,74000.],false),
        ([104750.,71500.],false), ([102500.,69250.],false),
        ([100000.,71500.],false), ([98750.,77750.],false),
        ([96250.,80250.],false), ([95500.,80250.],false),
        ([91750.,82000.],false), ([90000.,93250.],false),
        ([80250.,103000.],true), ([75250.,107250.],true),
        ([70000.,108000.],false), ([67000.,105000.],false),
    ] {
        let steps = ((to[0] - from[0]).hypot(to[1] - from[1]) / 2500.).ceil() as u32;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let xy = [from[0] + t * (to[0] - from[0]), from[1] + t * (to[1] - from[1])];
            points.push(json!({"name":format!("Return to the Sylph well {}",points.len()+1),
                "xy":xy,"radius":140,"run":run}));
        }
        from = to;
    }
    for drink in 1..=10 {
        points.push(json!({"name":format!("Drink from the Sylph well {drink}"),
            "xy":from,"radius":900,"run":false,"aim":"well2"}));
    }
    let path = play.root.join("sylph-well.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap()).unwrap();
    let (healed, viewer) = leg(&play, 1, "SylphWellHealed", 2300,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains("walk-route at Drink from the Sylph well 10"));
    assert_eq!(viewer.lines().filter(|line| line
        .contains("test --aim well2 (camera through gameplay pixel")
        && line.contains("hits well2 first")
        && !line.contains("walk-route turns the camera")).count(), 10);
    let doors = log(&play, "SylphWellHealed", "doors.log");
    assert_eq!(doors.matches("KQStatsChangeRequestEvent 5 0 0 0 0 to Connor").count(), 10);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = json!(110);
    let (restored, _) = leg(&play, 2, "SylphWellReload", 45, &[]);
    for saved in [&healed, &restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert_eq!(saved["kept"]["deadcity"]["prop_placements"],
            start["kept"]["deadcity"]["prop_placements"]);
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name == "weeper16"));
    }
    assert!(!log(&play, "SylphWellReload", "doors.log")
        .contains("KQStatsChangeRequestEvent 5 0 0 0 0 to Connor"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_CENTRAL_GUARDS_SLOT"]
fn central_guard_drops_heal_and_preserve_supplies_after_reload() {
    let play = Play::new("earned-central-supplies");
    let source = earned_slot("KQ8_CENTRAL_GUARDS_SLOT")
        .expect("set KQ8_CENTRAL_GUARDS_SLOT to an earned CentralGuardsReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "CentralGuardsReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"][4], 7112);
    let drops: Vec<_> = start["items"].as_array().unwrap().iter().filter(|item| {
        let p = &item["place"]["World"]["position"];
        p[0].as_f64().is_some_and(|x| (127000.0..135000.0).contains(&x))
            && p[1].as_f64().is_some_and(|y| (85000.0..105000.0).contains(&y))
            && matches!(item["kind"].as_str(),
                Some("INVITEM_Mushroom" | "INVITEM_SilverCoins" | "INVITEM_Strength"))
    }).collect();
    assert_eq!(drops.len(), 7);
    assert_eq!(drops.iter().filter(|d| d["kind"]=="INVITEM_Mushroom").count(), 4);
    assert_eq!(drops.iter().filter(|d| d["kind"]=="INVITEM_SilverCoins")
        .map(|d| d["quantity"].as_i64().unwrap()).sum::<i64>(), 25);
    assert_eq!(drops.iter().filter(|d| d["kind"]=="INVITEM_Strength").count(), 1);
    let mut points = Vec::new();
    for y in [102000,100000,98000] {
        points.push(json!({"name":format!("Return to the archer drops {y}"),
            "xy":[129500,y],"radius":140,"run":false}));
    }
    for item in drops.iter().filter(|d| d["place"]["World"]["position"][1].as_f64().unwrap() > 95000.) {
        points.push(json!({"name":format!("Collect central drop {}",item["name"]),
            "xy":[129500,98000],"radius":900,"aim":item["name"]}));
    }
    for y in [95000,92500,90500] {
        points.push(json!({"name":format!("Return to the melee guard drops {y}"),
            "xy":[129500,y],"radius":140,"run":false}));
    }
    for item in drops.iter().filter(|d| d["place"]["World"]["position"][1].as_f64().unwrap() < 95000.) {
        points.push(json!({"name":format!("Collect central drop {}",item["name"]),
            "xy":[129500,90500],"radius":900,"aim":item["name"]}));
    }
    for (index,y) in [93000,95500,98000,100500,103000,104000].into_iter().enumerate() {
        let mut point = json!({"name":format!("Return with central supplies {y}"),
            "xy":[129500,y],"radius":140,"run":false});
        if index < 4 { point["inventory_slot"] = json!(1); }
        points.push(point);
    }
    let path = play.root.join("central-supplies.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap()).unwrap();
    let (supplied, viewer) = leg(&play, 1, "CentralSupplies", 1200,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains("walk-route at Return with central supplies 104000"));
    for item in &drops {
        let name = item["name"].as_str().unwrap();
        assert!(viewer.lines().any(|line| line
            .contains(&format!("test --aim {name} (camera through gameplay pixel"))
            && line.contains(&format!("hits {name} first"))));
    }
    assert_eq!(log(&play, "CentralSupplies", "items.log")
        .matches("INVITEM_Mushroom, slot used").count(), 4);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = json!(start["journey"]["stats"]["stats"][0].as_i64().unwrap()+20);
    let (restored, _) = leg(&play, 2, "CentralSuppliesReload", 45, &[]);
    for saved in [&supplied, &restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        let items = saved["journey"]["items"].as_array().unwrap();
        for original in start["journey"]["items"].as_array().unwrap() { assert!(items.contains(original)); }
        assert!(items.iter().any(|i| i["kind"]=="INVITEM_SilverCoins" && i["quantity"]==25));
        assert!(items.iter().any(|i| i["kind"]=="INVITEM_Strength" && i["quantity"]==1));
        assert!(!items.iter().any(|i| i["kind"]=="INVITEM_Mushroom"));
        assert_eq!(items.len(), start["journey"]["items"].as_array().unwrap().len()+2);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        for name in ["skeleton1","skeleton2","skeleton3"] {
            assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
                .iter().any(|gone| gone==name));
        }
    }
    assert_eq!(supplied["journey"]["items"], restored["journey"]["items"]);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_CENTRAL_SUPPLIES_SLOT"]
fn western_passage_guard_uses_earned_strength_and_reloads_after_expiry() {
    let play = Play::new("earned-western-passage-guard");
    let source = earned_slot("KQ8_CENTRAL_SUPPLIES_SLOT")
        .expect("set KQ8_CENTRAL_SUPPLIES_SLOT to an earned CentralSuppliesReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "CentralSuppliesReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([43,80,25,10,7112]));
    assert!(carries_one(&start, "INVITEM_Strength"));
    assert!(carries_one(&start, "INVITEM_Mace"));
    let mut points = Vec::new();
    for xy in [
        [129500,107000],[129500,110000],[129500,112500],
        [129250,115000],[129250,117000],[126500,117000],
        [124000,117000],[121500,117000],
    ] {
        points.push(json!({"name":format!("Northern passage {}",points.len()+1),
            "xy":xy,"radius":140,"run":false}));
    }
    points.push(json!({"name":"Use the earned Strength before the western guard",
        "xy":[120000,117000],"radius":140,"run":false,"inventory_slot":6}));
    points.push(json!({"name":"Fight the western passage guard",
        "xy":[118000,117000],"radius":140,"run":false,"fight":"skeleton52","retreat":true}));
    for x in [115500,113000,110500,108250] {
        points.push(json!({"name":format!("Western passage cleared {x}"),
            "xy":[x,117000],"radius":140,"run":false}));
    }
    let path = play.root.join("western-guard.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":points})).unwrap()).unwrap();
    let (won, viewer) = leg(&play, 1, "WesternPassageGuard", 1100, &[
        "--walk-route",path.to_str().unwrap(),"--key","left:IDCTL_INVBAR_SHORT,8"
    ]);
    assert!(viewer.contains("walk-route at Western passage cleared 108250"));
    let combat = log(&play, "WesternPassageGuard", "fight.log");
    assert!(combat.contains("skeleton52 missing Connor, too far away"));
    assert!(combat.contains("skeleton52: hit by Connor"));
    assert_eq!(combat.matches("skeleton52: its death ended (rule 32)").count(), 1);
    let items = log(&play, "WesternPassageGuard", "items.log");
    assert!(items.contains("INVITEM_Strength, slot used"));
    assert_eq!(items.matches("strength: hand number 80 -> 120").count(), 1);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = won["journey"]["stats"]["stats"][0].clone();
    expected["stats"][4] = json!(7512);
    assert!(expected["stats"][0].as_i64().unwrap() <= 43);
    let remaining: Vec<_> = start["journey"]["items"].as_array().unwrap().iter()
        .filter(|item| item["kind"]!="INVITEM_Strength").cloned().collect();
    let (restored, _) = leg(&play, 2, "WesternPassageReload", 45, &[]);
    for saved in [&won,&restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], json!(remaining));
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name=="skeleton52"));
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert!((saved["position"][0].as_f64().unwrap()-108250.).abs()<900.);
        assert!((saved["position"][1].as_f64().unwrap()-117000.).abs()<900.);
    }
    assert!(!log(&play, "WesternPassageReload", "fight.log")
        .contains("skeleton52: its death ended (rule 32)"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_WESTERN_PASSAGE_SLOT"]
fn graveyard_entrance_preserves_earned_state_and_reloads() {
    let play = Play::new("earned-graveyard-entrance");
    let source = earned_slot("KQ8_WESTERN_PASSAGE_SLOT")
        .expect("set KQ8_WESTERN_PASSAGE_SLOT to an earned WesternPassageReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "WesternPassageReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([39,80,25,10,7512]));
    let points: Vec<_> = [
        [106250,114750],[106250,112000],[106250,109250],
        [106250,106500],[106250,103750],[106250,101000],
        [105920,98500],[105580,96000],[105250,93750],
        [105250,93000],[106000,92750],[108500,93400],
        [110750,94000],[111000,94750],[111000,97000],[111000,99000],
    ].into_iter().enumerate().map(|(i,xy)| json!({
        "name":format!("Graveyard entrance {}",i+1),
        "xy":xy,"radius":140,"run":false
    })).collect();
    let path = play.root.join("graveyard-entrance.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":points})).unwrap()).unwrap();
    let (entered, viewer) = leg(&play, 1, "GraveyardEntrance", 1400,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains("walk-route at Graveyard entrance 16"));
    let (restored, _) = leg(&play, 2, "GraveyardEntranceReload", 45, &[]);
    for saved in [&entered,&restored] {
        assert_eq!(saved["journey"]["stats"], start["journey"]["stats"]);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert!((saved["position"][0].as_f64().unwrap()-111000.).abs()<900.);
        assert!((saved["position"][1].as_f64().unwrap()-99000.).abs()<900.);
        assert!(!saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name=="zombie6"));
    }
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "known failing underground-target approach; requires GOG, graphics and KQ8_GRAVEYARD_ENTRANCE_SLOT"]
fn graveyard_zombie_and_plate_approach_survive_reload() {
    let play = Play::new("earned-graveyard-zombie");
    let source = earned_slot("KQ8_GRAVEYARD_ENTRANCE_SLOT")
        .expect("set KQ8_GRAVEYARD_ENTRANCE_SLOT to an earned GraveyardEntranceReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "GraveyardEntranceReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([39,80,25,10,7512]));
    assert!(carries_one(&start, "INVITEM_SmCrossbow"));
    assert_eq!(start["view_flags"]["active"], false);
    let path = play.root.join("graveyard-zombie.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":[
        {"name":"Shoot the graveyard zombie from the entrance","xy":[111014,99472],
            "radius":140,"run":false,"fight":"zombie6","ranged":true},
        {"name":"Cleared southern graveyard floor","xy":[111000,102500],"radius":140,"run":false},
        {"name":"Cleared western graveyard floor","xy":[111000,105500],"radius":140,"run":false},
        {"name":"Western plate approach","xy":[111000,107500],"radius":140,"run":false},
        {"name":"Face the western plate","xy":[112000,108500],"radius":140,"run":false},
        {"name":"Outside the western plate","xy":[112800,108676],"radius":140,"run":false}
    ]})).unwrap()).unwrap();
    let (won, viewer) = leg(&play, 1, "GraveyardZombie", 800, &[
        "--walk-route",path.to_str().unwrap(),"--key","physical:F2,3"
    ]);
    assert!(viewer.contains("walk-route at Outside the western plate"));
    assert!(viewer.lines().any(|line| line
        .contains("test --aim zombie6 (camera through gameplay pixel")
        && line.contains("the aim pick meets character zombie6")));
    let fight = log(&play, "GraveyardZombie", "fight.log");
    assert!(fight.contains("zombie6: hit by Connor, Projectile"));
    assert!(!fight.contains("test --swing zombie6"));
    assert_eq!(fight.matches("zombie6: its death ended (rule 32)").count(), 1);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = won["journey"]["stats"]["stats"][0].clone();
    expected["stats"][4] = json!(7914);
    assert!(expected["stats"][0].as_i64().unwrap()<=39);
    let (restored, _) = leg(&play, 2, "GraveyardZombieReload", 45, &[]);
    for saved in [&won,&restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name=="zombie6"));
        assert!((saved["position"][0].as_f64().unwrap()-112800.).abs()<900.);
        assert!((saved["position"][1].as_f64().unwrap()-108676.).abs()<900.);
    }
    assert!(!log(&play, "GraveyardZombieReload", "fight.log")
        .contains("zombie6: its death ended (rule 32)"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

/// Previously traversed roads around the ridges, not a straight line
/// across the high plateau or the still-closed graveyard portals.
fn graveyard_well_road(mut from: [f32; 2], towards_well: bool) -> Vec<Value> {
    let mut road = vec![
        [111014.,99472.],[111000.,97000.],[111000.,94750.],
        [110750.,94000.],[108500.,93400.],[106000.,92750.],
        [105250.,93000.],[105250.,93750.],[105580.,96000.],
        [105920.,98500.],[106250.,101000.],[106250.,103750.],
        [106250.,106500.],[106250.,109250.],[106250.,112000.],
        [106250.,114750.],[108250.,117000.],[110500.,117000.],
        [113000.,117000.],[115500.,117000.],[118000.,117000.],
        [120000.,117000.],[121500.,117000.],[124000.,117000.],
        [126500.,117000.],[129250.,117000.],[129250.,115000.],
        [129500.,112500.],[129500.,110000.],[129500.,107000.],
        [129500.,104000.],[129500.,102000.],[129500.,99000.],
        [129500.,96000.],[129500.,93500.],[129250.,91000.],
        [129250.,88750.],[129250.,87750.],[121000.,87750.],
        [107250.,87750.],[106250.,87000.],[106250.,81750.],
        [107250.,81500.],[108500.,80250.],[106250.,78000.],
        [105000.,74000.],[104750.,71500.],[102500.,69250.],
        [100000.,71500.],[98750.,77750.],[96250.,80250.],
        [95500.,80250.],[91750.,82000.],[90000.,93250.],
        [80250.,103000.],[75250.,107250.],[70000.,108000.],
        [67000.,105000.],
    ];
    if !towards_well { road.reverse(); }
    let destination = if towards_well { "well" } else { "graveyard" };
    let mut points = Vec::new();
    for to in road {
        let steps = ((to[0]-from[0]).hypot(to[1]-from[1])/2500.).ceil().max(1.) as u32;
        for i in 1..=steps {
            let t = i as f32/steps as f32;
            let xy = [from[0]+t*(to[0]-from[0]),from[1]+t*(to[1]-from[1])];
            points.push(json!({"name":format!("Prepared return to the {destination} {}",points.len()+1),
                "xy":xy,"radius":140,"run":false}));
        }
        from = to;
    }
    points
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_GRAVEYARD_ENTRANCE_SLOT"]
fn graveyard_return_to_the_well_heals_and_preserves_campaign() {
    let play = Play::new("earned-graveyard-well");
    let source = earned_slot("KQ8_GRAVEYARD_ENTRANCE_SLOT")
        .expect("set KQ8_GRAVEYARD_ENTRANCE_SLOT to an earned GraveyardEntranceReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "GraveyardEntranceReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([39,80,25,10,7512]));
    let from = [start["position"][0].as_f64().unwrap() as f32,
        start["position"][1].as_f64().unwrap() as f32];
    let mut points = graveyard_well_road(from, true);
    for drink in 1..=15 {
        points.push(json!({"name":format!("Graveyard preparation drink {drink}"),
            "xy":[67000,105000],"radius":900,"run":false,"aim":"well2"}));
    }
    let path = play.root.join("graveyard-well.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":points})).unwrap()).unwrap();
    let (healed, viewer) = leg(&play, 1, "GraveyardWell", 3400,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains("walk-route at Graveyard preparation drink 15"));
    assert_eq!(viewer.lines().filter(|line| line
        .contains("test --aim well2 (camera through gameplay pixel")
        && line.contains("hits well2 first")
        && !line.contains("walk-route turns the camera")).count(), 15);
    assert_eq!(log(&play, "GraveyardWell", "doors.log")
        .matches("KQStatsChangeRequestEvent 5 0 0 0 0 to Connor").count(), 15);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][0] = json!(110);
    let (restored, _) = leg(&play, 2, "GraveyardWellReload", 45, &[]);
    for saved in [&healed,&restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert_eq!(saved["kept"]["deadcity"]["prop_placements"],
            start["kept"]["deadcity"]["prop_placements"]);
        for name in ["skeleton1","skeleton2","skeleton3","skeleton52","weeper16"] {
            assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
                .iter().any(|gone| gone==name));
        }
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
    }
    assert!(!log(&play, "GraveyardWellReload", "doors.log")
        .contains("KQStatsChangeRequestEvent 5 0 0 0 0 to Connor"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_GRAVEYARD_WELL_SLOT"]
fn healed_return_to_graveyard_preserves_health_and_earned_state() {
    let play = Play::new("earned-healed-graveyard-return");
    let source = earned_slot("KQ8_GRAVEYARD_WELL_SLOT")
        .expect("set KQ8_GRAVEYARD_WELL_SLOT to an earned GraveyardWellReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "GraveyardWellReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([110,80,25,10,7512]));
    let from = [start["position"][0].as_f64().unwrap() as f32,
        start["position"][1].as_f64().unwrap() as f32];
    let points = graveyard_well_road(from, false);
    let final_waypoint = points.last().unwrap()["name"].as_str().unwrap().to_string();
    let path = play.root.join("healed-graveyard-return.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":points})).unwrap()).unwrap();
    let (arrived, viewer) = leg(&play, 1, "HealedGraveyard", 3400,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains(&format!("walk-route at {final_waypoint}")));
    let (restored, _) = leg(&play, 2, "HealedGraveyardReload", 45, &[]);
    for saved in [&arrived,&restored] {
        assert_eq!(saved["journey"]["stats"], start["journey"]["stats"]);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert_eq!(saved["kept"]["deadcity"]["prop_placements"],
            start["kept"]["deadcity"]["prop_placements"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        for name in ["skeleton1","skeleton2","skeleton3","skeleton52","weeper16"] {
            assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
                .iter().any(|gone| gone==name));
        }
        assert!((saved["position"][0].as_f64().unwrap()-111014.).abs()<900.);
        assert!((saved["position"][1].as_f64().unwrap()-99472.).abs()<900.);
        assert!(!saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name=="zombie6"));
    }
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_HEALED_GRAVEYARD_SLOT"]
fn healed_graveyard_zombie_and_western_plate_survive_reload() {
    let play = Play::new("earned-healed-graveyard-zombie");
    let source = earned_slot("KQ8_HEALED_GRAVEYARD_SLOT")
        .expect("set KQ8_HEALED_GRAVEYARD_SLOT to an earned HealedGraveyardReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "HealedGraveyardReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([110,80,25,10,7512]));
    assert!(carries_one(&start, "INVITEM_Mace"));
    let path = play.root.join("healed-graveyard-zombie.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":[
        {"name":"Approach the original zombie alarm radius","xy":[111000,101000],"radius":140,"run":false},
        {"name":"Use the earned Mace after the zombie rises","xy":[112000,103000],"radius":700,
            "run":false,"fight":"zombie6","big_swing":false},
        {"name":"Cleared western graveyard floor","xy":[111000,105500],"radius":140,"run":false},
        {"name":"Western plate approach","xy":[111000,107500],"radius":140,"run":false},
        {"name":"Face the western plate","xy":[112000,108500],"radius":140,"run":false},
        {"name":"Stop onto the western pressure plate","xy":[112800,108676],"radius":140,"run":false}
    ]})).unwrap()).unwrap();
    let (won, viewer) = leg(&play, 1, "WesternPlateHeld", 800, &[
        "--walk-route",path.to_str().unwrap(),"--key","left:IDCTL_INVBAR_SHORT,8"
    ]);
    assert!(viewer.contains("walk-route at Stop onto the western pressure plate"));
    assert!(log(&play, "WesternPlateHeld", "characters.log")
        .contains("zombie6: rises (rule 30)"));
    let fight = log(&play, "WesternPlateHeld", "fight.log");
    assert!(fight.contains("zombie6: hit by Connor, Normal"));
    assert_eq!(fight.matches("zombie6: its death ended (rule 32)").count(), 1);
    let mut expected = start["journey"]["stats"].clone();
    expected["stats"][4] = json!(7914);
    let traps = log(&play, "WesternPlateHeld", "traps.log");
    assert_eq!(traps.matches("trap triptrp5: On Connor").count(), 1);
    assert!(traps.contains("collision portrap5 +4: mask 0x0 -> 0x4"));
    assert!(traps.contains("collision portrp5b +4: mask 0x0 -> 0x4"));
    assert!(traps.contains("addMapIcon dc_port1.bmp 92 240 dedmap.pbm"));
    assert!(log(&play, "WesternPlateHeld", "doors.log")
        .contains("KQAnimPlayEvent down to object trip5"));
    let (restored, _) = leg(&play, 2, "WesternPlateHeldReload", 45, &[]);
    for saved in [&won,&restored] {
        assert_eq!(saved["journey"]["stats"], expected);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name=="zombie6"));
        let states = saved["traps"]["states"].as_array().unwrap();
        for name in ["portrap5","portrp5b"] {
            assert_eq!(states.iter().find(|t| t["name"]==name).unwrap()["mask"], 4);
        }
        let plate = states.iter().find(|t| t["name"]=="triptrp5").unwrap();
        assert!(plate["inside"].as_array().unwrap().iter().any(|n| n=="Connor"));
        // The authored plate box extends 250 units from its center;
        // Connor's 200-unit body can touch it from either side.
        assert!((saved["position"][0].as_f64().unwrap()-113992.).abs()<=450.);
        assert!((saved["position"][1].as_f64().unwrap()-108676.).abs()<=450.);
    }
    assert!(!log(&play, "WesternPlateHeldReload", "fight.log")
        .contains("zombie6: its death ended (rule 32)"));
    assert!(!log(&play, "WesternPlateHeldReload", "traps.log")
        .contains("trap triptrp5: On Connor"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets, graphics and a genuine KQ8_WESTERN_PLATE_SLOT"]
fn western_plate_release_disables_portal_and_reloads() {
    let play = Play::new("earned-western-plate-release");
    let source = earned_slot("KQ8_WESTERN_PLATE_SLOT")
        .expect("set KQ8_WESTERN_PLATE_SLOT to an earned WesternPlateHeldReload save");
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "WesternPlateHeldReload");
    assert_eq!(start["world"], "DeadCity");
    assert_eq!(start["journey"]["stats"]["stats"], json!([110,80,25,10,7914]));
    let states = start["traps"]["states"].as_array().unwrap();
    for name in ["portrap5","portrp5b"] {
        assert_eq!(states.iter().find(|t| t["name"]==name).unwrap()["mask"], 4);
    }
    let path = play.root.join("western-plate-release.json");
    fs::write(&path, serde_json::to_vec(&json!({"start":25,"landmarks":[
        {"name":"Leave the western pressure plate","xy":[112800,108676],"radius":140,"run":false}
    ]})).unwrap()).unwrap();
    let (released, viewer) = leg(&play, 1, "WesternPlateReleased", 220,
        &["--walk-route",path.to_str().unwrap()]);
    assert!(viewer.contains("walk-route at Leave the western pressure plate"));
    let traps = log(&play, "WesternPlateReleased", "traps.log");
    assert_eq!(traps.matches("trap triptrp5: Off Connor").count(), 1);
    assert!(traps.contains("collision portrap5 -4: mask 0x4 -> 0x0"));
    assert!(traps.contains("collision portrp5b -4: mask 0x4 -> 0x0"));
    assert!(traps.contains("removeMapIcon 92 240"));
    assert!(log(&play, "WesternPlateReleased", "doors.log")
        .contains("KQAnimPlayEvent up to object trip5"));
    let (restored, _) = leg(&play, 2, "WesternPlateReleasedReload", 45, &[]);
    for saved in [&released,&restored] {
        assert_eq!(saved["journey"]["stats"], start["journey"]["stats"]);
        assert_eq!(saved["journey"]["items"], start["journey"]["items"]);
        assert_eq!(saved["journey"]["flags"], start["journey"]["flags"]);
        let states = saved["traps"]["states"].as_array().unwrap();
        for name in ["portrap5","portrp5b"] {
            assert_eq!(states.iter().find(|t| t["name"]==name).unwrap()["mask"], 0);
        }
        assert!(states.iter().find(|t| t["name"]=="triptrp5").unwrap()["inside"]
            .as_array().unwrap().is_empty());
        assert!(saved["kept"]["deadcity"]["gone"].as_array().unwrap()
            .iter().any(|name| name=="zombie6"));
        assert!(region_flag(saved, "DeadCity::Sylph::metConnor", "3"));
        assert!(region_flag(saved, "DeadCity::Sylph::isIn", "true"));
    }
    assert!(!log(&play, "WesternPlateReleasedReload", "traps.log")
        .contains("trap triptrp5: Off Connor"));
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}
