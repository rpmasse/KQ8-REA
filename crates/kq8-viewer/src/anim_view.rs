//! Animated objects playing their movements in the ground view (M3 steps
//! 3c and 3i-4, docs/DESIGN.md; rules: docs/behaviour/objects.md
//! "Animated objects at load", docs/behaviour/anim-scripts.md).
//!
//! - Since 3i-4 the movements are played by `kq8_core` (`Active`'s
//!   `anims`, the same player as doors and chests), so scripts reach them
//!   (`KQAnimPlayEvent`, `pauseAnim`, `deleteObject`, `EndLoop`). This
//!   file only draws what core says. Unused objects (shown only with H)
//!   are not in the world's scripts: they keep a player of their own here.
//! - Each object is a body entity at its placement with one child per
//!   frame of every movement; only the shown frame is visible (as doors).
//!   If the shown movement has whole-object steps (frame transforms), the
//!   body is moved by the shown frame's chained steps, as doors are
//!   turned. The original adds the steps up and the data's cue puts the
//!   object back at each wrap; for Gnome's rolling rocks both give the
//!   same picture, so this stays (objects.md Q51 rules 1–7, checked by the
//!   user). Such an object's collider is moved to the same pose every
//!   update (Q51 rule 5, M3 step 3r-4).
//! - Hidden and unused objects keep their H-key visibility; see-through
//!   objects keep their opacity.
//!
//! Log: `kq8-artifacts/logs/anim.log`.

use bevy::prelude::*;
use kq8_core::active::AnimObject;
use kq8_core::anim::pose_at_frame;
use kq8_core::collide::Placement;
use kq8_core::door::DoorMovement;
use kq8_core::emitter::Rng;
use kq8_core::named_anim::NamedAnimPlayer;
use kq8_data::shape::Transform as GameTransform;

use crate::door_view::DoorSim;
use crate::load::{AnimatedAssets, Animation};
use crate::log::Log;
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::terrain_view::{bevy_transform_from, ObjectColliders, ShownWithH};

/// Seed for the `flags` bit 8 random starts and speeds.
const SEED: u64 = 0x0A11_3C00_5EED_0001;

/// Objects whose phase and frame are logged every update (for a while).
const TRACKED: [&str; 4] = ["wellwatr", "windprop", "millwheel", "Davtorch.01"];
const TRACK_UPDATES: u64 = 120;

/// Who plays an object's movements.
enum Source {
    /// `Active`'s `anims[i]` (the world's scripts reach it).
    Core(usize),
    /// Its own player (unused objects, shown only with H).
    Own(Box<NamedAnimPlayer>),
}

/// One movement as drawn: its number of frames and whole-object steps
/// (empty when no frame has one).
struct DrawnMovement {
    frames: usize,
    steps: Vec<GameTransform>,
}

struct Playing {
    name: String,
    source: Source,
    /// `script=` (for a copy brought in by `loadKQ`, M3 3k-4).
    script: Option<String>,
    /// The object's placement (rotation, position).
    rotation: [[f32; 3]; 3],
    position: [f32; 3],
    /// Smooth animation (S-5): its place between the last two updates.
    glide: crate::smooth_view::Glide,
    movements: Vec<DrawnMovement>,
    body: Entity,
    /// (movement, frame) shown.
    shown: Option<(usize, usize)>,
    tracked: bool,
    /// Its entry in `ObjectColliders` (only for objects with steps):
    /// `None` = not looked up yet, `Some(None)` = it has none.
    collider: Option<Option<usize>>,
    /// A character prepared for a script's `loadKQ` (M4 4d-3): its states
    /// and its draw-distance test, handed over when it comes in.
    character: Option<Box<kq8_core::character::Character>>,
    test: Option<crate::draw_distance::DistanceTest>,
    /// Its own copies of the materials showing its face texture, the
    /// image of each mouth shape, and the shape drawn (M4 4d-5, rule 25).
    face: Vec<Handle<ObjectMaterial>>,
    lip_images: Vec<Option<Handle<Image>>>,
    mouth_shown: u16,
    /// Its drawn parts (for `makeTranslucent`, rule 22), the opacity drawn
    /// now, and its own material copies with their opacity at 1 (the
    /// alpha of the colour before any see-through).
    parts: Vec<Entity>,
    opacity_shown: f32,
    own: std::collections::HashMap<AssetId<ObjectMaterial>, (Handle<ObjectMaterial>, f32)>,
    /// The character's own mask and original opacity, independent of the
    /// menu's live world mask (rule 36), and its shadow mesh once made.
    shadow_mask: u32,
    shadow_translucency: f32,
    shadow: Option<crate::shadow_view::ShadowSlot>,
    /// A character loaded underground: whether its body is shown now
    /// (`character-movement.md` rules 28, 30; M5 5a-8).
    birth_shown: Option<bool>,
    /// Its drawing (Animation memory step 4); `None` if its animations
    /// couldn't be packed.
    new_draw: Option<crate::anim_gpu::Slots>,
    /// `changeBmp` (batch 2 D-6, `characters.md` rule 4a): its materials'
    /// original texture names, the pair drawn now, and the pictures the
    /// pair replaced (put back when it changes).
    texture_names: Vec<((usize, i32), String)>,
    bitmaps: BitmapOverrides,
    /// An unused object's player as built at load: what a script's
    /// `loadKQ` brings in, so its movement starts as at the start
    /// (`objects.md` rule 6; G-4). Its own player keeps playing for H.
    at_load: Option<Box<NamedAnimPlayer>>,
}

/// The built character and movement player for one original .kq type file.
pub struct PreparedMonster {
    pub file: String,
    pub keys: kq8_data::characters::CharacterKeys,
    pub combat_scaled: bool,
    pub actor: AnimObject,
}

/// What a new copy of an unused object needs (M5 5d-4): each movement's
/// frames as (mesh, material) lists, its whole-object steps, its script.
struct Recipe {
    /// Per movement: its frames and whole-object steps.
    movements: Vec<(usize, Vec<GameTransform>)>,
    script: Option<String>,
    /// What `Slots::spawn` needs for a copy drawn the new way (step 4e):
    /// per movement its animation, model and ball; its materials.
    slots: (Vec<SlotMovement>, SlotMaterials),
    /// Its materials' original texture names (batch 2 D-6).
    textures: Vec<((usize, i32), String)>,
    /// Character-only presentation of a new copy of this type.
    character: Option<CharacterRecipe>,
}

struct CharacterRecipe {
    distance: crate::draw_distance::DistanceTest,
    face_keys: Vec<(usize, i32)>,
    lip_images: Vec<Option<Handle<Image>>>,
    casts_shadow: bool,
    shadow_mask: u32,
    shadow_translucency: f32,
}

/// A copy's materials for `Slots::spawn`: (animation, material index).
type SlotMaterials = Vec<((usize, i32), Handle<ObjectMaterial>)>;

/// A movement for `Slots::spawn`: its animation's index, model and ball.
type SlotMovement = (usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall);

#[derive(Resource)]
pub struct AnimSim {
    playing: Vec<Playing>,
    /// The unused objects' recipes, by lower-case name (M5 5d-4).
    recipes: std::collections::BTreeMap<String, Recipe>,
    /// The pictures `changeBmp` swaps in, by lower-case name (batch 2 D-6).
    swap_images: std::collections::BTreeMap<String, Handle<Image>>,
    updates: u64,
    log: Log,
    /// `characters.log` (M4 4d), if it could be made.
    characters: Option<Log>,
    /// `fight.log` (M5 5c), if it could be made.
    fight: Option<Log>,
}

/// Class-controlled replacements overlay normal object overrides by original
/// texture name. Reveal changes its named targets, not unrelated materials.
fn bitmap_pairs(world: &kq8_core::active::WorldState, name: &str) -> Vec<(String, String)> {
    let mut pairs: Vec<_> = world.bmp_swaps.get(&name.to_ascii_lowercase()).cloned().into_iter().collect();
    pairs.extend(world.tile_bitmap_pairs(name));
    pairs
}

fn replacement(original: &str, pairs: &[(String, String)], images: &std::collections::BTreeMap<String, Handle<Image>>) -> Option<Handle<Image>> {
    let (_, name) = pairs.iter().rev().find(|(old, _)| old.eq_ignore_ascii_case(original))?;
    if name.eq_ignore_ascii_case(original) { return None; }
    images.get(&name.to_ascii_lowercase()).cloned()
}

/// Each object's saved originals are separate from the image cache. Aliased
/// animation slots save a material only once, so restoration cannot put a
/// replacement back over its original after processing another slot.
#[derive(Default)]
struct BitmapOverrides {
    shown: Vec<(String, String)>,
    saved: Vec<(Handle<ObjectMaterial>, Option<Handle<Image>>)>,
}

