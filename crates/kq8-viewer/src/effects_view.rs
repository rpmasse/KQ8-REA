//! Emitters in the ground view (M3 step 3b, docs/DESIGN.md; rules:
//! docs/behaviour/emitters.md).
//!
//! - The particles live in `kq8_core::emitter` and move at 15 updates per
//!   second, after the doors and Connor.
//! - Particle catchers run after the emitters in the same update and hide
//!   their emitter's particles inside a building's box (the original's
//!   order is not settled; this one is fine, emitters.md catcher rule 5).
//! - Each particle is drawn as its emitter's `shape` (frame 0) at its
//!   position and turn, only while it is on screen (visible and age not 0),
//!   blended between updates except right after a restart.
//! - Haze (M1 step 6g, object-lighting.md rule 30): all particles of one
//!   emitter share one haze, measured from the emitter's location (`loc`,
//!   which moves for following emitters); each emitter has its own
//!   material copies for that.
//! - Draw distance (M1 step 6h, rules 35-37): an emitter is near / far /
//!   off by its distance from Connor minus 250; when its group bits have
//!   no draw bit, none of its particles is shown.
//!
//! Everything is logged to `kq8-artifacts/logs/effects.log`: each emitter's
//! settings and, for `stream1` and `leaf1` (the note's "How to check"),
//! particle 0's choices, then per update its age, position and visibility,
//! and at each restart how its life compares with its first; each
//! catcher's box and every particle it hides.

use bevy::prelude::*;
use kq8_data::effects::CatcherSettings;
use kq8_core::collide::BuildingCollider;
use kq8_core::emitter::{Catcher, Emitter, Rng};

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::game_camera::GameCam;
use crate::load::EmitterAssets;
use crate::log::Log;
use crate::terrain_view::bevy_transform_from;
use crate::object_light::{for_object, ObjectMaterial};
use crate::Shared;

/// Seed of the particles' random choices: the same every run, so two runs
/// give the same log.
const SEED: u64 = 0x4B51_3820_E417_7E25;

/// Emitters whose particle 0 is followed in the log (emitters.md "How to
/// check"); if a world has none of them, its first emitter.
const TRACKED: [&str; 2] = ["stream1", "leaf1"];

/// Per-update lines for a followed particle stop after this many updates
/// (the restart summaries go on).
const TRACK_UPDATES: u64 = 450;

/// A followed particle: its path in its first full life (by age) and the
/// numbers of the life now running.
struct Tracker {
    emitter: usize,
    first_life: Option<Vec<[f32; 3]>>,
    this_life: Vec<[f32; 3]>,
    lives: u32,
    life_ticks: u32,
    hidden_ticks: u32,
}

/// What a following emitter follows (emitters.md rule 11).
#[derive(Debug, Clone)]
pub enum Target {
    Connor,
    /// The game camera: position only. The original turns the offset by
    /// the camera's turn (emitters.md rule 11 c); no emitter in any world
    /// follows the camera, so that is not done.
    Camera,
    /// A placed object that doesn't move: (position, angles).
    Fixed([f32; 3], [f32; 3]),
    /// A character's current named model part.
    Animated(String),
    /// A projectile's moving origin and turn.
    Bolt(String),
}

/// Finds a following emitter's target by name: what it is, its position
/// and turn now.
pub type TargetLookup<'a> = &'a dyn Fn(&str) -> Option<(Target, [f32; 3], [f32; 3])>;

struct DrawnEmitter {
    emitter: Emitter,
    target: Option<Target>,
    dynamic: bool,
    /// A live copy made by a script's `loadKQ`, rather than a head attack.
    runtime_copy: bool,
    /// One entity per particle (its model's meshes are children).
    entities: Vec<Entity>,
    /// Positions before the last update, for blending.
    previous: Vec<[f32; 3]>,
    /// The emitter's own material copies, and the location their haze was
    /// last measured from (rule 30).
    materials: Vec<Handle<ObjectMaterial>>,
    haze_from: Option<[f32; 3]>,
    /// `tooFar`, its three group masks, and its state now (rules 36, 37).
    too_far: f32,
    groups: [u32; 3],
    activity: Option<kq8_data::objects::Activity>,
}

