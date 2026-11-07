//! Synthetic checks of traps.md rules 4-9 and thrown-items.md rules 7/11.
//! No original assets or campaign state are embedded here.
use crate::active::Active;
use crate::collide::{BoxCollider, ColliderKind, Placement};
use crate::connor_mover::tests::Flat;
use crate::item::{Flight, Items, Place};
use kq8_data::world_script::Vars;
use crate::trap::{SavedTraps, Trap, TrapSettings};

const EVENTS: &str = "if test $2 == On\n set Test::on $3\nendif\n\
if test $2 == Cue\n set Test::cue $3 $4\nendif\n\
if test $2 == Off\n set Test::off $3\nendif\n";

fn world(script: &str) -> Active {
    let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".into()],
        [("plate.cs".into(), script.as_bytes().to_vec())]);
    a.world.items = Items::new(crate::item::tests::types(),
        [("crystl.kqs".into(), [-5., -5., 0.], [5., 5., 10.])]);
    a
}

fn plate(a: &mut Active, name: &str, at: [f32; 3], half: f32, mask: u32) {
    a.add_traps([Trap::new(TrapSettings { name: name.into(), position: at,
        min: [-half; 3], max: [half; 3], script: Some("plate.cs".into()),
        lookup_call: false, mask, trap_type: 0, inner_percent: None, spirit: None })]);
}

fn launch(a: &mut Active, kind: &str, from: [f32; 3], velocity: [f32; 3]) -> String {
    let name = a.world.items.make(kind, 1, Place::World { position: from, angles: [0.; 3] })
        .unwrap().name.clone();
    a.world.known.insert(name.to_ascii_lowercase());
    let shape = a.world.items.get(&name).and_then(|o| a.world.items.click_box(o));
    let mut f = Flight::launch(&name, from, [0.; 3], shape).with_tumble(0.5);
    // Synthetic speeds expose the difference between a swept L-shaped
    // path and its diagonal in one update. No native throw is modified.
    f.velocity = velocity;
    a.world.flights.push(f);
    name
}

fn inside(a: &Active, trap: &str) -> Vec<String> {
    a.world.traps.iter().find(|t| t.settings.name == trap).unwrap().inside.clone()
}

#[test]
fn tossed_item_uses_vertical_and_across_passes_not_the_diagonal() {
    let mut a = world(EVENTS);
    plate(&mut a, "up", [0., 0., 500.], 20., 512);
    plate(&mut a, "across", [500., 0., 982.], 20., 512);
    plate(&mut a, "diagonal", [500., 0., 491.], 20., 512);
    let item = launch(&mut a, "Crystal", [0.; 3], [15000., 0., 15000.]);
    a.flights_step(&Flat(-10000.), &[], &[]);
    assert_eq!(inside(&a, "up"), [item.clone()]);
    assert_eq!(inside(&a, "across"), [item.clone()]);
    assert!(inside(&a, "diagonal").is_empty());
    let p = a.world.items.get(&item).unwrap().placement().unwrap().position;
    assert!((p[0] - 1000.).abs() < 0.01 && (p[2] - 982.0333).abs() < 0.01,
        "traps must not stop the ordinary flight: {p:?}");
    a.flights_step(&Flat(-10000.), &[], &[]);
    assert!(inside(&a, "up").is_empty() && inside(&a, "across").is_empty());
    assert!(a.world.trap_log.iter().any(|s| s.contains(&format!("trap up: Off {item}"))));
}

#[test]
fn tossed_item_stopped_by_a_wall_does_not_enter_a_trap_beyond_it() {
    let mut a = world(EVENTS);
    plate(&mut a, "near", [200., 0., 982.], 20., 512);
    plate(&mut a, "behind", [500., 0., 982.], 20., 512);
    let item = launch(&mut a, "Crystal", [0.; 3], [15000., 0., 15000.]);
    let wall = BoxCollider { name: "wall".into(), min: [300., -500., -100.],
        max: [320., 500., 3000.], placement: Placement::at([0.; 3]),
        kind: ColliderKind { collision_type: 1, ..Default::default() } };
    a.flights_step(&Flat(-10000.), &[], &[wall]);
    assert_eq!(inside(&a, "near"), [item.clone()]);
    assert!(inside(&a, "behind").is_empty());
    assert!(a.world.items.get(&item).unwrap().placement().unwrap().position[0] < 300.);
}

#[test]
fn tossed_item_needs_a_shape_and_matching_mask_while_placement_does_not_enter() {
    let mut a = world(EVENTS);
    plate(&mut a, "items", [0., 0., 200.], 300., 512);
    plate(&mut a, "connor", [0., 0., 200.], 300., 4);
    a.world.items.make("Crystal", 1, Place::World { position: [0.; 3], angles: [0.; 3] }).unwrap();
    let shaped = launch(&mut a, "Crystal", [0.; 3], [0., 0., 3000.]);
    launch(&mut a, "SilverCoins", [0.; 3], [0., 0., 3000.]);
    a.flights_step(&Flat(-10000.), &[], &[]);
    assert_eq!(inside(&a, "items"), [shaped]);
    assert!(inside(&a, "connor").is_empty());
}

