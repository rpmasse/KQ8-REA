//! CPU actor construction shared by presentation and ordered world loading.
//!
//! These constructors consume already resolved placement/section values. They
//! neither register actors nor run scripts, restore saved state, or create GPU
//! resources. Animation indices retain the caller's original order and random
//! animation starts consume the caller's stream exactly once per actor.

use std::collections::BTreeMap;
use std::ops::Range;

use kq8_core::active::{ActiveChest, ActiveDoor, ActiveObject};
use kq8_core::character::Character;
use kq8_core::collide::Placement;
use kq8_core::door::{Cue, Door, DoorMovement};
use kq8_core::emitter::Rng;
use kq8_core::named_anim::NamedAnimPlayer;
use kq8_data::objects::{CurrentLoop, Placement as ObjectPlacement};
use kq8_data::shape::{FrameParts, Transform};

use crate::load::{AnimatedAssets, CharacterAssets, DoorAssets};

/// Borrowed animation data, independent of meshes, images and draw bindings.
/// Doors supply `first_in_file: true` and no parts, as their existing player
/// does not use the character wrap-step/attachment rules.
pub struct AnimationMotion<'a> {
    pub duration: f32,
    pub steps: &'a [Transform],
    pub first_in_file: bool,
    pub parts: &'a [FrameParts],
}

pub fn movements<'a>(
    entries: &[(String, CurrentLoop, usize)],
    mut motion: impl FnMut(usize) -> AnimationMotion<'a>,
) -> Vec<DoorMovement> {
    entries.iter().map(|(name, line, index)| {
        let animation = motion(*index);
        DoorMovement {
            name: name.clone(),
            start: line.start,
            speed: line.speed,
            repeat: line.repeat,
            cut_short: line.unknown != 0,
            play_time: animation.duration,
            frame_steps: animation.steps.to_vec(),
            first_in_file: animation.first_in_file,
            frame_parts: animation.parts.to_vec(),
            cues: line.cues.iter().map(|(phase, action)| Cue {
                phase: *phase, action: action.clone(),
            }).collect(),
            transitions: line.transitions.clone(),
        }
    }).collect()
}

fn placement(p: &ObjectPlacement) -> Placement {
    Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position }
}

pub fn scripted_object(d: &DoorAssets, movements: Vec<DoorMovement>) -> ActiveObject {
    ActiveObject {
        player: NamedAnimPlayer::new(&d.placement.name, movements, d.current, placement(&d.placement))
            .with_stop_names(d.anim_names.0.as_deref(), d.anim_names.1.as_deref())
            .with_anim_pos(d.anim_pos),
        script: d.script.clone(),
        bounds: d.bounds,
        collision_type: d.collider_type,
        collision_mask: d.collider_mask,
        accepts_inventory: d.accepts_inventory.clone(),
        accept_all_items: d.accept_all_items,
        too_far_away_action: d.too_far_away_action,
        do_not_point_at: d.do_not_point_at,
        hidden: !d.placement.drawn(),
        gone: false,
        delete_when_done: false,
    }
}

pub fn chest(
    d: &DoorAssets,
    settings: kq8_core::chest::ChestSettings,
    movements: Vec<DoorMovement>,
) -> ActiveChest {
    let mut chest = kq8_core::chest::Chest::new(
        &d.placement.name, settings, movements, d.current, placement(&d.placement),
    );
    chest.player = chest.player.with_names_at_load(d.anim_names.0.as_deref(), d.anim_names.1.as_deref());
    ActiveChest {
        bounds: d.bounds,
        collision_type: d.collider_type,
        collision_mask: d.collider_mask,
        hidden: !d.placement.drawn(),
        do_not_point_at: d.do_not_point_at,
        script: d.script.clone(),
        chest,
        gone: false,
    }
}

