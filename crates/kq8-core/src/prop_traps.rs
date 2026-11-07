//! Synthetic physical-push checks from traps.md and pushing.md.
use crate::active::{Active, Clickable};
use crate::collide::{BoxCollider, ColliderKind, Placement};
use crate::connor_mover::{tests::Flat, MoverWorld, PropMove, Scene};
use crate::trap::{Trap, TrapSettings, Visitor, VisitorShape};

fn world(script: &str) -> Active {
    Active::new(Default::default(), vec![], vec![], ["Connor".into()],
        [("plate.cs".into(), script.as_bytes().to_vec())])
}

fn plate(a: &mut Active, name: &str, position: [f32; 3], half: f32, mask: u32) {
    a.add_traps([Trap::new(TrapSettings { name: name.into(), position,
        min: [-half; 3], max: [half; 3], script: Some("plate.cs".into()),
        lookup_call: false, mask, trap_type: 0, inner_percent: None, spirit: None })]);
}

fn prop(a: &mut Active, name: &str, position: [f32; 3], balls: bool) -> BoxCollider {
    let b = BoxCollider { name: name.into(), min: [-5.; 3], max: [5.; 3],
        placement: Placement::at(position), kind: ColliderKind { collision_type: 0x121,
            collision_mask: 11, ball_stack: balls, keep_on_ground: false } };
    a.world.known.insert(name.to_ascii_lowercase());
    a.world.prop_placements.insert(name.to_ascii_lowercase(), None);
    a.world.clickables.push(Clickable::plain(name, "KQObject", b.min, b.max, b.placement, b.kind.collision_type));
    b
}

fn push(a: &mut Active, b: &mut BoxCollider, delta: [f32; 3]) -> Vec<String> {
    let m = PropMove { name: b.name.clone(), delta, path: vec![([0.; 3], delta)] };
    let calls = a.prop_moved(b, &m);
    b.placement.position = std::array::from_fn(|i| b.placement.position[i] + delta[i]);
    calls
}

#[test]
fn pushed_rotated_thin_box_uses_its_shape_not_a_ball_or_world_bounding_box() {
    let mut a = world("");
    plate(&mut a, "tip", [6., 6., 0.], 0.2, 1);
    plate(&mut a, "empty_corner", [7., -7., 0.], 0.2, 1);
    let mut b = prop(&mut a, "crate", [0., 0., -0.1], false);
    b.min = [-10., -1., -1.]; b.max = [10., 1., 1.];
    b.placement.rotation = kq8_data::objects::rotation([0., 0., std::f32::consts::FRAC_PI_4]);
    assert_eq!(push(&mut a, &mut b, [0., 0., 0.1]), ["tip On crate"]);
    assert!(a.world.traps[1].inside.is_empty());
}

#[test]
fn pushed_box_crosses_a_thin_trigger_then_leaves_on_its_next_move() {
    let mut a = world("");
    plate(&mut a, "plate", [0.; 3], 0.1, 1);
    let mut b = prop(&mut a, "crate", [-1000., 0., 0.], false);
    assert_eq!(push(&mut a, &mut b, [2000., 0., 0.]), ["plate On crate"]);
    assert_eq!(push(&mut a, &mut b, [1., 0., 0.]), ["plate Off crate"]);
}

#[test]
fn rotated_box_collision_preserves_the_existing_logical_centre_rule() {
    let mut a = world("");
    plate(&mut a, "plate", [0., 100., 0.], 1., 1);
    let rotation = kq8_data::objects::rotation([0., 0., std::f32::consts::FRAC_PI_2]);
    let v = Visitor { name: "offcentre", collision_type: 1, from: [0.; 3], position: [0.; 3],
        min: [90., -1., -1.], max: [110., 1., 1.], shape: VisitorShape::Box(rotation) };
    let entered = a.world.traps[0].check(&v);
    // Collision follows the rotated box. Do not silently change the older
    // unturned logical-centre approximation while its source rule is Q204.
    assert!(entered.touching && !entered.centre_inside);
    assert_eq!(entered.call.unwrap(), ["On", "offcentre"]);
}

