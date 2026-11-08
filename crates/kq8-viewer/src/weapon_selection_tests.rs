//! Inventory.md 12a: actual ordinary assignment, separate from requests,
//! internal list changes and recreation checkpoint reconstruction.
use super::*;
use kq8_core::{active::Active, item::{Items, Place}, weapons::{Ask, Request}};
use kq8_data::{characters::ConnorAnimLists, items::ItemTypes, GameDir};

struct Flat;
impl kq8_core::connor_mover::GroundLookup for Flat {
    fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> { Some((0., [0., 0., 1.])) }
}

fn controller() -> ConnorSim {
    // Reuse the real constructor with empty renderer assets, then install
    // short, deterministic clips for the actual swap controller to play.
    let mut world = World::new();
    let placement = Placement::from_section(&kq8_data::kq_text::Section { name: "Connor".into(), entries: vec![] });
    let assets = ConnorAssets { base_list: "fixture.anm".into(), movements: vec![], animations: vec![],
        keys: vec![], camera_keys: vec![], collision_box: None, lips: None,
        potion_looks: Default::default(), late: None };
    let light = ObjectLight::new(Default::default(), crate::light_view::LightMode::X1,
        kq8_data::haze::Haze::new(1000., 2000., [0.; 3]), false);
    let mut queue = bevy::ecs::world::CommandQueue::default();
    spawn_connor(&mut Commands::new(&mut queue, &world), (placement, assets), None,
        &mut Assets::default(), &light, &mut Assets::default(),
        &mut Log::create("weapon-selection-controller").unwrap(), &crate::anim_gpu::AnimGpu::default());
    queue.apply(&mut world);
    let mut sim = world.remove_resource::<ConnorSim>().unwrap();
    for bank in ["", "fists.anm", "bow.anm", "fpown.anm", "fpfists.anm", "fpbow.anm"] {
        for action in ["stop", "away", "out", "draw", "undraw", "throw"] {
            let name: &'static str = Box::leak(if bank.is_empty() { action.to_string() }
                else { format!("{bank}/{action}") }.into_boxed_str());
            sim.movements.push(DrawnMovement { name, qualified: name, first_person: bank.starts_with("fp"),
                movement_loop: MovementLoop { duration: 0.2, frames: 2, repeat: action == "stop",
                    cut_short: true, speed: 1., ..Default::default() }, frames: 2, anim: 0, parts: vec![] });
        }
    }
    sim
}

fn active(types: &ItemTypes) -> (Active, String) {
    let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], []);
    a.world.items = Items::new(types.clone(), []);
    for kind in ["Fists", "SmCrossbow"] {
        a.world.items.make(kind, 1, Place::Carried("Connor".into())).unwrap();
    }
    let held = a.world.items.make("Candle", 1, Place::Carried("Connor".into())).unwrap().name.clone();
    a.world.hand = Some(held.clone());
    a.world.connor_fight = Some(kq8_core::fight::ConnorFight {
        ouch_sound: "sword.aud".into(), min_damage_factor: 1., max_damage_factor: 1.,
        third_person: ConnorAnimLists { stowed: None, hand: vec![("Fists".into(), "fists.anm".into())],
            ranged: vec![("SmCrossbow".into(), "bow.anm".into())] },
        first_person: ConnorAnimLists { stowed: Some("fpown.anm".into()),
            hand: vec![("Fists".into(), "fpfists.anm".into())], ranged: vec![("SmCrossbow".into(), "fpbow.anm".into())] },
        shoot_offsets: vec![], start_projectile: None, start_offset: [0.; 3],
    });
    (a, held)
}

fn request(sim: &mut ConnorSim, active: &mut Active, ask: Ask) -> bool {
    let request = Request { ask, how: "weapon selection matrix".into() };
    let Some(change) = active.world.take_weapon_request(&request, sim.may_ready(), sim.hands_off) else { return false };
    let list = crate::door_view::weapon_list(&active.world);
    sim.ready_weapon_request(&mut active.world, list, &change, false);
    true
}

fn tick(sim: &mut ConnorSim, active: &mut Active) {
    sim.tick_with_state_requests(Some(active), Intent::default(), false, &Flat);
    sim.finish_weapon_step(Some(&mut active.world));
}

fn finish(sim: &mut ConnorSim, active: &mut Active) {
    for _ in 0..40 {
        if !sim.readying { return; }
        tick(sim, active);
    }
    panic!("swap did not finish: {:?} {} {}", sim.walker.mode, sim.walker.movement, sim.walker.phase);
}

