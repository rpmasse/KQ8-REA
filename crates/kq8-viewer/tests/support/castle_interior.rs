//! The fireplace passage beyond Castle Daventry's arrival portal.
use super::{
    carries_one,
    castle::enter_castle,
    earned_slot, hero_flag,
    shadow::{leg, log, prepare},
    Play,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn corridor(points: &[[i32; 2]]) -> Vec<Value> {
    points
        .iter()
        .enumerate()
        .map(|(i, xy)| {
            json!({
                "name":format!("Passage {}",i+1),"xy":xy,"radius":125
            })
        })
        .collect()
}

fn save_route(play: &Play, name: &str, points: Vec<Value>) -> PathBuf {
    let path = play.root.join(name);
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    path
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_ENTERED_CASTLE_SLOT"]
fn fireplace_passage_to_ashes_and_fresh_reload() {
    let play = Play::new("castle-ashes-campaign");
    let source = earned_slot("KQ8_ENTERED_CASTLE_SLOT").unwrap_or_else(enter_castle);
    let arrival = prepare(&play, &source);
    assert_eq!(arrival["world"], "CastleD");
    assert_eq!(arrival["label"], "EnteredCastle");
    assert!(hero_flag(&arrival, "Daventry::Castle::foundPortal", "1"));
    let ashes = arrival["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "INVITEM_Ashes" && i["place"]["World"].is_object())
        .unwrap();
    let name = ashes["name"].as_str().unwrap();
    let mut hall = corridor(&[
        [9720, 107140],
        [10000, 106860],
        [11680, 105180],
        [11820, 105040],
        [12940, 105040],
        [13640, 104900],
    ]);
    hall.push(
        json!({"name":"Operate the outer light","xy":[13950,102760],"radius":145,"aim":"cdlght1"}),
    );
    let path = save_route(&play, "outer-light.json", hall);
    let (opened, viewer) = leg(
        &play,
        1,
        "FireplaceOpen",
        420,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.lines().any(|line| line
        .contains("test --aim cdlght1 (camera through gameplay pixel")
        && line.contains("hits cdlght1 first")));
    let doors = log(&play, "FireplaceOpen", "doors.log");
    assert!(doors.contains(
        "KQDoorEvent 0 to firedor (sent by cdlight.cs line 18): state closed -> opening"
    ));
    assert_eq!(
        doors
            .matches("door firedor: movement 'open' finished; state opened")
            .count(),
        1
    );
    assert_eq!(
        opened["kept"]["castled"]["interactions"]["doors"]["firedor"]["state"],
        "Opened"
    );
    let mut passage = corridor(&[
        [13920, 101960],
        [15600, 97900],
        [15880, 97620],
        [17280, 96360],
        [18260, 96360],
        [24980, 93980],
    ]);
    passage.push(json!({"name":"Take the Ashes","xy":[25400,94400],"radius":155,"aim":name}));
    let path = save_route(&play, "fireplace-passage.json", passage);
    let (taken, viewer) = leg(
        &play,
        2,
        "CastleAshes",
        620,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.lines().any(|line| line
        .contains(&format!("test --aim {name} (camera through gameplay pixel"))
        && line.contains(&format!("hits {name} first"))));
    assert!(carries_one(&taken, "INVITEM_Ashes"));
    assert_eq!(
        log(&play, "CastleAshes", "items.log")
            .lines()
            .filter(|line| line.contains("inventory (picked up):")
                && line.contains("INVITEM_Ashes 0 -> 1"))
            .count(),
        1
    );
    let (restored, _) = leg(&play, 3, "AshesReload", 50, &[]);
    assert!(carries_one(&restored, "INVITEM_Ashes"));
    assert_eq!(restored["journey"]["stats"], taken["journey"]["stats"]);
    assert_eq!(
        restored["kept"]["castled"]["interactions"]["doors"]["firedor"]["state"],
        "Opened"
    );
    assert!(hero_flag(&restored, "Daventry::Wizard::hasTorch", "false"));
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
}