#[derive(Clone)]
struct PreparedEmitter {
    assets: EmitterAssets,
    parts: Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>,
}

/// A catcher with the index of its emitter, and how many particles it has
/// hidden so far.
struct ActiveCatcher {
    catcher: Catcher,
    emitter: usize,
    hidden: u64,
}

#[derive(Resource)]
pub struct EffectsSim {
    emitters: Vec<DrawnEmitter>,
    prepared: Vec<PreparedEmitter>,
    rng: Rng,
    fire_active: std::collections::BTreeSet<String>,
    bolt_started: std::collections::BTreeSet<String>,
    runtime_started: std::collections::BTreeSet<String>,
    catchers: Vec<ActiveCatcher>,
    trackers: Vec<Tracker>,
    updates: u64,
    log: Log,
}

/// Builds the emitters and their particles' entities. `built` is each
/// object model's (mesh, material) list, indexed like `Objects::shapes`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_emitters(
    commands: &mut Commands,
    emitters: Vec<EmitterAssets>,
    catchers: Vec<(String, CatcherSettings)>,
    buildings: &[BuildingCollider],
    targets: TargetLookup,
    built: &[Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>],
    materials: &mut Assets<ObjectMaterial>,
    main_log: &mut Log,
) {
    let mut log = match Log::create("effects") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("effects: cannot create effects.log: {e}; emitters not started"));
            return;
        }
    };
    let mut rng = Rng::new(SEED);
    let mut drawn = Vec::new();
    let mut prepared = Vec::new();
    let (mut particles, mut not_started) = (0, Vec::new());
    for a in emitters {
        if a.start_on_attack || a.start_on_load || a.start_on_projectile {
            if let Some(shape) = a.shape {
                prepared.push(PreparedEmitter { parts: built[shape].clone(), assets: a });
            } else {
                main_log.summary(format!("effects: prepared emitter {} has no particle model", a.file));
            }
            continue;
        }
        let s = &a.settings;
        let at = |g: Option<f32>| g.map_or("none".to_string(), |g| format!("{g:.0}"));
        let below = a.ground.iter().flatten().any(|g| a.position[2] < *g);
        let ground = format!("floor {} surface {}{}", at(a.ground[0]), at(a.ground[1]), if below { ", BELOW THE GROUND" } else { "" });
        let moved = if a.written != a.position { " CHANGED BY THE PLACEMENT RULES" } else { "" };
        let line = format!(
            "emitter [{}] shape {:?} at ({:.0}, {:.0}, {:.0}) (written ({:.0}, {:.0}, {:.0}){moved}; ground there: {}); radius {:?} force {:?}..{:?} gravity {:?} rot {:?}..{:?} maxSinPeriod {:?} maxLife {} maxParticles {} delayStart {} translucency {}{}",
            a.name,
            s.shape,
            a.position[0],
            a.position[1],
            a.position[2],
            a.written[0],
            a.written[1],
            a.written[2],
            ground,
            s.radius,
            s.force_min,
            s.force_max,
            s.gravity,
            s.rot_min,
            s.rot_max,
            s.max_sin_period,
            s.max_life,
            s.max_particles,
            s.delay_start,
            a.translucency,
            s.follow.as_ref().map_or(String::new(), |f| format!(" follow {f} followMesh {} followOffset {:?}", s.follow_mesh, s.follow_offset))
        );
        main_log.detail(&line);
        log.detail(line);
        let Some(shape) = a.shape else {
            not_started.push(format!("{} (no particle model)", a.name));
            continue;
        };
        // Following (rule 11): Connor, the camera or a placed object. The
        // note doesn't say what happens without a target: not started.
        let (emitter, target) = match &s.follow {
            None => (Emitter::new(&a.name, a.position, a.settings.clone(), &mut rng), None),
            Some(f) => match targets(f) {
                Some((target, at, angles)) => {
                    let e = Emitter::new_following(&a.name, at, angles, a.settings.clone(), &mut rng);
                    let line = format!(
                        "emitter [{}] follows {f} ({target:?}) from ({:.0}, {:.0}, {:.0}) turn {:?}: loc ({:.0}, {:.0}, {:.0}){}",
                        a.name,
                        at[0],
                        at[1],
                        at[2],
                        angles,
                        e.loc[0],
                        e.loc[1],
                        e.loc[2],
                        if s.follow_mesh { "; followMesh: restarts take the target's turn and a new velocity" } else { "" }
                    );
                    main_log.summary(format!("effects: {line}"));
                    log.detail(line);
                    (e, Some(target))
                }
                None => {
                    not_started.push(format!("{} (follows {f}: not in this world)", a.name));
                    continue;
                }
            },
        };
        // The emitter's translucency on every particle (likely, not traced:
        // emitters.md rule 9, open question 2).
        let opacity = if a.translucency >= 256.0 { 1.0 } else { (a.translucency / 255.0).clamp(0.0, 1.0) };
        // Its own material copies (rule 30: haze from the emitter's
        // location), with its b_hazeSolid and hazeMultiplier.
        let parts: Vec<(Handle<Mesh>, Handle<ObjectMaterial>)> = built[shape]
            .iter()
            .map(|(mesh, material)| {
                let material = match materials.get(material).cloned() {
                    Some(m) => materials.add(m),
                    None => material.clone(),
                };
                (mesh.clone(), for_object(materials, &material, opacity, a.haze.0, a.haze.1))
            })
            .collect();
        let mut entities = Vec::with_capacity(emitter.particles.len());
        for p in &emitter.particles {
            let transform = bevy_transform_from(kq8_data::objects::rotation(p.angles), p.position);
            let visibility = if p.drawn() { Visibility::Inherited } else { Visibility::Hidden };
            let parent = commands.spawn((transform, visibility)).id();
            for (mesh, material) in &parts {
                let child = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                commands.entity(parent).add_child(child);
            }
            entities.push(parent);
        }
        particles += emitter.particles.len();
        let previous = emitter.particles.iter().map(|p| p.position).collect();
        let materials = parts.iter().map(|(_, m)| m.clone()).collect();
        drawn.push(DrawnEmitter { emitter, target, dynamic: false, runtime_copy: false, entities, previous, materials, haze_from: None, too_far: a.too_far, groups: a.groups, activity: None });
    }
    // Particle catchers: names looked up once, after the start-up scripts
    // (rule 2); any case. Nothing found: the catcher does nothing.
    let mut active_catchers = Vec::new();
    for (name, c) in catchers {
        let emitter = c.emitter.as_ref().and_then(|n| drawn.iter().position(|d| d.emitter.name.eq_ignore_ascii_case(n)));
        let building = c.interior.as_ref().and_then(|n| buildings.iter().find(|b| b.name.eq_ignore_ascii_case(n)));
        let line = match (emitter, building) {
            (Some(e), Some(b)) => {
                let catcher = Catcher::new(&name, b.bounds.0, b.bounds.1, b.placement);
                let line = format!(
                    "catcher [{name}]: emitter {} in building {} at ({:.0}, {:.0}, {:.0}); box (own space) {:?}..{:?}; top z {:.0}",
                    drawn[e].emitter.name,
                    b.name,
                    b.placement.position[0],
                    b.placement.position[1],
                    b.placement.position[2],
                    b.bounds.0,
                    b.bounds.1,
                    catcher.top
                );
                active_catchers.push(ActiveCatcher { catcher, emitter: e, hidden: 0 });
                line
            }
            _ => format!(
                "catcher [{name}]: does nothing: emitter {:?} {}, building {:?} {}",
                c.emitter,
                if emitter.is_some() { "found" } else { "not running" },
                c.interior,
                if building.is_some() { "found" } else { "not found" }
            ),
        };
        main_log.summary(format!("effects: {line}"));
        log.detail(line);
    }
    // The followed particles.
    let mut tracked: Vec<usize> =
        drawn.iter().enumerate().filter(|(_, d)| TRACKED.iter().any(|t| d.emitter.name.eq_ignore_ascii_case(t))).map(|(k, _)| k).collect();
    if tracked.is_empty() && !drawn.is_empty() {
        tracked.push(0);
    }
    let mut trackers = Vec::new();
    for k in tracked {
        let e = &drawn[k].emitter;
        match e.particles.first() {
            Some(p) => {
                log.detail(format!(
                    "follow [{}] particle 0: offset {:?} velocity {:?} spin {:?} sway speed {:?} starting age {} position {:?} visible {}",
                    e.name, p.offset, p.velocity, p.spin, p.sway_speed, p.age, p.position, p.visible
                ));
                trackers.push(Tracker { emitter: k, first_life: None, this_life: Vec::new(), lives: 0, life_ticks: 0, hidden_ticks: 0 });
            }
            None => log.detail(format!("follow [{}]: no particles", e.name)),
        }
    }
    main_log.summary(format!(
        "effects: {} emitters running, {particles} particles; not started: {not_started:?}; following particle 0 of {:?}; log in {}",
        drawn.len(),
        trackers.iter().map(|t| drawn[t.emitter].emitter.name.clone()).collect::<Vec<_>>(),
        log.path().display()
    ));
    main_log.summary(format!("effects: {} delayed emitter recipes prepared", prepared.len()));
    commands.insert_resource(EffectsSim { emitters: drawn, prepared, rng, fire_active: Default::default(), bolt_started: Default::default(), runtime_started: Default::default(), catchers: active_catchers, trackers, updates: 0, log });
}

