//! Items lying in the world (M3 step 3e-3, docs/DESIGN.md; rules:
//! docs/behaviour/items.md).
//!
//! The item objects live in `kq8_core::active` (`DoorSim`), which picks
//! them up when clicked. Here each item lying in the world (and not
//! hidden) is drawn with its type's model, turned and placed as made
//! (rule 5), see-through by its type's translucency (as objects,
//! `objects.md` Q14). An item made while playing gets its entity when it
//! first appears; one picked up, deleted or hidden is hidden.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::door_view::DoorSim;
use crate::terrain_view::bevy_transform_from;
use crate::draw_distance::{DistanceTest, ShapeBalls};
use crate::object_light::{see_through, ObjectMaterial};

/// One drawn model: (mesh, material) per material group.
pub type Parts = Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>;

#[derive(Resource)]
pub struct ItemDraw {
    /// Each item type (`IDName`) that has a model here.
    parts: BTreeMap<String, Parts>,
    /// Each item object drawn so far, by name, and the place it was last
    /// drawn at.
    drawn: BTreeMap<String, (Entity, kq8_core::collide::Placement)>,
    /// Each item type's draw distance test (M1 6h).
    tests: BTreeMap<String, DistanceTest>,
    /// Smooth animation (S-5): each item's place between the last two
    /// updates (a thrown one flies smoothly).
    glides: BTreeMap<String, crate::smooth_view::Glide>,
}

/// `looks`: (`IDName`, index into `built`, opacity) per type.
pub fn spawn_items(commands: &mut Commands, looks: Vec<(String, usize, f32, String)>, built: &[Parts], materials: &mut Assets<ObjectMaterial>, balls: &ShapeBalls) {
    let mut parts = BTreeMap::new();
    let mut tests = BTreeMap::new();
    for (kind, slot, opacity, shape) in looks {
        let Some(model) = built.get(slot) else { continue };
        tests.insert(kind.clone(), balls.item(&kind, &shape));
        let p: Parts = model
            .iter()
            .map(|(mesh, material)| {
                let material = see_through(materials, material, opacity);
                (mesh.clone(), material)
            })
            .collect();
        parts.insert(kind, p);
    }
    commands.insert_resource(ItemDraw { parts, drawn: BTreeMap::new(), tests, glides: BTreeMap::new() });
}

/// Shows the items lying in the world, hides the others.
pub fn item_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<ItemDraw>>, mut visibility: Query<&mut Visibility>, mut transforms: Query<&mut Transform>, fixed: Res<Time<Fixed>>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    let (update, a) = (sim.active.world.updates, fixed.overstep_fraction());
    for o in &sim.active.world.items.objects {
        let at = o.placement().filter(|_| !o.hidden);
        // Smooth animation (S-5): between the last two updates; out of the
        // world (carried, hidden) its last place is forgotten.
        let at = match at {
            Some(p) => Some(draw.glides.entry(o.name.clone()).or_default().drawn(update, p, a)),
            None => {
                draw.glides.remove(&o.name);
                None
            }
        };
        match (draw.drawn.get(&o.name), at) {
            (Some(&(e, shown)), at) => {
                // M5 5c-8: a thrown item moves while it flies; fix 6: also
                // to where it landed (its flight ends in that update).
                if let Some(p) = at.filter(|p| p.position != shown.position || p.rotation != shown.rotation) {
                    if let Ok(mut t) = transforms.get_mut(e) {
                        *t = bevy_transform_from(p.rotation, p.position);
                    }
                    draw.drawn.insert(o.name.clone(), (e, p));
                }
                let want = if at.is_some() { Visibility::Inherited } else { Visibility::Hidden };
                if let Ok(mut v) = visibility.get_mut(e) {
                    if *v != want {
                        *v = want;
                    }
                }
            }
            (None, Some(p)) => {
                let Some(parts) = draw.parts.get(&o.kind) else { continue };
                let mut body = commands.spawn((bevy_transform_from(p.rotation, p.position), Visibility::Inherited));
                if let Some(test) = draw.tests.get(&o.kind) {
                    // Its own name: the collider switch and clicks go by it.
                    let mut test = test.another();
                    test.name = o.name.clone();
                    body.insert(test);
                }
                let body = body.id();
                for (mesh, material) in parts {
                    let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                    commands.entity(body).add_child(part);
                }
                draw.drawn.insert(o.name.clone(), (body, p));
            }
            (None, None) => {}
        }
    }
}

/// The corpses' models (`fighting.md` rule 32; M5 5c-7), and the corpses
/// drawn so far.
#[derive(Resource)]
pub struct CorpseDraw {
    /// Lower-case shape name -> mesh, shared material and original texture.
    parts: BTreeMap<String, Vec<(Handle<Mesh>, Handle<ObjectMaterial>, Option<String>)>>,
    /// How many of the world's corpses have their entity.
    drawn: usize,
    /// The bolts in the air drawn now (M5 5c-9), by name, and their places
    /// between the last two updates (smooth animation, S-5).
    bolts: BTreeMap<String, (Entity, crate::smooth_view::Glide)>,
}

