//! Stateful doors and chests retained when leaving or saving a world.
//! A chest remembers the actual item made on its first click. An empty
//! chest and a door that finished opening must stay in those states.
//! See world-change.md rules 12 and 15, and chests.md rules 7–9.

use crate::active::{ActiveChest, ActiveDoor, WorldState};
use crate::chest::ChestState;
use crate::collide::Placement;
use crate::door::DoorState;
use crate::item::{ItemObject, Items};
use crate::named_anim::NamedAnimPlayer;
use kq8_data::objects::TransitionLine;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeptInteractions {
    doors: BTreeMap<String, KeptDoor>,
    chests: BTreeMap<String, KeptChest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeptDoor {
    state: DoorState,
    animation: Playback,
    collision_type: u32,
    collision_mask: u32,
    do_not_point_at: bool,
    script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeptChest {
    state: ChestState,
    animation: Playback,
    remembered: Option<String>,
    /// The loose item a chest already put out but Connor has yet to take.
    loot: Option<ItemObject>,
    collision_type: u32,
    collision_mask: u32,
    hidden: bool,
    do_not_point_at: bool,
    script: Option<String>,
}

/// Retain the clock and movement choice, without copying a model or
/// its animation data into the saved game.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Playback {
    movement: String,
    requested: String,
    phase: f32,
    waiting: Option<String>,
    playing: bool,
    paused: bool,
    cut_short: bool,
    started: bool,
    placement: Placement,
    random_speed: Option<f32>,
    chosen: Option<TransitionLine>,
    in_transition: Option<TransitionLine>,
}

impl Playback {
    fn take(p: &NamedAnimPlayer) -> Self {
        Self {
            movement: p.movement().map(|m| m.name.clone()).unwrap_or_default(),
            requested: p.current_name.clone(),
            phase: p.phase,
            waiting: p.waiting.clone(),
            playing: p.playing,
            paused: p.paused,
            cut_short: p.cut_short,
            started: p.started_this_update,
            placement: p.base,
            random_speed: p.random_speed,
            chosen: p.chosen.clone(),
            in_transition: p.in_transition.clone(),
        }
    }
    fn restore(&self, p: &mut NamedAnimPlayer) {
        p.current = p
            .movements
            .iter()
            .position(|m| m.name.eq_ignore_ascii_case(&self.movement))
            .unwrap_or(0);
        p.current_name.clone_from(&self.requested);
        p.phase = self.phase;
        p.waiting.clone_from(&self.waiting);
        p.playing = self.playing;
        p.paused = self.paused;
        p.cut_short = self.cut_short;
        p.started_this_update = self.started;
        p.base = self.placement;
        p.random_speed = self.random_speed;
        p.chosen.clone_from(&self.chosen);
        p.in_transition.clone_from(&self.in_transition);
    }
}

impl KeptInteractions {
    /// A remembered loose item belongs to exactly this open chest. Older
    /// slots load the empty default, which has no relationships to check.
    pub fn valid_loot(&self) -> bool {
        let mut remembered = BTreeSet::new();
        self.chests.values().all(|chest| {
            chest.loot.as_ref().is_none_or(|item| {
                chest.state == ChestState::Open
                    && item.in_world()
                    && chest.remembered.as_deref() == Some(&item.name)
                    && remembered.insert(item.name.to_ascii_lowercase())
                    && item.added < u64::MAX / 2
            })
        })
    }

    pub fn take(w: &WorldState) -> Self {
        let doors = w
            .doors
            .iter()
            .filter(|d| !d.gone)
            .map(|d| {
                (
                    d.door.name.to_ascii_lowercase(),
                    KeptDoor {
                        state: d.door.state,
                        animation: Playback::take(&d.door.player),
                        collision_type: d.collision_type,
                        collision_mask: d.collision_mask,
                        do_not_point_at: d.do_not_point_at,
                        script: d.script.clone(),
                    },
                )
            })
            .collect();
        let chests = w
            .chests
            .iter()
            .filter(|c| !c.gone)
            .map(|c| {
                let loot = c
                    .chest
                    .remembered
                    .as_deref()
                    .and_then(|n| w.items.get(n))
                    .filter(|item| item.in_world())
                    .cloned();
                (
                    c.chest.name.to_ascii_lowercase(),
                    KeptChest {
                        state: c.chest.state,
                        animation: Playback::take(&c.chest.player),
                        remembered: c.chest.remembered.clone(),
                        loot,
                        collision_type: c.collision_type,
                        collision_mask: c.collision_mask,
                        hidden: c.hidden,
                        do_not_point_at: c.do_not_point_at,
                        script: c.script.clone(),
                    },
                )
            })
            .collect();
        Self { doors, chests }
    }

    /// Apply before making the renderer and collision objects, so they
    /// start at the correct phase and placement on the first frame.
    pub fn restore_door(&self, d: &mut ActiveDoor) {
        let Some(kept) = self.doors.get(&d.door.name.to_ascii_lowercase()) else {
            return;
        };
        d.door.state = kept.state;
        kept.animation.restore(&mut d.door.player);
        d.collision_type = kept.collision_type;
        d.collision_mask = kept.collision_mask;
        d.do_not_point_at = kept.do_not_point_at;
        d.script.clone_from(&kept.script);
    }

    pub fn restore_chest(&self, c: &mut ActiveChest, items: &mut Items) {
        let Some(kept) = self.chests.get(&c.chest.name.to_ascii_lowercase()) else {
            return;
        };
        c.chest.state = kept.state;
        c.chest.remembered.clone_from(&kept.remembered);
        kept.animation.restore(&mut c.chest.player);
        c.collision_type = kept.collision_type;
        c.collision_mask = kept.collision_mask;
        c.hidden = kept.hidden;
        c.do_not_point_at = kept.do_not_point_at;
        c.script.clone_from(&kept.script);
        if let Some(item) = &kept.loot {
            match items.index(&item.name) {
                Some(i) => items.objects[i] = item.clone(),
                None => items.objects.push(item.clone()),
            }
        }
    }
}
