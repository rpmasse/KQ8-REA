use super::{carries_one, check_mill_rope_save, check_saved_church_climb, earned_slot, hero_flag, Play};
use serde_json::Value;
use std::{fs, path::{Path, PathBuf}};

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_CHURCH_ROOF_SLOT"]
fn earned_roof_to_candle_and_fresh_reload() {
    earn_candle();
}

fn earn_candle() -> PathBuf {
    let play = Play::new("church-candle-campaign");
    let source = earned_slot("KQ8_CHURCH_ROOF_SLOT").unwrap_or_else(|| check_saved_church_climb(false));
    fs::create_dir_all(&play.slots).unwrap();
    fs::copy(source, play.slots.join("slot-1.json")).unwrap();
    let roof = play.slot(1);
    assert_eq!(roof["label"], "ChurchRoof");
    check_mill_rope_save(&roof);
    assert!(!carries_one(&roof, "INVITEM_Candle"));

    let routes = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes");
    let (walk, _) = play.run("cross", &[
        "--menu-input", "down,enter,release,enter,release", "--walk-route", routes.join("daventry_church_roof.json").to_str().unwrap(),
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=InsideLedge:enter:release:escape,225",
        "--key", "menu:up:enter:release:left:enter:release,240", "--frames", "2250",
    ]);
    assert!(walk.contains("walk-route at Stop on the roof above the west aisle"));
    let ledge = play.slot(2);
    assert_eq!(ledge["label"], "InsideLedge");
    assert_eq!(ledge["rope_place"]["settings"]["status"], "Top");
    assert_eq!(ledge["rope_place"]["settings"]["place"], serde_json::json!([201745.0,183184.0,13445.0]));
    check_mill_rope_save(&ledge);

    play.run("descend", &[
        "--menu-input", "down,enter,release,down,enter,release", "--key", "left:IDCTL_INVBAR_ROPE,8",
        "--aim-point", "201745,183184,12500,14", "--hold", "-1,0,0,160,239",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchInside:enter:release:escape,400",
        "--key", "menu:up:enter:release:left:enter:release,415", "--frames", "3500",
    ]);
    let descent = fs::read_to_string(play.root.join("descend/kq8-artifacts/logs/rope.log")).unwrap();
    assert!(descent.contains("rope state: starts from the top"));
    assert_eq!(descent.matches("climb: reached the bottom end").count(), 1);
    assert_eq!(descent.matches("rope state: over (rule 26)").count(), 1);
    let inside = play.slot(3);
    assert_eq!(inside["label"], "ChurchInside");
    check_mill_rope_save(&inside);
    assert!(inside["position"][2].as_f64().unwrap() < ledge["position"][2].as_f64().unwrap() - 4000.0);

    let (viewer, items) = play.run("candle", &[
        "--menu-input", "down,enter,release,down,down,enter,release", "--walk-route", routes.join("daventry_church_candle.json").to_str().unwrap(),
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchCandle:enter:release:escape,320",
        "--key", "menu:up:enter:release:left:enter:release,335", "--frames", "3500",
    ]);
    assert!(viewer.lines().any(|line| line.contains("test --aim candle (camera through gameplay pixel") && line.contains("hits candle first")));
    assert_eq!(items.matches("INVITEM_Candle 0 -> 1").count(), 1);
    assert!(items.contains("INVITEM_Candle added 0.3 s ago: +25 experience"));
    let before = play.slot(4);
    assert_eq!(before["label"], "ChurchCandle");
    check_candle(&before);
    if before["journey"]["stats"]["level"] != inside["journey"]["stats"]["level"] {
        let gui = fs::read_to_string(play.root.join("candle/kq8-artifacts/logs/gui.log")).unwrap();
        assert!(gui.contains("level-up flame starts and LEVEL.AUD plays (17 frames, 100 ms each)"));
        for effect in ["the level-up flame", "armour number's flash", "hand weapon number's flash", "ranged weapon number's flash"] {
            let prefix = format!("{effect} played out after ");
            let time = gui.lines().find_map(|line| line.strip_prefix(&prefix)).unwrap();
            let (elapsed, art) = time.split_once(" ms ").unwrap();
            assert!(elapsed.parse::<u64>().unwrap() >= 1800, "{effect} finished at {elapsed} ms");
            assert_eq!(art, "(17 frames, 100 ms each)");
        }
    }

    let (viewer, items) = play.run("reload", &[
        "--menu-input", "down,enter,release,down,down,down,enter,release",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=CandleReload:enter:release:escape,30",
        "--key", "menu:up:enter:release:left:enter:release,45", "--frames", "800",
    ]);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    assert!(!items.contains("INVITEM_Candle 0 -> 1"));
    let after = play.slot(5);
    assert_eq!(after["label"], "CandleReload");
    check_candle(&after);
    assert_eq!(after["journey"]["stats"], before["journey"]["stats"]);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
    play.slots.join("slot-5.json")
}

