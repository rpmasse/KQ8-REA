//! Existing-file failure gate, using the production reader and menu update.
//! Successful AVI-clock, native movie and Credits checks are retained.
use super::*;
use crate::{Shared, Stats, door_view::DoorSim, log::Log, movie_player::Playback, world_build};
use kq8_core::active::Active;
use kq8_data::{avi::Movie, world_script::Vars};
use std::{
    os::windows::process::CommandExt,
    process::{Command as Process, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};

fn decoder(movie: Movie, complete_pictures: usize, trailing_bytes: usize, exit: u8) -> Playback {
    let bytes = movie.width as usize * movie.height as usize * 4;
    // This helper writes synthetic RGBA bytes, never original extracted media.
    // Each argument is passed directly to the process; there is no shell file.
    let program = format!(
        "$stream = [Console]::OpenStandardOutput(); $picture = New-Object byte[] {bytes}; \
         for ($i = 0; $i -lt {complete_pictures}; $i++) {{ $stream.Write($picture, 0, $picture.Length) }}; \
         if ({trailing_bytes} -gt 0) {{ $stream.Write($picture, 0, {trailing_bytes}) }}; \
         $stream.Flush(); exit {exit}"
    );
    let child = Process::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", &program])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(0x08000000)
        .spawn()
        .expect("controlled hidden decoder process");
    Playback::from_decoder(movie, child)
}

fn scene(game: &Path, playable: bool) -> App {
    let mut app = App::new();
    app.add_systems(DisplayMenu, show);
    app.insert_resource(Source { game: game.to_owned(), sound: false });
    app.insert_resource(Preferences::load(game));
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Time<Real>>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<PresentationQueue>();
    app.init_resource::<world_build::WorldChange>();
    app.insert_resource(Shared {
        log: Arc::new(Mutex::new(Log::create("movie-failure-matrix").unwrap())),
        stats: Arc::new(Mutex::new(Stats::default())),
    });
    app.insert_resource(DoorSim::for_test(
        Active::new(Vars::default(), vec![], vec![], ["Connor".into()], []),
        Log::create("movie-failure-active").unwrap(),
    ));
    let world = app.world_mut();
    {
        let mut state = world.resource_mut::<world_build::WorldChange>();
        state.current = "Temple4".into();
        state.game_started = true;
        state.gameplay_available = playable;
    }
    let mut menu = Menu::open(game, vec![], false, &mut world.resource::<Shared>().log.lock().unwrap()).unwrap();
    menu.state = session_menu_choice(world);
    world.insert_resource(menu);
    world.spawn((PrimaryWindow, bevy::window::CursorOptions { visible: false, ..default() }));
    world.run_schedule(DisplayMenu);
    app
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; grouped headless decoder failure and return matrix"]
fn movie_failures_acknowledge_and_preserve_return_ownership() {
    let game_path = std::path::PathBuf::from(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"));
    let game = GameDir::open(&game_path).unwrap();
    let bytes = std::fs::read(game.find("W32ndg_1.dll").unwrap()).unwrap();
    for (index, (after, count, tail, status, expected)) in [
        (MovieReturn::Title, 0, 0, 7, "ended after 0"),
        (MovieReturn::Menu, 1, 3, 0, "truncated an RGBA picture"),
        (MovieReturn::Gameplay, 1, 0, 0, "ended after 1"),
        (MovieReturn::Menu, 1, 0, 9, "exited with"),
    ].into_iter().enumerate() {
        let ending = index == 3;
        let mut movie = Movie::read(&bytes).unwrap();
        if ending {
            // Exercise the terminal status branch using a one-interval stream.
            // The full original stream clock already has its own passing test.
            movie.frames = 1;
            movie.holds.truncate(1);
        }
        let playback = decoder(movie, count, tail, status);
        let mut app = scene(&game_path, !ending);
        let world = app.world_mut();
        let rows = world.resource::<Menu>().art.rows;
        let retained = world.spawn(world_build::WorldEntity).id();
        let updates = world.resource::<DoorSim>().active.world.updates;
        world.resource_mut::<Menu>().movie = Some(MoviePage {
            name: "W32ndg_1.dll".into(), after, playback: Ok(playback), cursor_visible: Some(true),
        });
        if ending {
            world.resource_mut::<PresentationQueue>().0.push_back(credits::Presentation::Credits);
        }
        let timeout = Instant::now() + Duration::from_secs(15);
        let mut observed_picture = false;
        loop {
            if let Ok(player) = &mut world.resource_mut::<Menu>().movie.as_mut().unwrap().playback {
                if player.started.is_some() {
                    observed_picture = true;
                    player.started = Some(Instant::now() - Duration::from_secs(120));
                }
            }
            play_movie_frame(world);
            let menu = world.resource::<Menu>();
            if let Err(message) = &menu.movie.as_ref().unwrap().playback {
                assert!(message.contains(expected), "case {index}: {message}");
                let image = world.resource::<Assets<Image>>().get(&menu.image).unwrap();
                assert_eq!(image.data.as_ref().unwrap(), &menu.art.movie_screen(Some("W32ndg_1.dll")).rgba);
                break;
            }
            assert!(Instant::now() < timeout, "case {index}: decoder failure did not arrive");
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(observed_picture, count != 0, "case {index}: actual picture delivery");
        // Failure waits for acknowledgement, without consuming Credits or
        // exposing a gameplay frame. Space is not an acknowledgement.
        world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
        play_movie_frame(world);
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        assert!(world.resource::<Menu>().movie.as_ref().unwrap().playback.is_err());
        assert_eq!(world.resource::<PresentationQueue>().0.len(), usize::from(ending));
        match index {
            0 => world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter),
            1 => world.resource_mut::<Menu>().keys.push_back(Key::Escape),
            2 => world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape),
            _ => world.resource_mut::<Menu>().keys.push_back(Key::Enter),
        }
        play_movie_frame(world);
        assert!(world.get_entity(retained).is_ok());
        assert_eq!(world.resource::<DoorSim>().active.world.updates, updates);
        assert!(world.resource::<PresentationQueue>().0.is_empty());
        assert!(world.query_filtered::<&bevy::window::CursorOptions, With<PrimaryWindow>>().single(world).unwrap().visible);
        if matches!(after, MovieReturn::Gameplay) {
            assert!(!world.contains_resource::<Menu>());
            assert!(gameplay_available(world));
        } else {
            let menu = world.resource::<Menu>();
            assert!(menu.movie.is_none());
            assert_eq!(menu.credits.is_some(), ending);
            assert_eq!(gameplay_available(world), !ending);
            if ending {
                assert_eq!(menu.credits.as_ref().unwrap().index, 0);
            }
        }
        // Ending state controls the original New warning, independently of
        // whether retained Connor/world resources still exist.
        let mut choice = session_menu_choice(world);
        let [left, top, right, bottom] = rows[0];
        assert_eq!(choice.input(Key::Click((left + right) / 2, (top + bottom) / 2), &rows), Action::Stay);
        assert_eq!(choice.input(Key::Enter, &rows), Action::Stay);
        assert_eq!(choice.input(Key::Release, &rows), if ending { Action::Start(2) } else { Action::Stay },
            "ended sessions go straight to Normal; playable sessions first default to Cancel restart");
        println!("MOVIE FAILURE case {index} PASS: {expected}; picture={observed_picture}; return={after:?}; ending={ending}");
    }
}
