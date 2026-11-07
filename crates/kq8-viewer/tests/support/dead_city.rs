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
