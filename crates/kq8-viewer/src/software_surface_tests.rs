//! Production palette selection across actual Window modes; no native window.
use super::*;
use crate::{display_mode, gui_view, live_palette::{self, Values}, Shared, Stats};
use std::sync::Mutex;

#[test]
#[ignore = "requires original KQ8_GAME_DIR; CPU palette/menu/GUI matrix"]
fn software_surface_original_data_window_fullscreen_refresh_matrix() {
    let path: std::path::PathBuf = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR").into();
    let game = GameDir::open(&path).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = crate::log::Log::create("software-surface").unwrap();
    let gui = gui_view::load_gui(&game, &region, &mut log).unwrap();
    let mut preferences = Preferences::load(&path);
    preferences.settings.graphics.mode = "Software".into();
    preferences.settings.graphics.brightness = 100;
    preferences.settings.graphics.high_resolution = false;
    let mut menu = Menu::open(&path, vec![], false, &mut log).unwrap();
    menu.art.resize(false);
    let mut app = App::new();
    app.add_message::<bevy::window::WindowResized>();
    app.init_resource::<Assets<Image>>();
    app.insert_resource(preferences);
    app.insert_resource(Shared { log: Arc::new(Mutex::new(log)), stats: Arc::new(Mutex::new(Stats::default())) });
    app.insert_resource(gui);
    menu.image = app.world_mut().resource_mut::<Assets<Image>>().add(image(&menu.art, &menu.state, 0));
    app.insert_resource(menu);
    display_mode::register(&mut app, display_mode::Capabilities { high: false });
    let world = app.world_mut();
    let window = world.spawn((Window { resolution: [640,480].into(), ..default() }, PrimaryWindow)).id();
    let values = Values { brightness: 100, output: kq8_data::palette::Output::Software };
    live_palette::apply_to_surface(world, values, "window matrix");
    let window_colors = world.resource::<gui_view::Gui>().palette;
    let (system, source) = crate::system_palette::read().unwrap();
    let window_output = kq8_data::palette::Output::SoftwareWindow { system };
    let expected = kq8_data::palette::for_output([[0;3];256], 100, window_output);
    assert_eq!(&window_colors[..10], &expected[..10]);
    assert_eq!(&window_colors[246..], &expected[246..]);

    let mut expected_art = Art::open(&path).unwrap();
    expected_art.resize(false);
    expected_art.set_palette(100, window_output);
    let assert_menu = |world: &World, art: &Art| {
        let menu = world.resource::<Menu>();
        let actual = world.resource::<Assets<Image>>().get(&menu.image).unwrap();
        let expected = image(art, &menu.state, menu.mask_frame);
        assert_eq!(actual.data, expected.data, "the menu uses the active Software surface colors");
        assert_eq!((actual.width(),actual.height()), (640,480));
    };
    assert_menu(world, &expected_art);

    world.get_mut::<Window>(window).unwrap().mode = bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current);
    live_palette::apply_to_surface(world, values, "fullscreen matrix");
    let fullscreen_colors = world.resource::<gui_view::Gui>().palette;
    assert_eq!(&window_colors[10..246], &fullscreen_colors[10..246]);
    // This installation's twenty palette-edge colors equal the Windows
    // halftone entries. Compare the source-backed result, not inequality.
    let (palette_paths, _) = crate::load::screen_palette_paths(&game, "Daventry");
    let palette_path = palette_paths.iter().find_map(|name|game.find(name)).unwrap();
    let original = kq8_data::Palette::parse(&std::fs::read(palette_path).unwrap()).unwrap();
    assert_eq!(fullscreen_colors, kq8_data::palette::for_output(original.shown_colors(), 100, kq8_data::palette::Output::Software));
    expected_art.set_palette(100, kq8_data::palette::Output::Software);
    assert_menu(world, &expected_art);

    world.get_mut::<Window>(window).unwrap().mode = bevy::window::WindowMode::Windowed;
    live_palette::apply_to_surface(world, Values { brightness: 0, ..values }, "dim matrix");
    assert_ne!(world.resource::<gui_view::Gui>().palette, window_colors);
    display_mode::sync_world_presentation(world);
    assert_eq!(world.resource::<gui_view::Gui>().palette, window_colors, "world/display restoration selects window colors from immutable sources");
    expected_art.set_palette(100, window_output);
    assert_menu(world, &expected_art);
    println!("Software PASS: window/fullscreen dispatch, 20 static colors via {source:?}, original menu pixels, GUI middle entries and restoration; 640x480 unchanged");
}
