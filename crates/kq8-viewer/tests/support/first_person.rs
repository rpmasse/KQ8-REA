use super::{carries_one, earn_hero_ring, Play};
use std::{fs, path::PathBuf};

#[test]
#[ignore = "requires GOG assets and graphics; optional earned KQ8_CHURCH_YARD_SLOT"]
fn church_climb_restores_first_person_after_a_fresh_process_dismount() {
    super::check_saved_church_climb(true);
}

fn import_earned_ring(play: &Play) {
    let source = std::env::var_os("KQ8_HERO_RING_SLOT").filter(|v| !v.is_empty())
        .map(PathBuf::from).unwrap_or_else(earn_hero_ring);
    fs::create_dir_all(&play.slots).unwrap();
    fs::copy(source, play.slots.join("slot-1.json")).unwrap();
    assert_eq!(play.slot(1)["label"], "HeroRingReload");
}

#[test]
#[ignore = "requires GOG assets and graphics; KQ8_HERO_RING_SLOT repeats the Daventry leg"]
fn save_with_the_first_person_camera_while_a_script_owns_the_previous_model() {
    let play = Play::new("first-person-scripted-model");
    import_earned_ring(&play);
    let (viewer, _) = play.run("switch", &[
        "--menu-input", "down,enter,release,enter,release",
        "--special", "AnimList=conmill.anm;AnimListOn;axe,10", "--key", "physical:f2,15",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=MillAction:enter:release:escape,23",
        "--key", "menu:up:enter:release:left:enter:release,65", "--frames", "1400",
    ]);
    assert!(viewer.contains("CameraFirstPerson: first person true"));
    let saved = play.slot(2);
    assert_eq!(saved["view_flags"]["active"], true);
    assert_eq!(saved["weapon_state"]["actor_first_person"], false);
    assert_eq!(saved["special"]["movement"], "conmill.anm/axe");
    let (resume, _) = play.run("resume", &[
        "--menu-input", "down,enter,release,down,enter,release",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ResumedEye:enter:release:escape,75",
        "--key", "menu:up:enter:release:left:enter:release,85", "--frames", "1500",
    ]);
    assert!(resume.contains("save restored in Daventry without replaying Begin"));
    let completed = play.slot(3);
    assert!(completed["special"].is_null());
    assert_eq!(completed["view_flags"]["active"], true);
    assert_eq!(completed["weapon_state"]["actor_first_person"], true);
    assert_eq!(completed["weapon_state"]["weapon_set"], "connerFp.anm");
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets and graphics"]
fn change_view_while_drawing_the_crossbow_then_reload_mid_swap() {
    let play = Play::new("first-person-mid-swap");
    play.run("start", &[
        "--world", "DeadCity", "--no-ai", "--god", "--item", "INVITEM_SmCrossbow",
        "--command", "KQConner::setDoMode 1,25", "--key", "physical:f2,26",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=TurnDuringDraw:enter:release:escape,29",
        "--updates", "55", "--frames", "1300",
    ]);
    let first = play.slot(1);
    assert_eq!(first["view_flags"]["active"], true);
    assert_eq!(first["label"], "TurnDuringDraw");
    assert_eq!(first["special"]["movement"], "conner.anm/away");
    assert_eq!(first["special"]["wanted"], "away");
    assert!(first["special"]["phase"].as_f64().is_some_and(|p| p > 0. && p < 1.));
    assert_eq!(first["weapon_state"]["pending_set"][0], "Consbowfp.anm");
    assert_eq!(first["special"]["steps"][1]["Play"][0], "consbowfp.anm/out");
    let (viewer, _) = play.run("resume", &[
        "--menu-input", "down,enter,release,enter,release",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=BowDrawn:enter:release:escape,60",
        "--key", "menu:up:enter:release:left:enter:release,75", "--frames", "1400",
    ]);
    assert!(viewer.contains("save restored in DeadCity without replaying Begin"));
    let connor = fs::read_to_string(play.root.join("resume/kq8-artifacts/logs/connor.log")).unwrap();
    assert_eq!(connor.matches("weapon: the set switches now").count(), 1);
    assert!(connor.contains("consbowfp.anm/out"));
    let resumed = play.slot(2);
    assert_eq!(resumed["weapon_state"]["weapon_set"], "Consbowfp.anm");
    assert_eq!(resumed["weapon_state"]["readying"], false);
    assert_eq!(resumed["view_flags"]["active"], true);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets and graphics"]
fn original_first_person_crossbow_fires_and_resumes_in_third_person() {
    let play = Play::new("first-person-crossbow");
    let (viewer, _) = play.run("fire", &[
        "--world", "DeadCity", "--no-ai", "--god", "--item", "INVITEM_SmCrossbow",
        "--command", "move skeleton1 0 184500 43091 864 0 0 0,5",
        "--key", "physical:f2,15", "--command", "KQConner::setDoMode 1,25",
        "--aim-point", "184000,44000,1800,65",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=FirstBow:enter:release:escape,115",
        "--updates", "135", "--frames", "1500",
    ]);
    assert!(viewer.contains("CameraFirstPerson: first person true"));
    let saved = play.slot(1);
    assert_eq!(saved["label"], "FirstBow");
    assert_eq!(saved["weapon_state"]["weapon_set"], "Consbowfp.anm");
    assert_eq!(saved["journey"]["weapon_ready"], "INVITEM_SmCrossbow");
    assert_eq!(saved["view_flags"]["active"], true);
    assert_eq!(saved["bolts"]["bolts_shot"], 1);
    let (reload, _) = play.run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--key", "physical:f2,28",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ThirdBow:enter:release:escape,60",
        "--key", "menu:up:enter:release:left:enter:release,75", "--frames", "1400",
    ]);
    assert!(reload.contains("save restored in DeadCity without replaying Begin"));
    assert!(reload.contains("CameraFirstPerson: first person false"));
    let restored = play.slot(2);
    assert_eq!(restored["label"], "ThirdBow");
    assert_eq!(restored["view_flags"]["active"], false);
    assert_eq!(restored["weapon_state"]["weapon_set"], "conscbow.anm");
    assert_eq!(restored["bolts"]["bolts_shot"], 1);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}

