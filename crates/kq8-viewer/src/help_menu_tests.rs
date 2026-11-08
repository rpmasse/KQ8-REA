//! Headless help/Main/Return integration, using original message/layout data.
//! The outer ConnorSim admission gate and GPU capture are not exercised:
//! the actual menu constructor is called after popup_step requests Main.
use super::*;
use crate::{gui_view::{self, Gui, KeyAt}, door_view::DoorSim, log::Log, Shared, Stats, world_build};
use kq8_core::active::Active;
use std::{sync::Mutex, time::{Duration, Instant}};

fn tick_help(app: &mut App) { app.world_mut().run_schedule(Update); }
fn key(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);
    tick_help(app);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
}
fn popup_key(app: &mut App, word: &str) {
    app.world_mut().resource_mut::<KeyAt>().0.push((format!("popup:{word}"), 0));
    tick_help(app);
}
fn navigation_sounds(app: &mut App, steps: &[(&str, usize, bool)]) {
    for &(word, highlight, changes) in steps {
        app.world_mut().resource_mut::<Gui>().panels.sounds.clear();
        popup_key(app, word);
        let gui = app.world().resource::<Gui>();
        assert_eq!(gui.popup.as_ref().unwrap().highlight, highlight);
        let expected: &[&str] = if changes { &["button1.aud"] } else { &[] };
        assert_eq!(gui.panels.sounds.as_slice(), expected, "{word}: cue only for a changed highlight");
    }
}
fn clock(app: &App) -> f32 { app.world().resource::<DoorSim>().active.world.now() as f32 }
fn run_to(app: &mut App, simulation: &mut Schedule, time: f32) {
    for _ in 0..200 {
        if clock(app) >= time { return; }
        simulation.run(app.world_mut());
        tick_help(app);
    }
    panic!("simulation did not reach {time}; clock {}", clock(app));
}
fn request(app: &mut App, case: u8) -> f32 {
    let now = clock(app);
    assert_eq!(app.world_mut().resource_mut::<DoorSim>().active.console_command(
        &format!("doPopup {case}"), "help menu matrix"), Some("0".into()));
    now + 3.
}
fn main_pause_return(app: &mut App, simulation: &mut Schedule) {
    key(app, KeyCode::Escape);
    assert!(app.world().resource::<Gui>().popup.is_none());
    assert!(app.world().resource::<MenuRequest>().0);
    // The constructor used for Main retains the live game. The omitted
    // outer opener consumes this request and performs Connor/GPU checks.
    app.world_mut().resource_mut::<MenuRequest>().0 = false;
    return_to_title(app.world_mut()).unwrap();
    assert!(app.world().resource::<Menu>().state.has_game());
    let before = (clock(app), app.world().resource::<DoorSim>().active.world.updates);
    for _ in 0..20 {
        app.world_mut().resource_mut::<Time<Real>>().advance_by(Duration::from_secs(30));
        simulation.run(app.world_mut());
        tick_help(app);
    }
    assert_eq!((clock(app), app.world().resource::<DoorSim>().active.world.updates), before);
    {
        let mut menu = app.world_mut().resource_mut::<Menu>();
        menu.shown = Some(Instant::now() - Duration::from_secs(1));
        menu.keys.push_back(Key::Enter); // actual default Return -> Action::Resume
    }
    step(app.world_mut());
    assert!(!app.world().contains_resource::<Menu>());
    tick_help(app);
    assert!(app.world().resource::<Gui>().popup.is_none());
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and isolated KQ8_SAVE_DIR; original help/Main/Return matrix"]
fn help_escape_main_return_and_game_time_wait_matrix() {
    let path = std::path::PathBuf::from(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"));
    let game = GameDir::open(&path).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = Log::create("help-menu-matrix").unwrap();
    let gui = gui_view::load_gui(&game, &region, &mut log).unwrap();
    let bytes = std::fs::read(game.find("GAME/ENGLISH/500.MSG").unwrap()).unwrap();
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], []);
    active.world.connor_state = Some(kq8_core::character::state_number::WAIT);
    active.world.talk.book.add("GAME/ENGLISH", kq8_data::msg::MessageFile::parse(500, &bytes).unwrap(), 500);
    let mut app = App::new();
    app.add_systems(DisplayMenu, show);
    app.add_systems(Update, gui_view::popup_step.run_if(world_build::playing));
    app.insert_resource(Source { game: path.clone(), sound: false });
    let mut preferences = Preferences::load(&path);
    preferences.settings.help = true;
    app.insert_resource(preferences);
    app.insert_resource(Shared { log: Arc::new(Mutex::new(log)), stats: Arc::new(Mutex::new(Stats::default())) });
    app.insert_resource(gui);
    app.insert_resource(DoorSim::for_test(active, Log::create("help-menu-active").unwrap()));
    app.init_resource::<Assets<Image>>().init_resource::<Time<Real>>()
        .init_resource::<ButtonInput<KeyCode>>().init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<KeyAt>().init_resource::<MenuRequest>().init_resource::<world_build::WorldChange>();
    app.insert_resource(gui_view::ScreenFrame([0., 0., 640., 480.]));
    crate::keyboard_input::register(&mut app, &path);
    app.world_mut().spawn((Window { resolution: [640, 480].into(), ..default() }, PrimaryWindow));
    {
        let mut change = app.world_mut().resource_mut::<world_build::WorldChange>();
        change.current = "Daventry".into();
        change.game_started = true;
        change.gameplay_available = true;
    }
    let mut simulation = Schedule::default();
    simulation.add_systems((|mut sim: ResMut<DoorSim>| sim.active.update()).run_if(world_build::running));

    // Both popup kinds share the production navigation/sound path. A
    // minimal portal supplies state; its labels come from original 500.MSG.
    let section = kq8_data::kq_text::Section { name: "soundtest".into(),
        entries: vec![("classType".into(), "KQPortal".into())] };
    {
        let mut sim = app.world_mut().resource_mut::<DoorSim>();
        sim.active.world.portals.insert("soundtest".into(),
            kq8_data::magic_map::PortalKeys::from_section(&section).unwrap());
        assert_eq!(sim.active.console_command("KQMonster::setState Connor Portal soundtest",
            "popup sound matrix"), Some("0".into()));
    }
    tick_help(&mut app);
    navigation_sounds(&mut app, &[("left",0,false), ("right",1,true),
        ("right",1,false), ("left",0,true)]);
    popup_key(&mut app, "no");
    assert!(!app.world().resource::<DoorSim>().active.world.portal_asking());
    key(&mut app, KeyCode::F1);
    navigation_sounds(&mut app, &[("left",0,false), ("right",1,true),
        ("right",2,true), ("right",2,false), ("left",1,true), ("left",0,true)]);
    assert_eq!(app.world().resource::<Gui>().help_snapshot(), Some((127,Some(0),0)),
        "focus navigation keeps the displayed help page");
    popup_key(&mut app, "enter");

    let due = request(&mut app, 130);
    run_to(&mut app, &mut simulation, due);
    assert_eq!(app.world().resource::<Gui>().help_snapshot(), Some((130, None, 0)));
    let before = clock(&app);
    simulation.run(app.world_mut());
    assert_eq!(clock(&app), before, "the visible help panel holds world time");
    app.world_mut().resource_mut::<DoorSim>().active.world.connor_hands_off = true;
    key(&mut app, KeyCode::Escape);
    assert!(app.world().resource::<Gui>().popup.is_some());
    assert!(!app.world().resource::<MenuRequest>().0);
    app.world_mut().resource_mut::<DoorSim>().active.world.connor_hands_off = false;
    main_pause_return(&mut app, &mut simulation);
    key(&mut app, KeyCode::F1);
    popup_key(&mut app, "next");
    assert_eq!(app.world().resource::<Gui>().help_snapshot().unwrap().1, Some(1));
    main_pause_return(&mut app, &mut simulation);
    key(&mut app, KeyCode::F1);
    assert_eq!(app.world().resource::<Gui>().help_snapshot(), Some((127, Some(0), 0)));
    popup_key(&mut app, "enter");

    // Pause Main with an initial delay still outstanding; Return retains
    // its remaining game time, rather than spending ten wall-clock minutes.
    let due = request(&mut app, 135);
    run_to(&mut app, &mut simulation, due - 2.);
    key(&mut app, KeyCode::F1);
    main_pause_return(&mut app, &mut simulation);
    assert!(clock(&app) < due);
    assert!(app.world().resource::<Gui>().popup.is_none());
    run_to(&mut app, &mut simulation, due - 0.1);
    assert!(app.world().resource::<Gui>().popup.is_none());
    run_to(&mut app, &mut simulation, due);
    assert_eq!(app.world().resource::<Gui>().help_snapshot().unwrap().0, 135);
    popup_key(&mut app, "enter");

    // Then freeze a one-second busy retry in Main and resume that same wait.
    let due = request(&mut app, 140);
    app.world_mut().resource_mut::<DoorSim>().active.world.connor_hands_off = true;
    run_to(&mut app, &mut simulation, due);
    let retry = clock(&app) + 1.;
    assert!(app.world().resource::<Gui>().popup.is_none());
    app.world_mut().resource_mut::<DoorSim>().active.world.connor_hands_off = false;
    key(&mut app, KeyCode::F1);
    main_pause_return(&mut app, &mut simulation);
    run_to(&mut app, &mut simulation, retry - 0.1);
    assert!(app.world().resource::<Gui>().popup.is_none());
    run_to(&mut app, &mut simulation, retry);
    assert_eq!(app.world().resource::<Gui>().help_snapshot().unwrap().0, 140);
    println!("help/Main/Return PASS: Portal/help navigation cues and silent focus boundaries; actual popup Escape; Main Return action; fresh F1 focus; initial and retry game-clock waits; ConnorSim admission/GPU capture omitted");
}
