//! Script placement is a final-position stay check, never an entry sweep.
//! Synthetic cases for traps.md rule 5(b), using actual accepted pushes to enter.
use crate::active::{Active, Clickable, Request};
use crate::collide::{BoxCollider, ColliderKind, Placement};
use crate::connor_mover::{tests::Flat, MoverWorld, Scene};
use crate::trap::{Trap, TrapSettings};

fn world(scripts: &[(&str, &str)]) -> Active {
    Active::new(Default::default(), vec![], vec![], ["Connor".into()],
        scripts.iter().map(|(name, text)| ((*name).into(), text.as_bytes().to_vec())))
}

fn plate(a: &mut Active, name: &str, position: [f32; 3], half: f32, script: &str) {
    a.add_traps([Trap::new(TrapSettings { name: name.into(), position,
        min: [-half; 3], max: [half; 3], script: Some(script.into()),
        lookup_call: false, mask: 1, trap_type: 0, inner_percent: None, spirit: None })]);
}

fn prop(name: &str, position: [f32; 3], balls: bool) -> BoxCollider {
    BoxCollider { name: name.into(), min: [-5.; 3], max: [5.; 3],
        placement: Placement::at(position), kind: ColliderKind { collision_type: 0x121,
            collision_mask: 11, ball_stack: balls, keep_on_ground: false } }
}

fn register(a: &mut Active, b: &BoxCollider) {
    a.world.known.insert(b.name.to_ascii_lowercase());
    a.world.prop_placements.insert(b.name.to_ascii_lowercase(), None);
    a.world.clickables.push(Clickable::plain(&b.name, "KQObject", b.min, b.max,
        b.placement, b.kind.collision_type));
    a.register_prop_colliders([b.clone()]);
}

fn push(a: &mut Active, b: &mut BoxCollider, delta: [f32; 3]) -> Vec<String> {
    let objects = [b.clone()];
    let ground = Flat(-1000.);
    let scene = Scene::new(&ground, &[], &objects);
    assert_eq!(scene.push_object(&b.name, delta).0, delta);
    let moves = scene.moved.take();
    assert_eq!(moves.len(), 1);
    let calls = a.prop_moved(b, &moves[0]);
    b.placement.position = std::array::from_fn(|i| b.placement.position[i] + delta[i]);
    calls
}

#[test]
fn script_out_and_back_keeps_occupancy_and_exposes_each_logical_placement() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Off\n set Test::off 1\nendif\n"),
        ("roundtrip.cs", "move crate 0 100 0 0\ngetLoc crate\nset Test::away $KQObject::loc\nmove crate 0 0 0 0\ngetLoc crate\nset Test::back $KQObject::loc\n"),
    ]);
    plate(&mut a, "plate", [0.; 3], 20., "plate.cs");
    let mut b = prop("crate", [-100., 0., 0.], false);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]), ["plate On crate"]);
    assert!(a.run_script("roundtrip.cs", &[]));
    assert_eq!(a.scripts.vars.get("Test::off"), None);
    assert_eq!(a.world.traps[0].inside, ["crate"]);
    assert!(a.scripts.vars.get("Test::away").unwrap().starts_with("100 0 0"));
    assert!(a.scripts.vars.get("Test::back").unwrap().starts_with("0 0 0"));
    assert_eq!(a.world.prop_placements["crate"].unwrap().position, [0.; 3]);
    a.world.clickables.clear();
    assert_eq!(a.world.place_of("crate").unwrap().position, [0.; 3]);
}

#[test]
fn nested_script_departure_waits_for_outer_state_and_requests() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Off\n set Test::seen $Test::phase\n getLoc crate\n set Test::offLoc $KQObject::loc\n KQCamera::follow none\nendif\n"),
        ("inner.cs", "move crate 0 100 0 0\n"),
        ("outer.cs", "set Test::phase before\ninner.cs\nset Test::phase after\nhandsOff 1\n"),
    ]);
    plate(&mut a, "plate", [0.; 3], 20., "plate.cs");
    let mut b = prop("crate", [-100., 0., 0.], false);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]), ["plate On crate"]);
    assert!(a.run_script("outer.cs", &[]));
    assert_eq!(a.scripts.vars.get("Test::seen"), Some("after"));
    assert!(a.scripts.vars.get("Test::offLoc").unwrap().starts_with("100 0 0"));
    assert!(a.world.traps[0].inside.is_empty());
    assert!(matches!(&a.world.requests[..], [
        Request::Move { who, .. }, Request::HandsOff { on: true, .. },
        Request::CameraFollow { connor: false, .. },
    ] if who == "crate"));
}