impl BitmapOverrides {
    /// A later per-object opacity copy inherits the same saved texture,
    /// including when a replacement is already showing.
    fn copied(&mut self, from: &Handle<ObjectMaterial>, to: Handle<ObjectMaterial>) {
        if let Some((_, original)) = self.saved.iter().find(|(handle, _)| handle == from) {
            self.saved.push((to, original.clone()));
        }
    }

    fn apply<'a>(
        &mut self,
        pairs: Vec<(String, String)>,
        bindings: impl IntoIterator<Item = (&'a str, Handle<ObjectMaterial>)>,
        images: &std::collections::BTreeMap<String, Handle<Image>>,
        materials: &mut Assets<ObjectMaterial>,
    ) -> Option<usize> {
        if self.shown == pairs { return None; }
        for (handle, original) in self.saved.drain(..) {
            if let Some(mut material) = materials.get_mut(&handle) { material.texture = original; }
        }
        let mut seen = std::collections::HashSet::new();
        for (original, handle) in bindings {
            let Some(image) = replacement(original, &pairs, images) else { continue };
            if !seen.insert(handle.id()) { continue; }
            let Some(mut material) = materials.get_mut(&handle) else { continue };
            if material.texture.as_ref() == Some(&image) { continue; }
            self.saved.push((handle, material.texture.clone()));
            material.texture = Some(image);
        }
        let changed = self.saved.len();
        self.shown = pairs;
        Some(changed)
    }
}

/// Only static Swamp tiles need this adapter. Their model geometry remains
/// shared, while a material becomes private on its first texture substitution.
#[derive(Component)]
pub struct StaticTileBitmap {
    original: String,
    material: Handle<ObjectMaterial>,
    owned: bool,
    state: BitmapOverrides,
}

impl StaticTileBitmap {
    pub fn new(original: String, material: Handle<ObjectMaterial>) -> Self {
        // Look carries its source label (e.g. vol:Swamp/8bit/GASTRAP.BMP),
        // while script/class rules name the file, as animated slots do.
        let original = original.rsplit(['/', '\\']).next().unwrap_or(&original).to_string();
        Self { original, material, owned: false, state: Default::default() }
    }

    fn apply(&mut self, handle: &mut Handle<ObjectMaterial>, pairs: Vec<(String, String)>, images: &std::collections::BTreeMap<String, Handle<Image>>, materials: &mut Assets<ObjectMaterial>) -> Option<usize> {
        if self.state.shown == pairs { return None; }
        let replaced = !self.state.saved.is_empty() || replacement(&self.original, &pairs, images).is_some();
        if !self.owned && replaced {
            let material = materials.get(&self.material)?.clone();
            self.material = materials.add(material);
            self.owned = true;
        }
        let changed = self.state.apply(pairs, [(self.original.as_str(), self.material.clone())], images, materials);
        if replaced {
            // unlit_step may be drawing a highlighted copy. Return its
            // normal handle so it refreshes the bright copy from this
            // texture, preserving the original lighting for highlight off.
            *handle = self.material.clone();
        }
        changed
    }
}

#[cfg(test)]
mod bitmap_tests {
    use super::*;

    fn material(image: Handle<Image>, images: &mut Assets<Image>) -> ObjectMaterial {
        let light = crate::object_light::ObjectLight::new(
            kq8_data::lights::Lights::default(), crate::light_view::LightMode::X1,
            kq8_data::haze::Haze::new(100.0, 1000.0, [0.0; 3]), true,
        );
        let mut material = crate::object_light::object_material(
            crate::load::Look::Color([255; 3]), crate::load::Shading::PLAIN, &light, images,
        );
        material.texture = Some(image);
        material.settings.textured = 1;
        material.settings.lighting = 2;
        material.settings.haze_multiplier = 0.75;
        material.settings.base_colour.w = 0.5;
        material.alpha_mode = AlphaMode::Blend;
        material
    }

    #[test]
    fn tile_bitmap_material_matrix() {
        let mut materials = Assets::<ObjectMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut swaps = std::collections::BTreeMap::new();
        for name in kq8_core::tile_objects::REVEAL_BITMAPS.into_iter()
            .chain(kq8_core::secondary_tiles::LAVA_BITMAPS).chain(["normal.bmp", "other.bmp"]) {
            swaps.insert(name.to_ascii_lowercase(), images.add(Image::default()));
        }
        let mut world = kq8_core::active::Active::new(Default::default(), vec![], vec![], vec![], vec![]).world;
        for (height, names) in [(1, ["gastrap.bmp", "wtower5.bmp"]), (12, ["trapring.bmp", "trunk.bmp"])] {
            let originals: Vec<_> = names.iter().map(|_| images.add(Image::default())).collect();
            let base: Vec<_> = originals.iter().map(|image| materials.add(material(image.clone(), &mut images))).collect();
            let mut drawn = base.clone();
            let untouched_copy = base.clone();
            let mut parts: Vec<_> = names.iter().zip(&base).enumerate().map(|(index, (name, handle))| {
                let label = if index == 0 { format!("vol:Swamp/8bit/{name}") } else { format!("original\\Swamp\\{name}") };
                StaticTileBitmap::new(label, handle.clone())
            }).collect();
            world.swamp_tiles.insert("tile".into(), kq8_core::tile_objects::SwampTile {
                has_script: true, height_num: height, damage_factor: 0.48, last_contact: 0.0, revealed: false,
            });
            world.bmp_swaps.insert("tile".into(), (names[0].to_uppercase(), "NORMAL.BMP".into()));
            // Highlight may already be drawing a clone before the first swap.
            let mut bright = materials.get(&drawn[0]).unwrap().clone();
            bright.settings.lighting = 0;
            drawn[0] = materials.add(bright);
            let ordinary = bitmap_pairs(&world, "TiLe");
            assert_eq!(parts[0].apply(&mut drawn[0], ordinary.clone(), &swaps, &mut materials), Some(1));
            assert_eq!(parts[1].apply(&mut drawn[1], ordinary, &swaps, &mut materials), Some(0));
            assert_eq!(materials.get(&drawn[0]).unwrap().texture, Some(swaps["normal.bmp"].clone()));
            assert_eq!(drawn[1], base[1], "unaffected material remains shared");
            world.swamp_tiles.get_mut("tile").unwrap().revealed = true;
            let reveal = bitmap_pairs(&world, "tile");
            assert_eq!(reveal.len(), 3, "ordinary pair followed by both class pairs");
            for index in 0..2 {
                assert_eq!(parts[index].apply(&mut drawn[index], reveal.clone(), &swaps, &mut materials), Some(1));
                assert_eq!(materials.get(&drawn[index]).unwrap().texture, Some(swaps[&reveal[index + 1].1.to_ascii_lowercase()].clone()));
                assert_ne!(drawn[index], untouched_copy[index]);
                assert_eq!(materials.get(&untouched_copy[index]).unwrap().texture, Some(originals[index].clone()));
                let shown = materials.get(&drawn[index]).unwrap();
                assert_eq!((shown.settings.lighting, shown.settings.haze_multiplier, shown.settings.base_colour.w), (2, 0.75, 0.5));
                assert_eq!(shown.alpha_mode, AlphaMode::Blend);
            }
            let count = materials.len();
            let own = drawn.clone();
            for index in 0..2 {
                assert_eq!(parts[index].apply(&mut drawn[index], reveal.clone(), &swaps, &mut materials), None);
            }
            assert_eq!(drawn, own);
            assert_eq!(materials.len(), count, "repeat does not allocate material copies");
            // A later highlight copy must not become the saved normal material.
            let mut bright = materials.get(&drawn[0]).unwrap().clone();
            bright.settings.lighting = 0;
            drawn[0] = materials.add(bright);
            world.swamp_tiles.get_mut("tile").unwrap().revealed = false;
            for index in 0..2 {
                parts[index].apply(&mut drawn[index], bitmap_pairs(&world, "tile"), &swaps, &mut materials);
            }
            assert_eq!(drawn, own, "restore hands highlight its normal canonical handles");
            assert_eq!(materials.get(&drawn[0]).unwrap().texture, Some(swaps["normal.bmp"].clone()));
            assert_eq!(materials.get(&drawn[1]).unwrap().texture, Some(originals[1].clone()));
            world.bmp_swaps.remove("tile");
            for index in 0..2 {
                parts[index].apply(&mut drawn[index], bitmap_pairs(&world, "tile"), &swaps, &mut materials);
                assert_eq!(materials.get(&drawn[index]).unwrap().texture, Some(originals[index].clone()));
            }
            // Animated objects share this replacement helper, with their own
            // material handles. Multiple animation keys may alias one handle.
            let mut animated = BitmapOverrides::default();
            let bindings = || [(names[0], own[0].clone()), (names[0], own[0].clone()), (names[1], own[1].clone())];
            assert_eq!(animated.apply(reveal, bindings(), &swaps, &mut materials), Some(2));
            assert_eq!(animated.saved.len(), 2);
            assert_eq!(animated.apply(vec![], bindings(), &swaps, &mut materials), Some(0));
            for index in 0..2 { assert_eq!(materials.get(&own[index]).unwrap().texture, Some(originals[index].clone())); }
        }
        // Lava primes affect only the top, across aliased movement slots.
        let original = images.add(Image::default());
        let top = materials.add(material(original.clone(), &mut images));
        let side = materials.add(material(original.clone(), &mut images));
        let mut lava = BitmapOverrides::default();
        let bindings = || [("lvtile01.bmp", top.clone()), ("lvtile01.bmp", top.clone()), ("side.bmp", side.clone())];
        for (index, name) in kq8_core::secondary_tiles::LAVA_BITMAPS.into_iter().enumerate() {
            world.lava_tiles.insert("lava".into(), kq8_core::tile_objects::LavaTile { bitmap: Some(name.into()), ..Default::default() });
            assert_eq!(lava.apply(bitmap_pairs(&world, "LAVA"), bindings(), &swaps, &mut materials), Some(usize::from(index != 0)));
            let expected = if index == 0 { &original } else { &swaps[&name.to_ascii_lowercase()] };
            assert_eq!(materials.get(&top).unwrap().texture.as_ref(), Some(expected));
            assert_eq!(materials.get(&side).unwrap().texture, Some(original.clone()));
        }
        let later_copy = materials.add(materials.get(&top).unwrap().clone());
        lava.copied(&top, later_copy.clone());
        // Unrelated ordinary pairs survive class overlays; a missing replacement
        // or an identity pair restores the original rather than a stale image.
        let pairs = vec![("side.bmp".into(), "other.bmp".into()), ("lvtile01.bmp".into(), "missing.bmp".into())];
        assert_eq!(lava.apply(pairs, bindings(), &swaps, &mut materials), Some(1));
        assert_eq!(materials.get(&top).unwrap().texture, Some(original.clone()));
        assert_eq!(materials.get(&later_copy).unwrap().texture, Some(original.clone()), "later opacity copy also restores the original");
        assert_eq!(materials.get(&side).unwrap().texture, Some(swaps["other.bmp"].clone()));
        let identity = vec![("side.bmp".into(), "SIDE.BMP".into())];
        assert_eq!(lava.apply(identity, bindings(), &swaps, &mut materials), Some(0));
        assert_eq!(materials.get(&side).unwrap().texture, Some(original));
        println!("tile bitmap matrix: both Swamp pairs, eight Lava tops, instance isolation, highlight handoff, alias restore, repeats and ordinary fallback passed");
    }
}