fn pending_save(sim: &ConnorSim) -> (SavedWeaponState, SavedSpecial) {
    let special = sim.saved_special().unwrap();
    let weapon = sim.saved_weapon_state(Some(&special)).unwrap();
    let value = serde_json::to_string(&(weapon, special)).unwrap();
    serde_json::from_str(&value).unwrap()
}

fn restore(sim: &mut ConnorSim, weapon: &SavedWeaponState, special: &SavedSpecial) {
    sim.first_person = weapon.actor_first_person(false);
    sim.restore_weapon_state(weapon, Some(special)).unwrap();
    sim.restore_special(special).unwrap();
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original item types and production swap/selection lifecycle"]
fn original_weapon_assignment_selection_and_checkpoint_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let types = ItemTypes::parse(&std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap()).0;

    // None -> fists assigns before draw. Fists -> none keeps the held
    // item throughout undraw, then assigns and clears in that same tick.
    let (mut a, held) = active(&types);
    let mut sim = controller();
    a.world.rope_first_person_lock = true;
    assert!(request(&mut sim, &mut a, Ask::Draw));
    assert_eq!(sim.set_mode, WeaponMode::Hand);
    assert!(a.world.hand.is_none());
    assert!(a.world.items.get(&held).unwrap().carried_by("Connor"));
    assert!(a.world.rope_first_person_lock, "generic assignment does not release rope's independent lock");
    finish(&mut sim, &mut a);
    a.world.hand = Some(held.clone());
    assert!(!request(&mut sim, &mut a, Ask::Script(WeaponMode::Hand)));
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    assert!(request(&mut sim, &mut a, Ask::Away));
    assert_eq!(sim.set_mode, WeaponMode::Hand);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    let mut retained_ticks = 0;
    for _ in 0..40 {
        tick(&mut sim, &mut a);
        if sim.set_mode == WeaponMode::None {
            assert!(a.world.hand.is_none(), "clear shares the actual assignment tick");
            break;
        }
        retained_ticks += 1;
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    }
    assert!(retained_ticks > 0 && sim.set_mode == WeaponMode::None);
    a.world.hand = Some(held.clone());
    finish(&mut sim, &mut a);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()), "completed assignment never clears a later selection twice");

    // Refused or no-op requests never produce an assignment notice.
    for case in 0..6 {
        let (mut a, held) = active(&types);
        let mut sim = controller();
        let ask = match case {
            0 => { sim.hands_off = true; Ask::Draw }
            1 => { sim.walker.start_jump(kq8_core::connor::JumpKind::Stand); Ask::Draw }
            2 => { a.world.connor_roping = true; Ask::Script(WeaponMode::Hand) }
            3 => Ask::Away,
            4 => Ask::Script(WeaponMode::None),
            _ => {
                a.world.items.objects.retain(|o| !o.kind.eq_ignore_ascii_case("INVITEM_SmCrossbow"));
                a.world.items.make("NoRanged", 1, Place::Carried("Connor".into())).unwrap();
                a.world.set_weapon(WeaponMode::Hand, Some(kq8_core::weapons::FISTS.into()));
                sim.ready_set(Some("fists.anm".into()), WeaponMode::Hand,
                    Some(kq8_core::weapons::FISTS.into()), true, false, false);
                Ask::Draw
            }
        };
        assert!(!request(&mut sim, &mut a, ask));
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
        assert_eq!(sim.weapon_selection, WeaponSelection::default());
    }

    // A pending ordinary swap survives both a real F2 bank refresh and
    // own-format serialization. Restoring itself does not clear anything.
    let (mut a, held) = active(&types);
    let mut sim = controller();
    assert!(request(&mut sim, &mut a, Ask::Script(WeaponMode::Ranged)));
    assert_eq!(sim.set_mode, WeaponMode::None);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    a.world.first_person = true;
    assert!(sim.refresh_perspective(&a.world).is_some());
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    assert!(sim.weapon_selection.pending_ordinary);
    let (weapon, special) = pending_save(&sim);
    let mut resumed = controller();
    restore(&mut resumed, &weapon, &special);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    assert!(resumed.weapon_selection.pending_ordinary);
    for _ in 0..40 {
        tick(&mut resumed, &mut a);
        if resumed.set_mode == WeaponMode::Ranged { break; }
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    }
    assert_eq!(resumed.set_mode, WeaponMode::Ranged);
    assert!(a.world.hand.is_none());
    a.world.hand = Some(held.clone());
    finish(&mut resumed, &mut a);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));

    // An old Rust slot cannot distinguish ordinary from internal swaps.
    // Missing origin preserves its old no-clear behavior; no guessed type.
    let mut legacy = serde_json::to_value(&weapon).unwrap();
    legacy.as_object_mut().unwrap().remove("selection");
    let legacy: SavedWeaponState = serde_json::from_value(legacy).unwrap();
    let mut old = controller();
    restore(&mut old, &legacy, &special);
    assert_eq!(old.weapon_selection, WeaponSelection::default());
    finish(&mut old, &mut a);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));

    // A completed-but-undrained notice is also serialized. Loading does
    // not emit another one; its original notice drains exactly once.
    let (mut a, held) = active(&types);
    let mut sim = controller();
    request(&mut sim, &mut a, Ask::Script(WeaponMode::Ranged));
    for _ in 0..40 {
        sim.tick_with_state_requests(Some(&mut a), Intent::default(), false, &Flat);
        sim.weapon_step(); // Deliberately hold the world-side boundary for serialization.
        if sim.weapon_selection.assigned_ordinary { break; }
    }
    assert!(sim.weapon_selection.assigned_ordinary);
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    let (completed, playing) = pending_save(&sim);
    let mut restored = controller();
    restore(&mut restored, &completed, &playing);
    assert!(restored.apply_weapon_assignment(&mut a.world));
    assert!(a.world.hand.is_none());
    a.world.hand = Some(held.clone());
    assert!(!restored.apply_weapon_assignment(&mut a.world));
    let (consumed, playing) = pending_save(&restored);
    let mut restored_again = controller();
    restore(&mut restored_again, &consumed, &playing);
    assert!(!restored_again.apply_weapon_assignment(&mut a.world));
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));

    // Generic internal list synchronization and ThrowRock's temporary
    // away/restore use the preserving path, even when the mode changes.
    let (mut a, held) = active(&types);
    let mut sim = controller();
    sim.ready_set(Some("fists.anm".into()), WeaponMode::Hand, Some(kq8_core::weapons::FISTS.into()), true, false, false);
    assert!(!sim.apply_weapon_assignment(&mut a.world));
    assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
    let rock = a.world.items.make("Rock", 2, Place::Carried("Connor".into())).unwrap().name.clone();
    a.world.hand = Some(rock.clone());
    assert!(sim.start_rock_throw("rock.kq".into(), [100., 0., 0.], None));
    sim.throw_ended.push(sim.walker.movement.into());
    sim.rock_step();
    assert_eq!(sim.set_mode, WeaponMode::None);
    assert!(!sim.apply_weapon_assignment(&mut a.world));
    assert_eq!(a.world.hand.as_deref(), Some(rock.as_str()));
    sim.throw_ended.push(sim.walker.movement.into());
    sim.rock_step();
    assert_eq!(sim.set_mode, WeaponMode::Hand);
    assert!(!sim.apply_weapon_assignment(&mut a.world));
    assert_eq!(a.world.hand.as_deref(), Some(rock.as_str()));

    // If a swap is replaced before its indexed item, the existing
    // weapon_step fallback still assigns; drain belongs to that boundary.
    let (mut a, _) = active(&types);
    let mut sim = controller();
    request(&mut sim, &mut a, Ask::Script(WeaponMode::Ranged));
    sim.walker.enter_talk();
    sim.finish_weapon_step(Some(&mut a.world));
    assert_eq!(sim.set_mode, WeaponMode::Ranged);
    assert!(a.world.hand.is_none());

    // The pending-death end handler has its own production completion
    // boundary; it must drain before ownership passes to death.
    let (mut a, held) = active(&types);
    let mut sim = controller();
    request(&mut sim, &mut a, Ask::Draw);
    finish(&mut sim, &mut a);
    a.world.hand = Some(held.clone());
    request(&mut sim, &mut a, Ask::Away);
    sim.walker.phase = 1.;
    assert!(sim.release_pending_die_after_end("undraw", &mut a.world));
    assert_eq!(sim.set_mode, WeaponMode::None);
    assert!(a.world.hand.is_none());
    assert!(!sim.apply_weapon_assignment(&mut a.world));

    // An explicitly invoked ordinary assignment clears even if its
    // numeric mode is unchanged. A same-mode request above does not.
    a.world.hand = Some(held);
    sim.assign_swap_set(None, WeaponMode::None, true);
    assert!(sim.apply_weapon_assignment(&mut a.world));
    assert!(a.world.hand.is_none());
    println!("weapon selection: immediate/delayed assignment; refusals/no-ops; F2; pending/completed/legacy checkpoints; internal/ThrowRock preservation; interruption boundary");
}