fn check_candle(save: &Value) {
    check_mill_rope_save(save);
    assert!(carries_one(save, "INVITEM_Candle"));
    assert!(hero_flag(save, "Daventry::candle::full", "false"));
    assert!(save["kept"]["daventry"]["gone"].as_array().unwrap().iter().any(|name| name == "candle"));
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_CHURCH_CANDLE_SLOT"]
fn earned_candle_to_urn_and_open_tomb_door() {
    let play = Play::new("church-urn-campaign");
    let source = earned_slot("KQ8_CHURCH_CANDLE_SLOT").unwrap_or_else(earn_candle);
    fs::create_dir_all(&play.slots).unwrap();
    fs::copy(source, play.slots.join("slot-1.json")).unwrap();
    let before = play.slot(1);
    assert_eq!(before["label"], "CandleReload");
    check_candle(&before);
    assert!(hero_flag(&before, "Daventry::Church::urn", "0"));
    let route = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/routes/daventry_church_urn.json");
    let (viewer, _) = play.run("push", &[
        "--menu-input", "down,enter,release,enter,release", "--walk-route", route.to_str().unwrap(),
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchUrn:enter:release:escape,760",
        "--key", "menu:up:enter:release:left:enter:release,775", "--frames", "5500",
    ]);
    assert!(viewer.contains("walk-route at Push the church urn"));
    let log = fs::read_to_string(play.root.join("push/kq8-artifacts/logs/doors.log")).unwrap();
    assert!(log.contains("urn1.cs line 16: set Daventry::Church::urn"));
    assert_eq!(log.matches("door tombdor: movement 'open' finished").count(), 1);
    assert!(log.contains("urn1.cs line 33: sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 35"));
    let saved = play.slot(2);
    assert_eq!(saved["label"], "ChurchUrn");
    check_open_tomb(&saved);
    assert_eq!(saved["journey"]["stats"]["stats"][4].as_i64(), before["journey"]["stats"]["stats"][4].as_i64().map(|xp| xp + 35));
    let (reload, _) = play.run("reload", &[
        "--menu-input", "down,enter,release,down,enter,release",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ChurchUrnReload:enter:release:escape,30",
        "--key", "menu:up:enter:release:left:enter:release,45", "--frames", "800",
    ]);
    assert!(reload.contains("save restored in Daventry without replaying Begin"));
    let restored = play.slot(3);
    assert_eq!(restored["label"], "ChurchUrnReload");
    check_open_tomb(&restored);
    assert_eq!(restored["journey"]["stats"], saved["journey"]["stats"]);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

fn check_open_tomb(save: &Value) {
    check_candle(save);
    assert!(hero_flag(save, "Daventry::Church::urn", "1"));
    let region = &save["kept"]["daventry"];
    let door = &region["interactions"]["doors"]["tombdor"];
    assert_eq!(door["state"], "Opened");
    assert_eq!(door["collision_type"].as_u64().unwrap() & 32, 0);
    let urn = region["states"].as_array().unwrap().iter().find(|obj| obj["name"] == "urn1").unwrap();
    assert_eq!(urn["placement"]["position"], serde_json::json!([216850.0,185249.0,8370.0]));
}