/// A step that does nothing (no move, no turn), within rounding.
fn is_still(t: &GameTransform) -> bool {
    let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    t.p.iter().all(|v| v.abs() < 0.001) && (0..3).all(|i| (0..3).all(|j| (t.m[i][j] - id[i][j]).abs() < 1e-5))
}

/// The pose `pose` (own space) followed by the placement: m = pose.m × R,
/// p = pose.p × R + position (row vectors).
fn combine(pose: &GameTransform, r: &[[f32; 3]; 3], position: [f32; 3]) -> ([[f32; 3]; 3], [f32; 3]) {
    let m = std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| pose.m[i][k] * r[k][j]).sum()));
    let p = std::array::from_fn(|j| (0..3).map(|k| pose.p[k] * r[k][j]).sum::<f32>() + position[j]);
    (m, p)
}

impl AnimSim {
    /// A copy prepared for a script's `loadKQ` comes into the world (M3
    /// 3k-4): its player goes to `active`, which plays it from now on.
    /// `false` if no unused copy by that name plays here.
    pub fn bring_in(&mut self, name: &str, active: &mut kq8_core::active::Active) -> bool {
        let Some(p) = self.playing.iter_mut().find(|p| p.name.eq_ignore_ascii_case(name) && matches!(p.source, Source::Own(_))) else {
            return false;
        };
        let index = active.world.anims.len();
        let Source::Own(player) = std::mem::replace(&mut p.source, Source::Core(index)) else { unreachable!() };
        let player = p.at_load.take().unwrap_or(player);
        active.add_anims([AnimObject { player: *player, script: p.script.clone(), gone: false, character: p.character.take(), translucency: None }]);
        true
    }

    /// A new copy of an unused object (M5 5d-4, a script's `loadKQ` while
    /// playing): its body and frames from the template's recipe, at its
    /// place, played by core's `anims[index]`. Returns the log line.
    pub fn spawn_copy(
        &mut self,
        commands: &mut Commands,
        template: &str,
        name: &str,
        placement: Placement,
        index: usize,
        unborn: bool,
        (gpu, materials): (Option<&crate::anim_gpu::AnimGpu>, &mut Assets<ObjectMaterial>),
    ) -> String {
        let Some(r) = self.recipes.get(&template.to_ascii_lowercase()) else {
            return format!("copy {name}: no recipe for {template}; not drawn");
        };
        let mut body = commands.spawn((bevy_transform_from(placement.rotation, placement.position), Visibility::default(), crate::terrain_view::ObjectName(name.to_string())));
        if r.character.is_some() && unborn { body.insert((Visibility::Hidden, ShownWithH)); }
        let body = body.id();
        let distance = r.character.as_ref().map(|c| {
            let mut test = c.distance.another();
            test.name = name.to_string();
            test.hidden = false;
            if unborn {
                test.born_groups = Some(test.groups);
                test.groups = [test.groups[0] | 1, 0, 0];
            }
            test
        });
        if let Some(test) = distance.clone() { commands.entity(body).insert(test); }
        // Step 4e: drawn the new way too.
        let new_draw = gpu.and_then(|gpu| crate::anim_gpu::Slots::spawn(commands, body, gpu, crate::anim_gpu::List::Objects, r.slots.0.clone(), r.slots.1.clone(), materials, &mut Default::default()));
        let face: Vec<Handle<ObjectMaterial>> = match (&new_draw, &r.character) {
            (Some(draw), Some(c)) => c.face_keys.iter().filter_map(|key| draw.materials.get(key).cloned()).collect(),
            _ => Vec::new(),
        };
        let lip_images = r.character.as_ref().map_or_else(Vec::new, |c| c.lip_images.clone());
        let casts = r.character.as_ref().is_some_and(|c| c.casts_shadow);
        let movements: Vec<DrawnMovement> = r.movements.iter().map(|(frames, steps)| DrawnMovement { frames: *frames, steps: steps.clone() }).collect();
        let parts = Vec::new();
        let line = format!(
            "copy {name} of {template}: drawn, {} movement(s), {} part entities, played by the world's rules (M5 5d-4); character distance {}, face {} material(s), shadow {}, unborn {}",
            movements.len(),
            new_draw.as_ref().map_or(0, |s| s.len()),
            distance.as_ref().map_or("none".to_string(), |d| format!("radius {:.0} ({})", d.radius, d.ball_from)),
            face.len(), if casts { "eligible" } else { "off" }, unborn,
        );
        self.playing.push(Playing {
            name: name.to_string(),
            source: Source::Core(index),
            script: r.script.clone(),
            glide: Default::default(),
            rotation: placement.rotation,
            position: placement.position,
            movements,
            body,
            shown: None,
            tracked: false,
            // Its collider (`copies_view::add_collider`) is found by name
            // and moves with its steps (G-3; the death-hall stones).
            collider: None,
            character: None,
            test: None,
            face,
            lip_images,
            mouth_shown: 0,
            parts,
            opacity_shown: 1.0,
            own: Default::default(),
            shadow_mask: r.character.as_ref().map_or(0, |c| c.shadow_mask),
            shadow_translucency: r.character.as_ref().map_or(256.0, |c| c.shadow_translucency),
            shadow: None,
            birth_shown: unborn.then_some(false),
            new_draw,
            texture_names: r.textures.clone(),
            bitmaps: Default::default(),
            at_load: None,
        });
        line
    }

    pub fn has_recipe(&self, file: &str) -> bool {
        self.recipes.contains_key(&file.to_ascii_lowercase())
    }

    /// The draw-distance test of a character that just came in (M4 4d-3),
    /// with its body, to be put on it.
    pub fn take_test(&mut self, name: &str) -> Option<(Entity, crate::draw_distance::DistanceTest)> {
        self.playing.iter_mut().find(|p| p.name.eq_ignore_ascii_case(name) && p.test.is_some()).and_then(|p| Some((p.body, p.test.take()?)))
    }

    /// `deleteObject` (3f): its collider is no longer moved.
    pub fn forget_collider(&mut self, name: &str) {
        for p in self.playing.iter_mut().filter(|p| p.name.eq_ignore_ascii_case(name)) {
            p.collider = Some(None);
        }
    }
}

/// The player of `p`: in core, or its own.
fn player<'a>(p: &'a Playing, doors: Option<&'a DoorSim>) -> Option<&'a NamedAnimPlayer> {
    match &p.source {
        Source::Core(i) => doors.and_then(|d| d.active.world.anims.get(*i)).map(|a| &a.player),
        Source::Own(player) => Some(player),
    }
}