/// The current animated part in world space, including the movement's
/// frame turn and the character's placement (as for a dropped weapon).
fn animated_part(actor: &kq8_core::active::AnimObject, part: &str) -> Option<([f32; 3], [f32; 3], usize)> {
    let player = &actor.player;
    let movement = player.movement()?;
    let frame = player.frame();
    let (_, local) = movement.frame_parts.get(frame)?.iter().find(|(name, _)| name.eq_ignore_ascii_case(part))?;
    let base = kq8_data::shape::Transform { flags: 0, m: player.base.rotation, p: player.base.position };
    let owner = kq8_core::door::chain(&base, &movement.turn_at_frame(frame));
    let world = kq8_core::door::chain(&owner, local);
    Some((world.p, kq8_data::objects::angles_of(world.m), frame))
}

/// Begin a prepared effect after its owner enters the world. A script
/// copy and a Dragon attack share the same particle and material rules.
fn start_prepared(
    sim: &mut EffectsSim, commands: &mut Commands, materials: &mut Assets<ObjectMaterial>,
    recipe: &PreparedEmitter, name: &str, target: Option<Target>,
    position: [f32; 3], angles: [f32; 3], runtime_copy: bool, update: u64,
) {
    let a = &recipe.assets;
    let emitter = if target.is_some() {
        Emitter::new_following(name, position, angles, a.settings.clone(), &mut sim.rng)
    } else {
        Emitter::new(name, position, a.settings.clone(), &mut sim.rng)
    };
    let opacity = if a.translucency >= 256.0 { 1.0 } else { (a.translucency / 255.0).clamp(0.0, 1.0) };
    let parts: Vec<_> = recipe.parts.iter().map(|(mesh, material)| {
        let material = materials.get(material).cloned().map(|m| materials.add(m)).unwrap_or_else(|| material.clone());
        (mesh.clone(), for_object(materials, &material, opacity, a.haze.0, a.haze.1))
    }).collect();
    let mut entities = Vec::with_capacity(emitter.particles.len());
    for p in &emitter.particles {
        let transform = bevy_transform_from(kq8_data::objects::rotation(p.angles), p.position);
        let visibility = if p.drawn() { Visibility::Inherited } else { Visibility::Hidden };
        let parent = commands.spawn((transform, visibility)).id();
        for (mesh, material) in &parts {
            let child = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
            commands.entity(parent).add_child(child);
        }
        entities.push(parent);
    }
    sim.log.detail(format!("update {update} [{name}]: started {} from {} with {} particles at {:?}; {} model parts, groups {:?}",
        a.name, a.file, emitter.particles.len(), emitter.loc, parts.len(), a.groups));
    sim.emitters.push(DrawnEmitter { previous: emitter.particles.iter().map(|p| p.position).collect(),
        emitter, target, dynamic: true, runtime_copy, entities,
        materials: parts.iter().map(|(_, m)| m.clone()).collect(), haze_from: None,
        too_far: a.too_far, groups: a.groups, activity: None });
}

