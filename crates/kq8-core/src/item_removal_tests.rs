//! Live item destruction and carry relocation, from traps.md Q203.
use crate::active::{Active, ActiveChest, ClickResult};
use crate::chest::ChestState;
use crate::item::{Items, Place};
use crate::trap::{Trap, TrapSettings, STICKY};

fn world(scripts: &[(&str, &str)]) -> Active {
    let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".into()],
        scripts.iter().map(|(name, text)| ((*name).into(), text.as_bytes().to_vec())));
    a.world.items = Items::new(crate::item::tests::types(),
        [("crystl.kqs".into(), [-5., -5., 0.], [5., 5., 10.])]);
    a
}

fn plate(a: &mut Active, name: &str, position: [f32; 3], script: &str) {
    a.add_traps([Trap::new(TrapSettings { name: name.into(), position,
        min: [-20.; 3], max: [20.; 3], script: Some(script.into()),
        lookup_call: false, mask: 512, trap_type: STICKY, inner_percent: Some(0.5), spirit: None })]);
}

fn make(a: &mut Active, position: [f32; 3]) -> String {
    assert_eq!(a.console_command(&format!("KQMonster::inventory world Crystal 1 {} {} {}",
        position[0], position[1], position[2]), "fixture").as_deref(), Some("0"));
    a.scripts.vars.get("NewInvItem").unwrap().into()
}

const DEPARTURE: &str = "if test $2 == Off\n set Test::off $3\n set Test::phaseAtOff $Test::phase\n getLoc $3\n set Test::locAtOff $KQObject::loc\nendif\nif test $2 == Cue\n if test $3 == 302\n  set Test::leave $4\n endif\nendif\n";

#[test]
fn merged_pickup_waits_for_outer_script_and_keeps_the_departing_name_and_transform() {
    let mut a = world(&[("plate.cs", DEPARTURE),
        ("pickup.cs", "set Test::phase pickup\nset Test::offInPickup $Test::off\ngetLoc $1\nset Test::pickupLoc $KQObject::loc\n"),
        ("outer.cs", "set Test::phase before\nconClickItem $1\nset Test::afterPickup $Test::phase\nset Test::offBeforeEnd $Test::off\nconClickItem $1\nset Test::duplicate $Console::Return\nset Test::phase finished\n")]);
    let carried = a.world.items.make("Crystal", 2, Place::Carried("Connor".into())).unwrap().name.clone();
    plate(&mut a, "plate", [100., 200., 300.], "plate.cs");
    let name = make(&mut a, [100., 200., 300.]);
    a.console_command(&format!("KQObject::setScript {name} pickup.cs"), "fixture");
    a.scripts.vars.set("Test::off", "none".into());
    assert!(a.run_script("outer.cs", &[name.clone()]));
    assert_eq!(a.scripts.vars.get("Test::offInPickup"), Some("none"));
    assert_eq!(a.scripts.vars.get("Test::offBeforeEnd"), Some("none"));
    assert_eq!(a.scripts.vars.get("Test::afterPickup"), Some("pickup"));
    assert_eq!(a.scripts.vars.get("Test::duplicate"), Some("1"));
    assert_eq!(a.scripts.vars.get("Test::phaseAtOff"), Some("finished"));
    assert_eq!(a.scripts.vars.get("Test::off"), Some(name.as_str()));
    assert_eq!(a.scripts.vars.get("Test::pickupLoc"), a.scripts.vars.get("Test::locAtOff"));
    assert!(a.scripts.vars.get("Test::locAtOff").unwrap().starts_with("100 200 300"));
    assert_eq!(a.world.items.get(&carried).unwrap().quantity, 3);
    assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 3);
    assert!(a.world.traps[0].inside.is_empty() && a.world.traps[0].shrunk);
    assert_eq!(a.world.items.get(&name).unwrap().place, Place::Gone);
    assert!(a.world.is_deleted(&name));
    assert!(a.world.place_of(&name).is_none());
}

