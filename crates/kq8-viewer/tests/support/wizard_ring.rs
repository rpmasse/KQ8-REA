//! Deliver the castle ingredient to the Wizard after the return portal.
use super::{
    carries_one,
    castle_interior::return_with_ashes,
    earned_slot, hero_flag,
    shadow::{leg, log, prepare},
    Play,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_RETURNED_ASHES_SLOT"]
fn wizard_casts_ring_of_light() {
    earn_ring();
}

fn earn_ring() -> PathBuf {
    let play = Play::new("wizard-ring-campaign");
    let source = earned_slot("KQ8_RETURNED_ASHES_SLOT").unwrap_or_else(return_with_ashes);
    let start = prepare(&play, &source);
    assert_eq!(start["label"], "ReturnedWithAshes");
    assert_eq!(start["world"], "Daventry");
    assert!(carries_one(&start, "INVITEM_Ashes"));
    let mut points: Vec<Value> = Vec::new();
    for xy in [
        [158350, 302170],
        [158440, 300040],
        [152980, 301860],
        [152840, 302000],
        [151500, 303900],
    ] {
        points.push(json!({"name":format!("Exit the cave {}",points.len()+1),"xy":xy,"radius":155,"run":false}));
    }
    for xy in [
        [143400, 296500],
        [143400, 282900],
        [151400, 272500],
        [157000, 266400],
        [163800, 266400],
        [170800, 266400],
        [180600, 263700],
        [187800, 259300],
        [189400, 259300],
        [199000, 249700],
        [207000, 244900],
        [205000, 232500],
        [197000, 221300],
        [196600, 207700],
        [192200, 198100],
    ] {
        points.push(json!({"name":format!("Return road {}",points.len()+1),"xy":xy,"radius":275,"run":true}));
    }
    points.push(json!({"name":"Bring the Ashes to the Wizard","xy":[191980,198274],"radius":210,"run":false}));
    let path = play.root.join("homeward.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (near, viewer) = leg(
        &play,
        1,
        "WizardWithAshes",
        1750,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Bring the Ashes to the Wizard"));
    assert!(carries_one(&near, "INVITEM_Ashes"));
    let aim = play.root.join("offer.json");
    let at = &near["position"];
    fs::write(&aim,serde_json::to_vec(&json!({"start":16,"landmarks":[{"name":"Offer the last ingredient","xy":[at[0],at[1]],"radius":250,"aim":"Wizard"}]})).unwrap()).unwrap();
    let (cast, viewer) = leg(
        &play,
        2,
        "RingOfLight",
        1050,
        &[
            "--key",
            "left:IDCTL_INVBAR_QUEST2,8",
            "--walk-route",
            aim.to_str().unwrap(),
        ],
    );
    assert!(viewer.lines().any(|line| line
        .contains("test --aim Wizard (camera through gameplay pixel")
        && line.contains("hits Wizard first")));
    let doors = log(&play, "RingOfLight", "doors.log");
    assert_eq!(doors.matches("INVITEM_RingOfLight 0 -> 1").count(), 1);
    for stage in 1..=4 {
        assert_eq!(
            doors
                .matches(&format!("movement 'castring{stage}' ended: tells Wizard"))
                .count(),
            1
        );
    }
    assert_eq!(
        doors
            .matches("KQStatsChangeRequestEvent 0 0 0 0 50 to Connor (sent by wizard.cs line 331)")
            .count(),
        1
    );
    assert!(doors.contains("wizard.cs line 333: set Daventry::Wizard::metConnor"));
    assert!(
        log(&play, "RingOfLight", "conversation.log").contains("the conversation with Wizard ends")
    );
    let (restored, _) = leg(&play, 3, "RingReload", 55, &[]);
    assert_eq!(restored["journey"]["stats"], cast["journey"]["stats"]);
    for state in [&cast, &restored] {
        assert!(carries_one(state, "INVITEM_RingOfLight"));
        assert!(!state["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == "INVITEM_Ashes"));
        assert!(hero_flag(state, "Daventry::Wizard::metConnor", "3"));
        for f in [
            "Daventry::Wizard::hasCandle",
            "Daventry::Wizard::hasRing",
            "Daventry::Wizard::hasTorch",
        ] {
            assert!(hero_flag(state, f, "true"));
        }
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-4.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_RING_OF_LIGHT_SLOT"]
fn ring_opens_dead_city_and_arrival_survives_reload() {
    enter_dead_city();
}

pub(super) fn enter_dead_city() -> PathBuf {
    let play = Play::new("earned-dead-city-campaign");
    let source = earned_slot("KQ8_RING_OF_LIGHT_SLOT").unwrap_or_else(earn_ring);
    let first = prepare(&play, &source);
    assert_eq!(first["label"], "RingReload");
    assert!(carries_one(&first, "INVITEM_RingOfLight"));
    assert!(hero_flag(&first, "Daventry::BeenThere::DoneThat", "0"));
    let mut points = Vec::new();
    for xy in [
        [199860, 186440],
        [200000, 177060],
        [200560, 176500],
        [221560, 177060],
        [226380, 181750],
    ] {
        points.push(json!({"name":format!("Approach the tomb {}",points.len()+1),"xy":xy,"radius":275,"run":true}));
    }
    for xy in [
        [227020, 182520],
        [227300, 182660],
        [228040, 183120],
        [233600, 183120],
    ] {
        points.push(json!({"name":format!("Inside the tomb {}",points.len()+1),"xy":xy,"radius":145,"run":false}));
    }
    let path = play.root.join("ring-to-tomb.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({"start":3,"landmarks":points})).unwrap(),
    )
    .unwrap();
    let (tomb, viewer) = leg(
        &play,
        1,
        "AtDeathGate",
        920,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Inside the tomb 9 on update"));
    assert!(hero_flag(&tomb, "Daventry::BeenThere::DoneThat", "0"));
    let ring = tomb["journey"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "INVITEM_RingOfLight" && item["place"]["Carried"] == "Connor")
        .unwrap();
    let ring_name = ring["name"].as_str().unwrap();
    let path = play.root.join("shadow-gate.json");
    let gate = json!({"start":3,"landmarks":[{"name":"Enter the gate with the Ring of Light","xy":[235180,183080],"radius":110,
        "until_var":{"name":"Daventry::BeenThere::DoneThat","value":"1"}}]});
    fs::write(&path, serde_json::to_vec(&gate).unwrap()).unwrap();
    let (arrived, viewer) = leg(
        &play,
        2,
        "ReachedDeadCity",
        430,
        &[
            "--walk-route",
            path.to_str().unwrap(),
            "--key",
            "popup:yes,3",
        ],
    );
    assert_eq!(viewer.matches("trap sdwtrap1: On Connor at").count(), 1);
    assert!(viewer.contains("davgate4 gets Accept"));
    assert!(viewer.contains("world read: DeadCity"));
    let doors = log(&play, "ReachedDeadCity", "doors.log");
    assert_eq!(
        doors
            .matches(&format!(
                "shadbat.cs line 10: deleteObject {ring_name}: item gone"
            ))
            .count(),
        1
    );
    let (restored, _) = leg(&play, 3, "DeadCityReload", 60, &[]);
    for state in [&arrived, &restored] {
        assert_eq!(state["world"], "DeadCity");
        assert!(hero_flag(state, "Daventry::BeenThere::DoneThat", "1"));
        assert!(!state["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["kind"] == "INVITEM_RingOfLight"));
        assert!(state["kept"]["deadcity"]["vars"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry[0] == "DeadCity::Shadow::Alive" && entry[1] == "0"));
        assert!(
            carries_one(state, "INVITEM_RopeAndHook") && carries_one(state, "INVITEM_SmCrossbow")
        );
    }
    assert_eq!(restored["journey"]["stats"], arrived["journey"]["stats"]);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-4.json")
}