#[test]
fn script_departure_ignores_removed_mask_and_never_enters_jump_or_destination_traps() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == On\n set Test::on $1\nendif\nif test $2 == Off\n set Test::off $1 $3\nendif\n"),
        ("jump.cs", "KQObject::collision source -1\nmove crate 0 1000 0 0\n"),
    ]);
    plate(&mut a, "source", [0.; 3], 20., "plate.cs");
    plate(&mut a, "crossed", [500., 0., 0.], 20., "plate.cs");
    plate(&mut a, "destination", [1000., 0., 0.], 20., "plate.cs");
    let mut b = prop("crate", [-100., 0., 0.], true);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]), ["source On crate"]);
    a.scripts.vars.set("Test::on", "none".into());
    assert!(a.run_script("jump.cs", &[]));
    assert_eq!(a.scripts.vars.get("Test::off"), Some("source crate"));
    assert_eq!(a.scripts.vars.get("Test::on"), Some("none"));
    assert_eq!(a.world.traps[0].settings.mask, 0);
    assert!(a.world.traps.iter().all(|t| t.inside.is_empty()));
    assert_eq!(a.world.prop_placements["crate"].unwrap().position, [1000., 0., 0.]);
}

#[test]
fn script_departure_names_the_visitor_in_cue_302_and_retains_other_occupants() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Cue\n set Test::cue $3 $4\nendif\nif test $2 == Off\n set Test::off 1\nendif\n"),
        ("leave.cs", "move leaving 0 100 0 0\n"),
    ]);
    plate(&mut a, "plate", [0.; 3], 20., "plate.cs");
    let mut other = prop("other", [-100., 0., 0.], false);
    let mut leaving = prop("leaving", [-100., 0., 0.], true);
    register(&mut a, &other);
    register(&mut a, &leaving);
    assert_eq!(push(&mut a, &mut other, [100., 0., 0.]), ["plate On other"]);
    assert_eq!(push(&mut a, &mut leaving, [100., 0., 0.]), ["plate Cue 301 leaving"]);
    assert!(a.run_script("leave.cs", &[]));
    assert_eq!(a.scripts.vars.get("Test::cue"), Some("302 leaving"));
    assert_eq!(a.scripts.vars.get("Test::off"), None);
    assert_eq!(a.world.traps[0].inside, ["other"]);
}

#[test]
fn departure_callback_moves_another_occupant_before_return_with_ordered_requests() {
    let mut a = world(&[
        ("first.cs", "if test $2 == Off\n set Test::firstSeen $Test::phase\n move second 0 2000 0 0\n handsOff 0\nendif\n"),
        ("second.cs", "if test $2 == Off\n set Test::secondSeen $Test::phase\n KQCamera::follow none\nendif\n"),
        ("leave.cs", "set Test::phase before\nmove first 0 500 0 0\nhandsOff 1\nset Test::phase after\n"),
    ]);
    plate(&mut a, "first_plate", [0.; 3], 20., "first.cs");
    plate(&mut a, "second_plate", [1000., 0., 0.], 20., "second.cs");
    let mut first = prop("first", [-100., 0., 0.], false);
    let mut second = prop("second", [900., 0., 0.], true);
    register(&mut a, &first);
    register(&mut a, &second);
    assert_eq!(push(&mut a, &mut first, [100., 0., 0.]), ["first_plate On first"]);
    assert_eq!(push(&mut a, &mut second, [100., 0., 0.]), ["second_plate On second"]);
    assert!(a.run_script("leave.cs", &[]));
    assert!(a.world.traps.iter().all(|t| t.inside.is_empty()));
    assert_eq!(a.scripts.vars.get("Test::firstSeen"), Some("after"));
    assert_eq!(a.scripts.vars.get("Test::secondSeen"), Some("after"));
    assert_eq!(a.world.place_of("second").unwrap().position, [2000., 0., 0.]);
    assert!(matches!(&a.world.requests[..], [
        Request::Move { who: first, .. }, Request::HandsOff { on: true, .. },
        Request::Move { who: second, .. }, Request::HandsOff { on: false, .. },
        Request::CameraFollow { connor: false, .. },
    ] if first == "first" && second == "second"));
}