/// Builds the objects and returns the ones the world's scripts can reach,
/// in order, for `Active::add_anims` (3i-4). `placed` is
/// `Objects::placed`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_animated(
    commands: &mut Commands,
    animated: Vec<AnimatedAssets>,
    animations: Vec<Animation>,
    script_shapes: &std::collections::BTreeMap<String, (usize, DoorMovement)>,
    placed: &[(kq8_data::objects::Placement, usize, bool)],
    templates: &std::collections::BTreeSet<usize>,
    monster_targets: &std::collections::BTreeSet<String>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    balls: &crate::draw_distance::ShapeBalls,
    character_lines: &[String],
    main_log: &mut Log,
    gpu: &crate::anim_gpu::AnimGpu,
    swap_looks: &[(String, crate::load::Look)],
) -> (Vec<AnimObject>, Vec<NamedAnimPlayer>, Vec<PreparedMonster>) {
    let mut log = match Log::create("anim") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("animated objects: cannot create anim.log: {e}; not started"));
            return (Vec::new(), Vec::new(), Vec::new());
        }
    };
    // Per animation: its number of frames, the materials by index, its
    // length and steps.
    struct Built {
        frames: usize,
        materials: Vec<(i32, Handle<ObjectMaterial>)>,
        duration: f32,
        steps: Vec<GameTransform>,
        moves: bool,
        /// The loop starts at the file's first frame (Q164).
        first_in_file: bool,
        /// Each frame's lights (M1 7c, rule 26b).
        lights: std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>,
        /// Each material's texture file name (for a character's face).
        textures: Vec<(i32, Option<String>)>,
        /// Its `.kqs` ball (rule 37 (2), M4 4d-7b-5b).
        ball: crate::shadow_view::ModelBall,
        /// Each frame's named parts (`ranged-weapons.md` rule 24, M5 5f-2).
        parts: Vec<kq8_data::shape::FrameParts>,
        /// Its model (step 4).
        source: Option<crate::load::FrameShape>,
    }
    let built: Vec<Built> = animations
        .into_iter()
        .map(|a| {
            let textures = a
                .looks
                .iter()
                .map(|(m, look, _)| match look {
                    crate::load::Look::Texture { name, .. } => (*m, Some(name.rsplit(['/', '\\']).next().unwrap_or(name).to_string())),
                    _ => (*m, None),
                })
                .collect();
            let materials = a.looks.into_iter().map(|(m, look, l)| (m, materials.add(object_material(look, l, light, images)))).collect();
            let frames = a.frame_count;
            let source = a.source.clone();
            let moves = a.frame_transforms.iter().skip(1).any(|t| !is_still(t));
            let ball = crate::shadow_view::ModelBall::of(a.ball);
            Built { frames, materials, duration: a.duration, steps: a.frame_transforms, moves, first_in_file: a.first_in_file, lights: std::sync::Arc::new(a.frame_lights), textures, ball, parts: a.frame_parts, source }
        })
        .collect();
    let mut rng = Rng::new(SEED);
    let mut characters: Vec<String> = Vec::new();
    let mut playing = Vec::new();
    let mut core = Vec::new();
    // M5 5d-4: the unused objects' players and recipes (templates).
    let mut own_players = Vec::new();
    let mut monster_recipes = Vec::new();
    let mut recipes = std::collections::BTreeMap::new();
    for (file, (index, _)) in script_shapes {
        let model = &built[*index];
        let own = model.materials.iter().map(|(m, material)| ((*index, *m), material.clone())).collect();
        recipes.insert(file.clone(), Recipe {
            movements: vec![(model.frames, model.steps.clone())], script: None,
            slots: (vec![(*index, model.source.clone(), model.ball)], own), textures: vec![], character: None,
        });
    }
    let mut moving = Vec::new();
    let mut characters_new = 0;
    let (mut random, mut switchable) = (0, 0);
    let mut stop_starts: Vec<String> = Vec::new();
    for mut a in animated {
        let ranges = a.append_alternate_lists();
        let (p, shape_index, unused) = &placed[a.placed];
        // F-6: a template from a start-up placement needs its recipe too.
        let template = templates.contains(&a.placed);
        random += usize::from(a.random);
        let opacity = p.opacity();
        let rotation = kq8_data::objects::rotation(p.angles);
        // The movements as core plays them (anim-scripts.md rules 1–10).
        let movements = crate::actor_recipes::movements(&a.movements, |i| crate::actor_recipes::AnimationMotion {
            duration: built[i].duration, steps: &built[i].steps,
            first_in_file: built[i].first_in_file, parts: &built[i].parts,
        });
        // ranged-weapons.md rule 24 (M5 5f-2): where a bolt would start, per
        // frame of each movement that has an `arrow` part (for the checks).
        for m in &movements {
            let arrow: Vec<String> = m
                .frame_parts
                .iter()
                .enumerate()
                .filter_map(|(k, parts)| parts.iter().find(|(n, _)| n.eq_ignore_ascii_case("arrow")).map(|(_, q)| format!("{k}: ({:.0}, {:.0}, {:.0})", q.p[0], q.p[1], q.p[2])))
                .collect();
            if !arrow.is_empty() {
                log.detail(format!("[{}] '{}' ({} frames) part arrow by frame: {}", p.name, m.name, m.frames(), arrow.join(", ")));
            }
        }
        let crate::actor_recipes::AnimatedPlayer { player, selected_at_load } =
            crate::actor_recipes::animated_player(&p.name, Placement { rotation, position: p.position }, &a, movements, ranges, &mut rng);
        if selected_at_load != a.current {
            let was = player.movements.get(a.current).map_or("none", |m| m.name.as_str()).to_string();
            stop_starts.push(format!("{} starts on '{}' (movement {}), not '{was}' (movement {}): no currAnimName, so the name Stop (anim-scripts.md rule 3 corrected)", p.name, player.movements.get(selected_at_load).map_or("none", |m| m.name.as_str()), selected_at_load, a.current));
        }
        // Q164: its walk and run speeds by `objects.md` "Loops" rule 6a.
        if player.loops_move_base {
            let speeds: Vec<String> = player
                .movements
                .iter()
                .filter(|m| m.repeat && (m.name.eq_ignore_ascii_case("walk") || m.name.eq_ignore_ascii_case("run")))
                .map(|m| format!("{} {:.0} units/s ({} frames, {:.4} s, from the file's first frame: {})", m.name, m.loop_ground_speed(m.speed), m.frames(), m.play_time, m.first_in_file))
                .collect();
            if !speeds.is_empty() {
                log.detail(format!("[{}] character speeds (objects.md \"Loops\" rule 6a): {}", p.name, speeds.join("; ")));
            }
        }
        let mut body = commands.spawn((bevy_transform_from(rotation, p.position), Visibility::default(), crate::terrain_view::ObjectName(p.name.clone())));
        // A character not born yet (characters.md rule 16): not drawn;
        // only with H, as hidden objects.
        let unborn = a.character.as_ref().is_some_and(|c| c.unborn);
        if *unused || !p.drawn() || unborn {
            body.insert((Visibility::Hidden, ShownWithH));
        }
        // Draw distance (M1 6h); unused objects are only shown with H.
        // A character's ball is its rule-9 box.
        // A character's states (M4 4d-3).
        let mut character = a.character.as_ref().map(|c| crate::actor_recipes::character(
            p, c, &player, balls.character_box(&p.name, c.box_type.as_deref()),
        ));
        let mut prepared_test = None;
        if *unused {
            if let Some(c) = &a.character {
                prepared_test = Some(balls.test_character(p, c.box_type.as_deref()));
            }
        }
        if !*unused {
            let test = match &a.character {
                Some(c) => {
                    let mut test = balls.test_character(p, c.box_type.as_deref());
                    if c.unborn {
                        // Rule 16: groupsAlways plus bit 1 only, no draw
                        // or collider bits (its own again once risen).
                        test.born_groups = Some(test.groups);
                        test.groups = [p.groups_always | 1, 0, 0];
                    }
                    test
                }
                None => balls.test_at(p, *shape_index),
            };
            if let Some(c) = &a.character {
                let l = &a.movements[player.current].1;
                characters.push(format!(
                    "[{}] {}: from {}; at ({:.0}, {:.0}, {:.0}) heading {:.2}; {} movements, shown at load '{}' ({} {} speed {} repeat {} cut short {}); flags {:#x}{}{}; ball from {} (centre ({:.0}, {:.0}, {:.0}) m, radius {:.0}); wanders {}; birthDelay {}{}{}{}; movement rules: {}",
                    p.name,
                    c.keys.class.name(),
                    c.from,
                    p.position[0],
                    p.position[1],
                    p.position[2],
                    p.angles[2],
                    a.movements.len(),
                    a.movements[player.current].0,
                    l.shape,
                    l.sequence,
                    l.speed,
                    l.repeat,
                    l.unknown != 0,
                    c.keys.flags,
                    if c.keys.no_turn { ", noTurn" } else { "" },
                    if c.keys.gravity { "" } else { ", gravity off" },
                    test.ball_from,
                    test.centre.x,
                    test.centre.y,
                    test.centre.z,
                    test.radius,
                    c.wanders,
                    c.birth_delay,
                    if c.unborn { "; has Birth: NOT DRAWN until born (rule 16)" } else { "" },
                    c.lips.as_ref().map_or(String::new(), |l| format!("; lips [{}] face {}", l.section, l.face)),
                    c.skin.as_ref().map_or(String::new(), |(f, t)| format!("; skin {f} -> {t}")),
                    c.ai.describe(),
                ));
            }
            body.insert(test);
            // The lights it carries (M1 7c), by the movement and frame shown.
            let lights = a.movements.iter().map(|(_, _, i)| built[*i].lights.clone()).collect();
            if let Some(c) = crate::scene_lights::CarriedLights::new(lights, bevy_transform_from(rotation, p.position), p.turn_off_dyn_lighting) {
                body.insert(c);
            }
        }
        let body = body.id();
        // A character's mouth (M4 4d-5, rule 25): its own copy of every
        // material showing the face texture, in every movement, so the
        // shape is kept across movements and not shared with others.
        let face_name = a.character.as_ref().and_then(|c| c.lips.as_ref()).map(|l| l.face.clone());
        let face_keys: Vec<(usize, i32)> = face_name.as_ref().map_or_else(Vec::new, |f| a
            .movements.iter().flat_map(|(_, _, i)| built[*i].textures.iter()
                .filter(|(_, t)| t.as_ref().is_some_and(|t| t.eq_ignore_ascii_case(f)))
                .map(move |(m, _)| (*i, *m))).collect());
        let lip_images: Vec<Option<Handle<Image>>> = a
            .lip_looks
            .iter()
            .map(|l| match l {
                Some(look) => light.palette.look(look.clone(),images,bevy::image::ImageAddressMode::Repeat),
                _ => None,
            })
            .collect();
        let parts = Vec::new();
        // Steps 4b, 4d, 4e: every animated object is drawn the new way
        // too (the unused ones, copy templates, only with H; their copies
        // from the recipe).
        let own: SlotMaterials = a
            .movements
            .iter()
            .flat_map(|(_, _, i)| built[*i].materials.iter().map(move |(m, h)| ((*i, *m), h.clone())))
            .map(|(k, h)| (k, crate::object_light::for_object(materials, &h, opacity, p.haze_solid, p.haze_multiplier)))
            .collect();
        let slot_movements: Vec<SlotMovement> = a.movements.iter().map(|(_, _, i)| (*i, built[*i].source.clone(), built[*i].ball)).collect();
        let new_draw = crate::anim_gpu::Slots::spawn(commands, body, gpu, crate::anim_gpu::List::Objects, slot_movements.clone(), own.clone(), materials, &mut Default::default());
        characters_new += usize::from(a.character.is_some() && new_draw.is_some());
        // Step 4d: the new way's face materials take the mouth shapes too.
        let new_faces: Vec<Handle<ObjectMaterial>> = match &new_draw {
            Some(nd) => face_keys.iter().filter_map(|k| nd.materials.get(k).cloned()).collect(),
            _ => Vec::new(),
        };
        let drawn: Vec<DrawnMovement> = a
            .movements
            .iter()
            .map(|(_, _, i)| DrawnMovement { frames: built[*i].frames, steps: if built[*i].moves { built[*i].steps.clone() } else { Vec::new() } })
            .collect();
        let recipe: Vec<(usize, Vec<GameTransform>)> = drawn.iter().map(|d| (d.frames, d.steps.clone())).collect();
        // Batch 2 D-6: each material's original texture name, keyed as
        // its drawing's materials are.
        let texture_names: Vec<((usize, i32), String)> =
            a.movements.iter().flat_map(|(_, _, i)| built[*i].textures.iter().filter_map(move |(m, t)| t.as_ref().map(|t| ((*i, *m), t.clone())))).collect();
        if monster_targets.contains(&a.source_file) && !monster_recipes.iter().any(|r: &PreparedMonster| r.file == a.source_file) {
            if let (Some(keys), Some(ch)) = (a.character.as_ref(), character.as_ref()) {
                recipes.insert(a.source_file.clone(), Recipe { movements: recipe.clone(), script: a.script.clone(), slots: (slot_movements.clone(), own.clone()), textures: texture_names.clone(),
                    character: Some(CharacterRecipe { distance: balls.test_character(p, keys.box_type.as_deref()), face_keys: face_keys.clone(), lip_images: lip_images.clone(), casts_shadow: keys.casts_shadow, shadow_mask: keys.keys.shadow_mask, shadow_translucency: p.translucency }) });
                monster_recipes.push(PreparedMonster { file: a.source_file.clone(), keys: keys.keys.clone(), combat_scaled: keys.combat_scaled,
                    actor: AnimObject { player: player.clone(), script: a.script.clone(), gone: false, character: Some(ch.clone()), translucency: None } });
            }
        }
        if (*unused || template) && a.character.is_none() {
            recipes.insert(p.name.to_ascii_lowercase(), Recipe { movements: recipe, script: a.script.clone(), slots: (slot_movements, own), textures: texture_names.clone(), character: None });
        }
        // Whole-object steps of the movement shown at load: how far the
        // chained pose goes over the loop.
        let b = &built[a.movements[player.current].2];
        let mut reach = String::from("none");
        if b.moves {
            let (mut far, mut turn) = (0.0f32, 0.0f32);
            for k in 0..b.steps.len() {
                let pose = pose_at_frame(&b.steps, k);
                far = far.max(pose.p.iter().map(|v| v * v).sum::<f32>().sqrt());
                turn = turn.max(kq8_core::door::turn_degrees(&pose).abs());
            }
            let last = pose_at_frame(&b.steps, b.steps.len().saturating_sub(1));
            reach = format!(
                "up to {far:.1} units and {turn:.1} deg about z; at the last frame ({:.1}, {:.1}, {:.1}), {:.1} deg",
                last.p[0],
                last.p[1],
                last.p[2],
                kq8_core::door::turn_degrees(&last)
            );
            moving.push(p.name.clone());
        }
        let (_, l, _) = &a.movements[player.current];
        let more = if a.movements.len() > 1 {
            switchable += 1;
            format!("; movements {:?}", a.movements.iter().map(|(n, _, _)| n.as_str()).collect::<Vec<_>>())
        } else {
            String::new()
        };
        log.detail(format!(
            "[{}] {} frames, {:.4} s, start {} speed {} repeat {} random {} -> phase {:.4} speed {:.4}, frame {}; whole-object steps: {reach}{}{more}",
            p.name,
            b.frames,
            b.duration,
            l.start,
            l.speed,
            l.repeat,
            a.random,
            player.phase,
            player.speed(),
            player.frame(),
            if *unused || !p.drawn() { "; hidden (H)" } else { "" }
        ) + if player.current_name.is_empty() && player.waiting.is_some() {
            // anim-scripts.md rule 3 (Q85).
            "; no currAnimName: the first update starts the waiting movement plainly (phase from its loop= line, its own speed; the random part is lost)"
        } else {
            ""
        });
        let tracked = TRACKED.iter().any(|t| t.eq_ignore_ascii_case(&p.name));
        let at_load = unused.then(|| Box::new(player.clone()));
        let source = if *unused {
            if a.character.is_none() {
                own_players.push(player.clone());
            }
            Source::Own(Box::new(player))
        } else {
            core.push(AnimObject { player, script: a.script.clone(), gone: false, character: character.take(), translucency: None });
            Source::Core(core.len() - 1)
        };
        playing.push(Playing {
            name: p.name.clone(),
            source,
            script: a.script.clone(),
            glide: Default::default(),
            rotation,
            position: p.position,
            movements: drawn,
            body,
            shown: None,
            tracked,
            collider: None,
            character,
            test: prepared_test,
            face: new_faces,
            lip_images,
            mouth_shown: 0,
            parts,
            opacity_shown: opacity,
            own: Default::default(),
            shadow_mask: a.character.as_ref().map_or(0, |c| c.keys.shadow_mask),
            shadow_translucency: p.translucency,
            shadow: None,
            birth_shown: unborn.then_some(false),
            new_draw,
            texture_names,
            bitmaps: Default::default(),
            at_load,
        });
    }
    let new_way = playing.iter().filter(|p| p.new_draw.is_some()).count();
    let slots: usize = playing.iter().filter_map(|p| p.new_draw.as_ref()).map(|n| n.len()).sum();
    main_log.summary(format!(
        "animated objects drawn from the compact data (Animation memory step 4): {new_way} of {} (characters among them: {characters_new}), {slots} part entities",
        playing.len()
    ));
    main_log.summary(format!(
        "animated objects: {} playing ({} run by the world's scripts, {} with more than one movement, {} with a random start and speed, flags bit 8), {} different loops; whole-object steps in {} objects {moving:?}; log in {}",
        playing.len(),
        core.len(),
        switchable,
        random,
        built.len(),
        moving.len(),
        log.path().display()
    ));
    main_log.summary(format!("animated objects and characters: {} start on another movement than the loader's because the names start as Stop (M4 4d-8); listed in the log", stop_starts.len()));
    for line in &stop_starts {
        main_log.detail(format!("names Stop: {line}"));
    }
    // Characters (M4 4d-2): one line each, and the loader's lines.
    let mut clog = Log::create("characters").ok();
    let drawn = characters.len();
    if let Some(c) = clog.as_mut() {
        for line in character_lines.iter().chain(&characters) {
            c.detail(line);
        }
    }
    main_log.summary(format!(
        "characters: {drawn} drawn from their movement lists ({} not born yet: hidden), {} loader notes; log in kq8-artifacts/logs/characters.log",
        characters.iter().filter(|l| l.contains("NOT DRAWN")).count(),
        character_lines.len()
    ));
    let fight = Log::create("fight").ok();
    if let Some(f) = fight.as_ref() {
        main_log.summary(format!("fight: log in {}", f.path().display()));
    }
    // Batch 2 D-6: the pictures `changeBmp` swaps in.
    let swap_images = swap_looks
        .iter()
        .filter_map(|(n, l)| match l {
            crate::load::Look::Texture { .. } => light.palette.look(l.clone(),images,bevy::image::ImageAddressMode::Repeat).map(|h| (n.to_ascii_lowercase(),h)),
            _ => None,
        })
        .collect();
    commands.insert_resource(AnimSim { playing, recipes, swap_images, updates: 0, log, characters: clog, fight });
    (core, own_players, monster_recipes)
}