pub fn door(d: &DoorAssets, movements: Vec<DoorMovement>) -> ActiveDoor {
    let mut door = Door::new(&d.placement.name, movements, d.current, placement(&d.placement));
    door.player = door.player.with_names_at_load(d.anim_names.0.as_deref(), d.anim_names.1.as_deref());
    ActiveDoor {
        door,
        script: d.script.clone(),
        bounds: d.bounds,
        collision_type: d.collider_type,
        collision_mask: d.collider_mask,
        accepts_inventory: d.accepts_inventory.clone(),
        accept_all_items: d.accept_all_items,
        too_far_away_action: d.too_far_away_action,
        do_not_point_at: d.do_not_point_at,
        gone: false,
    }
}

pub struct AnimatedPlayer {
    pub player: NamedAnimPlayer,
    /// Selection after authored names/phase, before random start. Presentation
    /// uses this only for its existing load diagnostic.
    pub selected_at_load: usize,
}

pub fn animated_player(
    name: &str,
    at: Placement,
    a: &AnimatedAssets,
    movements: Vec<DoorMovement>,
    ranges: BTreeMap<String, Range<usize>>,
    rng: &mut Rng,
) -> AnimatedPlayer {
    let mut player = NamedAnimPlayer::new(name, movements, a.current, at);
    if let Some(file) = a.character.as_ref().and_then(|c| c.keys.anim_file.as_deref()) {
        player.set_lists(file, ranges);
    }
    player = player.with_stop_names(a.anim_names.0.as_deref(), a.anim_names.1.as_deref())
        .with_anim_pos(a.anim_pos);
    player.once = a.once;
    let selected_at_load = player.current;
    if a.random {
        player = player.with_random_start(rng);
    }
    if let Some(c) = &a.character {
        player.character_steps(c.keys.class);
    }
    AnimatedPlayer { player, selected_at_load }
}

pub fn character(
    p: &ObjectPlacement,
    c: &CharacterAssets,
    player: &NamedAnimPlayer,
    bounds: Option<([f32; 3], [f32; 3])>,
) -> Box<Character> {
    use kq8_data::characters::Class;
    let k = &c.keys;
    let has_birth = player.available().iter().any(|m| m.name.eq_ignore_ascii_case("Birth"));
    let mut ch = Box::new(Character::new(
        k.class == Class::GeometryTalker, k.flags, c.birth_delay, has_birth,
        k.no_turn, k.gravity, Character::seed_for(&p.name),
    ));
    if k.class == Class::Lucreto {
        ch.lucreto_react_pause = k.react_pause;
    }
    ch.lucreto_change_dist = k.lucreto_change_dist;
    ch.flying = k.flying;
    ch.mouth_moves = k.class == Class::Death;
    ch.hits_buildings = k.collider_mask & 0x8 != 0;
    (ch.collision_type, ch.collision_mask) = (k.collider_type, k.collider_mask);
    ch.bounds = bounds;
    let eye = bounds.map_or(1000.0, |(_, max)| max[2]);
    let has_hide = player.available().iter().any(|m| m.name.eq_ignore_ascii_case("hide"));
    ch.unborn = c.unborn;
    let mut ai = kq8_core::character_ai::Ai::new(c.ai.clone(), p.position, eye, has_hide);
    if let Some(keys) = k.tower {
        ch.tower = Some(kq8_core::tower_skeleton::Tower::new(keys));
    }
    ch.fight = c.fight.clone().map(|s| {
        let mut f = kq8_core::fight::Fight::new(s);
        f.combat_scaled = c.combat_scaled;
        f
    });
    ai.has_birth = has_birth;
    ch.ai = Some(ai);
    if let Some((min, max)) = bounds {
        let keep = match p.keep_on_ground {
            kq8_data::objects::KeepOnGround::Floor => -1,
            kq8_data::objects::KeepOnGround::Surface => 1,
            kq8_data::objects::KeepOnGround::Off => 0,
        };
        let step = kq8_core::connor_mover::STEP;
        ch.mover = Some(Box::new(kq8_core::connor_mover::Mover::character(
            p.position, k.gravity, min, max, (k.collider_type, k.collider_mask),
            (k.step_up.unwrap_or(step), k.step_down.unwrap_or(step)), keep,
        )));
    }
    ch
}
