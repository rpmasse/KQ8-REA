//! Real viewer entry/Back/reentry and modal physical-input routing.
use super::*;
use crate::{Shared, Stats};
use kq8_core::graphics_devices::{Device, Inventory};
use std::{sync::Mutex, time::Duration};

#[test]
#[ignore = "requires KQ8_GAME_DIR; original Graphics frontend lifecycle"]
fn graphics_runtime_retained_screen_focus_and_modal_pointer_matrix() {
    let path: std::path::PathBuf = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR").into();
    let mut log = crate::log::Log::create("graphics-lifecycle").unwrap();
    let mut menu = Menu::open(&path,vec![],false,&mut log).unwrap();
    menu.art.resize(false);
    menu.options = Some(options_menu::Screen::open(&path,&menu.art).unwrap());
    let mut app = App::new();
    app.add_systems(DisplayMenu,show);
    app.insert_resource(Source { game:path.clone(),sound:false });
    let mut settings = Preferences::load(&path).settings;
    settings.graphics.high_resolution = false;
    settings.graphics.mode = "D3D-display".into();
    app.insert_resource(Preferences::isolated_for_test(&path,settings));
    app.insert_resource(Shared { log:Arc::new(Mutex::new(log)), stats:Arc::new(Mutex::new(Stats::default())) });
    app.init_resource::<Assets<Image>>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<crate::world_build::WorldChange>();
    menu.image = app.world_mut().resource_mut::<Assets<Image>>().add(image(&menu.art,&menu.state,0));
    app.insert_resource(menu);
    let world = app.world_mut();
    let window = world.spawn((Window { resolution:[640,480].into(), ..default() },PrimaryWindow)).id();
    let software = Device::software(true);
    let d3d = Device::direct3d("display",true);
    let inventory = Inventory::new(vec![software,d3d.clone()]);
    option_key(world,Key::Enter);
    world.resource_mut::<Menu>().graphics.as_mut().unwrap().set_device_inventory(inventory.clone(),d3d.clone(),false);
    for _ in 0..3 { graphics_key(world,Key::Down); }
    let before = {
        let menu = world.resource::<Menu>();
        menu.graphics.as_ref().unwrap().render(&world.resource::<Preferences>().settings.graphics,&menu.art,0).rgba
    };
    graphics_key(world,Key::Escape);
    assert!(world.resource::<Menu>().graphics.is_none());
    assert!(world.contains_resource::<RetainedGraphics>());
    option_key(world,Key::Enter);
    assert!(!world.contains_resource::<RetainedGraphics>());
    let after = {
        let menu = world.resource::<Menu>();
        menu.graphics.as_ref().unwrap().render(&world.resource::<Preferences>().settings.graphics,&menu.art,0).rgba
    };
    assert_eq!(before,after,"Back and Options reentry retain focus and targets");
    graphics_key(world,Key::Escape);
    close_menu(world,false);
    assert!(world.contains_resource::<RetainedGraphics>());
    return_to_title(world).unwrap();
    {
        let mut menu = world.resource_mut::<Menu>();
        menu.options = Some(options_menu::Screen::open(&path,&menu.art).unwrap());
    }
    option_key(world,Key::Enter);
    let after_reopen = {
        let menu = world.resource::<Menu>();
        menu.graphics.as_ref().unwrap().render(&world.resource::<Preferences>().settings.graphics,&menu.art,0).rgba
    };
    assert_eq!(before,after_reopen,"closing and reopening Main retains the screen");

    // A freshly focused DirectDraw row opens the original D3D confirmation.
    let mut screen = graphics_menu::Screen::open(&path,&world.resource::<Menu>().art).unwrap();
    screen.set_device_inventory(inventory,d3d,false);
    {
        let mut menu = world.resource_mut::<Menu>();
        menu.graphics = Some(screen);
        menu.shown = Some(std::time::Instant::now()-Duration::from_secs(1));
    }
    graphics_key(world,Key::Enter);
    assert!(world.resource::<Menu>().graphics.as_ref().unwrap().modal_open());
    world.get_mut::<Window>(window).unwrap().set_cursor_position(Some(Vec2::new(43.,59.)));
    world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter);
    step(world);
    assert_eq!(world.get::<Window>(window).unwrap().cursor_position(),Some(Vec2::new(405.,241.)));
    assert_eq!(world.resource::<Menu>().return_pointer,Some(Vec2::new(43.,59.)));
    world.resource_mut::<ButtonInput<KeyCode>>().clear();
    world.resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::Enter);
    step(world);
    assert_eq!(world.get::<Window>(window).unwrap().cursor_position(),Some(Vec2::new(43.,59.)));
    assert!(!world.resource::<Menu>().graphics.as_ref().unwrap().modal_open());
    world.resource_mut::<ButtonInput<KeyCode>>().reset_all();

    graphics_key(world,Key::Enter);
    world.get_mut::<Window>(window).unwrap().set_cursor_position(Some(Vec2::new(200.,225.)));
    world.resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    step(world);
    world.resource_mut::<ButtonInput<MouseButton>>().clear();
    world.get_mut::<Window>(window).unwrap().set_cursor_position(Some(Vec2::new(50.,50.)));
    world.resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Left);
    step(world);
    let mut menu = world.resource_mut::<Menu>();
    let page = menu.graphics.as_mut().unwrap();
    assert!(page.modal_open(),"release outside does not accept the pressed Yes button");
    assert!(page.take_device_selection().is_none());
    println!("Graphics runtime PASS: original focus survives Back and Main lifetime; Enter pointer moves/restores; mouse release uses current position");
}