#[test]
fn pushed_ball_stack_dispatches_its_real_wall_slide_not_the_chord() {
    let mut a = world("");
    plate(&mut a, "slide", [277.32233, 700., 100.], 10., 1);
    plate(&mut a, "chord", [138.66, 500., 100.], 10., 1);
    let b = prop(&mut a, "crate", [0., 0., 100.], true);
    let wall = BoxCollider { name: "wall".into(), min: [300., -500., -100.], max: [320., 2000., 3000.],
        placement: Placement::at([0.; 3]), kind: ColliderKind { collision_type: 1, ..Default::default() } };
    let objects = [b.clone(), wall];
    let ground = Flat(0.);
    let scene = Scene::new(&ground, &[], &objects);
    let (delta, _) = scene.push_object("crate", [1000., 1000., 0.]);
    assert!((delta[0] - 277.32233).abs() < 0.01 && (delta[1] - 1000.).abs() < 0.01);
    let moved = scene.moved.take();
    assert_eq!(moved.len(), 1);
    assert_eq!(a.prop_moved(&b, &moved[0]), ["slide On crate"]);
    assert!(a.world.traps[1].inside.is_empty());
}

#[test]
fn rejected_uphill_push_and_nonmatching_type_do_not_enter() {
    let mut a = world("");
    plate(&mut a, "plate", [0.; 3], 50., 4);
    let mut b = prop(&mut a, "crate", [0.; 3], false);
    assert!(push(&mut a, &mut b, [1., 0., 0.]).is_empty(), "mask accepts only Connor");
    a.world.traps[0].settings.mask = 1;
    b.kind.keep_on_ground = true;
    let objects = [b];
    let ground = Flat(100.);
    let scene = Scene::new(&ground, &[], &objects);
    assert_eq!(scene.push_object("crate", [10., 0., 0.]).0, [0.; 3]);
    assert!(scene.moved.take().is_empty());
    assert!(a.world.traps[0].inside.is_empty());
}

#[test]
fn pushed_prop_callback_resolves_trap_and_current_object_positions() {
    let mut a = world("if test $2 == On\n checkDistance $1 crate 20\n set Test::near $Console::Return\n getLoc crate\n set Test::loc $KQObject::loc\nendif\n");
    plate(&mut a, "plate", [100., 0., 0.], 10., 1);
    let mut b = prop(&mut a, "crate", [0.; 3], true);
    assert_eq!(push(&mut a, &mut b, [100., 0., 0.]), ["plate On crate"]);
    assert_eq!(a.scripts.vars.get("Test::near"), Some("1"));
    assert!(a.scripts.vars.get("Test::loc").unwrap().starts_with("100 "));
    assert_eq!(a.world.prop_placements["crate"].unwrap().position, [100., 0., 0.]);
    a.world.clickables.clear();
    assert_eq!(a.world.place_of("crate").unwrap().position, [100., 0., 0.], "a moved non-clickable prop is resolvable too");
}

#[test]
fn pushed_prop_leaving_an_occupied_trigger_sends_cue_302_with_its_name() {
    let mut a = world("if test $2 == Cue\n set Test::cue $3 $4\nendif\n");
    plate(&mut a, "plate", [0.; 3], 20., 1);
    let mut first = prop(&mut a, "other", [0.; 3], false);
    let mut leaving = prop(&mut a, "crate", [0.; 3], true);
    assert_eq!(push(&mut a, &mut first, [1., 0., 0.]), ["plate On other"]);
    assert_eq!(push(&mut a, &mut leaving, [1., 0., 0.]), ["plate Cue 301 crate"]);
    push(&mut a, &mut leaving, [100., 0., 0.]);
    assert_eq!(push(&mut a, &mut leaving, [1., 0., 0.]), ["plate Cue 302 crate"]);
    assert_eq!(a.scripts.vars.get("Test::cue"), Some("302 crate"));
    assert_eq!(a.world.traps[0].inside, ["other"]);
}

#[test]
fn pushed_prop_uses_a_type_changed_by_an_earlier_callback_in_the_same_update() {
    let mut a = world("if test $2 == On\n KQObject::collisionType next -1\nendif\n");
    plate(&mut a, "plate", [0.; 3], 20., 1);
    let mut first = prop(&mut a, "first", [-100., 0., 0.], false);
    let mut next = prop(&mut a, "next", [-100., 0., 0.], false);
    assert_eq!(push(&mut a, &mut first, [100., 0., 0.]), ["plate On first"]);
    assert_eq!(a.world.collision_type_of("next"), Some(0x120));
    assert!(push(&mut a, &mut next, [100., 0., 0.]).is_empty(),
        "the queued collider still has bit 1, but the live object no longer does");
    assert_eq!(a.world.traps[0].inside, ["first"]);
}
