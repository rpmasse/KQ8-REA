//! One headless original-data check of the ending's retained game lifetime.
//! The established decoder/native Credits checks are reused, not replayed.
use super::*;
use crate::{Shared, Stats, door_view::DoorSim, log::Log, world_build};
use kq8_core::{
    active::Active,
    stats::{ConnorStats, Stats as ConnorValues},
};
use kq8_data::{VolArchive, world_script::Vars};
use std::{sync::Mutex, time::Duration};

fn set_score(world: &mut World, level: i64, experience: i64) {
    world.resource_mut::<DoorSim>().active.world.connor_stats = Some(ConnorStats {
        level,
        stats: ConnorValues([0, 0, 0, 0, experience]),
    });
}

fn reach_final_card(world: &mut World, expected: i64) {
    for expected_index in 1..25 {
        assert!(
            world
                .resource::<Menu>()
                .credits
                .as_ref()
                .unwrap()
                .score()
                .is_none()
        );
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(60));
        update_credits(world, None);
        assert_eq!(
            world.resource::<Menu>().credits.as_ref().unwrap().index,
            expected_index
        );
    }
    assert_eq!(
        world.resource::<Menu>().credits.as_ref().unwrap().score(),
        Some(expected)
    );
}

fn exit_credits(world: &mut World) {
    assert!(!update_credits(world, Some(Key::Escape)));
    world
        .resource_mut::<Time<Real>>()
        .advance_by(Duration::from_secs(1));
    assert!(update_credits(world, None));
    assert!(world.resource::<Menu>().credits.is_none());
    assert!(!world.resource::<Menu>().state.has_game());
    assert_eq!(
        world
            .resource_mut::<Menu>()
            .state
            .input(Key::Escape, &[[0; 4]; 7]),
        Action::Stay
    );
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; headless original ending lifetime matrix"]
fn ending_lifetime_grouped_retention_movie_queue_score_and_replacement() {
    let game_path = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
    let game_path = std::path::PathBuf::from(game_path);
    let game = GameDir::open(&game_path).unwrap();
    let vol = VolArchive::open(&game.volume_path("temple4", "kq").unwrap()).unwrap();
    let script = vol.unpack(vol.find("vortrap.cs").unwrap()).unwrap().data;
    let mut active = Active::new(
        Vars::default(),
        vec![],
        vec![],
        ["Connor".into()],
        [("vortrap.cs".into(), script)],
    );
    let args = ["vortrap", "Cue", "300", "0", "c_imdone.kqs"].map(str::to_owned);
    assert!(active.scripts.call("vortrap.cs", &args, &mut active.world));
    let mut requests = std::mem::take(&mut active.world.presentations);
    assert_eq!(requests.pop_front(), Some(credits::Presentation::EndGame));
    let Some(credits::Presentation::Movie(movie)) = requests.pop_front() else {
        panic!("ending movie request")
    };
    assert!(movie.eq_ignore_ascii_case("w32ndg_1.dll"));
    assert_eq!(requests.pop_front(), Some(credits::Presentation::Credits));
    assert!(requests.is_empty());

    let mut app = App::new();
    app.add_systems(DisplayMenu, show);
    app.insert_resource(Source {
        game: game_path.clone(),
        sound: false,
    });
    app.insert_resource(Preferences::load(&game_path));
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Time<Real>>();
    app.init_resource::<PresentationQueue>();
    app.init_resource::<world_build::WorldChange>();
    app.insert_resource(crate::connor_view::StartAt(None));
    app.insert_resource(Shared {
        log: Arc::new(Mutex::new(Log::create("ending-lifetime").unwrap())),
        stats: Arc::new(Mutex::new(Stats::default())),
    });
    app.insert_resource(DoorSim::for_test(
        active,
        Log::create("ending-lifetime-active").unwrap(),
    ));
    let world = app.world_mut();
    {
        let mut state = world.resource_mut::<world_build::WorldChange>();
        state.current = "Temple4".into();
        state.generation = 1;
        state.game_started = true;
        state.gameplay_available = true;
    }
    set_score(world, 16, 7);
    assert!(gameplay_available(world));
    let retained = world.spawn(world_build::WorldEntity).id();
    let mut simulation = Schedule::default();
    simulation
        .add_systems((|mut sim: ResMut<DoorSim>| sim.active.update()).run_if(world_build::playing));
    simulation.run(world);
    let stopped_at = world.resource::<DoorSim>().active.world.updates;
    assert_eq!(
        stopped_at, 1,
        "the production run condition permits the started game"
    );

    // Consume the real final callback's first request. The actual movie
    // decoder is intentionally not invoked by this lifetime check.
    world
        .resource_mut::<PresentationQueue>()
        .0
        .push_back(credits::Presentation::EndGame);
    present_next(world);
    assert!(world.resource::<world_build::WorldChange>().game_started);
    assert!(
        !world
            .resource::<world_build::WorldChange>()
            .gameplay_available
    );
    assert!(!world.resource::<Menu>().state.has_game());
    assert!(!gameplay_available(world));
    assert!(world.get_entity(retained).is_ok());
    simulation.run(world);
    let menu = world.remove_resource::<Menu>().unwrap();
    simulation.run(world);
    assert_eq!(
        world.resource::<DoorSim>().active.world.updates,
        stopped_at,
        "killGame blocks simulation even if a menu is temporarily absent"
    );
    world.insert_resource(menu);

    // Feed the production movie-return bridge its completed-player state.
    // It must continue straight to Credits while preserving the game.
    world.resource_mut::<Menu>().movie = Some(MoviePage {
        name: movie,
        after: MovieReturn::Menu,
        playback: Err("headless completed-player fixture; decoder tested separately".into()),
        cursor_visible: None,
    });
    world
        .resource_mut::<PresentationQueue>()
        .0
        .push_back(credits::Presentation::Credits);
    leave_movie(world, "headless completion fixture");
    assert!(world.resource::<PresentationQueue>().0.is_empty());
    assert!(world.resource::<Menu>().movie.is_none());
    assert_eq!(world.resource::<Menu>().credits.as_ref().unwrap().index, 0);
    assert!(
        world
            .resource::<Menu>()
            .credits
            .as_ref()
            .unwrap()
            .score()
            .is_none()
    );
    // Change Connor after the ending/movie/credits request. Only final-card
    // preparation reads the live object; there is no early snapshot.
    set_score(world, 17, 7);
    reach_final_card(world, 463_837);
    exit_credits(world);
    assert!(world.get_entity(retained).is_ok());
    assert_eq!(world.resource::<DoorSim>().active.world.updates, stopped_at);

    // Main -> Credits again remains a completed, non-resumable game.
    open_ending_credits(world).unwrap();
    set_score(world, 17, 11);
    reach_final_card(world, 463_841);
    exit_credits(world);

    // A missing filename returns synchronously to the following request.
    // No game asset is hidden, changed or copied for this failure case.
    world.resource_mut::<PresentationQueue>().0.extend([
        credits::Presentation::Movie("missing-ending-lifetime-fixture.dll".into()),
        credits::Presentation::Credits,
    ]);
    present_next(world);
    assert!(world.resource::<Menu>().movie.is_none());
    assert_eq!(world.resource::<Menu>().credits.as_ref().unwrap().index, 0);
    assert!(world.resource::<PresentationQueue>().0.is_empty());
    exit_credits(world);

    // A title session that has never started is different even if an old
    // Connor is present in this deliberately adversarial fixture.
    world
        .resource_mut::<world_build::WorldChange>()
        .game_started = false;
    open_ending_credits(world).unwrap();
    reach_final_card(world, 0);
    exit_credits(world);
    world
        .resource_mut::<world_build::WorldChange>()
        .game_started = true;

    // Use the real confirmed-New path to destroy the retained completed
    // game. Its loading worker is not advanced; this checks ownership of
    // cleanup, not another full Daventry load.
    close_menu(world, false);
    world_build::first_world(world);
    assert!(world.get_entity(retained).is_err());
    assert!(!world.contains_resource::<DoorSim>());
    let state = world.resource::<world_build::WorldChange>();
    assert!(!state.game_started && !state.gameplay_available);
    assert!(state.current.is_empty());
    assert!(!world.contains_resource::<Menu>());
    println!(
        "ENDING LIFETIME PASS: original callback order, retained Connor/world, paused simulation, movie-return queue, live final score/reentry, missing movie continuation, never-started zero, confirmed-New cleanup; no window or decoder launch"
    );
}
