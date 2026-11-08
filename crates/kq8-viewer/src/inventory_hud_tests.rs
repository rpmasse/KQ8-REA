//! Q109/Q110: production GUI dispatch/specification/CPU draw, original art.
use super::*;
use kq8_core::{active::Active, item::{Items, Place}};
use std::sync::{Arc, Mutex};

fn add(app: &mut App, kind: &str, count: i64) -> String {
    app.world_mut().resource_mut::<DoorSim>().active.world.items
        .make(kind, count, Place::Carried("Connor".into())).unwrap().name.clone()
}

fn step(app: &mut App) { app.world_mut().run_schedule(Update); }

fn named(app: &mut App, control: &str) {
    app.world_mut().resource_mut::<KeyAt>().0.push((control.into(), 0));
    step(app);
}

fn parts(gui: &mut Gui, panel: Panel, now: f64) -> ([i32; 2], Vec<Spec>) {
    gui.observe_inventory_pictures(Drawn::Panel(panel), now);
    gui.specs(Drawn::Panel(panel)).unwrap()
}

fn rect(gui: &Gui, panel: Panel, name: &str) -> [i32; 4] {
    gui.controls(gui.control(panel.layout_id()).unwrap()).into_iter()
        .find(|(c, _)| gui.name_of(c) == name).unwrap().1
}

fn picture(gui: &Gui, panel: Panel, specs: &[Spec], name: &str) -> String {
    let target = rect(gui, panel, name);
    specs.iter().find_map(|s| match s {
        Spec::Pic { rect, file, .. } if *rect == target => Some(file.clone()), _ => None,
    }).unwrap()
}

fn text(gui: &Gui, panel: Panel, specs: &[Spec], name: &str) -> Option<String> {
    let target = rect(gui, panel, name);
    specs.iter().find_map(|s| match s {
        Spec::Text { rect, text, .. } if *rect == target => Some(text.clone()), _ => None,
    })
}