#[test]
fn tossed_item_follows_contact_then_wall_slide_for_trigger_entry() {
    let mut a = world(EVENTS);
    plate(&mut a, "slide", [295., 700., 982.], 20., 512);
    plate(&mut a, "behind", [500., 500., 982.], 20., 512);
    let item = launch(&mut a, "Crystal", [0.; 3], [15000., 15000., 15000.]);
    let wall = BoxCollider { name: "wall".into(), min: [300., -500., -100.],
        max: [320., 2000., 3000.], placement: Placement::at([0.; 3]),
        kind: ColliderKind { collision_type: 1, ..Default::default() } };
    a.flights_step(&Flat(-10000.), &[], &[wall]);
    assert_eq!(inside(&a, "slide"), [item.clone()]);
    assert!(inside(&a, "behind").is_empty());
    let p = a.world.items.get(&item).unwrap().placement().unwrap().position;
    // The existing 25-unit pull-back follows the diagonal approach:
    // x = wall 300 - radius 5 - 25/sqrt(2), then the slide continues in y.
    assert!((p[0] - 277.32233).abs() < 0.01 && (p[1] - 1000.).abs() < 0.01, "{p:?}");
}

#[test]
fn tossed_item_on_callback_changes_the_mask_before_the_next_item_moves() {
    let mut a = world("if test $2 == On\n KQObject::collision $1 -512\nendif\n");
    plate(&mut a, "items", [0., 0., 200.], 300., 512);
    let first = launch(&mut a, "Crystal", [0.; 3], [0., 0., 3000.]);
    launch(&mut a, "Crystal", [0.; 3], [0., 0., 3000.]);
    a.flights_step(&Flat(-10000.), &[], &[]);
    assert_eq!(inside(&a, "items"), [first]);
    assert_eq!(a.world.traps[0].settings.mask, 0);
    assert!(a.world.flights.iter().all(|f| f.updates == 1));
}

#[test]
fn tossed_item_multi_occupant_entry_and_exit_use_the_original_cues() {
    let mut a = world(EVENTS);
    plate(&mut a, "items", [0., 0., 200.], 300., 512);
    let first = launch(&mut a, "Crystal", [0.; 3], [15000., 0., 0.]);
    let second = launch(&mut a, "Crystal", [0.; 3], [0., 0., 3000.]);
    a.flights_step(&Flat(-10000.), &[], &[]);
    assert_eq!(a.scripts.vars.get("Test::on"), Some(first.as_str()));
    assert_eq!(a.scripts.vars.get("Test::cue"), Some(format!("301 {second}").as_str()));
    a.flights_step(&Flat(-10000.), &[], &[]);
    assert_eq!(inside(&a, "items"), [second.clone()]);
    assert_eq!(a.scripts.vars.get("Test::cue"), Some(format!("302 {first}").as_str()));
    for _ in 0..4 { a.flights_step(&Flat(-10000.), &[], &[]); }
    assert!(inside(&a, "items").is_empty());
    assert_eq!(a.scripts.vars.get("Test::off"), Some(second.as_str()));
}

#[test]
fn settled_tossed_item_keeps_pressure_and_saved_membership_without_reentry() {
    let mut a = world(EVENTS);
    plate(&mut a, "items", [0., 0., 100.], 200., 512);
    let item = launch(&mut a, "Crystal", [0., 0., 150.], [0., 0., -1000.]);
    for _ in 0..12 {
        a.flights_step(&Flat(0.), &[], &[]);
        if a.world.flights.is_empty() { break; }
    }
    assert!(a.world.flights.is_empty(), "the item must actually settle");
    assert_eq!(inside(&a, "items"), [item.clone()]);
    assert_eq!(a.world.trap_log.iter().filter(|s| s.contains("trap items: On ")).count(), 1);
    let saved = serde_json::to_vec(&SavedTraps::capture(a.world.traps.iter())).unwrap();
    let restored: SavedTraps = serde_json::from_slice(&saved).unwrap();
    let mut fresh = world(EVENTS);
    plate(&mut fresh, "items", [0., 0., 100.], 200., 512);
    fresh.world.items.objects = a.world.items.objects.clone();
    restored.restore(&mut fresh.world.traps, &[item.to_ascii_lowercase()].into_iter().collect()).unwrap();
    fresh.flights_step(&Flat(0.), &[], &[]);
    assert_eq!(inside(&fresh, "items"), [item]);
    assert_eq!(fresh.scripts.vars.get("Test::on"), None, "restore does not emit On again");
}
