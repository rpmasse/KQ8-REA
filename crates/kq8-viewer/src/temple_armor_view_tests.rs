use super::*;
use kq8_core::{
    active::Active,
    fight::ConnorFight,
    item::{Items, Place},
    journey::Journey,
};
use kq8_data::{characters::ConnorAnimLists, items::ItemTypes, GameDir};

struct Flat;
impl kq8_core::connor_mover::GroundLookup for Flat {
    fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> { Some((0., [0., 0., 1.])) }
}

fn tick(sim: &mut ConnorSim, active: &mut Active) {
    sim.tick_with_state_requests(Some(active), Intent::default(), false, &Flat);
    sim.finish_weapon_step(Some(&mut active.world));
}

fn controller() -> ConnorSim {
    let mut world = World::new();
    let placement = Placement::from_section(&kq8_data::kq_text::Section {
        name: "Connor".into(),
        entries: vec![],
    });
    let assets = ConnorAssets {
        base_list: "conner.anm".into(),
        movements: vec![],
        animations: vec![],
        keys: vec![],
        camera_keys: vec![],
        collision_box: None,
        lips: None,
        potion_looks: Default::default(),
        late: None,
    };
    let light = ObjectLight::new(
        Default::default(),
        crate::light_view::LightMode::X1,
        kq8_data::haze::Haze::new(1000., 2000., [0.; 3]),
        false,
    );
    let mut queue = bevy::ecs::world::CommandQueue::default();
    spawn_connor(
        &mut Commands::new(&mut queue, &world),
        (placement, assets),
        None,
        &mut Assets::default(),
        &light,
        &mut Assets::default(),
        &mut Log::create("armour-controller").unwrap(),
        &crate::anim_gpu::AnimGpu::default(),
    );
    queue.apply(&mut world);
    let mut sim = world.remove_resource::<ConnorSim>().unwrap();
    for bank in [
        "conner.anm",
        "oldhand.anm",
        "oldranged.anm",
        "conner2.anm",
        "contpsd2.anm",
        "conicbw2.anm",
        "fpnone.anm",
        "fphand.anm",
        "fpranged.anm",
        "fpother.anm",
        "newranged.anm",
        "fpnewranged.anm",
    ] {
        for action in ["stop", "attack", "away", "out"] {
            let qualified = kq8_core::connor::intern(&format!("{bank}/{action}"));
            let name = if bank == "conner.anm" {
                kq8_core::connor::intern(action)
            } else {
                qualified
            };
            sim.movements.push(DrawnMovement {
                name,
                qualified,
                first_person: bank.starts_with("fp"),
                movement_loop: MovementLoop {
                    duration: 1.,
                    frames: 4,
                    repeat: action == "stop",
                    cut_short: true,
                    speed: 1.,
                    ..Default::default()
                },
                frames: 4,
                anim: usize::MAX,
                parts: vec![],
            });
        }
    }
    sim
}

fn actor(types: &ItemTypes) -> Active {
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], []);
    active.world.items = Items::new(types.clone(), []);
    for kind in ["Dagger", "SmCrossBow"] {
        active
            .world
            .items
            .make(kind, 1, Place::Carried("Connor".into()))
            .unwrap();
    }
    let held = active
        .world
        .items
        .make("Candle", 1, Place::Carried("Connor".into()))
        .unwrap()
        .name
        .clone();
    active.world.hand = Some(held);
    active.world.connor_fight = Some(ConnorFight {
        ouch_sound: "conrow3.aud".into(),
        min_damage_factor: 1.,
        max_damage_factor: 1.,
        third_person: ConnorAnimLists {
            stowed: Some("conner.anm".into()),
            hand: vec![("Dagger".into(), "oldhand.anm".into())],
            ranged: vec![("SmCrossBow".into(), "oldranged.anm".into())],
        },
        first_person: ConnorAnimLists {
            stowed: Some("fpnone.anm".into()),
            hand: vec![("Dagger".into(), "fphand.anm".into())],
            ranged: vec![("SmCrossBow".into(), "fpranged.anm".into())],
        },
        list_refresh_pending: false,
        shoot_offsets: vec![],
        start_projectile: None,
        start_offset: [0.; 3],
    });
    active
}

