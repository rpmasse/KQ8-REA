//! Fresh active world-item registration, distinct from script placement (Q203).
use crate::active::Active;
use crate::item::{ItemObject, Items};
use crate::trap::{SavedTraps, Trap, TrapSettings};

fn world(scripts: &[(&str, &str)]) -> Active {
    let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".into()],
        scripts.iter().map(|(name, text)| ((*name).into(), text.as_bytes().to_vec())));
    a.world.items = Items::new(crate::item::tests::types(),
        [("crystl.kqs".into(), [-5., -5., 0.], [5., 5., 10.])]);
    a
}

fn plate(a: &mut Active, name: &str, position: [f32; 3], half: f32, mask: u32, script: &str) {
    a.add_traps([Trap::new(TrapSettings { name: name.into(), position,
        min: [-half; 3], max: [half; 3], script: Some(script.into()),
        lookup_call: false, mask, trap_type: 0, inner_percent: None, spirit: None })]);
}

#[test]
fn fresh_stationary_item_enters_before_the_next_script_line_and_resolves_its_location() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == On\n set Test::visitor $3\n set Test::seen $Test::phase\n getLoc $3\n set Test::loc $KQObject::loc\n checkDistance $1 $3 1\n set Test::near $Console::Return\nendif\nif test $2 == Cue\n set Test::cue $3 $4\nendif\n"),
        ("make.cs", "set Test::phase before\nKQMonster::inventory world Crystal 1 100 200 300\nset Test::phase after\nset Test::first $NewInvItem\nset Test::completed $Test::visitor\nKQMonster::inventory world Crystal 1 100 200 300\nset Test::second $NewInvItem\n"),
    ]);
    plate(&mut a, "plate", [100., 200., 300.], 20., 512, "plate.cs");
    assert!(a.run_script("make.cs", &[]));
    let first = a.scripts.vars.get("Test::first").unwrap();
    let second = a.scripts.vars.get("Test::second").unwrap();
    assert_ne!(first, second);
    assert_eq!(a.scripts.vars.get("Test::seen"), Some("before"));
    assert_eq!(a.scripts.vars.get("Test::completed"), Some(first));
    assert_eq!(a.scripts.vars.get("Test::near"), Some("1"));
    assert!(a.scripts.vars.get("Test::loc").unwrap().starts_with("100 200 300"));
    assert_eq!(a.scripts.vars.get("Test::cue"), Some(format!("301 {second}").as_str()));
    assert_eq!(a.world.traps[0].inside, [first, second]);
    assert!(a.world.flights.is_empty(), "registration does not manufacture a flight");
    assert_eq!(a.world.items.get(first).unwrap().placement().unwrap().position, [100., 200., 300.]);
}

#[test]
fn fresh_registration_rejects_outside_missing_shape_and_nonmatching_mask() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == On\n set Test::entered $3\nendif\n"),
        ("make.cs", "KQMonster::inventory world Crystal 1 100 0 0\nKQMonster::inventory world SilverCoins 1 0 0 0\nKQMonster::inventory world Crystal 1 1000 0 0\n"),
    ]);
    plate(&mut a, "items", [0., 0., 5.], 10., 512, "plate.cs");
    plate(&mut a, "connor_only", [1000., 0., 5.], 10., 4, "plate.cs");
    assert!(a.run_script("make.cs", &[]));
    assert_eq!(a.world.items.objects.len(), 3, "all valid items are still created");
    assert!(a.world.traps.iter().all(|t| t.inside.is_empty()));
    assert_eq!(a.scripts.vars.get("Test::entered"), None);
}