#[test]
fn scriptless_nonmerged_pickup_keeps_identity_but_releases_a_distant_plate() {
    let mut a = world(&[("plate.cs", DEPARTURE)]);
    plate(&mut a, "plate", [100., 0., 0.], "plate.cs");
    let name = make(&mut a, [100., 0., 0.]);
    // A pickup resets orientation as well as position; the original item's
    // world orientation must not survive as its carried collider pose.
    a.console_command(&format!("move {name} 0 100 0 0 0 1.2 0.7"), "fixture");
    assert_eq!(a.click(&name, [100., 0., 0.], 100., None), ClickResult::PickedUp(name.clone()));
    assert!(a.world.items.get(&name).unwrap().carried_by("Connor"));
    assert!(!a.world.is_deleted(&name));
    assert_eq!(a.world.place_of(&name).unwrap(), crate::collide::Placement::at([0.; 3]));
    assert!(a.world.traps[0].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::off"), Some(name.as_str()));
    assert!(a.scripts.vars.get("Test::locAtOff").unwrap().starts_with("0 0 0"));
}

#[test]
fn nonmerged_pickup_can_remain_in_an_origin_trap_and_does_not_enter_new_ones() {
    let mut a = world(&[("plate.cs", DEPARTURE)]);
    plate(&mut a, "origin", [0.; 3], "plate.cs");
    let name = make(&mut a, [0.; 3]);
    plate(&mut a, "new_origin", [0.; 3], "plate.cs");
    assert_eq!(a.click(&name, [0.; 3], 100., None), ClickResult::PickedUp(name.clone()));
    assert_eq!(a.world.traps[0].inside, [name.clone()]);
    assert!(!a.world.traps[0].shrunk);
    assert!(a.world.traps[1].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::off"), None);
    // Deletion is different from relocation: it releases even an origin trap.
    assert_eq!(a.console_command(&format!("deleteObject {name}"), "test").as_deref(), Some("0"));
    assert!(a.world.traps[0].inside.is_empty() && a.world.traps[0].shrunk);
    assert_eq!(a.scripts.vars.get("Test::off"), Some(name.as_str()));
}

#[test]
fn deletion_uses_cue_302_until_the_last_occupant_even_with_the_mask_removed() {
    let mut a = world(&[("plate.cs", DEPARTURE)]);
    plate(&mut a, "plate", [100., 0., 0.], "plate.cs");
    let first = make(&mut a, [100., 0., 0.]);
    let second = make(&mut a, [100., 0., 0.]);
    a.console_command("KQObject::collision plate -512", "fixture");
    a.console_command(&format!("deleteObject {}", first.to_ascii_uppercase()), "test");
    assert_eq!(a.world.traps[0].inside, [second.clone()]);
    assert_eq!(a.scripts.vars.get("Test::leave"), Some(first.as_str()));
    assert_eq!(a.scripts.vars.get("Test::off"), None);
    assert!(!a.world.traps[0].shrunk);
    a.console_command(&format!("deleteObject {second}"), "test");
    assert_eq!(a.scripts.vars.get("Test::off"), Some(second.as_str()));
    assert!(a.world.traps[0].shrunk);
}

#[test]
fn explicit_deletion_finishes_the_outer_event_before_callbacks_and_is_not_reentrant() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Off\n set Test::phaseAtOff $Test::phase\n getLoc $3\n set Test::locAtOff $KQObject::loc\n deleteObject $3\n set Test::duplicate $Console::Return\n deleteObject $Test::other\nendif\n"),
        ("other.cs", "if test $2 == Off\n getLoc $3\n set Test::otherLoc $KQObject::loc\n set Test::otherOff $3\nendif\n"),
        ("delete.cs", "set Test::phase before\ndeleteObject $1\ngetLoc $1\nset Test::during $KQObject::loc\nset Test::phase after\n")]);
    plate(&mut a, "plate", [100., 0., 0.], "plate.cs");
    plate(&mut a, "other", [200., 0., 0.], "other.cs");
    let first = make(&mut a, [100., 0., 0.]);
    let second = make(&mut a, [200., 0., 0.]);
    a.scripts.vars.set("Test::other", second.clone());
    assert!(a.run_script("delete.cs", &[first.clone()]));
    assert_eq!(a.scripts.vars.get("Test::phaseAtOff"), Some("after"));
    assert_eq!(a.scripts.vars.get("Test::during"), a.scripts.vars.get("Test::locAtOff"));
    assert_eq!(a.scripts.vars.get("Test::duplicate"), Some("1"));
    assert_eq!(a.scripts.vars.get("Test::otherOff"), Some(second.as_str()));
    assert!(a.scripts.vars.get("Test::otherLoc").unwrap().starts_with("200 0 0"));
    assert!(a.world.traps.iter().all(|t| t.inside.is_empty()));
    assert!(a.world.is_deleted(&first) && a.world.is_deleted(&second));
    let log = a.take_item_log();
    assert_eq!(log.iter().filter(|line| line.contains("item destruction") && line.contains(": trap ")).count(), 2);
}

#[test]
fn chest_pickup_merges_then_releases_the_remembered_world_item() {
    let mut a = world(&[("plate.cs", DEPARTURE)]);
    let chest = crate::chest::tests::test_chest(Some("INVITEM_Crystal"));
    let position = chest.settings.item_position;
    let chest_name = chest.name.clone();
    a.add_chests([ActiveChest { chest, script: None,
        bounds: Some(([-480., -310., 0.], [470., 290., 2060.])),
        collision_type: crate::chest::CHEST_TYPE, collision_mask: 0,
        hidden: false, do_not_point_at: true, gone: false }]);
    plate(&mut a, "plate", position, "plate.cs");
    a.world.items.make("Crystal", 2, Place::Carried("Connor".into())).unwrap();
    assert_eq!(a.click(&chest_name, [0., -1310., 0.], 2350., None),
        ClickResult::Chest { before: ChestState::Closed, after: ChestState::Open });
    let name = a.world.chests[0].chest.remembered.clone().unwrap();
    let quantity = a.world.items.get(&name).unwrap().quantity;
    assert_eq!(a.world.traps[0].inside, [name.clone()]);
    assert_eq!(a.click(&chest_name, [0., -1310., 0.], 2350., None),
        ClickResult::Chest { before: ChestState::Open, after: ChestState::Empty });
    assert!(a.world.traps[0].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::off"), Some(name.as_str()));
    assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 2 + quantity);
    assert_eq!(a.world.items.get(&name).unwrap().place, Place::Gone);
}

#[test]
fn item_destruction_uses_the_spirit_traps_own_departure_handler() {
    let mut a = world(&[("generic.cs", "if test $2 == Off\n set Test::wrongHandler 1\nendif\n")]);
    plate(&mut a, "spirit", [100., 0., 0.], "generic.cs");
    // The trap was activated by Connor, with a surviving non-Connor occupant.
    a.world.traps[0].settings.spirit = Some(crate::spirit_trap::SpiritTrap {
        trap_status: 1, ..Default::default()
    });
    let name = make(&mut a, [100., 0., 0.]);
    assert_eq!(a.world.traps[0].inside, [name.clone()]);
    a.console_command(&format!("deleteObject {name}"), "test");
    assert!(a.world.traps[0].inside.is_empty());
    assert_eq!(a.world.traps[0].settings.spirit.unwrap().trap_status, 0);
    assert_eq!(a.scripts.vars.get("Test::wrongHandler"), None);
    assert!(a.world.is_deleted(&name));
}
