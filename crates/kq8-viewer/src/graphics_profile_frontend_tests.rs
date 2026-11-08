//! Real original-menu dispatch with supplied host replies, never a claim
//! of native exclusive acceptance. The unattended integration test owns it.
use super::*;
use crate::{
    graphics_runtime::{self, HostPresentation, HostState, Presentation, Profile, Runtime},
    Shared, Stats,
};
use bevy::{
    camera::RenderTargetInfo,
    window::{MonitorSelection, VideoMode, VideoModeSelection, WindowMode},
};
use kq8_core::graphics_devices::Family;
use std::sync::Mutex;

fn video(high: bool) -> VideoMode {
    VideoMode {
        physical_size: UVec2::from_array(kq8_core::caption::screen(high)),
        bit_depth: 32,
        refresh_rate_millihertz: 60_000,
    }
}

fn host(high: bool) -> HostState {
    let pixels = kq8_core::caption::screen(high);
    HostState {
        presentation: HostPresentation::Exclusive,
        window_pixels: pixels,
        monitor_pixels: Some(pixels),
        mode: Some(video(high)),
    }
}

fn camera_target(world: &mut World, camera: Entity, high: bool) {
    world
        .get_mut::<Camera>(camera)
        .unwrap()
        .computed
        .target_info = Some(RenderTargetInfo {
        physical_size: video(high).physical_size,
        scale_factor: 1.0,
    });
}

fn reply(world: &mut World, window: Entity, camera: Entity, high: bool) {
    let serial = world.resource::<Runtime>().pending().unwrap().serial;
    {
        let mut window = world.get_mut::<Window>(window).unwrap();
        let [w, h] = kq8_core::caption::screen(high);
        window.resolution.set_physical_resolution(w, h);
        window.mode = WindowMode::Fullscreen(
            MonitorSelection::Primary,
            VideoModeSelection::Specific(video(high)),
        );
    }
    camera_target(world, camera, high);
    assert!(graphics_runtime::accept_host(world, serial, host(high)).unwrap());
}

