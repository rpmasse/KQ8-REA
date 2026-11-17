//! Viewer Options → Mouse → saved preference route.
use super::*;
use crate::{Shared, Stats};
use kq8_core::mouse_menu::MouseSettings;
use std::sync::Mutex;

#[test]
#[ignore = "requires KQ8_GAME_DIR"]
fn mouse_settings_menu_applies_live_and_survives_back_and_reentry() {
    let path: std::path::PathBuf = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR").into();
    let mut log = crate::log::Log::create("mouse-menu-route").unwrap();
    let mut menu = Menu::open(&path,vec![],false,&mut log).unwrap();
    menu.art.resize(false);
    menu.options = Some(options_menu::Screen::open_modern(&path,&menu.art).unwrap());
    let mut app = App::new();
    app.insert_resource(Source { game:path.clone(),sound:false });
    app.insert_resource(Preferences::isolated_for_test(&path,options_menu::Settings::default()));
    app.insert_resource(Shared { log:Arc::new(Mutex::new(log)),stats:Arc::new(Mutex::new(Stats::default())) });
    app.init_resource::<Assets<Image>>();
    menu.image = app.world_mut().resource_mut::<Assets<Image>>().add(image(&menu.art,&menu.state,0));
    app.insert_resource(menu);
    let world = app.world_mut();
    option_key(world,Key::Down);
    option_key(world,Key::Enter);
    assert!(world.resource::<Menu>().mouse.is_some());
    mouse_key(world,Key::Right);
    let changed = world.resource::<Preferences>().mouse;
    assert_eq!(changed,MouseSettings { camera_sensitivity_percent: 105 });
    assert_eq!(changed.camera_gain(),1.3125,"the changed preference reaches camera input immediately");
    mouse_key(world,Key::Escape);
    assert!(world.resource::<Menu>().mouse.is_none());
    let saved = world.resource::<Preferences>().isolated_path().with_file_name("mouse.json");
    assert_eq!(serde_json::from_slice::<MouseSettings>(&std::fs::read(&saved).unwrap()).unwrap(),changed);
    option_key(world,Key::Enter);
    assert!(world.resource::<Menu>().mouse.is_some());
    assert_eq!(world.resource::<Preferences>().mouse,changed);
    std::fs::remove_file(saved).unwrap();
}