/// One update of every emitter (15 per second), particles in list order.
pub fn effects_step(
    sim: Option<ResMut<EffectsSim>>, shared: Res<Shared>, connor: Option<Res<ConnorSim>>,
    camera: Option<Res<GameCam>>, mut door: Option<ResMut<DoorSim>>, mut commands: Commands,
    mut materials: ResMut<Assets<ObjectMaterial>>,
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    // A close target can remove a bolt in its launch update. Process its
    // world-entry notice before looking at the survivors below.
    if let Some(door) = &mut door {
        for spawn in std::mem::take(&mut door.active.world.bolt_effect_spawns) {
            let key = spawn.bolt.to_ascii_lowercase();
            if sim.bolt_started.contains(&key) { continue }
            if let Some(recipe) = sim.prepared.iter()
                .find(|p| p.assets.start_on_projectile && p.assets.file.eq_ignore_ascii_case(&spawn.file)).cloned() {
                let name = format!("{}:{}", spawn.bolt, recipe.assets.name);
                start_prepared(sim, &mut commands, &mut materials, &recipe, &name,
                    Some(Target::Bolt(spawn.bolt.clone())), spawn.position,
                    kq8_data::objects::angles_of(spawn.rotation), false, n);
            } else {
                sim.log.detail(format!("update {n} [{}]: projectile emitter {} was not prepared", spawn.bolt, spawn.file));
            }
            sim.bolt_started.insert(key);
        }
    }
    // The first update that a Dragon head shows attack2 starts a fresh
    // emitter. An overlapping later attack may have its own particles.
    if let Some(door) = &door {
        let mut live_bolts = std::collections::BTreeSet::new();
        let mut bolt_starts = Vec::new();
        for bolt in &door.active.world.bolts {
            let Some(file) = &bolt.emitter else { continue };
            let key = bolt.name.to_ascii_lowercase();
            live_bolts.insert(key.clone());
            if sim.bolt_started.contains(&key) { continue }
            let Some(recipe) = sim.prepared.iter()
                .find(|p| p.assets.start_on_projectile && p.assets.file.eq_ignore_ascii_case(file)).cloned() else {
                sim.log.detail(format!("update {n} [{}]: projectile emitter {file} was not prepared", bolt.name));
                sim.bolt_started.insert(key);
                continue;
            };
            let angles = kq8_data::objects::angles_of(bolt.rotation());
            bolt_starts.push((recipe, bolt.name.clone(), bolt.position, angles));
            sim.bolt_started.insert(key);
        }
        sim.bolt_started.retain(|key| live_bolts.contains(key));
        for (recipe, bolt, position, angles) in bolt_starts {
            let name = format!("{}:{}", bolt, recipe.assets.name);
            start_prepared(sim, &mut commands, &mut materials, &recipe, &name,
                Some(Target::Bolt(bolt)), position, angles, false, n);
        }
        let mut active = std::collections::BTreeSet::new();
        let mut starts = Vec::new();
        for actor in &door.active.world.anims {
            let Some(head) = actor.character.as_ref().and_then(|c| c.dragon_head.as_ref()) else { continue };
            if actor.gone || !actor.player.movement_name().eq_ignore_ascii_case("attack2") { continue }
            let name = actor.player.name.to_ascii_lowercase();
            active.insert(name.clone());
            if sim.fire_active.contains(&name) { continue }
            let Some(file) = &head.fire_file else { continue };
            if let Some(recipe) = sim.prepared.iter().find(|p| p.assets.file.eq_ignore_ascii_case(file)).cloned() {
                starts.push((recipe, name, actor.player.base.position, kq8_data::objects::angles_of(actor.player.base.rotation)));
            } else {
                sim.log.detail(format!("update {n} [{name}]: Dragon fire file {file} was not prepared"));
            }
        }
        sim.fire_active = active;
        for (recipe, target, position, angles) in starts {
            start_prepared(sim, &mut commands, &mut materials, &recipe, &recipe.assets.name,
                Some(Target::Animated(target)), position, angles, false, n);
        }
        // `loadKQ lucfire.kq` creates a normal live copy in core. Its
        // prepared effect begins only after that copy exists.
        let mut runtime_starts = Vec::new();
        for copy in &door.active.world.object_copies {
            if door.active.world.is_deleted(&copy.name) {
                sim.runtime_started.remove(&copy.name.to_ascii_lowercase());
                continue;
            }
            if sim.runtime_started.contains(&copy.name.to_ascii_lowercase()) {
                continue;
            }
            let Some(recipe) = sim.prepared.iter()
                .find(|r| r.assets.start_on_load && r.assets.file.eq_ignore_ascii_case(&copy.file)).cloned() else { continue };
            let (target, position, angles) = if let Some(follow) = &recipe.assets.settings.follow {
                let Some(actor) = door.active.world.anims.iter()
                    .find(|a| !a.gone && a.player.name.eq_ignore_ascii_case(follow)) else { continue };
                (Some(Target::Animated(follow.clone())), actor.player.base.position,
                    kq8_data::objects::angles_of(actor.player.base.rotation))
            } else {
                (None, copy.placement.position, kq8_data::objects::angles_of(copy.placement.rotation))
            };
            runtime_starts.push((recipe, copy.name.clone(), target, position, angles));
        }
        for (recipe, name, target, position, angles) in runtime_starts {
            start_prepared(sim, &mut commands, &mut materials, &recipe, &name,
                target, position, angles, true, n);
            sim.runtime_started.insert(name.to_ascii_lowercase());
        }
    }
    // Explicit deletion removes an object at once. Losing its animated
    // part is different: that begins the emitter's dying phase below.
    let mut i = 0;
    while i < sim.emitters.len() {
        if sim.emitters[i].runtime_copy
            && door.as_ref().is_some_and(|door| door.active.world.is_deleted(&sim.emitters[i].emitter.name)) {
            let removed = sim.emitters.remove(i);
            for entity in removed.entities { commands.entity(entity).despawn(); }
            sim.runtime_started.remove(&removed.emitter.name.to_ascii_lowercase());
            sim.log.detail(format!("update {n} [{}] deleted: particles removed", removed.emitter.name));
        } else {
            i += 1;
        }
    }
    for d in &mut sim.emitters {
        // Following emitters move with their target before the particles do.
        let at = match &d.target {
            None => None,
            Some(Target::Connor) => connor.as_ref().map(|c| (c.mover.position, [0.0, 0.0, c.walker.heading])),
            Some(Target::Camera) => camera.as_ref().map(|c| (c.position(), [0.0; 3])),
            Some(Target::Fixed(p, a)) => Some((*p, *a)),
            Some(Target::Bolt(name)) => {
                let bolt = (!d.emitter.dying).then_some(()).and_then(|_| door.as_ref())
                    .and_then(|door| door.active.world.bolts.iter().find(|b| b.name.eq_ignore_ascii_case(name)));
                match bolt {
                    Some(bolt) => {
                        let angles = kq8_data::objects::angles_of(bolt.rotation());
                        d.emitter.follow_to(bolt.position, angles);
                        if n % 5 == 0 {
                            sim.log.detail(format!("update {n} [{}] follows bolt {name}: loc {:?} turn {:?}, {} particles drawn",
                                d.emitter.name, d.emitter.loc, angles,
                                d.emitter.particles.iter().filter(|p| p.drawn()).count()));
                        }
                    }
                    None if !d.emitter.dying => {
                        d.emitter.begin_dying();
                        sim.log.detail(format!("update {n} [{}] bolt {name} ended: dying", d.emitter.name));
                    }
                    None => {}
                }
                None
            }
            Some(Target::Animated(name)) => {
                let pose = (!d.emitter.dying).then_some(()).and_then(|_| door.as_ref())
                    .and_then(|door| door.active.world.anims.iter().find(|a| !a.gone && a.player.name.eq_ignore_ascii_case(name)))
                    .filter(|a| d.runtime_copy || a.player.movement_name().eq_ignore_ascii_case("attack2"))
                    .and_then(|a| d.emitter.settings.follow_part.as_deref().and_then(|part| animated_part(a, part)));
                match pose {
                    Some((position, angles, frame)) => {
                        d.emitter.follow_part_to(position, angles);
                        if n % 5 == 0 { sim.log.detail(format!("update {n} [{}] follows {name} part {:?} frame {frame}: loc {:?} turn {:?}, {} particles drawn",
                            d.emitter.name, d.emitter.settings.follow_part, d.emitter.loc, angles,
                            d.emitter.particles.iter().filter(|p| p.drawn()).count())); }
                    }
                    None if !d.emitter.dying => {
                        d.emitter.begin_dying();
                        sim.log.detail(format!("update {n} [{}] part or target ended: dying", d.emitter.name));
                    }
                    None => {}
                }
                None
            }
        };
        if let Some((position, angles)) = at {
            d.emitter.follow_to(position, angles);
            if n % 15 == 0 && !matches!(d.target, Some(Target::Fixed(..))) {
                sim.log.detail(format!(
                    "update {n} [{}] follows ({:.0}, {:.0}, {:.0}) heading {:.3}: loc ({:.0}, {:.0}, {:.0})",
                    d.emitter.name, position[0], position[1], position[2], angles[2], d.emitter.loc[0], d.emitter.loc[1], d.emitter.loc[2]
                ));
            }
        }
        d.previous.clear();
        d.previous.extend(d.emitter.particles.iter().map(|p| p.position));
        d.emitter.tick();
    }
    // Catchers after the emitters, in the same update (emitters.md catcher
    // rule 5: the original's order is not settled, this one is fine).
    for c in &mut sim.catchers {
        let e = &mut sim.emitters[c.emitter].emitter;
        for k in c.catcher.catch(e) {
            c.hidden += 1;
            let p = &e.particles[k];
            sim.log.detail(format!(
                "update {n} catcher [{}] hides [{}] particle {k} at ({:.0}, {:.0}, {:.0}), age {} ({} hidden so far)",
                c.catcher.name, e.name, p.position[0], p.position[1], p.position[2], p.age, c.hidden
            ));
        }
    }
    for t in &mut sim.trackers {
        let e = &sim.emitters[t.emitter].emitter;
        let p = &e.particles[0];
        if n <= TRACK_UPDATES {
            sim.log.detail(format!(
                "update {n} [{}] particle 0: age {} position ({:.2}, {:.2}, {:.2}) angles ({:.3}, {:.3}, {:.3}) visible {} drawn {}{}",
                e.name,
                p.age,
                p.position[0],
                p.position[1],
                p.position[2],
                p.angles[0],
                p.angles[1],
                p.angles[2],
                p.visible,
                p.drawn(),
                if p.restarted { " RESTART" } else { "" }
            ));
        }
        t.life_ticks += 1;
        t.hidden_ticks += u32::from(!p.drawn());
        if p.restarted {
            // The life that just ended: complete only if it began with a restart.
            let line = if t.lives == 0 {
                format!("update {n} [{}] particle 0: first restart (its starting life was partial, {} updates)", e.name, t.life_ticks)
            } else {
                let path = std::mem::take(&mut t.this_life);
                let compare = match &t.first_life {
                    None => {
                        t.first_life = Some(path);
                        "first full life recorded".to_string()
                    }
                    Some(first) => {
                        let mut worst = [0.0f32; 3];
                        for (a, b) in first.iter().zip(&path) {
                            for k in 0..3 {
                                worst[k] = worst[k].max((a[k] - b[k]).abs());
                            }
                        }
                        format!("largest difference to the first full life by axis ({:.2}, {:.2}, {:.2})", worst[0], worst[1], worst[2])
                    }
                };
                format!(
                    "update {n} [{}] particle 0: life {} ended: {} updates, {} of them not drawn; {compare}",
                    e.name, t.lives, t.life_ticks, t.hidden_ticks
                )
            };
            shared.log.lock().unwrap().detail(format!("effects: {line}"));
            sim.log.detail(line);
            t.lives += 1;
            t.life_ticks = 0;
            t.hidden_ticks = 0;
            t.this_life.clear();
        } else if t.lives > 0 {
            t.this_life.push(p.position);
        }
    }
    let mut i = 0;
    while i < sim.emitters.len() {
        if sim.emitters[i].dynamic && sim.emitters[i].emitter.done() {
            let finished = sim.emitters.remove(i);
            for entity in finished.entities { commands.entity(entity).despawn(); }
            sim.log.detail(format!("update {n} [{}] all particles expired: removed", finished.emitter.name));
        } else {
            i += 1;
        }
    }
}