fn install() -> (GameDir, ItemTypes) {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let types =
        ItemTypes::parse(&std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap()).0;
    (game, types)
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original item types with production actor refresh/checkpoints"]
fn temple_armor_refresh_and_restore_matrix() {
    let (_, types) = install();
    for first_person in [false, true] {
        for mode in [
            WeaponMode::None,
            WeaponMode::Hand,
            WeaponMode::Ranged,
            WeaponMode::Raw(9),
        ] {
            for cut_short in [false, true] {
                let mut active = actor(&types);
                let world = &mut active.world;
                world.first_person = first_person;
                world.weapon_mode = mode;
                world.weapon_ready = carried_type(world, mode).map(str::to_owned);
                let held = world.hand.clone();
                let mut sim = controller();
                sim.first_person = first_person;
                let old_bank = if matches!(mode, WeaponMode::Raw(_)) {
                    Some("oldhand.anm".into())
                } else {
                    world
                        .connor_fight
                        .as_ref()
                        .unwrap()
                        .bank(first_person, mode, world.weapon_ready.as_deref())
                        .map(str::to_owned)
                };
                sim.switch_set(old_bank.clone(), mode);
                sim.weapon_target = old_bank;
                sim.target_mode = mode;
                sim.ready_item = world.weapon_ready.clone();
                sim.walker.movement = "attack";
                sim.walker.phase = 0.35;
                sim.walker.fresh = false;
                for movement in &mut sim.movements {
                    movement.movement_loop.cut_short = cut_short;
                }
                let old_model = sim.drawn("attack").unwrap().qualified;
                assert!(world.helmetize("contpsd2.anm", "conicbw2.anm", "refresh matrix"));

                // A checkpoint between the table mutation and refresh keeps
                // the old selected bank and the still-pending operation.
                let journey: Journey = serde_json::from_slice(
                    &serde_json::to_vec(&Journey::take(world, &active.scripts.vars, "temple1"))
                        .unwrap(),
                )
                .unwrap();
                let weapon: SavedWeaponState = serde_json::from_slice(
                    &serde_json::to_vec(&sim.saved_weapon_state(None).unwrap()).unwrap(),
                )
                .unwrap();
                let pose = sim.capture_controller_pose();
                let mut restored = actor(&types);
                journey.give(&mut restored.world, &mut restored.scripts.vars);
                restored.world.first_person = first_person;
                let mut restored_sim = controller();
                for movement in &mut restored_sim.movements {
                    movement.movement_loop.cut_short = cut_short;
                }
                restored_sim.first_person = first_person;
                restored_sim.restore_weapon_state(&weapon, None).unwrap();
                restored_sim
                    .restore_controller_pose(&pose, Mode::Free, "armour pending")
                    .unwrap();
                let line = sim.refresh_inventory_lists(world).unwrap();
                assert!(line.is_some());
                restored_sim
                    .refresh_inventory_lists(&mut restored.world)
                    .unwrap();
                assert_eq!(sim.weapon_set, restored_sim.weapon_set);
                assert_eq!(sim.walker.movement, restored_sim.walker.movement);
                assert_eq!(sim.walker.phase, restored_sim.walker.phase);
                assert_eq!(world.hand, held);
                assert_eq!(world.weapon_mode, mode);
                assert_eq!(sim.set_mode, mode);
                assert!(!sim.weapon_selection.assigned_ordinary);
                assert!(!sim.readying);
                assert!(!world.connor_fight.as_ref().unwrap().list_refresh_pending);
                assert!(
                    sim.refresh_inventory_lists(world).unwrap().is_none(),
                    "refresh is consumed once"
                );
                if matches!(mode, WeaponMode::Raw(_)) {
                    assert_eq!(sim.weapon_set.as_deref(), Some("oldhand.anm"));
                    assert_eq!(sim.walker.movement, "attack");
                    assert_eq!(sim.walker.phase, 0.35);
                } else if !cut_short {
                    assert_eq!(sim.walker.movement, old_model);
                    assert_eq!(sim.walker.phase, 0.35);
                }
                // A completed refresh retains an old non-cut-short model
                // through a subsequent save; it must not request it again.
                let completed = Journey::take(world, &active.scripts.vars, "temple1");
                let selected = sim.saved_weapon_state(None).unwrap();
                let pose = sim.capture_controller_pose();
                completed.give(&mut restored.world, &mut restored.scripts.vars);
                restored_sim.restore_weapon_state(&selected, None).unwrap();
                restored_sim
                    .restore_controller_pose(&pose, Mode::Free, "armour retained model")
                    .unwrap();
                assert!(restored_sim
                    .refresh_inventory_lists(&mut restored.world)
                    .unwrap()
                    .is_none());
                assert_eq!(restored_sim.walker.movement, sim.walker.movement);
                assert_eq!(restored_sim.walker.phase, sim.walker.phase);
            }
        }
    }
    // A destination starts with an empty actor controller. Its carried
    // ready mode is synchronized internally, without a draw or held clear.
    let mut origin = actor(&types);
    origin.world.weapon_mode = WeaponMode::Hand;
    origin.world.weapon_ready = Some("INVITEM_Dagger".into());
    assert!(origin
        .world
        .helmetize("contpsd2.anm", "conicbw2.anm", "travel"));
    let saved = Journey::take(&origin.world, &origin.scripts.vars, "temple1");
    let mut destination = actor(&types);
    saved.give_after_flags(&mut destination.world, &mut destination.scripts.vars);
    let mut sim = controller();
    let held = destination.world.hand.clone();
    assert_eq!(sim.set_mode, WeaponMode::None);
    sim.refresh_inventory_lists(&mut destination.world).unwrap();
    assert_eq!(sim.set_mode, WeaponMode::Hand);
    assert_eq!(sim.weapon_set.as_deref(), Some("contpsd2.anm"));
    assert_eq!(destination.world.hand, held);
    assert!(!destination.world.weapon_swapped);
    assert!(!sim.weapon_selection.assigned_ordinary);

    // Refresh during away/out remaps lists without advancing the pending
    // ordinary assignment or clearing its provenance.
    let mut active = actor(&types);
    active.world.weapon_mode = WeaponMode::Ranged;
    active.world.weapon_ready = Some("INVITEM_SmCrossBow".into());
    let held = active.world.hand.clone();
    let mut sim = controller();
    sim.switch_set(Some("oldhand.anm".into()), WeaponMode::Hand);
    sim.target_mode = WeaponMode::Ranged;
    sim.ready_item = active.world.weapon_ready.clone();
    sim.weapon_target = Some("oldranged.anm".into());
    sim.pending_set = Some((sim.weapon_target.clone(), WeaponMode::Ranged, 1));
    sim.readying = true;
    sim.weapon_selection.pending_ordinary = true;
    sim.walker.mode = Mode::Special;
    sim.walker.movement = "oldhand.anm/away";
    sim.walker.special = vec![
        kq8_core::connor::SpecialStep::Play("oldhand.anm/away", "away".into()),
        kq8_core::connor::SpecialStep::Play("oldranged.anm/out", "out".into()),
    ];
    sim.walker.special_at = 0;
    assert!(active
        .world
        .helmetize("contpsd2.anm", "conicbw2.anm", "during swap"));
    sim.refresh_inventory_lists(&mut active.world).unwrap();
    assert_eq!(sim.set_mode, WeaponMode::Hand);
    assert_eq!(sim.target_mode, WeaponMode::Ranged);
    assert_eq!(sim.weapon_set.as_deref(), Some("contpsd2.anm"));
    assert_eq!(
        sim.pending_set.as_ref().unwrap().0.as_deref(),
        Some("conicbw2.anm")
    );
    assert!(sim.weapon_selection.pending_ordinary && !sim.weapon_selection.assigned_ordinary);
    assert_eq!(active.world.hand, held);

    // Rule 8a: repeated aliases are selected by currently equipped item.
    // Enter a real swap: F2 is locked until explicit helmetize re-requests
    // the current action. The uncuttable old pose and target mode remain.
    let mut active = actor(&types);
    active.world.connor_at = Some([0.; 3]);
    active.world.weapon_mode = WeaponMode::Hand;
    active.world.weapon_ready = Some("INVITEM_Dagger".into());
    active.world.connor_fight.as_mut().unwrap().third_person.hand.push(("SmallAx".into(), "other.anm".into()));
    active.world.connor_fight.as_mut().unwrap().first_person.hand.push(("SmallAx".into(), "fpother.anm".into()));
    active.world.connor_fight.as_mut().unwrap().third_person.ranged.push(("CrossBow".into(), "newranged.anm".into()));
    active.world.connor_fight.as_mut().unwrap().first_person.ranged.push(("CrossBow".into(), "fpnewranged.anm".into()));
    assert!(active.world.helmetize("contpsd2.anm", "conicbw2.anm", "first alias"));
    let mut sim = controller();
    sim.ready_set(Some("contpsd2.anm".into()), WeaponMode::Hand, Some("INVITEM_Dagger".into()), true, false, false);
    assert!(sim.deliver_weapon_request(&mut active, &kq8_core::weapons::Request {
        ask: kq8_core::weapons::Ask::Draw, how: "alias key matrix".into() }, false));
    assert!(sim.readying && sim.walker.swap_owned_special && sim.walker.animation_input_lock);
    assert_eq!((sim.set_mode, sim.target_mode), (WeaponMode::Hand, WeaponMode::Ranged));
    assert!(!active.world.first_person_key(sim.hands_off || sim.walker.animation_input_lock));
    assert!(!active.world.first_person);
    let held = active.world.hand.clone();
    let old_model = sim.drawn(sim.walker.movement).unwrap().qualified;
    sim.movements.iter_mut().find(|m| m.qualified == old_model).unwrap().movement_loop.cut_short = false;
    sim.walker.phase = 0.35;
    sim.walker.fresh = false;
    let dagger = active.world.items.objects.iter_mut().find(|item| item.kind.eq_ignore_ascii_case("INVITEM_Dagger")).unwrap();
    dagger.place = Place::World { position: [0.; 3], angles: [0.; 3] };
    active.world.items.make("SmallAx", 1, Place::Carried("Connor".into())).unwrap();
    assert!(active.world.helmetize("contpsd2.anm", "conicbw2.anm", "second alias"));
    sim.hands_off = true;
    sim.refresh_inventory_lists(&mut active.world).unwrap();
    assert!(!sim.walker.animation_input_lock && sim.hands_off);
    assert_eq!(sim.walker.movement, old_model);
    assert_eq!(sim.walker.phase, 0.35);
    assert!(sim.readying && sim.pending_set.is_some());
    assert_eq!((sim.set_mode, active.world.weapon_mode), (WeaponMode::Hand, WeaponMode::Hand));
    assert_eq!(active.world.hand, held);
    assert!(!active.world.first_person_key(sim.hands_off || sim.walker.animation_input_lock));
    sim.hands_off = false;
    assert!(active.world.first_person_key(sim.hands_off || sim.walker.animation_input_lock));
    sim.refresh_perspective(&active.world).unwrap();
    assert_eq!(sim.weapon_set.as_deref(), Some("fpother.anm"), "current SmallAx, not old Dagger or pending ranged mode");
    assert_eq!(sim.weapon_target.as_deref(), Some("fpranged.anm"));
    assert_eq!(sim.pending_set.as_ref().unwrap().0.as_deref(), Some("fpranged.anm"));
    assert_eq!(sim.walker.movement, old_model);
    assert!(!sim.walker.animation_input_lock && sim.readying);

    // Existing checkpoint fields preserve the unlocked old clip while
    // subsequent list selection continues to use current equipment.
    let special = sim.saved_special().unwrap();
    let weapon = sim.saved_weapon_state(Some(&special)).unwrap();
    let timing = sim.saved_hit_timing().unwrap();
    let (weapon, special, timing): (SavedWeaponState, SavedSpecial, SavedHitTiming) = serde_json::from_slice(&serde_json::to_vec(&(weapon, special, timing)).unwrap()).unwrap();
    let mut restored = controller();
    restored.first_person = weapon.actor_first_person(false);
    restored.movements.iter_mut().find(|m| m.qualified == old_model).unwrap().movement_loop.cut_short = false;
    restored.restore_weapon_state(&weapon, Some(&special)).unwrap();
    restored.restore_special(&special).unwrap();
    restored.restore_hit_timing(&timing).unwrap();
    assert!(restored.walker.swap_owned_special && !restored.walker.animation_input_lock);
    assert_eq!((restored.walker.movement, restored.walker.phase), (old_model, 0.35));

    // Replace the pending mode's equipment after refresh. The queued out is
    // rebound before advance; the actual assignment still waits for away.
    let old_ranged = active.world.items.objects.iter_mut().find(|item| item.kind.eq_ignore_ascii_case("INVITEM_SmCrossBow")).unwrap();
    old_ranged.place = Place::World { position: [0.; 3], angles: [0.; 3] };
    active.world.items.make("CrossBow", 1, Place::Carried("Connor".into())).unwrap();
    tick(&mut sim, &mut active);
    assert_eq!(sim.pending_set.as_ref().unwrap().0.as_deref(), Some("fpnewranged.anm"));
    assert!(matches!(sim.walker.special.get(1), Some(kq8_core::connor::SpecialStep::Play("fpnewranged.anm/out", _))));
    assert_eq!(active.world.weapon_mode, WeaponMode::Hand);
    assert_eq!(active.world.hand, held);
    for _ in 0..40 {
        if sim.set_mode == WeaponMode::Ranged { break; }
        tick(&mut sim, &mut active);
    }
    assert_eq!(sim.weapon_set.as_deref(), Some("fpnewranged.anm"));
    assert!(active.world.weapon_ready.as_deref().is_some_and(|item| item.eq_ignore_ascii_case("INVITEM_CrossBow")));
    assert!(active.world.hand.is_none(), "ordinary assignment clears only at completion");
    assert!(sim.walker.animation_input_lock, "new out movement relocks input");
    assert!(!active.world.first_person_key(sim.hands_off || sim.walker.animation_input_lock));
    for _ in 0..40 { if !sim.readying { break; } tick(&mut sim, &mut active); }
    assert!(!sim.readying && !sim.walker.swap_owned_special, "locked swap finishes normally");

    let mut same_name = controller();
    same_name.walker.movement = "away";
    same_name.walker.start_swap_special(vec![kq8_core::connor::SpecialStep::Play("away", "away".into())]);
    assert!(same_name.walker.animation_input_lock, "first controller request locks even when an unrelated clip had the same name");
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original Temple1 banks through the actual late-loader then refresh schedule"]
fn original_temple_armor_live_banks_same_update_matrix() {
    let (game, types) = install();
    let mut active = actor(&types);
    active.world.weapon_mode = WeaponMode::Hand;
    active.world.weapon_ready = Some("INVITEM_Dagger".into());
    let held = active.world.hand.clone();
    let mut sim = controller();
    sim.switch_set(Some("oldhand.anm".into()), WeaponMode::Hand);
    sim.weapon_target = Some("oldhand.anm".into());
    sim.target_mode = WeaponMode::Hand;
    sim.ready_item = active.world.weapon_ready.clone();
    sim.movements.retain(|movement| {
        !["conner2.anm/", "contpsd2.anm/", "conicbw2.anm/"]
            .iter()
            .any(|prefix| movement.qualified.starts_with(prefix))
    });
    sim.late = Some(crate::load::temple_armor_late_fixture(&game));
    sim.late.as_mut().unwrap().sets.push(("invitem_dagger".into(), "CONTPSD2.ANM".into()));
    assert!(active
        .world
        .helmetize("contpsd2.anm", "conicbw2.anm", "same update"));
    assert!(sim.refresh_inventory_lists(&mut active.world).is_err());
    assert!(
        active
            .world
            .connor_fight
            .as_ref()
            .unwrap()
            .list_refresh_pending
    );
    assert_eq!(sim.weapon_set.as_deref(), Some("oldhand.anm"));
    assert_eq!(sim.missing_inventory_banks(&active.world).len(), 3);
    let candidates = vec![("legacy".into(), "CONTPSD2.ANM".into(), false),
        ("live".into(), "contpsd2.anm".into(), false), ("stale".into(), "OLDHAND.ANM".into(), false),
        ("separate view".into(), "contpsd2.anm".into(), true)];
    let planned = plan_late_banks(&sim.movements, candidates);
    assert_eq!(planned.len(), 2, "one new allocation per filename/view, no loaded bank allocation");
    assert!(!planned[0].2 && planned[1].2);
    let light = ObjectLight::new(
        Default::default(),
        crate::light_view::LightMode::X1,
        kq8_data::haze::Haze::new(1000., 2000., [0.; 3]),
        false,
    );
    let mut app = App::new();
    app.insert_resource(sim)
        .insert_resource(crate::door_view::DoorSim::for_test(
            active,
            Log::create("armour-live").unwrap(),
        ))
        .insert_resource(light)
        .insert_resource(Shared {
            log: std::sync::Arc::new(std::sync::Mutex::new(
                Log::create("armour-live-loader").unwrap(),
            )),
            stats: Default::default(),
        })
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ObjectMaterial>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<bevy::render::storage::ShaderBuffer>>()
        .add_systems(Update, (late_set_step, inventory_list_step).chain());
    app.world_mut().run_schedule(Update);
    let sim = app.world().resource::<ConnorSim>();
    let active = &app.world().resource::<crate::door_view::DoorSim>().active;
    assert!(sim.missing_inventory_banks(&active.world).is_empty());
    assert!(
        !active
            .world
            .connor_fight
            .as_ref()
            .unwrap()
            .list_refresh_pending
    );
    assert_eq!(sim.weapon_set.as_deref(), Some("contpsd2.anm"));
    assert_eq!(active.world.hand, held);
    for bank in ["conner2.anm", "contpsd2.anm", "conicbw2.anm"] {
        let name = format!("{bank}/stop");
        let movement = sim
            .movements
            .iter()
            .find(|m| m.qualified.eq_ignore_ascii_case(&name))
            .unwrap();
        assert!(
            movement.frames > 0 && movement.movement_loop.duration > 0.,
            "{name}"
        );
        assert!(!movement.first_person);
    }
    let allocations = (app.world().resource::<Assets<ObjectMaterial>>().len(), app.world().resource::<Assets<Image>>().len());
    app.world_mut().resource_mut::<ConnorSim>().late.as_mut().unwrap().sets
        .push(("invitem_dagger".into(), "CONTPSD2.ANM".into()));
    app.world_mut().run_schedule(Update);
    assert_eq!((app.world().resource::<Assets<ObjectMaterial>>().len(), app.world().resource::<Assets<Image>>().len()), allocations,
        "stale startup entry must be removed without allocating its already loaded bank again");
}