#[test]
fn fresh_registration_rotates_local_ball_centres_and_keeps_the_actual_radius() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == On\n set Test::entered $1\nendif\n"),
        ("make.cs", "KQMonster::inventory world Crystal 1 0 0 0 0 1.570796327 0\n"),
    ]);
    // Radius 1, local centres z=1,3,...,19. A quarter turn about Y
    // puts them along +X. The authored item-table radius is much larger.
    a.world.items = Items::new(crate::item::tests::types(),
        [("crystl.kqs".into(), [-1., -1., 0.], [1., 1., 20.])]);
    plate(&mut a, "touching_end", [20.25, 0., 0.], 0.25, 512, "plate.cs");
    plate(&mut a, "past_end", [20.6, 0., 0.], 0.1, 512, "plate.cs");
    plate(&mut a, "unturned_end", [0., 0., 19.], 0.1, 512, "plate.cs");
    assert!(a.run_script("make.cs", &[]));
    let item = a.scripts.vars.get("NewInvItem").unwrap();
    assert_eq!(a.world.traps[0].inside, [item]);
    assert!(a.world.traps[1].inside.is_empty());
    assert!(a.world.traps[2].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::entered"), Some("touching_end"));
}

#[test]
fn registration_callback_disabling_the_next_trap_prevents_its_contact() {
    let mut a = world(&[
        ("first.cs", "if test $2 == On\n set Test::first $3\n KQObject::collision second -512\nendif\n"),
        ("second.cs", "if test $2 == On\n set Test::second $3\nendif\n"),
    ]);
    plate(&mut a, "first", [0.; 3], 20., 512, "first.cs");
    plate(&mut a, "second", [0.; 3], 20., 512, "second.cs");
    assert_eq!(a.console_command("KQMonster::inventory world Crystal 1 0 0 0", "test").as_deref(), Some("0"));
    let item = a.scripts.vars.get("NewInvItem").unwrap();
    assert_eq!(a.world.traps[0].inside, [item]);
    assert_eq!(a.world.traps[1].settings.mask, 0);
    assert!(a.world.traps[1].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::second"), None);
}

#[test]
fn registration_callback_moving_the_new_item_changes_later_overlap_checks() {
    let mut a = world(&[
        ("first.cs", "if test $2 == On\n set Test::first $3\n move $3 0 1000 0 0\nendif\n"),
        ("second.cs", "if test $2 == On\n set Test::second $3\nendif\n"),
    ]);
    plate(&mut a, "first", [0.; 3], 20., 512, "first.cs");
    plate(&mut a, "second", [0.; 3], 20., 512, "second.cs");
    assert_eq!(a.console_command("KQMonster::inventory world Crystal 1 0 0 0", "test").as_deref(), Some("0"));
    let item = a.scripts.vars.get("NewInvItem").unwrap();
    assert_eq!(a.scripts.vars.get("Test::first"), Some(item));
    assert_eq!(a.scripts.vars.get("Test::second"), None);
    assert!(a.world.traps[1].inside.is_empty());
    assert_eq!(a.world.items.get(item).unwrap().placement().unwrap().position, [1000., 0., 0.]);
    assert_eq!(a.world.place_of(item).unwrap().position, [1000., 0., 0.]);
}

#[test]
fn registration_callback_deleting_the_new_item_prevents_later_contact_delivery() {
    let mut a = world(&[
        ("first.cs", "if test $2 == On\n set Test::first $3\n deleteObject $3\nendif\n"),
        ("second.cs", "if test $2 == On\n set Test::second $3\nendif\n"),
    ]);
    plate(&mut a, "first", [0.; 3], 20., 512, "first.cs");
    plate(&mut a, "second", [0.; 3], 20., 512, "second.cs");
    assert_eq!(a.console_command("KQMonster::inventory world Crystal 1 0 0 0", "test").as_deref(), Some("0"));
    let item = a.scripts.vars.get("Test::first").unwrap();
    assert!(a.world.is_deleted(item));
    assert_eq!(a.scripts.vars.get("Test::second"), None);
    assert!(a.world.traps[1].inside.is_empty());
}

#[test]
fn moving_an_existing_world_item_into_a_trap_does_not_register_it_again() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == On\n set Test::entered $1\nendif\n"),
        ("move.cs", "KQMonster::inventory world Crystal 1 -100 0 0\nset Test::item $NewInvItem\nmove $Test::item 0 100 0 0\n"),
    ]);
    plate(&mut a, "crossed", [0.; 3], 20., 512, "plate.cs");
    plate(&mut a, "destination", [100., 0., 0.], 20., 512, "plate.cs");
    assert!(a.run_script("move.cs", &[]));
    let item = a.scripts.vars.get("Test::item").unwrap();
    assert_eq!(a.world.items.get(item).unwrap().placement().unwrap().position, [100., 0., 0.]);
    assert!(a.world.traps.iter().all(|t| t.inside.is_empty()));
    assert_eq!(a.scripts.vars.get("Test::entered"), None);
}

