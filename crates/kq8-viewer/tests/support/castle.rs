//! Travel toward Castle Daventry from the Wizard's two-ingredient save.
use super::{
    carries_one, earned_slot, hero_flag,
    shadow::{leg, log, offer_ingredients, prepare},
    Play,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn check_ingredients(save: &Value) {
    for flag in ["Daventry::Wizard::hasCandle", "Daventry::Wizard::hasRing"] {
        assert!(hero_flag(save, flag, "true"));
    }
    assert!(hero_flag(save, "Daventry::Wizard::hasTorch", "false"));
    assert!(hero_flag(save, "Daventry::Wizard::metConnor", "2"));
    assert!(carries_one(save, "INVITEM_RopeAndHook"));
    assert!(carries_one(save, "INVITEM_SmCrossbow"));
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_TWO_INGREDIENTS_SLOT"]
fn wizard_to_village_fountain() {
    reach_fountain();
}

fn reach_fountain() -> PathBuf {
    let play = Play::new("daventry-fountain-campaign");
    let source = earned_slot("KQ8_TWO_INGREDIENTS_SLOT").unwrap_or_else(offer_ingredients);
    let before = prepare(&play, &source);
    assert_eq!(before["label"], "TwoIngredients");
    check_ingredients(&before);
    let mut landmarks = vec![
        json!({"name":"North path by the Wizard","xy":[196600,207700],"radius":285,"run":true}),
        json!({"name":"Village road","xy":[197000,221300],"radius":285,"run":true}),
        json!({"name":"Approach the fountain","xy":[205000,232500],"radius":285,"run":true}),
        json!({"name":"Walk to the fountain rim","xy":[207000,244900],"radius":165,"run":false}),
    ];
    for i in 1..=28 {
        landmarks.push(
            json!({"name":format!("Drink {i}"),"xy":[207000,244900],"radius":420,"aim":"fountain"}),
        );
    }
    let route = play.root.join("fountain.json");
    fs::write(
        &route,
        serde_json::to_vec(&json!({"start":3,"landmarks":landmarks})).unwrap(),
    )
    .unwrap();
    let (saved, viewer) = leg(
        &play,
        1,
        "VillageFountain",
        1150,
        &["--walk-route", route.to_str().unwrap()],
    );
    assert_eq!(
        viewer
            .lines()
            .filter(
                |line| line.contains("test --aim fountain (camera through gameplay pixel")
                    && line.contains("hits fountain first")
            )
            .count(),
        28
    );
    let events = log(&play, "VillageFountain", "doors.log");
    assert_eq!(
        events
            .matches("KQStatsChangeRequestEvent 2 0 0 0 0 to Connor (sent by fountain.cs")
            .count(),
        28
    );
    check_ingredients(&saved);
    let stats = &saved["journey"]["stats"];
    assert_eq!(stats["level"], before["journey"]["stats"]["level"]);
    assert_eq!(
        stats["stats"][0].as_i64().unwrap(),
        stats["level"].as_i64().unwrap() * 10
    );
    let (restored, _) = leg(&play, 2, "FountainReload", 50, &[]);
    assert_eq!(restored["journey"]["stats"], *stats);
    check_ingredients(&restored);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_FOUNTAIN_SLOT"]
fn village_fountain_to_waterfall_rope() {
    reach_waterfall();
}

fn reach_waterfall() -> PathBuf {
    let play = Play::new("castle-waterfall-campaign");
    let source = earned_slot("KQ8_FOUNTAIN_SLOT").unwrap_or_else(reach_fountain);
    let first = prepare(&play, &source);
    assert_eq!(first["label"], "FountainReload");
    check_ingredients(&first);
    let road = [
        [196200, 253700],
        [189400, 259300],
        [185800, 259300],
        [180600, 263700],
        [170800, 266400],
        [163800, 266400],
        [157000, 266400],
        [147400, 276500],
        [143400, 288500],
        [146600, 302100],
        [151500, 303900],
    ];
    let mut points: Vec<_> = road.into_iter().enumerate().map(|(i, xy)| {
        json!({"name":format!("Castle road {}", i+1),"xy":xy,"radius":280,"run":true})
    }).collect();
    points.push(json!({"name":"Approach the lower waterfall rope","xy":[153100,303900],"radius":175,"run":false}));
    let path = play.root.join("castle-road.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (foot, viewer) = leg(
        &play,
        1,
        "WaterfallFoot",
        1300,
        &["--walk-route", path.to_str().unwrap()],
    );
    for i in 1..=11 {
        assert!(viewer.contains(&format!("walk-route at Castle road {i} on update")));
    }
    assert!(viewer.contains("walk-route at Approach the lower waterfall rope"));
    assert!(
        log(&play, "WaterfallFoot", "traps.log").contains("script ropetrp3.cs ropetrp3 On Connor")
    );
    assert_eq!(foot["rope_place"]["settings"]["status"], "Bottom");
    assert_eq!(
        foot["rope_place"]["settings"]["place"],
        json!([155460.0, 303959.0, 9850.0])
    );
    check_ingredients(&foot);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-2.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_WATERFALL_FOOT_SLOT"]
fn waterfall_rope_and_brick_puzzle() {
    let play = Play::new("waterfall-bricks-campaign");
    let source = earned_slot("KQ8_WATERFALL_FOOT_SLOT").unwrap_or_else(reach_waterfall);
    let foot = prepare(&play, &source);
    assert_eq!(foot["label"], "WaterfallFoot");
    check_ingredients(&foot);
    assert!(hero_flag(&foot, "Daventry::bricks", "up"));
    let (top, _) = leg(
        &play,
        1,
        "WaterfallTop",
        530,
        &[
            "--key",
            "left:IDCTL_INVBAR_ROPE,8",
            "--aim-point",
            "155460,303959,12150,14",
            "--hold",
            "1,0,0,150,276",
        ],
    );
    assert_eq!(
        log(&play, "WaterfallTop", "rope.log")
            .matches("climb: reached the top end")
            .count(),
        1
    );
    assert!(top["position"][2].as_f64().unwrap() >= 14000.0);
    assert_eq!(top["rope_place"]["settings"]["status"], "Top");
    let path = play.root.join("push.json");
    let route = json!({"start":3, "landmarks":[
        {"name":"Approach the west side of the table","xy":[156650,303600],"radius":115},
        {"name":"Push the waterfall bricks","xy":[157550,303600],"radius":70,"until_var":{"name":"Daventry::bricks","value":"down"}},
    ]});
    fs::write(&path, serde_json::to_vec(&route).unwrap()).unwrap();
    let (blocked, _) = leg(
        &play,
        2,
        "WaterfallBlocked",
        380,
        &["--walk-route", path.to_str().unwrap()],
    );
    let script = log(&play, "WaterfallBlocked", "doors.log");
    assert_eq!(
        script
            .matches("KQStatsChangeRequestEvent 0 0 0 0 50 to Connor (sent by bricks.cs")
            .count(),
        1
    );
    assert!(script.contains("loadKQ waterfa2.kq: waterfall2 comes into the world"));
    assert!(script.contains("bricks.cs line 44: handsOff 0"));
    assert!(hero_flag(&blocked, "Daventry::bricks", "down"));
    let xp = |save: &Value| save["journey"]["stats"]["stats"][4].as_i64().unwrap();
    assert_eq!(xp(&blocked), xp(&foot) + 50);
    let (restored, _) = leg(&play, 3, "WaterfallReload", 55, &[]);
    assert_eq!(restored["journey"]["stats"], blocked["journey"]["stats"]);
    assert!(hero_flag(&restored, "Daventry::bricks", "down"));
    assert!(restored["kept"]["daventry"]["came_in"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v
            .as_str()
            .is_some_and(|file| file.eq_ignore_ascii_case("waterfa2.kq"))));
    assert!(restored["kept"]["daventry"]["gone"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case("waterfall"))));
    check_ingredients(&restored);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}