/// Each particle where it is, blended between the last two updates (not
/// across a restart), shown only while on screen. The emitter's materials
/// measure the haze from its location (rule 30) whenever it has moved.
pub fn effects_draw(
    sim: Option<ResMut<EffectsSim>>,
    fixed: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &mut Visibility)>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    point: Res<crate::draw_distance::ActivePoint>,
    shared: Res<Shared>,
) {
    use kq8_data::objects::{Activity, DRAW_BITS, EMITTER_RADIUS};
    let Some(mut sim) = sim else { return };
    let a = fixed.overstep_fraction();
    let mut activity_logs = Vec::new();
    for d in &mut sim.emitters {
        // Rules 35-37: radius 250 round the emitter's location.
        let activity = point.at.map(|at| {
            let l = d.emitter.loc;
            let distance = ((0..3).map(|k| (at[k] - l[k]).powi(2)).sum::<f32>()).sqrt() - EMITTER_RADIUS;
            (Activity::of(distance, point.limit, d.too_far), distance + EMITTER_RADIUS)
        });
        if activity.map(|a| a.0) != d.activity {
            if d.dynamic {
                let [always, near, far] = d.groups;
                let draw_groups = activity.is_none_or(|a| a.0.groups(always, near, far) & DRAW_BITS != 0);
                activity_logs.push(format!("draw [{}] activity {:?}, draw groups {}, {} particles currently drawn", d.emitter.name,
                    activity.map(|a| a.0), draw_groups, d.emitter.particles.iter().filter(|p| p.drawn()).count()));
            }
            if let (Some(before), Some((now, distance))) = (d.activity, activity) {
                shared.log.lock().unwrap().detail(format!(
                    "draw distance: emitter [{}] {before:?} -> {now:?}; location {distance:.0} from Connor",
                    d.emitter.name
                ));
            }
            d.activity = activity.map(|a| a.0);
        }
        if d.haze_from != Some(d.emitter.loc) {
            d.haze_from = Some(d.emitter.loc);
            let at = Vec3::from(crate::to_bevy(d.emitter.loc, [0.0; 3]));
            for m in &d.materials {
                if let Some(mut m) = materials.get_mut(m) {
                    m.settings.haze_from = at.extend(1.0);
                }
            }
        }
    }
    for line in activity_logs { sim.log.detail(line); }
    for d in &sim.emitters {
        let [always, near, far] = d.groups;
        let emitter_drawn = d.activity.is_none_or(|a| a.groups(always, near, far) & DRAW_BITS != 0);
        for ((p, e), before) in d.emitter.particles.iter().zip(&d.entities).zip(&d.previous) {
            let Ok((mut t, mut v)) = query.get_mut(*e) else { continue };
            let shown = if p.drawn() && emitter_drawn { Visibility::Inherited } else { Visibility::Hidden };
            if *v != shown {
                *v = shown;
            }
            if !p.drawn() {
                continue;
            }
            let at: [f32; 3] =
                if p.restarted { p.position } else { std::array::from_fn(|k| before[k] + (p.position[k] - before[k]) * a) };
            *t = bevy_transform_from(kq8_data::objects::rotation(p.angles), at);
        }
    }
}