#[test]
fn restored_world_item_and_saved_membership_do_not_repeat_registration_callbacks() {
    let scripts = [("plate.cs", "if test $2 == On\n set Test::entered $3\nendif\nif test $2 == Cue\n set Test::cue $3 $4\nendif\n")];
    let mut original = world(&scripts);
    plate(&mut original, "plate", [0.; 3], 20., 512, "plate.cs");
    assert_eq!(original.console_command("KQMonster::inventory world Crystal 1 0 0 0", "test").as_deref(), Some("0"));
    let name = original.scripts.vars.get("NewInvItem").unwrap().to_string();
    assert_eq!(original.world.traps[0].inside, [name.as_str()]);
    let bytes = serde_json::to_vec(&(original.world.items.objects.clone(), SavedTraps::capture(&original.world.traps))).unwrap();
    let (objects, traps): (Vec<ItemObject>, SavedTraps) = serde_json::from_slice(&bytes).unwrap();
    let mut restored = world(&scripts);
    plate(&mut restored, "plate", [0.; 3], 20., 512, "plate.cs");
    restored.world.items.restore_objects(objects, 1, 0);
    restored.world.known.insert(name.to_ascii_lowercase());
    traps.restore(&mut restored.world.traps, &restored.world.known).unwrap();
    assert_eq!(restored.console_command(&format!("getLoc {name}"), "restored item").as_deref(), Some("0"));
    assert_eq!(restored.world.traps[0].inside, [name]);
    assert_eq!(restored.scripts.vars.get("Test::entered"), None);
    assert_eq!(restored.scripts.vars.get("Test::cue"), None);
    assert!(restored.world.flights.is_empty());
}

#[test]
fn nested_fresh_rock_drops_keep_each_commands_post_callback_new_inv_item() {
    let mut a = world(&[
        ("outer_plate.cs", "if test $2 == On\n set Test::outerVisitor $3\n set Test::outerBefore $NewInvItem\n KQMonster::dropItem Connor Rock 1000 0 0 0 0 0\n set Test::childAfter $NewInvItem\nendif\n"),
        ("inner_plate.cs", "if test $2 == On\n set Test::innerVisitor $3\n set Test::innerBefore $NewInvItem\nendif\n"),
        ("drop.cs", "KQMonster::dropItem Connor Rock 0 0 0 0 0 0\nset Test::outerAfter $NewInvItem\n"),
    ]);
    let t = &mut a.world.items.types.types[0];
    t.name = "INVITEM_Rock".into();
    t.category = "Rock".into();
    t.stats = [0; 5];
    plate(&mut a, "outer_plate", [0.; 3], 20., 512, "outer_plate.cs");
    plate(&mut a, "inner_plate", [1000., 0., 0.], 20., 512, "inner_plate.cs");
    assert_eq!(a.console_command("give Rock 3", "setup").as_deref(), Some("0"));
    let carried = a.world.items.find("Connor", "Rock").unwrap().name.clone();
    assert!(a.run_script("drop.cs", &[]));
    let outer = a.scripts.vars.get("Test::outerVisitor").unwrap();
    let inner = a.scripts.vars.get("Test::innerVisitor").unwrap();
    assert_ne!(outer, carried);
    assert_ne!(inner, carried);
    assert_ne!(outer, inner);
    assert_eq!(a.scripts.vars.get("Test::outerBefore"), Some(carried.as_str()));
    assert_eq!(a.scripts.vars.get("Test::innerBefore"), Some(carried.as_str()));
    assert_eq!(a.scripts.vars.get("Test::childAfter"), Some(inner));
    assert_eq!(a.scripts.vars.get("Test::outerAfter"), Some(outer));
    assert_eq!(a.scripts.vars.get("NewInvItem"), Some(outer));
    assert_eq!(a.world.items.count("Connor", "INVITEM_Rock"), 1);
    assert_eq!(a.world.items.get(outer).unwrap().quantity, 1);
    assert_eq!(a.world.items.get(inner).unwrap().quantity, 1);
    assert_eq!(a.world.traps[0].inside, [outer]);
    assert_eq!(a.world.traps[1].inside, [inner]);
}