fn render(gui: &mut Gui, panel: Panel, size: [i32; 2], specs: &[Spec]) -> Vec<u8> {
    let control = gui.control(panel.layout_id()).unwrap();
    let origin = [control.rect[0], control.rect[1]];
    let canvas = gui.draw(origin, size, specs, 1.0);
    assert_eq!(canvas.rgba.len(), (size[0] * size[1] * 4) as usize);
    canvas.rgba
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; grouped original inventory HUD CPU rendering and dispatch"]
fn original_inventory_hud_binding_count_blink_render_and_dispatch_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = Log::create("inventory-hud-matrix").unwrap();
    let mut gui = load_gui(&game, &region, &mut log).unwrap();
    gui.resize([640, 480]);
    let table = std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap();
    let types = kq8_data::items::ItemTypes::parse(&table).0;
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], []);
    active.world.items = Items::new(types, []);
    let mut app = App::new();
    app.add_systems(Update, gui_step).insert_resource(gui)
        .insert_resource(DoorSim::for_test(active, log))
        .insert_resource(Shared { log: Arc::new(Mutex::new(Log::create("inventory-hud-viewer").unwrap())), stats: Arc::new(Mutex::new(crate::Stats::default())) })
        .init_resource::<KeyAt>().init_resource::<BarActions>()
        .init_resource::<crate::talk_view::Speech>()
        .init_resource::<crate::keyboard_input::GameKeys>()
        .init_resource::<crate::startup_menu::MenuRequest>();
    step(&mut app);
    {
        let mut gui = app.world_mut().resource_mut::<Gui>();
        for (panel, picture_name, count_name, file) in [
            (Panel::Top, "IDCTL_INVBAR_GOLD", "IDCTL_INVBAR_GOLDTEXT", "2gold0.pbm"),
            (Panel::BottomRight, "IDCTL_INVBAR_ROCKS", "IDCTL_INVBAR_ROCKSTEXT", "2rocks0.pbm"),
        ] {
            let (size, specs) = parts(&mut gui, panel, 0.0);
            assert_eq!(picture(&gui, panel, &specs, picture_name), file);
            assert_eq!(text(&gui, panel, &specs, count_name).as_deref(), Some("0"));
            render(&mut gui, panel, size, &specs);
        }
    }
    let money = add(&mut app, "SilverCoins", 1);
    let rocks = add(&mut app, "Rock", 1);
    let health = add(&mut app, "Mushroom", 1);
    let armor = add(&mut app, "PlateMail", 1);
    let weapon = add(&mut app, "Dagger", 1);
    let candle = add(&mut app, "Candle", 1);
    step(&mut app);
    let empty_armor;
    {
        let mut gui = app.world_mut().resource_mut::<Gui>();
        let (size, specs) = parts(&mut gui, Panel::BottomLeft, 0.0);
        assert_eq!(picture(&gui, Panel::BottomLeft, &specs, "IDCTL_INVBAR_ARMOR"), "2armr0.pbm");
        assert_eq!(picture(&gui, Panel::BottomLeft, &specs, "IDCTL_INVBAR_SHORT"), "2sord0.pbm");
        empty_armor = render(&mut gui, Panel::BottomLeft, size, &specs);
        let (_, specs) = parts(&mut gui, Panel::Top, 0.0);
        assert_eq!(picture(&gui, Panel::Top, &specs, "IDCTL_INVBAR_GOLD"), "2gold0.pbm");
        assert_eq!(text(&gui, Panel::Top, &specs, "IDCTL_INVBAR_GOLDTEXT").as_deref(), Some("1"));
        let (_, specs) = parts(&mut gui, Panel::Slots, 0.0);
        assert_eq!(text(&gui, Panel::Slots, &specs, "IDCTL_HEALTHBAR_1TEXT").as_deref(), Some("1"));
        assert!(!gui.inventory_bindings.contains_key(MINIMAP));
    }
    // Held coins use normal count-based art even during an empty phase.
    named(&mut app, "left:IDCTL_INVBAR_GOLD");
    assert_eq!(app.world().resource::<DoorSim>().active.world.hand.as_deref(), Some(money.as_str()));
    {
        let mut gui = app.world_mut().resource_mut::<Gui>();
        let (_, specs) = parts(&mut gui, Panel::Top, 0.0);
        assert_eq!(picture(&gui, Panel::Top, &specs, "IDCTL_INVBAR_GOLD"), "2gold1.pbm");
        assert_eq!(text(&gui, Panel::Top, &specs, "IDCTL_INVBAR_GOLDTEXT").as_deref(), Some("1"));
    }
    named(&mut app, "right:IDCTL_INVBAR_QUEST7");
    assert_eq!(app.world().resource::<DoorSim>().active.world.hand.as_deref(), Some(money.as_str()));
    named(&mut app, "left:IDCTL_INVBAR_QUEST7");
    assert!(app.world().resource::<DoorSim>().active.world.hand.is_none());
    {
        let mut gui = app.world_mut().resource_mut::<Gui>();
        let (size, specs) = parts(&mut gui, Panel::BottomLeft, 250.0);
        assert_ne!(render(&mut gui, Panel::BottomLeft, size, &specs), empty_armor);
        assert_eq!(gui.slot_items[0].as_deref(), Some(armor.as_str()));
        assert_eq!(gui.slot_items[1].as_deref(), Some(weapon.as_str()));
        for band in 1..=9 {
            for panel in [Panel::Top, Panel::BottomLeft, Panel::Slots, Panel::BottomRight] {
                parts(&mut gui, panel, band as f64 * 250.0);
            }
        }
        assert!(gui.blinks.is_empty());
    }
    for (item, panel, control, label, prefix, cases) in [
        (&money, Panel::Top, "IDCTL_INVBAR_GOLD", "IDCTL_INVBAR_GOLDTEXT", "2gold", vec![(0,0),(1,1),(24,1),(25,2),(49,2),(50,3)]),
        (&rocks, Panel::BottomRight, "IDCTL_INVBAR_ROCKS", "IDCTL_INVBAR_ROCKSTEXT", "2rocks", vec![(0,0),(1,1),(4,1),(5,2),(14,2),(15,3)]),
    ] {
        for (count, tier) in cases {
            let mut sim = app.world_mut().resource_mut::<DoorSim>();
            let index = sim.active.world.items.index(item).unwrap();
            sim.active.world.items.objects[index].quantity = count;
            drop(sim);
            step(&mut app);
            let mut gui = app.world_mut().resource_mut::<Gui>();
            assert!(gui.blinks.is_empty(), "count-only refresh must not reassign");
            let (size, specs) = parts(&mut gui, panel, 2500.0);
            assert_eq!(picture(&gui, panel, &specs, control), format!("{prefix}{tier}.pbm"));
            assert_eq!(text(&gui, panel, &specs, label), Some(count.to_string()));
            render(&mut gui, panel, size, &specs);
        }
    }
    // A retained zero-dose health item has neither its picture nor zero text.
    {
        let mut sim = app.world_mut().resource_mut::<DoorSim>();
        let index = sim.active.world.items.index(&health).unwrap();
        sim.active.world.items.objects[index].quantity = 0;
    }
    step(&mut app);
    {
        let mut gui = app.world_mut().resource_mut::<Gui>();
        let (_, specs) = parts(&mut gui, Panel::Slots, 2500.0);
        assert_eq!(picture(&gui, Panel::Slots, &specs, "IDCTL_HEALTHBAR_1"), "3empty.pbm");
        assert_eq!(text(&gui, Panel::Slots, &specs, "IDCTL_HEALTHBAR_1TEXT"), None);
    }
    add(&mut app, "LeatherGloves", 1);
    step(&mut app);
    assert!(app.world().resource::<Gui>().blinks.is_empty(), "gloves do not reassign armor");
    let replacement = add(&mut app, "Dagger", 1);
    step(&mut app);
    {
        let mut gui = app.world_mut().resource_mut::<Gui>();
        assert_eq!(gui.slot_items[1].as_deref(), Some(replacement.as_str()));
        assert!(gui.blinks.contains_key("IDCTL_INVBAR_SHORT"), "same picture, different identity");
        assert!(!gui.blinks.contains_key("IDCTL_INVBAR_ARMOR"));
        let (_, specs) = parts(&mut gui, Panel::BottomLeft, 2500.0);
        assert_eq!(picture(&gui, Panel::BottomLeft, &specs, "IDCTL_INVBAR_SHORT"), "2sord0.pbm");
    }
    named(&mut app, "left:IDCTL_INVBAR_QUEST1");
    assert_eq!(app.world().resource::<DoorSim>().active.world.hand.as_deref(), Some(candle.as_str()));
    named(&mut app, "1"); // the now-empty health slot shares the clear action
    assert!(app.world().resource::<DoorSim>().active.world.hand.is_none());
    println!("HUD PASS: original tags/art CPU draw; empty counts; thresholds; shared phases; equipment identity; no count restart; held coins and named empty cells");
}