/// One update (15 per second), after `door_step` has run core's update:
/// unused objects move their own loops; an object whose shown movement
/// moves it takes its collider along (3r-4).
pub fn anim_step(sim: Option<ResMut<AnimSim>>, colliders: Option<ResMut<ObjectColliders>>, mut doors: Option<ResMut<DoorSim>>, connor: Option<Res<crate::connor_view::ConnorSim>>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    // characters.log (M4 4d-3): core's lines, and every 10 s how many
    // characters are in each state.
    if let Some(d) = doors.as_mut() {
        // fight.log (M5 5c): core's fight lines.
        let fight = std::mem::take(&mut d.active.world.fight_log);
        if let Some(log) = sim.fight.as_mut() {
            for l in fight {
                log.detail(l);
            }
        }
        let lines = std::mem::take(&mut d.active.world.character_log);
        if let Some(log) = sim.characters.as_mut() {
            for l in lines {
                log.detail(l);
            }
            if n % 150 == 0 {
                let w = &d.active.world;
                let mut states: std::collections::BTreeMap<String, usize> = Default::default();
                for c in w.anims.iter().filter(|a| !a.gone).filter_map(|a| a.character.as_deref()) {
                    let key = if c.first_update.is_none() { "not updated yet".to_string() } else { format!("{:?}", c.state) };
                    *states.entry(key).or_default() += 1;
                }
                log.detail(format!("update {}: characters near or far now: {}; by state {states:?}", w.updates, w.characters_in_range.len()));
            }
        }
    }
    let mut contacts = Vec::new();
    let mut colliders = colliders;
    for p in &mut sim.playing {
        if let Source::Own(player) = &mut p.source {
            player.advance(1.0 / kq8_core::UPDATES_PER_SECOND as f32);
        }
        let Some(player) = player(p, doors.as_deref()) else { continue };
        let (movement, frame) = (player.current, player.frame());
        // From the place core keeps (a `move` or `KQMoveEvent` changes it;
        // G-3), as the drawing does.
        let base = player.base;
        if p.tracked && n <= TRACK_UPDATES {
            sim.log.detail(format!("update {n} [{}] phase {:.4} frame {}{}", p.name, player.phase, frame, if player.has_ended() { " finished" } else { "" }));
        }
        let Some(drawn) = p.movements.get(movement) else { continue };
        if drawn.steps.is_empty() {
            continue;
        }
        let Some(colliders) = colliders.as_mut() else { continue };
        let slot = *p.collider.get_or_insert_with(|| {
            let found: Vec<usize> = colliders.0.iter().enumerate().filter(|(_, c)| c.name == p.name).map(|(i, _)| i).collect();
            match found[..] {
                [i] => {
                    sim.log.detail(format!("[{}] its collider moves with it (objects.md Q51 rule 5)", p.name));
                    Some(i)
                }
                [] => {
                    sim.log.detail(format!("[{}] has no collider: nothing to move", p.name));
                    None
                }
                _ => {
                    sim.log.detail(format!("[{}] {} colliders by that name: none moved", p.name, found.len()));
                    None
                }
            }
        });
        let Some(i) = slot else { continue };
        // Switched off (out of the collision world, or deleted): stays off.
        if colliders.0[i].min == colliders.0[i].max {
            continue;
        }
        let frame = frame.min(drawn.frames.saturating_sub(1));
        let (rotation, position) = combine(&pose_at_frame(&drawn.steps, frame), &base.rotation, base.position);
        let collider = &colliders.0[i];
        if collider.kind.ball_stack && collider.placement.position != position &&
            doors.as_ref().is_some_and(|d| d.active.has_contact_script(&p.name)) {
            if let Some(con) = &connor {
                use kq8_core::collide::{BallStack, CylinderCollider, CONNOR_MASK, CONNOR_TYPE};
                use kq8_core::connor_mover::CONNOR_BOX;
                // The moving rock is the mover. Sweep its balls against
                // Connor's upright cylinder and leave its own movement intact.
                let target = CylinderCollider::from_box("Connor", con.mover.position, CONNOR_BOX.0, CONNOR_BOX.1, CONNOR_TYPE, CONNOR_MASK);
                let balls = BallStack::from_box(collider.min, collider.max);
                if let Some(hit) = target.sweep(&balls, collider.placement.position, position).contact {
                    contacts.push((p.name.clone(), hit.normal));
                }
            }
        }
        colliders.0[i].placement = kq8_core::collide::Placement { rotation, position };
        if n % 15 == 0 && n <= 150 {
            sim.log.detail(format!(
                "update {n} [{}] frame {frame}: collider at ({:.0}, {:.0}, {:.0})",
                p.name, position[0], position[1], position[2]
            ));
        }
    }
    if let Some(d) = doors.as_mut() {
        for (name, normal) in contacts {
            d.active.connor_contact(&name, normal);
        }
    }
}