#[test]
fn script_stay_uses_the_final_oriented_box_including_rotation_only_departure() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Off\n set Test::off 1\nendif\n"),
        ("stay.cs", "move crate 0 0.1 0 0 0 0 0.785398163\n"),
        ("turn.cs", "move crate 0 0.1 0 0 0 0 0\n"),
    ]);
    plate(&mut a, "tip", [6., 6., 0.], 0.2, "plate.cs");
    let mut b = prop("crate", [-1., 0., 0.], false);
    b.min = [-10., -1., -1.];
    b.max = [10., 1., 1.];
    b.placement.rotation = kq8_data::objects::rotation([0., 0., std::f32::consts::FRAC_PI_4]);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [1., 0., 0.]), ["tip On crate"]);
    assert!(a.run_script("stay.cs", &[]));
    assert_eq!(a.world.traps[0].inside, ["crate"]);
    assert_eq!(a.scripts.vars.get("Test::off"), None);
    assert!(a.run_script("turn.cs", &[]));
    assert!(a.world.traps[0].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::off"), Some("1"));
}

#[test]
fn script_stay_uses_the_ball_stack_and_does_not_sweep_back_to_the_old_spot() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Off\n set Test::off 1\nendif\n"),
        ("stay.cs", "move crate 0 0.1 0 0\n"),
        ("leave.cs", "move crate 0 -2 0 0\n"),
    ]);
    plate(&mut a, "side", [9., 0., 10.], 0.1, "plate.cs");
    let mut b = prop("crate", [-100., 0., 0.], true);
    b.min = [-1., -10., -20.];
    b.max = [1., 10., 20.];
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]), ["side On crate"]);
    assert!(a.run_script("stay.cs", &[]));
    assert_eq!(a.world.traps[0].inside, ["crate"]);
    assert_eq!(a.scripts.vars.get("Test::off"), None);
    assert!(a.run_script("leave.cs", &[]));
    assert!(a.world.traps[0].inside.is_empty());
    assert_eq!(a.scripts.vars.get("Test::off"), Some("1"));
}

#[test]
fn script_deleted_prop_does_not_fabricate_a_departure_from_a_stale_notice() {
    let mut a = world(&[
        ("plate.cs", "if test $2 == Off\n set Test::off 1\nendif\n"),
        ("delete.cs", "move crate 0 100 0 0\ndeleteObject crate\n"),
    ]);
    plate(&mut a, "plate", [0.; 3], 20., "plate.cs");
    let mut b = prop("crate", [-100., 0., 0.], false);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]), ["plate On crate"]);
    assert!(a.run_script("delete.cs", &[]));
    assert!(a.world.is_deleted("crate"));
    assert_eq!(a.scripts.vars.get("Test::off"), None,
        "Q203 leaves deletion occupancy unresolved; do not invent a moved callback");
    assert_eq!(a.world.prop_placements["crate"], None);
}

#[test]
fn first_departure_callback_repositions_the_same_prop_before_the_next_trap_checks_it() {
    let mut a = world(&[
        ("first.cs", "if test $2 == Off\n set Test::firstOff 1\n move crate 0 0 0 0\nendif\n"),
        ("other.cs", "if test $2 == Off\n set Test::otherOff 1\nendif\n"),
        ("leave.cs", "move crate 0 100 0 0\n"),
    ]);
    plate(&mut a, "first_plate", [0.; 3], 20., "first.cs");
    plate(&mut a, "other_plate", [0.; 3], 20., "other.cs");
    let mut b = prop("crate", [-100., 0., 0.], false);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]),
        ["first_plate On crate", "other_plate On crate"]);
    assert!(a.run_script("leave.cs", &[]));
    assert_eq!(a.scripts.vars.get("Test::firstOff"), Some("1"));
    assert_eq!(a.scripts.vars.get("Test::otherOff"), None,
        "the second trap must see the position changed by the first callback");
    assert!(a.world.traps[0].inside.is_empty(), "script placement back cannot re-enter");
    assert_eq!(a.world.traps[1].inside, ["crate"]);
    assert_eq!(a.world.place_of("crate").unwrap().position, [0.; 3]);
    assert_eq!(a.world.prop_placements["crate"].unwrap().position, [0.; 3]);
}