/// `shapes`: lower-case shape name -> index into `built`.
pub fn spawn_corpses(commands: &mut Commands, shapes: BTreeMap<String, usize>, built: &[Parts], textures: &[Vec<Option<String>>]) {
    let parts = shapes.into_iter().filter_map(|(k, slot)| {
        let parts = built.get(slot)?;
        let names = textures.get(slot)?;
        Some((k, parts.iter().zip(names).map(|((mesh, material), name)| (mesh.clone(), material.clone(), name.clone())).collect()))
    }).collect();
    commands.insert_resource(CorpseDraw { parts, drawn: 0, bolts: BTreeMap::new() });
}

/// Apply the authored skin first, then the current scripted replacement.
fn corpse_skin_target<'a>(original: &'a str, skins: &'a [(String, String)]) -> Option<&'a str> {
    // Loaded texture labels include the volume path; monster skin rules
    // name only the bitmap file.
    let original = original.rsplit(['/', '\\']).next().unwrap_or(original);
    let mut shown = original;
    for (from, to) in skins {
        if from.eq_ignore_ascii_case(shown) { shown = to; }
    }
    (!shown.eq_ignore_ascii_case(original)).then_some(shown)
}

/// Each new corpse: its model where the character stood, turned the same
/// way (rule 32). **Ours**: no shadow (not a caster) and no draw-distance
/// test (always drawn when in view).
pub fn corpse_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<CorpseDraw>>, anim: Option<Res<crate::anim_view::AnimSim>>, mut materials: ResMut<Assets<ObjectMaterial>>, shared: Res<crate::Shared>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let corpses = &sim.active.world.corpses;
    while draw.drawn < corpses.len() {
        let c = &corpses[draw.drawn];
        draw.drawn += 1;
        let Some(parts) = draw.parts.get(&c.shape.to_ascii_lowercase()) else {
            shared.log.lock().unwrap().summary(format!("corpse of {}: shape {} not loaded; not drawn", c.name, c.shape));
            continue;
        };
        let body = commands.spawn((bevy_transform_from(c.rotation, c.position), Visibility::Inherited, crate::terrain_view::ObjectName(c.drawn_name()))).id();
        let mut changed = 0;
        for (mesh, material, original) in parts {
            let replacement = original.as_deref().and_then(|name| corpse_skin_target(name, &c.skins))
                .and_then(|name| anim.as_ref()?.swap_image(name));
            let material = match replacement.and_then(|image| materials.get(material).cloned().map(|mut m| { m.texture = Some(image); m })) {
                Some(copy) => { changed += 1; materials.add(copy) },
                None => material.clone(),
            };
            let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material))).id();
            commands.entity(body).add_child(part);
        }
        shared.log.lock().unwrap().summary(format!("corpse of {}: {} drawn at ({:.0}, {:.0}, {:.0}); skin pairs {:?}, {changed} material(s) changed", c.name, c.shape, c.position[0], c.position[1], c.position[2], c.skins));
    }
}

/// The bolts in the air (`fighting.md` rule 36; M5 5c-9): each drawn with
/// its shape where it is, turned to its heading and pitch with its
/// model's front (+y) along its flight (`ranged-weapons.md` rule 27; M5
/// 5f-1); removed when it is gone.
pub fn bolt_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<CorpseDraw>>, mut transforms: Query<&mut Transform>, fixed: Res<Time<Fixed>>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    let (update, a) = (sim.active.world.updates, fixed.overstep_fraction());
    let bolts = &sim.active.world.bolts;
    draw.bolts.retain(|name, (e, _)| {
        let alive = bolts.iter().any(|b| &b.name == name);
        if !alive {
            commands.entity(*e).despawn();
        }
        alive
    });
    for b in bolts {
        let place = kq8_core::collide::Placement { rotation: b.rotation(), position: b.position };
        match draw.bolts.get_mut(&b.name) {
            Some((e, glide)) => {
                let p = glide.drawn(update, place, a);
                if let Ok(mut tr) = transforms.get_mut(*e) {
                    *tr = bevy_transform_from(p.rotation, p.position);
                }
            }
            None => {
                let Some(parts) = b.shape.as_ref().and_then(|s| draw.parts.get(&s.to_ascii_lowercase())) else { continue };
                let body = commands.spawn((bevy_transform_from(place.rotation, place.position), Visibility::Inherited)).id();
                for (mesh, material, _) in parts {
                    let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                    commands.entity(body).add_child(part);
                }
                let mut glide = crate::smooth_view::Glide::default();
                glide.drawn(update, place, a);
                draw.bolts.insert(b.name.clone(), (body, glide));
            }
        }
    }
}

#[cfg(test)]
mod corpse_tests {
    use super::corpse_skin_target;

    #[test]
    fn corpse_keeps_authored_skin_then_current_script_change() {
        let base = ("test2.bmp".into(), "test3.bmp".into());
        let script = ("test3.bmp".into(), "test4.bmp".into());
        let source = "vol:Daventry/resource/TEST2.BMP";
        assert_eq!(corpse_skin_target(source, &[]), None);
        assert_eq!(corpse_skin_target(source, &[base.clone()]), Some("test3.bmp"));
        assert_eq!(corpse_skin_target(source, &[base, script]), Some("test4.bmp"));
        assert_eq!(corpse_skin_target("other.bmp", &[("test2.bmp".into(), "test3.bmp".into())]), None);
    }
}
