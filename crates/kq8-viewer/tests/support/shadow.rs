//! Continue from an earned urn slot through both ropes, the tomb and the Wizard.
use super::{
    carries_one,
    church::{check_open_tomb, earn_open_tomb},
    earned_slot, hero_flag, Play,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn prepare(play: &Play, source: &Path) -> Value {
    fs::create_dir_all(&play.slots).unwrap();
    fs::copy(source, play.slots.join("slot-1.json")).unwrap();
    play.slot(1)
}

fn route(play: &Play, part: &str) -> PathBuf {
    let routes: Value =
        serde_json::from_str(include_str!("../routes/daventry_shadow.json")).unwrap();
    let path = play.root.join(format!("{part}.json"));
    fs::write(
        &path,
        serde_json::to_vec(routes.get(part).unwrap()).unwrap(),
    )
    .unwrap();
    path
}

/// Load the previous slot from the title, perform ordinary inputs and write
/// a new slot using the pause menu. Each call starts a fresh Windows process.
fn leg(play: &Play, slot: u32, label: &str, tick: u32, inputs: &[&str]) -> (Value, String) {
    let select = format!(
        "down,enter,release,{}enter,release",
        "down,".repeat((slot - 1) as usize)
    );
    let save = format!("menu:down:down:down:enter:release:enter:release:escape:type={label}:enter:release:escape,{tick}");
    let quit = format!("menu:up:enter:release:left:enter:release,{}", tick + 15);
    let limit = (tick * 8).max(900).to_string();
    let mut args = vec!["--menu-input", &select];
    args.extend_from_slice(inputs);
    args.extend(["--key", &save, "--key", &quit, "--frames", &limit]);
    let (viewer, _) = play.run(label, &args);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    let saved = play.slot(slot + 1);
    assert_eq!(saved["label"], label);
    assert!(saved["journey"]["stats"]["stats"][0].as_i64().unwrap() > 0);
    assert_eq!(saved["journey"]["god_mode"], false);
    assert_eq!(saved["journey"]["ai_off"], false);
    (saved, viewer)
}

fn log(play: &Play, part: &str, file: &str) -> String {
    fs::read_to_string(play.root.join(part).join("kq8-artifacts/logs").join(file)).unwrap()
}

fn check_shadow(save: &Value) {
    check_open_tomb(save);
    assert!(hero_flag(save, "Daventry::ShadowMonster::metConnor", "1"));
    assert!(hero_flag(save, "Daventry::BeenThere::DoneThat", "0"));
    assert!(!carries_one(save, "INVITEM_RingOfLight"));
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_CHURCH_URN_SLOT"]
fn urn_to_archer_and_shadow() {
    earn_shadow();
}

fn earn_shadow() -> PathBuf {
    let play = Play::new("daventry-shadow-campaign");
    let source = earned_slot("KQ8_CHURCH_URN_SLOT").unwrap_or_else(earn_open_tomb);
    let first = prepare(&play, &source);
    assert_eq!(first["label"], "ChurchUrnReload");
    check_open_tomb(&first);
    assert!(hero_flag(&first, "Daventry::ShadowMonster::metConnor", "0"));

    let approach = route(&play, "approach");
    let (foot, _) = leg(
        &play,
        1,
        "InwardFoot",
        450,
        &["--walk-route", approach.to_str().unwrap()],
    );
    assert_eq!(foot["rope_place"]["settings"]["status"], "Bottom");
    assert_eq!(
        foot["rope_place"]["settings"]["place"],
        serde_json::json!([201520.0, 183184.0, 8465.0])
    );
    let (roof, _) = leg(
        &play,
        2,
        "BackOnRoof",
        400,
        &[
            "--key",
            "left:IDCTL_INVBAR_ROPE,8",
            "--aim-point",
            "201520,183184,11000,14",
            "--hold",
            "1,0,0,150,232",
        ],
    );
    assert_eq!(
        log(&play, "BackOnRoof", "rope.log")
            .matches("climb: reached the top end")
            .count(),
        1
    );
    assert!(roof["rope_climb"].is_null());

    let roof_route = route(&play, "roof");
    let (ledge, _) = leg(
        &play,
        3,
        "OuterLedge",
        460,
        &["--walk-route", roof_route.to_str().unwrap()],
    );
    assert_eq!(ledge["rope_place"]["settings"]["status"], "Top");
    assert_eq!(
        ledge["rope_place"]["settings"]["place"],
        serde_json::json!([206337.0, 186393.0, 13285.0])
    );
    let (yard, _) = leg(
        &play,
        4,
        "OutOfChurch",
        400,
        &[
            "--key",
            "left:IDCTL_INVBAR_ROPE,8",
            "--aim-point",
            "206337,186393,12500,14",
            "--hold",
            "-1,0,0,160,241",
        ],
    );
    assert_eq!(
        log(&play, "OutOfChurch", "rope.log")
            .matches("climb: reached the bottom end")
            .count(),
        1
    );
    assert!(yard["rope_climb"].is_null());
    let tomb = route(&play, "tomb");
    let (shadow, viewer) = leg(
        &play,
        5,
        "ShadowMet",
        2400,
        &["--walk-route", tomb.to_str().unwrap()],
    );
    assert!(viewer.contains("walk-route at Meet the Shadow"));
    let fight = log(&play, "ShadowMet", "fight.log");
    assert!(
        fight.contains("sprgnbow: hit by Connor") && fight.contains("sprgnbow: dies (rule 30)")
    );
    assert_eq!(
        fight.matches("sprgnbow: its death ended (rule 32)").count(),
        1
    );
    let doors = log(&play, "ShadowMet", "doors.log");
    assert!(
        doors.contains("script shadow.cs Shadow Cue 10") && doors.contains("AnimList=Monst2.anm")
    );
    for n in 1..=12 {
        assert_eq!(
            doors
                .matches(&format!("movement 'CLSM1S{n}' ended: tells Shadow"))
                .count(),
            1
        );
    }
    assert_eq!(doors.matches("script shadow.cs Shadow Cue 20").count(), 1);
    assert!(doors.contains("shadow.cs line 16: set Daventry::ShadowMonster::metConnor"));
    assert_eq!(
        doors
            .matches("KQStatsChangeRequestEvent 0 0 0 0 20 to Connor (sent by shadow.cs")
            .count(),
        1
    );
    assert!(doors.contains("script shadow.cs Shadow Cue 25"));
    assert!(doors.contains("shadow.cs line 32: handsOff 0"));
    check_shadow(&shadow);
    assert_eq!(dropped_crossbow(&shadow)["quantity"], 1);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-6.json")
}

fn dropped_crossbow(save: &Value) -> &Value {
    let mut drops =
        save["items"].as_array().unwrap().iter().filter(|item| {
            item["kind"] == "INVITEM_SmCrossbow" && item["place"]["World"].is_object()
        });
    let item = drops.next().expect("the archer dropped a crossbow");
    assert!(drops.next().is_none(), "one dropped crossbow");
    item
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_SHADOW_SLOT"]
fn shadow_crossbow_to_wizard_ingredients() {
    reach_wizard();
}

fn reach_wizard() -> PathBuf {
    let play = Play::new("shadow-wizard-campaign");
    let source = earned_slot("KQ8_SHADOW_SLOT").unwrap_or_else(earn_shadow);
    let first = prepare(&play, &source);
    assert_eq!(first["label"], "ShadowMet");
    check_shadow(&first);
    assert!(hero_flag(&first, "Daventry::Wizard::metConnor", "1"));
    let drop = dropped_crossbow(&first);
    let name = drop["name"].as_str().unwrap();
    let position = &drop["place"]["World"]["position"];
    let crossbow_route = serde_json::json!({"start":3,"landmarks":[
        {"name":"Walk back through the mausoleum","xy":[228040,183120],"radius":145},
        {"name":"Exit the open tomb door","xy":[227040,183080],"radius":150},
        {"name":"Take the archer's dropped crossbow","xy":[position[0].as_f64().unwrap()+470.0,position[1]],"radius":165,"aim":name}
    ]});
    let path = play.root.join("crossbow.json");
    fs::write(&path, serde_json::to_vec(&crossbow_route).unwrap()).unwrap();
    let (armed, viewer) = leg(
        &play,
        1,
        "EarnedCrossbow",
        450,
        &["--walk-route", path.to_str().unwrap()],
    );
    assert!(viewer.lines().any(|l| l
        .contains(&format!("test --aim {name} (camera through gameplay pixel"))
        && l.contains(&format!("hits {name} first"))));
    assert!(carries_one(&armed, "INVITEM_SmCrossbow"));
    let items = log(&play, "EarnedCrossbow", "items.log");
    assert_eq!(
        items
            .lines()
            .filter(|line| line.contains("inventory (picked up):")
                && line.contains("INVITEM_SmCrossbow 0 -> 1"))
            .count(),
        1
    );
    assert!(!log(&play, "EarnedCrossbow", "doors.log").contains("script shadow.cs Shadow Cue 10"));
    let home = route(&play, "wizard");
    let (wizard, viewer) = leg(
        &play,
        2,
        "WizardIngredients",
        2650,
        &["--walk-route", home.to_str().unwrap()],
    );
    assert!(viewer.lines().any(
        |l| l.contains("test --aim Wizard (camera through gameplay pixel")
            && l.contains("hits Wizard first")
    ));
    let doors = log(&play, "WizardIngredients", "doors.log");
    assert!(doors.contains("Wizard: TalkComplete 1000 4 14 30 5"));
    assert!(doors.contains("wizard.cs line 189: set Daventry::Wizard::metConnor"));
    assert!(log(&play, "WizardIngredients", "conversation.log")
        .contains("the conversation with Wizard ends"));
    check_shadow(&wizard);
    assert!(
        hero_flag(&wizard, "Daventry::Wizard::metConnor", "2")
            && carries_one(&wizard, "INVITEM_SmCrossbow")
    );
    for flag in [
        "Daventry::Wizard::hasCandle",
        "Daventry::Wizard::hasRing",
        "Daventry::Wizard::hasTorch",
    ] {
        assert!(hero_flag(&wizard, flag, "false"));
    }
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-3.json")
}

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_WIZARD_INGREDIENTS_SLOT"]
fn wizard_accepts_the_earned_ring_and_candle() {
    offer_ingredients();
}

fn offer_ingredients() -> PathBuf {
    let play = Play::new("wizard-deliveries");
    let source = earned_slot("KQ8_WIZARD_INGREDIENTS_SLOT").unwrap_or_else(reach_wizard);
    let before = prepare(&play, &source);
    assert_eq!(before["label"], "WizardIngredients");
    check_shadow(&before);
    assert!(hero_flag(&before, "Daventry::Wizard::metConnor", "2"));
    let mut current = before;
    for (slot, label, kind, flag, response) in [
        (
            1,
            "RingOffered",
            "INVITEM_RingOfDeadHero",
            "Daventry::Wizard::hasRing",
            "1000 4 2 15 2",
        ),
        (
            2,
            "CandleOffered",
            "INVITEM_Candle",
            "Daventry::Wizard::hasCandle",
            "1000 4 1 5 2",
        ),
    ] {
        let item = current["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["kind"] == kind && i["place"]["Carried"] == "Connor")
            .unwrap();
        let at = &current["position"];
        let aim = serde_json::json!({"start":16,"landmarks":[{"name":label,"xy":[at[0],at[1]],"radius":210,"aim":"Wizard"}]});
        let path = play.root.join(format!("{label}.json"));
        fs::write(&path, serde_json::to_vec(&aim).unwrap()).unwrap();
        // The earned quest strip contains Piece1, ring, Candle. Removing
        // the ring moves the Candle into its former position in the strip.
        let (saved, viewer) = leg(
            &play,
            slot,
            label,
            870,
            &[
                "--key",
                "left:IDCTL_INVBAR_QUEST2,8",
                "--walk-route",
                path.to_str().unwrap(),
            ],
        );
        assert!(viewer.lines().any(|l| l
            .contains("test --aim Wizard (camera through gameplay pixel")
            && l.contains("hits Wizard first")));
        let items = log(&play, label, "items.log");
        assert!(items.contains(&format!(
            "bar click left on {} ({kind}",
            item["name"].as_str().unwrap()
        )));
        assert!(hero_flag(&saved, flag, "true"));
        assert!(!saved["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == kind));
        assert!(
            log(&play, label, "doors.log").contains(&format!("Wizard: TalkComplete {response}"))
        );
        current = saved;
    }
    let (restored, _) = leg(&play, 3, "TwoIngredients", 40, &[]);
    assert!(
        hero_flag(&restored, "Daventry::Wizard::hasRing", "true")
            && hero_flag(&restored, "Daventry::Wizard::hasCandle", "true")
    );
    assert!(
        hero_flag(&restored, "Daventry::Wizard::hasTorch", "false")
            && hero_flag(&restored, "Daventry::Wizard::metConnor", "2")
    );
    assert!(
        carries_one(&restored, "INVITEM_SmCrossbow")
            && carries_one(&restored, "INVITEM_RopeAndHook")
    );
    for item in [
        "INVITEM_Candle",
        "INVITEM_RingOfDeadHero",
        "INVITEM_RingOfLight",
    ] {
        assert!(!restored["journey"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == item));
    }
    assert_eq!(restored["journey"]["stats"], current["journey"]["stats"]);
    assert_eq!(
        fs::read(play.game.join("Options.cs")).unwrap(),
        play.options
    );
    play.slots.join("slot-4.json")
}