/// Each object shows its current movement's frame (and pose, if that
/// movement moves it).
#[allow(clippy::too_many_arguments)]
pub fn anim_draw(
    mut commands: Commands,
    sim: Option<ResMut<AnimSim>>,
    doors: Option<Res<DoorSim>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut carried: Query<&mut crate::scene_lights::CarriedLights>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    mut part_materials: Query<&mut MeshMaterial3d<ObjectMaterial>>,
    mut static_bitmaps: Query<(Entity, &crate::terrain_view::ObjectName, &mut StaticTileBitmap)>,
    gpu: Option<Res<crate::anim_gpu::AnimGpu>>,
    fixed: Res<Time<Fixed>>,
) {
    let Some(mut sim) = sim else { return };
    let doors = doors.as_deref();
    let sim = &mut *sim;
    let ahead = crate::smooth_view::ahead(&fixed);
    let smooth = crate::smooth_view::on();
    let (update, a) = (doors.map_or(sim.updates, |d| d.active.world.updates), fixed.overstep_fraction());
    if let Some(doors) = doors {
        for (entity, name, mut bitmap) in &mut static_bitmaps {
            let pairs = bitmap_pairs(&doors.active.world, &name.0);
            if bitmap.state.shown == pairs { continue; }
            let Ok(mut material) = part_materials.get_mut(entity) else { continue };
            if let Some(changed) = bitmap.apply(&mut material.0, pairs, &sim.swap_images, &mut materials) {
                sim.log.detail(format!("update {}: {}: static tile bitmap replacements {:?}; {changed} material(s)", sim.updates, name.0, bitmap.state.shown));
            }
        }
    }
    for p in &mut sim.playing {
        // makeTranslucent (characters.md rule 22, M4): every material of
        // every part at n / 255, at once (objects.md "See-through
        // objects" rule 2); the object gets its own copies first.
        if let (Source::Core(i), Some(d)) = (&p.source, doors) {
            if d.active.world.weeper_links.contains(i) {
                let gone = d.active.world.anims[*i].gone;
                let wanted = if gone { Visibility::Hidden } else { Visibility::Inherited };
                if let Ok(mut v) = visibility.get_mut(p.body) {
                    v.set_if_neq(wanted);
                }
                if gone {
                    continue;
                }
            }
        }
        let wanted = match (&p.source, doors) {
            (Source::Core(i), Some(d)) => d.active.world.anims.get(*i).and_then(|a| a.translucency),
            _ => None,
        };
        if let Some(n) = wanted {
            let opacity = if n >= 256.0 { 1.0 } else { (n / 255.0).clamp(0.0, 1.0) };
            if opacity != p.opacity_shown {
                let mut changed = 0;
                // Shared material → this object's copy (one per material).
                let mut copies: std::collections::HashMap<AssetId<ObjectMaterial>, Handle<ObjectMaterial>> = Default::default();
                for e in &p.parts {
                    let Ok(mut mm) = part_materials.get_mut(*e) else { continue };
                    let id = mm.0.id();
                    if !p.own.contains_key(&id) {
                        let Some(m) = materials.get(&mm.0).cloned() else { continue };
                        // Its alpha at opacity 1: the colour's, undone.
                        let full = if p.opacity_shown > 0.0 && m.alpha_mode == AlphaMode::Blend { m.settings.base_colour.w / p.opacity_shown } else { m.settings.base_colour.w };
                        if p.face.iter().any(|h| h.id() == id) {
                            // Already its own (the mouth's copies, 4d-5).
                            p.own.insert(id, (mm.0.clone(), full));
                        } else {
                            let h = copies.entry(id).or_insert_with(|| materials.add(m)).clone();
                            p.own.insert(h.id(), (h.clone(), full));
                            mm.0 = h;
                        }
                    }
                    changed += 1;
                }
                // Step 4: the new way's materials get own copies too.
                if let Some(nd) = p.new_draw.as_mut() {
                    for h in nd.materials.values_mut() {
                        if p.own.contains_key(&h.id()) {
                            continue;
                        }
                        let Some(m) = materials.get(&*h).cloned() else { continue };
                        let full = if p.opacity_shown > 0.0 && m.alpha_mode == AlphaMode::Blend { m.settings.base_colour.w / p.opacity_shown } else { m.settings.base_colour.w };
                        if p.face.iter().any(|f| f.id() == h.id()) {
                            // Already its own (the mouth's, 4d).
                            p.own.insert(h.id(), (h.clone(), full));
                            continue;
                        }
                        let copy = materials.add(m);
                        p.bitmaps.copied(h, copy.clone());
                        p.own.insert(copy.id(), (copy.clone(), full));
                        *h = copy;
                    }
                    nd.forget_shown();
                }
                for (h, full) in p.own.values() {
                    if let Some(mut m) = materials.get_mut(h) {
                        m.settings.base_colour.w = full * opacity;
                        m.alpha_mode = if opacity < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque };
                    }
                }
                if let Some(log) = sim.characters.as_mut() {
                    log.detail(format!("update {}: {}: makeTranslucent {n}: opacity {:.3} -> {opacity:.3} on {changed} part(s), {} own material(s) (rule 22)", sim.updates, p.name, p.opacity_shown, p.own.len()));
                }
                p.opacity_shown = opacity;
            }
        }
        // A character's mouth shape (M4 4d-5, rule 25): lip<n> on its face.
        let mouth = match (&p.source, doors) {
            (Source::Core(i), Some(d)) => d.active.world.anims.get(*i).and_then(|a| a.character.as_deref()).map(|c| c.mouth),
            _ => None,
        };
        if let Some(shape) = mouth.filter(|m| *m != p.mouth_shown && !p.face.is_empty()) {
            let n = if shape > 7 { 0 } else { shape as usize };
            let what = match p.lip_images.get(n).cloned().flatten() {
                Some(img) => {
                    for h in &p.face {
                        if let Some(mut m) = materials.get_mut(h) {
                            m.texture = Some(img.clone());
                        }
                    }
                    format!("lip{n} drawn on {} face material(s)", p.face.len())
                }
                None => format!("lip{n} has no texture: face unchanged"),
            };
            p.mouth_shown = shape;
            if let Some(log) = sim.characters.as_mut() {
                log.detail(format!("update {}: {}: mouth shape {shape}: {what} (rule 25)", sim.updates, p.name));
            }
        }
        // Shared changeBmp/class adapter; originals are restored before a
        // new replacement set, including Reveal off and marked Lava tops.
        let wanted = doors.map_or_else(Vec::new, |d| bitmap_pairs(&d.active.world, &p.name));
        if wanted != p.bitmaps.shown {
            let bindings = p.texture_names.iter().filter_map(|(key, name)| {
                p.new_draw.as_ref()?.materials.get(key).map(|handle| (name.as_str(), handle.clone()))
            });
            if let Some(changed) = p.bitmaps.apply(wanted, bindings, &sim.swap_images, &mut materials) {
                if let Some(nd) = p.new_draw.as_mut() { nd.forget_shown(); }
                sim.log.detail(format!("update {}: {}: bitmap replacements {:?}; {changed} material(s) (characters.md rule4a; secondary-tile-puzzles.md)", sim.updates, p.name, p.bitmaps.shown));
            }
        }
        // M5 5a-8: a zombie rises (shown) or goes back underground.
        if let (Some(shown), Source::Core(i), Some(d)) = (p.birth_shown, &p.source, doors) {
            let up = d.active.world.anims.get(*i).and_then(|a| a.character.as_deref()).is_some_and(|c| !c.unborn);
            if up != shown {
                if let Ok(mut v) = visibility.get_mut(p.body) {
                    *v = if up { Visibility::Inherited } else { Visibility::Hidden };
                }
                p.birth_shown = Some(up);
                if let Some(log) = sim.characters.as_mut() {
                    log.detail(format!("update {}: {}: {} (character-movement.md rules 28, 30)", sim.updates, p.name, if up { "risen: drawn" } else { "underground: not drawn" }));
                }
            }
        }
        let Some(player) = player(p, doors) else { continue };
        let movement = player.current.min(p.movements.len().saturating_sub(1));
        let Some(drawn) = p.movements.get(movement) else { continue };
        // Smooth animation (S-4): the frame and how far toward the next.
        let (frame, mix) = crate::smooth_view::frame_of(player, ahead);
        let frame = frame.min(drawn.frames.saturating_sub(1));
        // A character's repeating movement moves its place, not the
        // drawing (character-movement.md rule 1; M5 5a-2).
        let loop_moves_base = player.steps_move_base();
        // Its place may have changed (a script's `move`, objects.md Q20 d;
        // core keeps it): the body follows (M4 4d-2).
        let base = player.base;
        let moved = base.rotation != p.rotation || base.position != p.position;
        if moved {
            (p.rotation, p.position) = (base.rotation, base.position);
        }
        let same = p.shown == Some((movement, frame)) && !moved && p.new_draw.as_ref().is_none_or(|nd| !nd.stale(movement, frame));
        if same {
            if let (Some(nd), Some(gpu)) = (p.new_draw.as_mut(), gpu.as_deref()) {
                nd.set_amount(&mut commands, gpu, mix);
            }
            // Smooth animation (S-5) glides its place every drawn frame.
            if !smooth {
                continue;
            }
        } else {
            if let Ok(mut c) = carried.get_mut(p.body) {
                (c.movement, c.frame) = (movement, frame);
            }
            p.shown = Some((movement, frame));
            let join = match (p.new_draw.as_mut(), gpu.as_deref()) {
                (Some(nd), Some(gpu)) => nd.show_mixed(&mut commands, gpu, movement, frame, mix, &mut visibility),
                _ => None,
            };
            // Smooth joins (J-3): a change of movement drawn mixed.
            if let (Some(j), Some(now), Some(log)) = (join, self::player(p, doors), sim.characters.as_mut()) {
                let name = |k: usize| now.movements.get(k).map_or("?", |m| m.name.as_str());
                log.detail(format!("update {}: {}: {}", sim.updates, p.name, j.line(name(j.from), name(j.to))));
            }
        }
        let mut pose = if drawn.steps.is_empty() { kq8_core::door::identity() } else { pose_at_frame(&drawn.steps, frame) };
        // Smooth animation (S-5): the frame's steps mixed toward the next
        // frame's (not across a loop's wrap).
        if smooth && mix > 0 && frame + 1 < drawn.steps.len() {
            pose = kq8_core::door::mix_transform(&pose, &pose_at_frame(&drawn.steps, frame + 1), mix as f32 / 255.0);
        }
        if loop_moves_base {
            pose.p = [0.0; 3];
        }
        // Only bodies that move or just stopped moving need a new pose.
        if moved || smooth || !drawn.steps.is_empty() || p.movements.iter().any(|m| !m.steps.is_empty()) {
            let place = p.glide.drawn(update, kq8_core::collide::Placement { rotation: p.rotation, position: p.position }, a);
            let (m, at) = combine(&pose, &place.rotation, place.position);
            if let Ok(mut t) = transforms.get_mut(p.body) {
                *t = bevy_transform_from(m, at);
            }
        }
    }
}