#[test]
#[ignore = "requires GOG assets and graphics; KQ8_HERO_RING_SLOT repeats the Daventry leg"]
fn original_view_key_axe_attack_and_menu_save() {
    let play = Play::new("first-person-daventry");
    import_earned_ring(&play);
    let start = play.slot(1);
    assert_eq!(start["label"], "HeroRingReload");
    assert!(carries_one(&start, "INVITEM_SmallAx"));
    assert_eq!(start["journey"]["weapon_mode"], "None");
    let (viewer, _) = play.run("switch", &[
        "--menu-input", "down,enter,release,enter,release", "--key", "physical:f2,15",
        "--key", "physical:delete,40", "--swing", "coffnlid,100,big",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=FirstPerson:enter:release:escape,160",
        "--key", "menu:up:enter:release:left:enter:release,175", "--frames", "1600",
    ]);
    assert!(viewer.contains("CameraFirstPerson: first person true"));
    let fight = fs::read_to_string(play.root.join("switch/kq8-artifacts/logs/fight.log")).unwrap();
    assert!(fight.contains("first person true: bank Some(\"connerFp.anm\")"));
    assert!(fight.contains("Delete (Draw) pressed"));
    assert_eq!(fight.matches("Do (conaxefp.anm/attack2): hits object coffnlid").count(), 1);
    let first = play.slot(2);
    assert_eq!(first["label"], "FirstPerson");
    assert_eq!(first["view_flags"]["active"], true);
    assert_eq!(first["weapon_state"]["weapon_set"], "conAxeFp.anm");
    assert!(first["camera"]["first_person"].is_object());
    assert_eq!(first["journey"]["weapon_ready"], "INVITEM_SmallAx");
    let (viewer, _) = play.run("restore", &[
        "--menu-input", "down,enter,release,down,enter,release", "--key", "physical:f2,30",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=ThirdPerson:enter:release:escape,74",
        "--key", "menu:up:enter:release:left:enter:release,85", "--frames", "1500",
    ]);
    assert!(viewer.contains("save restored in Daventry without replaying Begin"));
    assert!(viewer.contains("CameraFirstPerson: first person false"));
    let restored = play.slot(3);
    assert_eq!(restored["label"], "ThirdPerson");
    assert_eq!(restored["view_flags"]["active"], false);
    assert!(restored["camera"]["first_person"].is_null());
    assert_eq!(restored["weapon_state"]["weapon_set"], "conAxe.anm");
    assert_eq!(restored["journey"]["weapon_ready"], "INVITEM_SmallAx");
    assert_eq!(restored["journey"]["god_mode"], false);
    assert_eq!(restored["journey"]["ai_off"], false);
    assert_eq!(fs::read(play.game.join("Options.cs")).unwrap(), play.options);
}
