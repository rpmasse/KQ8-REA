//! Original bar dispatch with actual controller owners; no native rendering.
use super::*;
use kq8_core::{
    active::Active,
    connor_inventory::BarOwner,
    item::{Items, Place},
};
use std::sync::{Arc, Mutex};

fn add(app: &mut App, kind: &str) -> String {
    app.world_mut()
        .resource_mut::<DoorSim>()
        .active
        .world
        .items
        .make(kind, 2, Place::Carried("Connor".into()))
        .unwrap()
        .name
        .clone()
}

fn step(app: &mut App) {
    app.world_mut().run_schedule(Update);
}

fn item(app: &mut App, item: &str) {
    app.world_mut()
        .resource_mut::<BarActions>()
        .0
        .push(BarAction::Item {
            item: item.into(),
            left: true,
            how: "selection-edge fixture".into(),
        });
    step(app);
}

/// Use the production constructor with empty render assets: these checks
/// need real controller ownership, not model loading or a graphics device.
fn spawn_controller(app: &mut App) {
    let placement = kq8_data::objects::Placement::from_section(&kq8_data::kq_text::Section {
        name: "Connor".into(),
        entries: vec![],
    });
    let assets = crate::load::ConnorAssets {
        base_list: "fixture.anm".into(),
        movements: vec![],
        animations: vec![],
        keys: vec![],
        camera_keys: vec![],
        collision_box: None,
        lips: None,
        potion_looks: Default::default(),
        late: None,
    };
    let light = crate::object_light::ObjectLight::new(
        Default::default(),
        crate::light_view::LightMode::X1,
        kq8_data::haze::Haze::new(1000.0, 2000.0, [0.0; 3]),
        false,
    );
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    crate::connor_view::spawn_connor(
        &mut commands,
        (placement, assets),
        None,
        &mut Assets::default(),
        &light,
        &mut Assets::default(),
        &mut Log::create("inventory-selection-controller").unwrap(),
        &crate::anim_gpu::AnimGpu::default(),
    );
    queue.apply(app.world_mut());
}

fn owner(app: &mut App, owner: BarOwner) {
    let mut connor = app.world_mut().remove_resource::<ConnorSim>().unwrap();
    connor.tile = None;
    connor.secondary_tile = None;
    connor.scales = None;
    match owner {
        BarOwner::Ordinary => {}
        BarOwner::Tile => {
            connor.enter_tile(&|_| None, "selection-edge fixture");
        }
        BarOwner::Tile2 | BarOwner::Tile3 => {
            let kind = if owner == BarOwner::Tile2 {
                kq8_core::secondary_tiles::Kind::Swamp
            } else {
                kq8_core::secondary_tiles::Kind::Barren
            };
            connor.enter_secondary_tile(
                kind,
                &app.world().resource::<DoorSim>().active.world,
                "selection-edge fixture",
            );
        }
        BarOwner::Scales => connor.enter_scales(),
    }
    assert_eq!(inventory_bar_owner(Some(&connor)), owner);
    app.insert_resource(connor);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original GUI dispatch with actual Tile/Tile2/Tile3/Scales ownership"]
fn original_inventory_selection_owner_scroll_and_rope_dispatch_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = Log::create("inventory-selection").unwrap();
    let gui = load_gui(&game, &region, &mut log).unwrap();
    let table = std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap();
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], []);
    active.world.items = Items::new(kq8_data::items::ItemTypes::parse(&table).0, []);
    active.world.connor_at = Some([1.0, 2.0, 3.0]);
    let mut app = App::new();
    app.add_systems(Update, gui_step)
        .insert_resource(gui)
        .insert_resource(DoorSim::for_test(active, log))
        .insert_resource(Shared {
            log: Arc::new(Mutex::new(
                Log::create("inventory-selection-viewer").unwrap(),
            )),
            stats: Arc::new(Mutex::new(crate::Stats::default())),
        })
        .init_resource::<KeyAt>()
        .init_resource::<BarActions>()
        .init_resource::<crate::talk_view::Speech>()
        .init_resource::<crate::keyboard_input::GameKeys>()
        .init_resource::<crate::startup_menu::MenuRequest>();
    spawn_controller(&mut app);
    let candle = add(&mut app, "Candle");
    let rock = add(&mut app, "Rock");
    let rope = add(&mut app, "RopeAndHook");
    let scroll = add(&mut app, "Scroll");
    step(&mut app);
    for current in [
        BarOwner::Ordinary,
        BarOwner::Tile,
        BarOwner::Tile2,
        BarOwner::Tile3,
        BarOwner::Scales,
    ] {
        owner(&mut app, current);
        {
            let w = &mut app.world_mut().resource_mut::<DoorSim>().active.world;
            w.hand = Some(candle.clone());
            w.talk.clear("Connor", w.now());
        }
        app.world_mut()
            .resource_mut::<KeyAt>()
            .0
            .push(("left:IDCTL_INVBAR_ROCKS".into(), 0));
        step(&mut app);
        let restricted = matches!(current, BarOwner::Tile | BarOwner::Tile3 | BarOwner::Scales);
        let w = &app.world().resource::<DoorSim>().active.world;
        assert_eq!(
            w.hand.as_deref(),
            Some(if restricted {
                candle.as_str()
            } else {
                rock.as_str()
            }),
            "{current:?}"
        );
        assert_eq!(w.items.get(&rock).unwrap().quantity, 2);
        if restricted {
            let saved = serde_json::to_value(w.talk.saved(w.now())).unwrap();
            assert_eq!(
                saved["queues"][0]["lines"][0]["tuple"],
                serde_json::json!({"file":0,"noun":0,"verb":0,"case":0,"seq":3})
            );
        }
    }
    owner(&mut app, BarOwner::Ordinary);
    for target in [Some(scroll.as_str()), None, Some(rope.as_str())] {
        {
            let w = &mut app.world_mut().resource_mut::<DoorSim>().active.world;
            w.hand = None;
            w.rope_first_person_lock = false;
            w.first_person = true;
            w.rope.status = kq8_core::rope::Status::Top;
            w.talk.clear("Connor", w.now());
        }
        item(&mut app, &rope);
        assert!(
            app.world()
                .resource::<DoorSim>()
                .active
                .world
                .rope_first_person_lock
        );
        match target {
            Some(target) => item(&mut app, target),
            None => {
                app.world_mut()
                    .resource_mut::<KeyAt>()
                    .0
                    .push(("1".into(), 0));
                step(&mut app);
            }
        }
        let w = &app.world().resource::<DoorSim>().active.world;
        assert!(w.hand.is_none());
        assert_eq!(w.rope_first_person_lock, target != Some(rope.as_str()));
        assert_eq!(w.first_person, target == Some(rope.as_str()));
    }
    println!("Inventory selection PASS: original Rocks control through actual ordinary/Tile/Tile2/Tile3/Scales owners; exact Rock refusal and hand retention; Scroll/empty selection preserve Top rope lock; same-rope put-back restores first person. Rendering and native input omitted.");
}