/// `characters.md` rules 36, 37 (M4 4d-7b-2): each character that casts a
/// shadow and is drawn (near or far, not hidden), and Connor (4d-7b-2b),
/// get, every frame, the triangles of their shown model frame thrown along
/// the sun onto the terrain (`kq8_core::shadow`), drawn unlit with multiply
/// blending at the strength 0.5, so each layer halves what is under it and
/// overlaps darken twice (the user's check); no haze. Runs after the
/// transforms are propagated, so it sees this frame's pose. Buildings do
/// not receive them yet (4d-7b-3). Once a second a line per shadow in
/// `characters.log`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shadow_draw(
    mut commands: Commands,
    sim: Option<ResMut<AnimSim>>,
    doors: Option<Res<DoorSim>>,
    connor: Option<Res<crate::connor_view::ConnorSim>>,
    connor_body: Query<(&GlobalTransform, &Visibility), With<crate::connor_view::ConnorBody>>,
    light: Option<Res<ObjectLight>>,
    ground: Option<Res<crate::terrain_view::GroundLookup>>,
    children: Query<&Children>,
    parts: crate::shadow_view::PartQuery,
    visibility: Query<&Visibility>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut connor_shadow: Local<Option<crate::shadow_view::ShadowSlot>>,
    mut frames: Local<u64>,
    (buildings, activity, mut spent, balls, change, mut seen, sources, mask, mut last_mask, shared): (
        Option<Res<crate::shadow_view::BuildingShadows>>,
        Option<Res<crate::draw_distance::ObjectActivity>>,
        Local<std::time::Duration>,
        Query<(&GlobalTransform, &crate::shadow_view::ModelBall)>,
        Res<crate::world_build::WorldChange>,
        Local<u64>,
        Option<Res<crate::shadow_view::FrameSources>>,
        Option<Res<crate::shadow_view::ShadowWorld>>,
        Local<Option<(u32, bool)>>,
        Res<crate::Shared>,
    ),
) {
    use kq8_core::shadow;
    let no_sources = crate::shadow_view::FrameSources::default();
    let sources = sources.as_deref().unwrap_or(&no_sources);
    // A new world (M6 6e-2): Connor's shadow is made again.
    if crate::world_build::new_world(&change, &mut seen) {
        *connor_shadow = None;
        *frames = 0;
        *spent = std::time::Duration::ZERO;
        *last_mask = None;
    }
    let (Some(mut sim), Some(ground)) = (sim, ground) else { return };
    let started = std::time::Instant::now();
    let sim = &mut *sim;
    *frames += 1;
    let report = (*frames).is_multiple_of(60);
    let dir = shadow::direction(light.as_ref().and_then(|l| l.lights.suns.first()).map(|s| s.aim));
    let material = material.get_or_insert_with(|| crate::shadow_view::shadow_material(&mut materials)).clone();
    let mask = mask.as_ref().map_or(1, |m| m.0);
    let allowed = doors.as_ref().is_none_or(|d| d.active.world.shadows_allowed);
    let mask_changed = *last_mask != Some((mask, allowed));
    *last_mask = Some((mask, allowed));
    let mut connor_pieces = 0;
    let mut character_shadows = 0;
    // Rule 37 (2): the near and far buildings receive too (M4 4d-7b-3).
    let active = buildings.as_deref().map(|b| b.active(activity.as_deref())).unwrap_or_default();
    let mut lines = Vec::new();
    // On log frames only: how many of the pieces lie on buildings.
    let line = |name: &str, model: &[[shadow::V3; 3]], on: &crate::shadow_view::Receivers, cast: usize, above: f32| {
        let pick = &on.pick;
        let search = pick.search.map_or("none (no downward sun)".to_string(), |(lo, hi)| format!("{:.0} x {:.0} x {:.0} (own space, z {:.0}..{:.0})", hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2], lo[2], hi[2]));
        format!(
            "update {}: {name}: shadow: {} model triangles, {} facing the sun, {} terrain triangles; buildings: s {:.0} ({}), box {search}, {} of {} active buildings in reach {:?}, {} building surfaces ({} triangles); {cast} pieces drawn, {} of them on buildings; {above:.0} above the drawn ground{} (rule 37)",
            sim.updates,
            model.len(),
            model.iter().filter(|t| shadow::faces_sun(t, dir)).count(),
            on.terrain,
            pick.s,
            if on.fallback { "OUR FALLBACK: box around its points" } else { "its model's stored ball" },
            pick.in_reach.len(),
            active.len(),
            pick.in_reach,
            pick.surfaces,
            on.triangles.len() - on.terrain,
            shadow::cast(model, dir, &on.triangles[on.terrain..]).len(),
            if above.abs() > shadow::TERRAIN_REACH { ": terrain beyond 1,500, none there" } else { "" }
        )
    };
    let height_above = |p: shadow::V3| p[2] - ground.0.height_at(p[0], p[1]).map_or(f32::NAN, |g| g.height);
    // Rule 36 / Q130: Connor's own mask is 1, subject to solid opacity
    // and the world's current mask, while his body is shown.
    if let Some(c) = connor.as_ref() {
        let body = connor_body.iter().next();
        let opacity = doors.as_ref().and_then(|d| d.active.world.connor_translucency).unwrap_or(256.0);
        let actor_mask = if doors.as_ref().is_some_and(|d| d.active.world.first_person) { 0 } else { 1 };
        let drawn = body.is_some_and(|(_, v)| *v != Visibility::Hidden)
            && kq8_data::characters::casts_shadow(mask, actor_mask, allowed, opacity);
        let shown = c.shown;
        let model = match shown.filter(|_| drawn) {
            Some(frame) => crate::shadow_view::frame_triangles(frame, &children, &parts, &meshes, sources),
            None => Vec::new(),
        };
        let at = body.map_or([0.0; 3], |(t, _)| {
            let v = t.translation();
            [v.x * 100.0, -v.z * 100.0, v.y * 100.0]
        });
        let corners: Vec<shadow::V3> = model.iter().flatten().copied().collect();
        let caster = shown.filter(|_| drawn).and_then(|f| balls.get(f).ok()).map(|(at, b)| crate::shadow_view::caster(at, b));
        let on = crate::shadow_view::receivers(&ground.0, at, &corners, dir, caster, &active);
        let cast = shadow::cast(&model, dir, &on.triangles);
        connor_pieces = cast.len();
        if report && drawn {
            lines.push(line("Connor", &model, &on, cast.len(), height_above(at)));
        }
        crate::shadow_view::show_shadow(&cast, &mut connor_shadow, &mut commands, &mut meshes, &material);
    }
    let Some(doors) = doors else { return };
    let world = &doors.active.world;
    for p in &mut sim.playing {
        let Source::Core(i) = p.source else { continue };
        let Some(a) = world.anims.get(i) else { continue };
        if !kq8_data::characters::casts_shadow(mask, p.shadow_mask, allowed, a.translucency.unwrap_or(p.shadow_translucency)) {
            crate::shadow_view::show_shadow(&[], &mut p.shadow, &mut commands, &mut meshes, &material);
            continue;
        }
        let drawn = !a.gone && world.characters_in_range.contains(&p.name.to_ascii_lowercase()) && visibility.get(p.body).is_ok_and(|v| *v != Visibility::Hidden);
        // Its parts shown (step 4: the slots).
        let frame = p.shown.filter(|_| drawn).and(p.new_draw.as_ref().map(|nd| nd.root()));
        let model = match frame {
            Some(frame) => crate::shadow_view::frame_triangles(frame, &children, &parts, &meshes, sources),
            None => Vec::new(),
        };
        let corners: Vec<shadow::V3> = model.iter().flatten().copied().collect();
        let caster = frame.and_then(|f| balls.get(f).ok()).map(|(at, b)| crate::shadow_view::caster(at, b));
        let on = crate::shadow_view::receivers(&ground.0, a.player.base.position, &corners, dir, caster, &active);
        let cast = shadow::cast(&model, dir, &on.triangles);
        character_shadows += usize::from(!cast.is_empty());
        if report && drawn {
            lines.push(line(&p.name, &model, &on, cast.len(), height_above(a.player.base.position)));
        }
        crate::shadow_view::show_shadow(&cast, &mut p.shadow, &mut commands, &mut meshes, &material);
    }
    *spent += started.elapsed();
    if mask_changed {
        shared.log.lock().unwrap().summary(format!("shadow renderer: mask {mask}, Connor {connor_pieces} pieces, {character_shadows} character shadows shown; allowShadows {}", u8::from(allowed)));
    }
    if report {
        lines.push(format!("update {}: shadows of Connor and the characters: {:.3} ms a frame on average (rule 37); {}", sim.updates, spent.as_secs_f64() * 1000.0 / *frames as f64, sources.line()));
    }
    if let Some(log) = sim.characters.as_mut() {
        for l in lines {
            log.detail(l);
        }
    }
}