#[test]
fn first_departure_callback_deleting_the_prop_suppresses_later_stale_move_notifications() {
    let mut a = world(&[
        ("first.cs", "if test $2 == Off\n set Test::firstOff 1\n deleteObject crate\nendif\n"),
        ("other.cs", "if test $2 == Off\n set Test::otherOff 1\nendif\n"),
        ("leave.cs", "move crate 0 100 0 0\n"),
    ]);
    plate(&mut a, "first_plate", [0.; 3], 20., "first.cs");
    plate(&mut a, "other_plate", [0.; 3], 20., "other.cs");
    let mut b = prop("crate", [-100., 0., 0.], true);
    register(&mut a, &b);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]),
        ["first_plate On crate", "other_plate On crate"]);
    assert!(a.run_script("leave.cs", &[]));
    assert_eq!(a.scripts.vars.get("Test::firstOff"), Some("1"));
    assert!(a.world.is_deleted("crate"));
    assert_eq!(a.scripts.vars.get("Test::otherOff"), None,
        "Q203 is unresolved; a deleted visitor must not get a stale movement departure");
    assert_eq!(a.world.prop_placements["crate"], None);
}

#[test]
fn restored_prop_occupancy_uses_loaded_shape_for_departure_without_another_push() {
    use crate::trap::SavedTraps;
    use std::collections::BTreeMap;
    let scripts = [
        ("plate.cs", "if test $2 == On\n set Test::entered 1\nendif\nif test $2 == Off\n set Test::left $3\n getLoc crate\n set Test::offLoc $KQObject::loc\nendif\n"),
        ("disable.cs", "KQObject::collision plate -1\n"),
        ("leave.cs", "move crate 0 100 0 0\n"),
    ];
    let mut original = world(&scripts);
    plate(&mut original, "plate", [0.; 3], 20., "plate.cs");
    let mut loaded_shape = prop("crate", [-100., 0., 0.], true);
    register(&mut original, &loaded_shape);
    assert_eq!(push(&mut original, &mut loaded_shape, [100., 0., 0.]), ["plate On crate"]);
    assert!(original.run_script("disable.cs", &[]));
    let bytes = serde_json::to_vec(&(SavedTraps::capture(&original.world.traps),
        &original.world.prop_placements)).unwrap();
    let (saved_traps, saved_places): (SavedTraps, BTreeMap<String, Option<Placement>>) =
        serde_json::from_slice(&bytes).unwrap();

    // A fresh world rebuilds shape metadata from loaded objects. The save
    // carries occupancy and placement, not the transient collider cache.
    let mut restored = world(&scripts);
    plate(&mut restored, "plate", [0.; 3], 20., "plate.cs");
    loaded_shape.placement = saved_places["crate"].unwrap();
    register(&mut restored, &loaded_shape);
    restored.world.prop_placements = saved_places;
    saved_traps.restore(&mut restored.world.traps, &restored.world.known).unwrap();
    assert_eq!(restored.world.traps[0].inside, ["crate"]);
    assert_eq!(restored.world.traps[0].settings.mask, 0);
    assert_eq!(restored.scripts.vars.get("Test::entered"), None);
    assert!(restored.world.requests.is_empty());

    // No physical movement is performed in the restored instance.
    assert!(restored.run_script("leave.cs", &[]));
    assert!(restored.world.traps[0].inside.is_empty());
    assert_eq!(restored.scripts.vars.get("Test::left"), Some("crate"));
    assert!(restored.scripts.vars.get("Test::offLoc").unwrap().starts_with("100 0 0"));
    assert_eq!(restored.scripts.vars.get("Test::entered"), None);
    assert_eq!(restored.world.prop_placements["crate"].unwrap().position, [100., 0., 0.]);
}