#[test]
fn carried_item_drop_identity_activation_and_continuation_matrix() {
    use crate::item::{Place, RegionItems, StoredPose};
    let empty = || std::iter::empty::<String>();
    for rock in [false, true] {
        for quantity in [1, 3] {
            for old_near in [false, true] {
                let mut a = world(&[
                    ("old.cs", "if test $2 == On\n set T::oldVisitor $3\n set T::oldBefore $NewInvItem\nendif\nif test $2 == Off\n set T::left $3\nendif\n"),
                    ("landing.cs", "if test $2 == On\n set T::landingVisitor $3\n set T::landingBefore $NewInvItem\n set T::landings $T::landings one\nendif\n"),
                    ("drop.cs", "set NewInvItem sentinel\n KQMonster::dropItem Connor Crystal 1000 0 0 0 0 0\n set T::after $NewInvItem\n"),
                ]);
                a.world.items.types.types[0].stats = [0; 5];
                if rock { a.world.items.types.types[0].category = "Rock".into(); }
                let reference = if old_near { [0.; 3] } else { [1000., 0., 0.] };
                let visible = if old_near { 30000. } else { 500. };
                a.sync_item_activity(reference, visible, empty());
                plate(&mut a, "old", [0.; 3], 20., 512, "old.cs");
                plate(&mut a, "landing", [1000., 0., 0.], 20., 512, "landing.cs");
                a.console_command(&format!("give Crystal {quantity}"), "setup");
                let selected = a.world.items.find("Connor", "Crystal").unwrap().name.clone();
                assert!(a.run_script("drop.cs", &[]));
                let after = a.scripts.vars.get("T::after").unwrap().to_string();
                let old = a.world.items.get(&selected).unwrap();
                assert_eq!(old.quantity, if quantity == 1 { 1 } else { quantity - 1 });
                assert_eq!(old.carried_by("Connor"), quantity > 1);
                assert_eq!(a.scripts.vars.get("T::oldVisitor"), (quantity == 1 && old_near).then_some(selected.as_str()));
                assert_eq!(a.scripts.vars.get("T::oldBefore"), (quantity == 1 && old_near).then_some("sentinel"));
                if rock {
                    assert_ne!(after, selected, "Rock always creates a fresh landing identity");
                    assert_eq!(a.world.items.objects.len(), 2);
                    assert_eq!(a.world.items.get(&after).unwrap().quantity, 1);
                    assert_eq!(a.world.items.get(&after).unwrap().placement().unwrap().position, [1000., 0., 0.]);
                    assert_eq!(a.scripts.vars.get("T::landingVisitor"), Some(after.as_str()));
                    assert_eq!(a.scripts.vars.get("T::landingBefore"), Some(selected.as_str()));
                    if quantity == 1 {
                        assert_eq!(old.placement().unwrap().position, [0.; 3]);
                        assert_eq!(a.world.traps[0].inside, if old_near { vec![selected.clone()] } else { vec![] });
                    }
                } else {
                    assert_eq!(after, selected);
                    assert_eq!(a.world.items.objects.len(), 1, "non-Rock never splits a new world object");
                    assert_eq!(old.stored_pose().position, [1000., 0., 0.]);
                    assert_eq!(a.world.place_of(&selected).unwrap().position, [1000., 0., 0.]);
                    assert_eq!(a.scripts.vars.get("T::landingVisitor"), None, "zero-time destination placement is not new registration");
                    assert!(a.world.traps[0].inside.is_empty());
                    assert_eq!(a.scripts.vars.get("T::left"), (quantity == 1 && old_near).then_some(selected.as_str()));
                    // The old inactive object can activate at its destination
                    // on the next ordinary frame, but a carried stack cannot.
                    a.sync_item_activity(reference, visible, empty());
                    let entered = quantity == 1 && !old_near;
                    assert_eq!(a.scripts.vars.get("T::landingVisitor"), entered.then_some(selected.as_str()));
                    a.sync_item_activity(reference, visible, empty());
                    assert_eq!(a.scripts.vars.get("T::landings"), entered.then_some("one"));
                }
                assert!(a.world.flights.is_empty());
                println!("drop matrix: Rock={rock} quantity={quantity} old_near={old_near} passed");
            }
        }
    }

    // A nested callback during old-identity registration finishes before the
    // outer removal publishes its selected name, then a fresh Rock registers.
    let mut a = world(&[
        ("old.cs", "if test $2 == On\n set T::oldBefore $NewInvItem\n KQMonster::inventory world SilverCoins 1 3000 0 0\n set T::nested $NewInvItem\nendif\n"),
        ("landing.cs", "if test $2 == On\n set T::landingBefore $NewInvItem\n move $3 0 2000 0 0\nendif\n"),
        ("later.cs", "if test $2 == On\n set T::wrongLater $3\nendif\n"),
        ("drop.cs", "set NewInvItem sentinel\n KQMonster::dropItem Connor Crystal 1000 0 0 0 0 0\n set T::after $NewInvItem\n"),
    ]);
    a.world.items.types.types[0].category = "Rock".into();
    a.world.items.types.types[0].stats = [0; 5];
    plate(&mut a, "old", [0.; 3], 20., 512, "old.cs");
    plate(&mut a, "landing", [1000., 0., 0.], 20., 512, "landing.cs");
    plate(&mut a, "later", [1000., 0., 0.], 20., 512, "later.cs");
    a.console_command("give Crystal 1", "setup");
    let selected = a.world.items.find("Connor", "Crystal").unwrap().name.clone();
    a.run_script("drop.cs", &[]);
    assert_eq!(a.scripts.vars.get("T::oldBefore"), Some("sentinel"));
    assert_eq!(a.scripts.vars.get("T::landingBefore"), Some(selected.as_str()));
    let fresh = a.scripts.vars.get("T::after").unwrap();
    assert_ne!(Some(fresh), a.scripts.vars.get("T::nested"));
    assert_eq!(a.world.items.get(fresh).unwrap().placement().unwrap().position, [2000., 0., 0.]);
    assert_eq!(a.scripts.vars.get("T::wrongLater"), None);

    // Carried pose survives disk item/journey serialization and a departed
    // region's non-Connor inventory. Missing legacy pose means origin only.
    let mut a = world(&[("stored.cs", "if test $2 == On\n set T::storedVisitor $3\nendif\n")]);
    a.console_command("give Crystal 3", "setup");
    a.console_command("KQMonster::dropItem Connor Crystal 100 200 300 0.1 0.2 0.3", "first non-Rock drop");
    let selected = a.world.items.find("Connor", "Crystal").unwrap().clone();
    let pose = StoredPose { position: [100., 200., 300.], angles: [0.1, 0.2, 0.3] };
    assert_eq!(selected.carried_pose, pose);
    let disk: ItemObject = serde_json::from_slice(&serde_json::to_vec(&selected).unwrap()).unwrap();
    assert_eq!(disk, selected);
    let journey = crate::journey::Journey::take(&a.world, &a.scripts.vars, "Daventry");
    let journey: crate::journey::Journey = serde_json::from_slice(&serde_json::to_vec(&journey).unwrap()).unwrap();
    assert_eq!(journey.items[0].carried_pose, pose);
    let mut legacy = serde_json::to_value(&selected).unwrap();
    legacy.as_object_mut().unwrap().remove("carried_pose");
    assert_eq!(serde_json::from_value::<ItemObject>(legacy).unwrap().carried_pose, StoredPose::default());
    let owner = a.world.items.make("SilverCoins", 2, Place::Carried("Wizard".into())).unwrap();
    owner.carried_pose = pose;
    let owner_name = owner.name.clone();
    let region = RegionItems::capture(&a.world.items, &Default::default());
    let region: RegionItems = serde_json::from_slice(&serde_json::to_vec(&region).unwrap()).unwrap();
    assert!(region.valid(a.world.items.no_name_counter()));
    let mut rebuilt = crate::item::tests::items();
    region.restore(&mut rebuilt);
    assert_eq!(rebuilt.get(&owner_name).unwrap().carried_pose, pose);
    a.console_command("KQMonster::dropItem Connor Crystal 400 500 600 0 0 0", "second non-Rock drop");
    assert_eq!(a.world.items.get(&selected.name).unwrap().quantity, 1);
    assert_eq!(a.world.items.get(&selected.name).unwrap().carried_pose.position, [400., 500., 600.]);
    plate(&mut a, "stored", [400., 500., 600.], 20., 512, "stored.cs");
    a.console_command("KQMonster::dropItem Connor Crystal 800 900 1000 0 0 0", "last non-Rock drop");
    assert_eq!(a.scripts.vars.get("T::storedVisitor"), Some(selected.name.as_str()));
    assert_eq!(a.world.items.get(&selected.name).unwrap().placement().unwrap().position, [800., 900., 1000.]);
    a.world.carry_item(&selected.name);
    assert_eq!(a.world.items.get(&selected.name).unwrap().carried_pose, StoredPose::default());

    let mut a = world(&[("plate.cs", "if test $2 == On\n set T::entered $T::entered once\nendif\n")]);
    a.initialize_item_activity([0.; 3], 500.);
    plate(&mut a, "far", [1000., 0., 0.], 20., 512, "plate.cs");
    a.console_command("KQMonster::inventory world Crystal 1 1000 0 0", "far creation");
    let name = a.scripts.vars.get("NewInvItem").unwrap().to_string();
    assert_eq!(a.scripts.vars.get("T::entered"), None);
    a.console_command(&format!("makeActive {name} 1"), "pending next frame");
    a.console_command("setActiveCamera 1", "pending camera reference");
    assert_eq!(a.world.item_activity(&name).unwrap().0, kq8_data::objects::Activity::Off);
    a.console_command(&format!("KQObject::setHidden {name} 1"), "hidden before activation");
    a.sync_item_activity([0.; 3], 500., [name.clone()]);
    assert_eq!(a.scripts.vars.get("T::entered"), None, "hidden wins over forced near");
    a.console_command(&format!("KQObject::setHidden {name} 0"), "show again");
    a.sync_item_activity([1000., 0., 0.], 500., empty());
    assert_eq!(a.scripts.vars.get("T::entered"), Some("once"), "the published camera reference permits stationary activation");
    a.sync_item_activity([1000., 0., 0.], 500., empty());
    assert_eq!(a.scripts.vars.get("T::entered"), Some("once"));
    assert!(a.world.flights.is_empty());

    // Exact near/far/off boundaries use the BOUNDS sphere, not table radius.
    let mut a = world(&[]);
    a.world.items = Items::new(crate::item::tests::types(), [("crystl.kqs".into(), [-3., -4., 0.], [3., 4., 0.])]);
    let item = a.world.items.make("Crystal", 1, StoredPose::default().world()).unwrap().name.clone();
    for (distance, expected) in [(88., kq8_data::objects::Activity::Near), (88.01, kq8_data::objects::Activity::Far), (105., kq8_data::objects::Activity::Off)] {
        a.initialize_item_activity([distance, 0., 0.], 100.);
        assert_eq!(a.world.item_activity(&item).unwrap().0, expected);
    }
    a.sync_item_activity([10000., 0., 0.], 100., [item.clone()]);
    assert_eq!(a.world.item_activity(&item).unwrap().0, kq8_data::objects::Activity::Near);
    println!("drop matrix: nested callbacks, late pose, disk/journey/region and legacy pose passed");
}