fn fixture(
    path: &Path,
    profile: Profile,
    presentation: Presentation,
    saved: &str,
) -> (App, Entity, Entity) {
    let mut log = crate::log::Log::create("graphics-profile-frontend").unwrap();
    let game = GameDir::open(path).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let gui = crate::gui_view::load_gui(&game, &region, &mut log).unwrap();
    let mut menu = Menu::open(path, vec![], false, &mut log).unwrap();
    menu.art.resize(false);
    menu.options = Some(options_menu::Screen::open(path, &menu.art).unwrap());
    let mut settings = Preferences::load(path).settings;
    settings.graphics.mode = saved.into();
    settings.graphics.high_resolution = false;
    let (mut runtime, validated) =
        Runtime::startup(profile, presentation, &settings.graphics).unwrap();
    settings.graphics = validated;
    runtime.confirm(1).unwrap(); // Explicit supplied starting surface in this headless fixture.
    let mut app = App::new();
    app.add_message::<AppExit>();
    app.add_message::<bevy::window::WindowResized>();
    app.add_systems(DisplayMenu, show);
    app.insert_resource(Source {
        game: path.to_path_buf(),
        sound: false,
    });
    app.insert_resource(Preferences::isolated_for_test(path, settings));
    app.insert_resource(Shared {
        log: Arc::new(Mutex::new(log)),
        stats: Arc::new(Mutex::new(Stats::default())),
    });
    app.init_resource::<Assets<Image>>();
    app.init_resource::<crate::world_build::WorldChange>();
    app.insert_resource(runtime);
    app.insert_resource(gui);
    crate::display_mode::register(&mut app, crate::display_mode::Capabilities { high: false });
    menu.image =
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .add(image(&menu.art, &menu.state, 0));
    app.insert_resource(menu);
    let world = app.world_mut();
    let mode = if presentation == Presentation::Windowed {
        WindowMode::Windowed
    } else {
        WindowMode::Fullscreen(
            MonitorSelection::Primary,
            VideoModeSelection::Specific(video(false)),
        )
    };
    let window = world
        .spawn((
            Window {
                resolution: [640, 480].into(),
                mode,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let camera = world.spawn(Camera::default()).id();
    camera_target(world, camera, false);
    graphics_runtime::apply_palette(world, "frontend fixture").unwrap();
    option_key(world, Key::Enter);
    (app, window, camera)
}

#[test]
#[ignore = "requires original KQ8_GAME_DIR; grouped profile frontend/persistence matrix"]
fn graphics_profile_frontend_live_size_window_palette_and_restart_matrix() {
    let path: std::path::PathBuf = std::env::var_os("KQ8_GAME_DIR")
        .expect("set KQ8_GAME_DIR")
        .into();
    let (mut app, window, camera) = fixture(
        &path,
        Profile::Direct3D,
        Presentation::Windowed,
        "D3D-display",
    );
    let world = app.world_mut();
    assert_eq!(
        world.resource::<Runtime>().active().unwrap().device.family,
        Family::Software
    );
    assert_eq!(
        world.resource::<Preferences>().settings.graphics.mode,
        "D3D-display"
    );
    let system = crate::system_palette::read().unwrap().0;
    assert_eq!(
        graphics_runtime::runtime_output(world.resource::<Runtime>()).unwrap(),
        kq8_data::palette::Output::SoftwareWindow { system }
    );
    let game = GameDir::open(&path).unwrap();
    let (paths, _) = crate::load::screen_palette_paths(&game, "Daventry");
    let palette_path = paths.iter().find_map(|name| game.find(name)).unwrap();
    let source = kq8_data::Palette::parse(&std::fs::read(palette_path).unwrap())
        .unwrap()
        .shown_colors();
    let brightness = world.resource::<Preferences>().settings.graphics.brightness;
    assert_eq!(
        world.resource::<crate::gui_view::Gui>().palette,
        kq8_data::palette::for_output(
            source,
            brightness,
            kq8_data::palette::Output::SoftwareWindow { system }
        )
    );
    let mut expected_art = Art::open(&path).unwrap();
    expected_art.resize(false);
    expected_art.set_palette(
        brightness,
        kq8_data::palette::Output::SoftwareWindow { system },
    );
    assert_eq!(
        world
            .resource::<Menu>()
            .art
            .render(&world.resource::<Menu>().state)
            .rgba,
        expected_art.render(&world.resource::<Menu>().state).rgba,
        "menu uses active Software window colors"
    );
    // A fresh-world loader initially reads the saved D3D spelling. The
    // production presentation restoration must replace that decode output.
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let gui = crate::gui_view::load_gui(
        &game,
        &region,
        &mut world.resource::<Shared>().log.lock().unwrap(),
    )
    .unwrap();
    world.insert_resource(gui);
    crate::display_mode::sync_world_presentation(world);
    assert_eq!(
        world.resource::<crate::gui_view::Gui>().palette,
        kq8_data::palette::for_output(
            source,
            brightness,
            kq8_data::palette::Output::SoftwareWindow { system }
        )
    );
    graphics_key(world, Key::Down); // Actual Software window → declared Direct3D fullscreen.
    graphics_key(world, Key::Enter);
    assert!(crate::display_mode::pending(world));
    let serial = world.resource::<Runtime>().pending().unwrap().serial;
    world
        .get_mut::<Camera>(camera)
        .unwrap()
        .computed
        .target_info = None;
    assert!(
        !graphics_runtime::accept_host(world, serial, host(false)).unwrap(),
        "camera acknowledgement is required"
    );
    assert_eq!(
        world.resource::<Runtime>().active().unwrap().device.family,
        Family::Software
    );
    reply(world, window, camera, false);
    assert_eq!(
        world.resource::<Runtime>().active().unwrap().device.family,
        Family::Direct3D
    );
    assert!(!crate::display_mode::high_available(world));
    assert_eq!(
        graphics_runtime::runtime_output(world.resource::<Runtime>()).unwrap(),
        kq8_data::palette::Output::Direct3D
    );
    assert_eq!(
        world.resource::<crate::gui_view::Gui>().palette,
        kq8_data::palette::for_output(source, brightness, kq8_data::palette::Output::Direct3D)
    );

    graphics_key(world, Key::Up); // DirectDraw restart modal.
    graphics_key(world, Key::Enter);
    graphics_key(world, Key::Enter); // Default Cancel, release-driven.
    graphics_key(world, Key::Release);
    assert!(world.resource::<Messages<AppExit>>().is_empty());
    assert_eq!(
        world.resource::<Preferences>().settings.graphics.mode,
        "D3D-display"
    );
    let preference_file = world
        .resource::<Preferences>()
        .isolated_path()
        .to_path_buf();
    std::fs::create_dir_all(&preference_file).unwrap(); // Force a real local rename/write failure.
    for key in [Key::Enter, Key::Left, Key::Enter, Key::Release] {
        graphics_key(world, key);
    }
    assert!(
        world.resource::<Messages<AppExit>>().is_empty(),
        "failed preference store cannot exit"
    );
    assert_eq!(
        world.resource::<Preferences>().settings.graphics.mode,
        "D3D-display"
    );
    assert_eq!(
        world.resource::<Runtime>().active().unwrap().device.family,
        Family::Direct3D
    );
    std::fs::remove_dir(&preference_file).unwrap();
    for key in [Key::Enter, Key::Left, Key::Enter, Key::Release] {
        graphics_key(world, key);
    }
    assert_eq!(
        world.resource::<Messages<AppExit>>().len(),
        1,
        "successful local store accepts the restart"
    );
    assert_eq!(
        world.resource::<Preferences>().settings.graphics.mode,
        "Software"
    );
    assert_eq!(
        world.resource::<Runtime>().active().unwrap().device.family,
        Family::Direct3D,
        "restart does not live-switch the renderer"
    );
    assert_eq!(
        options_menu::Settings::from_script(&std::fs::read(&preference_file).unwrap())
            .graphics
            .mode,
        "Software"
    );

    let (mut app, window, camera) =
        fixture(&path, Profile::Glide, Presentation::Fullscreen, "Software");
    let world = app.world_mut();
    graphics_key(world, Key::Down); // Skip unavailable D3D, select Glide.
    graphics_key(world, Key::Enter);
    reply(world, window, camera, false);
    assert_eq!(
        world.resource::<Preferences>().settings.graphics.mode,
        "Glide"
    );
    assert!(crate::display_mode::high_available(world));
    graphics_key(world, Key::Down); // 640
    graphics_key(world, Key::Down); // 800
    graphics_key(world, Key::Enter);
    assert!(
        !world
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution
    );
    reply(world, window, camera, true);
    assert!(
        world
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution
    );
    assert_eq!(world.resource::<Menu>().art.size(), [800, 600]);
    graphics_key(world, Key::Enter);
    assert!(
        !crate::display_mode::pending(world),
        "selected size is a no-op"
    );
    graphics_key(world, Key::Up);
    graphics_key(world, Key::Enter);
    reply(world, window, camera, false);
    assert_eq!(world.resource::<Menu>().art.size(), [640, 480]);
    assert!(
        !world
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution
    );
    println!("Graphics profile frontend PASS: actual Software window/saved D3D separation, live device and camera/palette acknowledgement, Glide640/800/640, no-op, restart Cancel/store-failure/store-success; native host replies were supplied");
}
