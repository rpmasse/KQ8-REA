//! Disk snapshots of the supported world-return state. Version 1 covers
//! Connor's journey, the active world's inventory and the worlds visited.
//! Version 2 also retains queued script events and end-of-loop listeners.
//! Updated version-2 saves include scripted characters in a Special movement.
//! Live combat and cutscene playback are still partial.
use super::*;
use kq8_core::active::Active;
use kq8_core::item::ItemObject;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::AtomicU64;

const VERSION: u32 = 2;
const MAX_BYTES: usize = 32 * 1024 * 1024;
const THUMB_BYTES: usize = 160 * 120 * 4;

#[derive(Serialize, Deserialize)]
pub(super) struct Snapshot {
    version: u32,
    label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thumbnail: Option<Vec<u8>>,
    world: String,
    pub(super) world_name_start: usize,
    position: [f32; 3],
    heading: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    actor_tilt: Option<[f32; 2]>,
    journey: Journey,
    kept: BTreeMap<String, KeptWorld>,
    items: Vec<ItemObject>,
    inventory_order: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    script_events: Option<kq8_core::events::SavedEvents>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    swing: Option<connor_view::SavedSwing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shot: Option<connor_view::SavedShot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shot_timing: Option<connor_view::SavedShotTiming>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hit_timing: Option<connor_view::SavedHitTiming>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    special: Option<connor_view::SavedSpecial>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    connor_dying: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    combat_death: Option<connor_view::SavedCombatDeath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pending_combat_death: Option<kq8_core::fight::HitKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    weapon_state: Option<connor_view::SavedWeaponState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    combat_cues: Option<connor_view::SavedCombatCues>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fight_rng: Option<kq8_core::emitter::Rng>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bolts: Option<kq8_core::fight::SavedBolts>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    drains: Option<Vec<kq8_core::weeper::SavedDrain>>,
    /// Absent in older slots: retain the loaded world's script default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shadows_allowed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    footstep_types: Option<BTreeMap<String, i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    traps: Option<kq8_core::trap::SavedTraps>,
    /// The start of the next sweep. A script or rope can place Connor
    /// inside a new trap without entering it until he next moves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trap_checked_at: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rope_place: Option<kq8_core::rope::SavedRope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rope_climb: Option<connor_view::SavedClimb>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tile: Option<connor_view::SavedTile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secondary_tile: Option<connor_view::SavedSecondaryTile>,
    /// Recreation bridge effects may remain after their controller exits.
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pending_secondary_calls: Vec<connor_view::SecondaryTileCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rope_camera_locked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    speech: Option<kq8_core::talk::SavedTalk>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    speech_sources: Option<std::collections::BTreeMap<u64, Option<[f32; 3]>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conversation: Option<kq8_core::conversation::SavedConversation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    talk_pose: Option<connor_view::SavedTalkPose>,
    /// Always present in new slots, including an empty slot. Absence alone
    /// identifies legacy Asked snapshots needing one start-message migration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pending_state: Option<connor_view::SavedPendingState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    choice: Option<connor_view::SavedChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    camera: Option<crate::game_camera::SavedCamera>,
    #[serde(default)]
    view_flags: kq8_core::camera_first_person::ModeFlags,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    panels: Option<kq8_core::panels::Panels>,
}

impl Snapshot {
    /// Older builds omitted Tile. At dhalltrp.cs's original starting
    /// point the next letter is known; later positions can be ambiguous.
    fn old_tile_entrance(&self) -> bool {
        self.tile.is_none() && self.world.eq_ignore_ascii_case("DeadCity")
            && self.position[0] == 238_340.0 && self.position[1] == 122_000.0
            && self.camera.as_ref().is_some_and(|camera| camera.has_tile_hold())
            && self.special.is_none() && self.talk_pose.is_none() && self.choice.is_none()
            && self.swing.is_none() && self.shot.is_none()
            && self.combat_death.is_none() && self.rope_climb.is_none()
            && self.kept.get("deadcity").is_some_and(|region| {
                ["DeadCity::DeadHall::TrapOn", "DeadCity::Azriel::metConnor"].iter().all(|key|
                    region.vars.iter().any(|(name, value)| name.eq_ignore_ascii_case(key) && value == "false"))
            })
    }
    pub(super) fn item_load(&self) -> load::ItemLoad {
        load::ItemLoad::Slot {
            objects: self.items.clone(),
            next_name: self.journey.next_no_name,
            order: self.inventory_order,
            first_person_banks: self.weapon_state.as_ref().map(|state| state.first_person_banks().to_vec()).unwrap_or_default(),
        }
    }
    // A conversation finishes `away` in trap_step after weapon synchronization.
    // Keep the old drawn list until the next door_step switches to bare hands.
    fn conversation_weapon_change_pending(&self) -> bool {
        self.journey.weapon_mode == kq8_core::weapons::WeaponMode::None && self.journey.weapon_ready.is_none()
            && self.special.is_none() && self.swing.is_none() && self.shot.is_none()
            && self.conversation.as_ref().is_some_and(|c| c.beginning_bare_hand_out())
            && self.talk_pose.as_ref().is_some_and(|p| p.finished_weapon_away())
            && self.weapon_state.as_ref().is_some_and(|w| w.retains_drawn_list())
    }

    fn validate(&self) -> Result<(), String> {
        if !connor_view::SecondaryTileCall::valid_queue(&self.pending_secondary_calls) {
            return Err("invalid pending secondary tile effects".into());
        }
        if self.secondary_tile.as_ref().is_some_and(|t|!t.valid()) || self.secondary_tile.is_some() && (self.tile.is_some() || self.rope_climb.is_some() || self.choice.is_some() || self.swing.is_some() || self.shot.is_some() || self.talk_pose.is_some() || self.combat_death.is_some()) {
            return Err("invalid secondary tile continuation or competing controller".into());
        }
        if !self.journey.terrain_hazards.valid() {
            return Err("save has invalid terrain sound continuation".into());
        }
        if self.tile.as_ref().is_some_and(|tile| !tile.valid(self.special.is_some())
            || self.special.as_ref().is_some_and(|s| s.combat_death())
            || self.camera.is_none() || self.rope_climb.is_some() || self.choice.is_some()
            || self.swing.is_some() || self.shot.is_some() || self.talk_pose.is_some()
            || self.combat_death.is_some() || self.hit_timing.as_ref().is_some_and(|t| t.has_reaction_pose())) {
            return Err("save has invalid Tile continuation or competing controller poses".into());
        }
        if self.trap_checked_at.is_some_and(|p| !p.into_iter().all(f32::is_finite)) {
            return Err("save has invalid trap sweep position".into());
        }
        if self.rope_place.as_ref().is_some_and(|r| !r.valid()) {
            return Err("save has invalid rope place settings".into());
        }
        if let Some(climb) = &self.rope_climb {
            if !climb.valid() || self.rope_place.is_none() || self.camera.is_none()
                || self.special.is_some() || self.swing.is_some() || self.shot.is_some()
                || self.talk_pose.is_some() || self.choice.is_some() || self.combat_death.is_some()
                || self.hit_timing.as_ref().is_some_and(|t| t.has_reaction_pose()) {
                return Err("save has inconsistent rope climb or movement ownership".into());
            }
        }
        if self.journey.help_history.as_ref().is_some_and(|history| !history.valid()) {
            return Err("save has invalid tutorial flags or duplicate held cases".into());
        }
        if self.choice.as_ref().is_some_and(|choice| !choice.valid() || !choice.compatible(self.special.is_some(),
            self.weapon_state.as_ref().is_some_and(|w| w.swapping()), self.swing.is_some() || self.shot.is_some() || self.talk_pose.is_some())) {
            return Err("save has invalid or unsupported Choice continuation".into());
        }
        if self.traps.as_ref().is_some_and(|s| !s.valid()) { return Err("save has invalid trap state".into()); }
        if self.footstep_types.as_ref().is_some_and(|m| m.len() > 4096 || m.keys().any(|n|
            n.is_empty() || n.len() > 1024 || n.chars().any(char::is_control) || n.to_ascii_lowercase() != *n)) {
            return Err("save has invalid scripted footstep types".into());
        }
        if self.conversation.as_ref().is_some_and(|s| !s.valid()) || self.talk_pose.as_ref().is_some_and(|s| !s.valid())
            || self.pending_state.as_ref().is_some_and(|s| !s.valid())
            || self.camera.as_ref().is_some_and(|s| !s.valid()) || self.panels.as_ref().is_some_and(|s| !s.valid_checkpoint()) {
            return Err("save has invalid conversation, pose, camera or panel state".into());
        }
        if self.speech.as_ref().is_some_and(|s| !s.valid()) { return Err("save has invalid speech queues".into()); }
        if self.speech_sources.as_ref().is_some_and(|sources| sources.len() > 4096 || sources.values().flatten().flatten().any(|v| !v.is_finite())) {
            return Err("save has invalid speech source positions".into());
        }
        if !(1..=VERSION).contains(&self.version) {
            return Err(format!(
                "unsupported save version {} (supported: 1 through {VERSION})",
                self.version
            ));
        }
        if self.version >= 2 && !self.script_events.as_ref().is_some_and(|e| e.valid()) {
            return Err("save has a missing or invalid script event queue".into());
        }
        if self.swing.as_ref().is_some_and(|s| !s.valid()) || (self.swing.is_some() && self.fight_rng.is_none()) {
            return Err("save has an invalid melee swing".into());
        }
        if self.shot.as_ref().is_some_and(|s| !s.valid()) || (self.shot.is_some() && self.swing.is_some()) {
            return Err("save has an invalid ranged shot".into());
        }
        if self.shot_timing.as_ref().is_some_and(|timing| !timing.valid()) {
            return Err("save has invalid moving-shot timing".into());
        }
        if self.hit_timing.as_ref().is_some_and(|timing| !timing.valid()) {
            return Err("save has invalid Connor hit timing".into());
        }
        if self.hit_timing.as_ref().is_some_and(|timing| timing.has_reaction_pose())
            && (self.special.is_some() || self.swing.is_some() || self.shot.is_some() || self.talk_pose.is_some()
                || self.choice.is_some() || self.combat_death.is_some()) {
            return Err("save has competing reaction/controller poses".into());
        }
        if self.special.as_ref().is_some_and(|s| !s.valid())
            || (self.special.is_some() && (self.shot.is_some() || self.swing.is_some())) {
            return Err("save has an invalid Special movement".into());
        }
        let entered_die = self.combat_death.is_some() || self.special.as_ref().is_some_and(|s| s.combat_death());
        if self.connor_dying.is_some_and(|dying| dying != entered_die)
            || entered_die && self.pending_combat_death.is_some()
            || self.combat_death.as_ref().is_some_and(|d| !d.valid() || self.special.is_some()
                || self.swing.is_some() || self.shot.is_some() || self.talk_pose.is_some() || self.choice.is_some()) {
            return Err("save has inconsistent Connor combat-death state".into());
        }
        if self.weapon_state.as_ref().is_some_and(|w| !w.valid(self.special.as_ref()))
            || (self.weapon_state.as_ref().is_some_and(|w| !w.matches_ready(self.journey.weapon_ready.as_deref(), self.journey.weapon_mode))
                && !self.conversation_weapon_change_pending()) {
            return Err("save has an invalid Connor weapon state".into());
        }
        if self.combat_cues.as_ref().is_some_and(|cues| !cues.valid()) {
            return Err("save has invalid pending combat cues".into());
        }
        if self.bolts.as_ref().is_some_and(|b| !b.valid()) {
            return Err("save has invalid live projectiles".into());
        }
        if self.drains.as_ref().is_some_and(|drains| {
            let mut bolts = std::collections::BTreeSet::new();
            drains.len() > 64 || drains.iter().any(|d| !d.valid() || !bolts.insert(d.bolt_name().to_ascii_lowercase()))
        }) {
            return Err("save has invalid Weeper drains".into());
        }
        let known = |name: &str| {
            kq8_data::game_dir::WORLDS
                .iter()
                .any(|w| w.eq_ignore_ascii_case(name))
        };
        if !known(&self.world)
            || self.world != self.journey.from
            || !self.kept.contains_key(&self.world.to_ascii_lowercase())
        {
            return Err("save has an unknown or inconsistent current world".into());
        }
        if self.label.trim().is_empty()
            || self.label.chars().count() > 128
            || self.label.chars().any(char::is_control)
        {
            return Err("save name must contain 1 to 128 printable characters".into());
        }
        if self
            .thumbnail
            .as_ref()
            .is_some_and(|pixels| pixels.len() != THUMB_BYTES)
        {
            return Err("save thumbnail has the wrong dimensions".into());
        }
        if !self.position.iter().all(|p| p.is_finite())
            || !self.heading.is_finite()
            || self.actor_tilt.is_some_and(|tilt| tilt.iter().any(|v| !v.is_finite()))
            || self.inventory_order >= u64::MAX / 2
            || self.items.iter().chain(&self.journey.items).any(|o| o.added >= u64::MAX / 2 || !o.carried_pose.valid())
            || self.journey.next_no_name >= usize::MAX / 2
            || self.world_name_start > self.journey.next_no_name
            || self.journey.potions.iter().any(|(p, t)| {
                p.place.is_some_and(|i| i >= kq8_core::potions::ICON_PLACES) || *t >= u64::MAX / 2
            })
        {
            return Err("save has invalid movement, inventory or potion state".into());
        }
        let safe_path = |s: &str| {
            !s.starts_with(['/', '\\'])
                && !s.contains(':')
                && !s.split(['/', '\\']).any(|p| matches!(p, "." | ".."))
        };
        for (world, kept) in &self.kept {
            if kept.items.as_ref().is_some_and(|items| !items.valid(self.journey.next_no_name)) {
                return Err(format!("save has invalid regional items for {world}"));
            }
            if kept.traps.as_ref().is_some_and(|traps| !traps.valid()) {
                return Err(format!("save has invalid trap state for {world}"));
            }
            if kept.terrain_changes.heights.iter().any(|op| !op.valid())
                || kept.terrain_changes.heights.iter().filter(|op| op.elapsed < op.frames).count() > 32 {
                return Err("save has invalid terrain changes".into());
            }
            if !kept.interactions.valid_loot() {
                return Err("save has inconsistent chest contents".into());
            }
            if !kept.interactions.valid_special_actors() {
                return Err("save has an invalid scripted character movement".into());
            }
            if !kept.interactions.valid_object_stats() {
                return Err("save has invalid ordinary-object combat state".into());
            }
            if !kept.interactions.valid_animations() || !kq8_core::object_checkpoint::valid_copies(&kept.object_copies) {
                return Err("save has invalid ordinary-object identity or animation state".into());
            }
            if kept.spawned_monsters.iter().any(|monster| !monster.valid()) {
                return Err("save has an invalid script-created monster".into());
            }
            if kept.characters.iter().any(|character| !character.valid()) {
                return Err("save has an invalid placed character".into());
            }
            if kept.active_list.as_ref().is_some_and(|list| list.len() > 256 || list.iter().any(|name| name.is_empty() || name.len() > 256 || name != &name.to_ascii_lowercase())) {
                return Err("save has an invalid active-monster list".into());
            }
            if !known(world)
                || *world != world.to_ascii_lowercase()
                || kept.came_in.iter().any(|f| !safe_path(f))
            {
                return Err("save has an invalid kept world or object path".into());
            }
        }
        for (&i, (file, map, _)) in &self.journey.other_maps {
            if i >= kq8_data::magic_map::MAP_COUNT
                || ![file, &map.full_map, &map.blank_map, &map.arrow_pba]
                    .iter()
                    .all(|s| safe_path(s))
                || map
                    .icons
                    .iter()
                    .any(|icon| !safe_path(&icon.file) || !safe_path(&icon.picture))
                || map
                    .teleport_portal
                    .iter()
                    .chain(&map.teleport_script)
                    .any(|p| !safe_path(p))
                || map
                    .runs
                    .iter()
                    .try_fold(0i64, |sum, &n| {
                        sum.checked_add(n).filter(|&s| n >= 0 && s <= 464 * 464)
                    })
                    .is_none()
            {
                return Err("save has an invalid magic map".into());
            }
        }
        Ok(())
    }

    fn restore_objects_and_vars(&mut self, active: &mut Active) {
        active.world.items.restore_objects(
            std::mem::take(&mut self.items),
            self.journey.next_no_name,
            self.inventory_order,
        );
        // On a world change the target's startup variables take priority.
        // A restore continues the same world; every saved value wins.
        active.scripts.vars = Default::default();
        for (name, value) in self
            .journey
            .vars
            .iter()
            .chain(&self.kept[&self.world.to_ascii_lowercase()].vars)
        {
            active.scripts.vars.set(name, value.clone());
        }
        active.world.connor_at = Some(self.position);
        active.world.connor_heading = self.heading;
        if let Some(saved) = &self.rope_place {
            active.world.rope = saved.restore(active.world.now());
            active.world.rope_note(format!("restored rope place: {}", active.world.rope.status.name()));
        }
        if let Some(locked) = self.rope_camera_locked {
            active.world.rope_first_person_lock = locked;
        }
        if let Some(climb) = &self.rope_climb {
            active.world.connor_roping = true;
            active.world.connor_on_rope = climb.on_rope();
        }
        // Older slots already retain the explicit hit-death list marker.
        // Health zero alone is alive and must never imply this state.
        active.world.connor_dying = self.connor_dying.unwrap_or_else(|| self.combat_death.is_some()
            || self.special.as_ref().is_some_and(|s| s.combat_death()));
        active.world.connor_death_entered = active.world.connor_dying;
        active.world.pending_combat_death = self.pending_combat_death;
        if let Some(on) = self.shadows_allowed { active.world.shadows_allowed = on; }
        if let Some(types) = &self.footstep_types { active.world.footstep_types.clone_from(types); }
        if self.version >= 2 {
            if let Some(events) = self.script_events.take() {
                active.world.restore_events(events);
            }
        }
    }

    fn restore_secondary_calls(&mut self, live:&mut Vec<connor_view::SecondaryTileCall>) {
        *live=std::mem::take(&mut self.pending_secondary_calls);
    }

    fn restore_conversation_state(&self, active: &mut Active) -> Result<(), String> {
        if let Some(conversation) = &self.conversation {
            active.world.restore_conversation(conversation)?;
            if self.pending_state.is_none() && conversation.awaiting_admission() {
                active.world.requests.push(kq8_core::active::Request::ConnorConversationStart);
            }
        }
        Ok(())
    }

    pub(super) fn restore_current(mut self, world: &mut bevy::prelude::World) -> Result<(), String> {
        let actor_view = self.weapon_state.as_ref().map_or(self.view_flags.active,
            |state| state.actor_first_person(self.view_flags.active));
        if let Some(sources) = &self.speech_sources {
            if let Some(mut speech) = world.get_resource_mut::<crate::talk_view::Speech>() { speech.sources.clone_from(sources); }
        }
        if let Some(mut sim) = world.get_resource_mut::<crate::door_view::DoorSim>() {
            sim.active.world.restore_view_flags(self.view_flags);
            if self.choice.as_ref().is_some_and(|choice| !sim.active.world.has_script(choice.script())) {
                return Err("saved Choice script is missing from the installed world".into());
            }
            self.restore_objects_and_vars(&mut sim.active);
            if let Some(speech) = &self.speech {
                let now = sim.active.world.now();
                let captions = sim.active.scripts.vars.get("Options::Captions") == Some("true");
                sim.active.world.talk.restore(speech, now, captions)?;
            }
            if let Some(state) = &self.weapon_state {
                let w = &sim.active.world;
                let bank = w.connor_fight.as_ref().and_then(|f| f.bank(actor_view, w.weapon_mode, w.weapon_ready.as_deref()));
                if !state.matches_list(bank) && !self.conversation_weapon_change_pending() {
                    return Err("saved weapon movement list does not match the restored world".into());
                }
            }
            if let Some(rng) = &self.fight_rng { sim.active.world.fight_rng = rng.clone(); }
            if let Some(bolts) = &self.bolts {
                sim.active.world.bolts.clone_from(&bolts.bolts);
                sim.active.world.bolts_shot = bolts.bolts_shot;
            }
            if let Some(kept) = self.kept.get(&self.world.to_ascii_lowercase()) {
                sim.active.world.restore_placed_characters(&kept.characters);
                sim.active.world.restore_spawned_monsters(&kept.spawned_monsters);
                if let Some(list) = kept.active_list.as_deref() { sim.active.world.restore_active_list(list); }
            }
            if let Some(traps) = &self.traps {
                let w = &mut sim.active.world;
                traps.restore(&mut w.traps, &w.known)?;
                sim.connor_checked_at = self.trap_checked_at;
            }
            if let Some(drains) = &self.drains {
                sim.active.world.restore_drains(drains, self.special.is_some() && self.tile.is_none())?;
            }
            self.restore_conversation_state(&mut sim.active)?;
        }
        let tile_stones = if self.tile.is_some() || self.old_tile_entrance() {
            let sim = world.get_resource::<crate::door_view::DoorSim>()
                .ok_or("the world is unavailable while restoring Tile")?;
            let (stones, missing) = kq8_core::tile_puzzle::Stones::read(&|name| {
                sim.active.world.place_of(name).map(|placement| placement.position)
            });
            Some((stones, 64 - missing.len()))
        } else {
            None
        };
        if let Some(mut connor) = world.get_resource_mut::<connor_view::ConnorSim>() {
            connor.first_person = actor_view;
            if let Some([pitch, roll]) = self.actor_tilt {
                connor.placement.angles = [pitch, roll, self.heading];
            }
            connor.put(self.position, self.heading);
            if let Some(weapon) = &self.weapon_state {
                connor.restore_weapon_state(weapon, self.special.as_ref())?;
            }
            if let Some(swing) = &self.swing {
                connor.restore_swing(swing)?;
            }
            if let Some(shot) = &self.shot {
                connor.restore_shot(shot)?;
            }
            if let Some(timing) = &self.shot_timing {
                connor.restore_shot_timing(timing)?;
            }
            if let Some(special) = &self.special {
                connor.restore_special(special)?;
            }
            if let Some(timing) = &self.hit_timing {
                connor.restore_hit_timing(timing)?;
            }
            if let Some(death) = &self.combat_death {
                connor.restore_combat_death(death)?;
            } else if self.special.as_ref().is_some_and(|s| s.combat_death()) {
                connor.walker.animation_input_lock = true;
            }
            if let Some(cues) = &self.combat_cues {
                connor.restore_combat_cues(cues)?;
            }
            if let Some(pose) = &self.talk_pose { connor.restore_talk_pose(pose)?; }
            if let Some(choice) = &self.choice { connor.restore_choice(choice)?; }
            if let Some(tile) = &self.secondary_tile {connor.restore_secondary_tile(tile)?;}
            self.restore_secondary_calls(&mut connor.secondary_calls);
            if let Some(climb) = &self.rope_climb { connor.restore_climb(climb)?; }
            if let Some((stones, found)) = tile_stones {
                if let Some(tile) = &self.tile {
                    connor.restore_tile(tile, stones, found)?;
                } else if found == 64 {
                    connor.restore_old_tile_entrance(stones);
                }
            }
            if let Some(pending) = &self.pending_state { connor.restore_pending_state(pending)?; }
        } else if self.swing.is_some() || self.shot.is_some() || self.special.is_some() || self.combat_death.is_some()
            || self.pending_state.is_some()
            || self.pending_combat_death.is_some() || self.weapon_state.is_some() || self.combat_cues.is_some() || self.choice.is_some()
            || self.hit_timing.as_ref().is_some_and(|timing| timing.has_reaction_pose()) {
            return Err("Connor is unavailable while restoring his attack".into());
        } else if self.rope_climb.is_some() {
            return Err("Connor is unavailable while restoring his rope climb".into());
        } else if self.tile.is_some() || self.secondary_tile.is_some() || !self.pending_secondary_calls.is_empty() {
            return Err("Connor is unavailable while restoring the Tile puzzle".into());
        }
        if let Some(saved) = &self.camera {
            world.get_resource_mut::<crate::game_camera::GameCam>().ok_or("camera unavailable on restore")?.restore(saved)?;
        }
        if let Some(saved) = &self.panels {
            world.get_resource_mut::<crate::gui_view::Gui>().ok_or("HUD unavailable on restore")?.restore_panels(saved)?;
        }
        if let Some(mut sim) = world.get_resource_mut::<crate::door_view::DoorSim>() {
            sim.active.world.queue_trap_lookups("disk restore");
        }
        Ok(())
    }
}

fn directory() -> PathBuf {
    std::env::var_os("KQ8_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("kq8-artifacts/saves"))
}

fn slot_path(dir: &Path, slot: u16) -> Result<PathBuf, String> {
    if slot == 0 {
        return Err("save slot zero is reserved".into());
    }
    Ok(dir.join(format!("slot-{slot}.json")))
}

fn read_slot(dir: &Path, slot: u16) -> Result<Snapshot, String> {
    let path = slot_path(dir, slot)?;
    let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err(format!("invalid save in slot {slot}: exceeds size limit"));
    }
    #[derive(Deserialize)]
    struct Header {
        version: u32,
    }
    let header: Header =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid save in slot {slot}: {e}"))?;
    if !(1..=VERSION).contains(&header.version) {
        return Err(format!(
            "unsupported save version {} (supported: 1 through {VERSION})",
            header.version
        ));
    }
    let snapshot: Snapshot =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid save in slot {slot}: {e}"))?;
    snapshot
        .validate()
        .map_err(|e| format!("invalid save in slot {slot}: {e}"))?;
    Ok(snapshot)
}

fn write_slot(dir: &Path, slot: u16, snapshot: &Snapshot) -> Result<(), String> {
    snapshot.validate()?;
    let path = slot_path(dir, slot)?;
    let bytes = serde_json::to_vec(snapshot).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("save exceeds the size limit".into());
    }
    // Non-finite floats can serialize to JSON null. Ensure the new bytes
    // can actually be restored before replacing an existing good slot.
    serde_json::from_slice::<Snapshot>(&bytes)
        .map_err(|e| format!("cannot round-trip save: {e}"))?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    static TEMP: AtomicU64 = AtomicU64::new(0);
    let (tmp, mut file) = loop {
        let serial = TEMP.fetch_add(1, Ordering::Relaxed);
        let tmp = dir.join(format!(".slot-{slot}-{}-{serial}.tmp", std::process::id()));
        match OpenOptions::new().create_new(true).write(true).open(&tmp) {
            Ok(file) => break (tmp, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    };
    let result = file.write_all(&bytes).and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| fs::rename(&tmp, &path));
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result.map_err(|e| format!("{}: {e}", path.display()))
}

pub fn list_slots() -> Result<Vec<(u16, String)>, String> {
    scan_slots().map(|(slots, _)| slots)
}

pub fn scan_slots() -> Result<(Vec<(u16, String)>, Option<u16>), String> {
    let dir = directory();
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], None)),
        Err(e) => return Err(e.to_string()),
    };
    let mut slots = Vec::new();
    let mut corrupt: Option<u16> = None;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file = entry.file_name();
        let Some(slot) = file.to_str().and_then(|s| {
            s.strip_prefix("slot-")?
                .strip_suffix(".json")?
                .parse::<u16>()
                .ok()
        }) else {
            continue;
        };
        if slot == 0 || file != format!("slot-{slot}.json").as_str() {
            continue;
        }
        let label = match read_slot(&dir, slot) {
            Ok(s) => s.label,
            Err(e) if e.starts_with("unsupported save version") => {
                format!("Unsupported save {slot}")
            }
            Err(e) if e.starts_with("invalid save in slot ") => {
                corrupt = Some(corrupt.map_or(slot, |other| other.min(slot)));
                format!("Unreadable save {slot}")
            }
            Err(_) => format!("Unavailable save {slot}"),
        };
        slots.push((slot, label));
    }
    slots.sort_by_key(|s| s.0);
    Ok((slots, corrupt))
}

pub fn delete_slot(slot: u16) -> Result<(), String> {
    fs::remove_file(slot_path(&directory(), slot)?).map_err(|e| e.to_string())
}

pub fn thumbnail(slot: u16) -> Result<Option<Vec<u8>>, String> {
    Ok(read_slot(&directory(), slot)?.thumbnail)
}

pub fn save_slot_with_thumbnail(
    world: &mut bevy::prelude::World,
    slot: u16,
    label: &str,
    thumbnail: Option<Vec<u8>>,
) -> Result<(), String> {
    let change = world.resource::<WorldChange>();
    if !change.accepts_menu() {
        return Err("no active world ready to save".into());
    }
    let sim = world
        .get_resource::<crate::door_view::DoorSim>()
        .ok_or("world simulation is unavailable")?;
    let connor = world
        .get_resource::<connor_view::ConnorSim>()
        .ok_or("Connor is unavailable")?;
    let mut kept = change.kept.clone();
    let key = change.current.to_ascii_lowercase();
    let current = KeptWorld::take_for_disk_save(&sim.active, &change.current, kept.get(&key));
    kept.insert(key, current);
    let swing = connor.saved_swing();
    if connor.swing.is_some() && swing.is_none() {
        return Err("Connor's current melee swing cannot be saved".into());
    }
    let shot = connor.saved_shot();
    if connor.shot.is_some() && shot.is_none() {
        return Err("Connor's current ranged shot cannot be saved".into());
    }
    let shot_timing = connor.saved_shot_timing().ok_or("Connor's moving-shot timing cannot be saved")?;
    let hit_timing = connor.saved_hit_timing().ok_or("Connor's hit timing cannot be saved")?;
    let special = connor.saved_special();
    let choice = connor.saved_choice();
    if connor.choice.is_some() && choice.is_none() { return Err("Connor's Choice state cannot be saved".into()); }
    let weapon_state = connor.saved_weapon_state(special.as_ref()).ok_or("Connor's weapon state cannot be saved")?;
    let combat_cues = connor.saved_combat_cues().ok_or("Connor's pending combat cues cannot be saved")?;
    let snapshot = Snapshot {
        version: VERSION,
        world: change.current.clone(),
        label: label.trim().into(),
        thumbnail,
        position: connor.mover.position,
        heading: connor.walker.heading,
        actor_tilt: Some([connor.placement.angles[0], connor.placement.angles[1]]),
        world_name_start: change.name_start,
        journey: take_journey(&sim.active, &change.current),
        kept,
        items: sim.active.world.items.objects.clone(),
        inventory_order: sim.active.world.items.inventory_order_counter(),
        script_events: Some(sim.active.world.saved_events()),
        swing,
        shot,
        shot_timing: Some(shot_timing),
        hit_timing: Some(hit_timing),
        special,
        connor_dying: Some(sim.active.world.connor_dying),
        combat_death: connor.saved_combat_death(),
        pending_combat_death: sim.active.world.pending_combat_death,
        weapon_state: Some(weapon_state),
        combat_cues: Some(combat_cues),
        fight_rng: Some(sim.active.world.fight_rng.clone()),
        bolts: Some(kq8_core::fight::SavedBolts {
            bolts: sim.active.world.bolts.clone(),
            bolts_shot: sim.active.world.bolts_shot,
        }),
        drains: Some(sim.active.world.saved_drains()?),
        shadows_allowed: Some(sim.active.world.shadows_allowed),
        footstep_types: Some(sim.active.world.footstep_types.clone()),
        traps: Some(kq8_core::trap::SavedTraps::capture(&sim.active.world.traps)),
        trap_checked_at: sim.connor_checked_at,
        rope_place: Some(kq8_core::rope::SavedRope::capture(&sim.active.world.rope, sim.active.world.now())),
        rope_climb: connor.saved_climb()?,
        tile: connor.saved_tile(),
        secondary_tile: connor.saved_secondary_tile(),
        pending_secondary_calls: connor.secondary_calls.clone(),
        rope_camera_locked: Some(sim.active.world.rope_first_person_lock),
        speech: Some(sim.active.world.talk.saved(sim.active.world.now())),
        speech_sources: world.get_resource::<crate::talk_view::Speech>().map(|s| s.sources.clone()),
        conversation: Some(sim.active.world.saved_conversation()),
        talk_pose: connor.saved_talk_pose(),
        pending_state: Some(connor.saved_pending_state()?),
        choice,
        camera: world.get_resource::<crate::game_camera::GameCam>().map(|c| c.saved()),
        view_flags: sim.active.world.view_flags(),
        panels: world.get_resource::<crate::gui_view::Gui>().map(|g| g.panels.saved()),
    };
    write_slot(&directory(), slot, &snapshot)
}

pub fn load_slot(world: &mut bevy::prelude::World, slot: u16) -> Result<(), String> {
    let save = read_slot(&directory(), slot)?;
    let change = world.resource::<WorldChange>();
    if !matches!(change.step, Step::Playing) || change.asked.is_some() {
        return Err("a world change is in progress".into());
    }
    let game = GameDir::open(change.game().ok_or("game folder unavailable")?)
        .map_err(|e| e.to_string())?;
    for volume in kq8_data::game_dir::VOLUMES {
        game.volume_path(&save.world, volume)
            .ok_or_else(|| format!("{} is missing {volume}.vol", save.world))?;
    }
    let terrain = format!("data/{}/KQ/Terrain/Terrain.kq", save.world);
    game.find(&terrain)
        .ok_or_else(|| format!("missing {terrain}"))?;
    let log = world.resource::<Shared>().log.clone();
    let frames = if change.current.is_empty() {
        0
    } else {
        FREE_FRAMES
    };
    if frames > 0 {
        take_down(world);
    }
    world.remove_resource::<BeginNow>();
    world.resource_mut::<connor_view::StartAt>().0 = None;
    crate::loading_screen::open(world, &save.world, &log);
    let request = Request {
        world: save.world.clone(),
        why: format!("load slot {slot}"),
        at: None,
        portal: None,
    };
    let mut change = world.resource_mut::<WorldChange>();
    change.current.clear();
    change.kept = save.kept.clone();
    change.step = Step::Down {
        request,
        frames,
        from: String::new(),
        journey: Some(Box::new(save.journey.clone())),
    };
    change.restoring = Some(Box::new(save));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_core::item::Place;
    use std::process::Command;

    fn active() -> Active {
        Active::new(Default::default(), vec![], vec![], [], [])
    }

    #[test]
    fn conversation_admission_grouped_snapshot_legacy_migration() {
        let mut original = active();
        original.world.conversation = Some(kq8_core::conversation::Conversation {
            partner: "Connor".into(), partner_at: [0.; 3], info: None,
            lines: kq8_core::talk::Speak { issuer: "Connor".into(), first: kq8_data::msg::Tuple::new(1000, 1, 1, 1, 1), last: 1, nofade: false },
            keep_weapon: false, resume_first_person: false, case: None, stage: kq8_core::conversation::Stage::Asked,
            walk_from: [0.; 3], approach_updates: 0, at: "legacy fixture".into(), cuts_stopped: false,
        });
        let mut saved = fixture();
        saved.conversation = Some(original.world.saved_conversation());
        let mut fresh = active();
        fresh.world.known.insert("connor".into());
        saved.restore_conversation_state(&mut fresh).unwrap();
        assert_eq!(fresh.world.requests.iter().filter(|r| matches!(r, kq8_core::active::Request::ConnorConversationStart)).count(), 1,
            "legacy Asked receives one explicit message when loaded");
        fresh.world.requests.clear();
        let mut walker = kq8_core::connor::Walker::new(0.);
        fresh.conversation_tick(&mut walker, [0.; 3], false);
        assert!(fresh.world.requests.is_empty(), "ordinary frames do not repeat migration");

        // A present empty slot means Talk was replaced. It must not be
        // reconstructed just because the prepared conversation still exists.
        let mut value = serde_json::to_value(&saved).unwrap();
        value["pending_state"] = serde_json::json!({ "request":null, "jump_finished":false, "fall_finished":false,
            "talk_exit_open":false, "preparation_stop":true, "interrupt_override":true, "airborne":null });
        let saved: Snapshot = serde_json::from_value(value).unwrap();
        let mut fresh = active();
        fresh.world.known.insert("connor".into());
        saved.restore_conversation_state(&mut fresh).unwrap();
        assert!(fresh.world.requests.is_empty(), "new empty slot does not revive replaced Talk");
        assert!(saved.pending_state.as_ref().unwrap().valid());
        let copy: Snapshot = serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert!(copy.pending_state.is_some(), "empty-slot presence survives disk encoding");
    }

    #[test]
    fn laying_a_carried_item_back_in_its_region_supersedes_the_old_removal() {
        let mut saved = fixture();
        let mut sim = active();
        let mut item = saved.items[0].clone();
        sim.world.items.restore_objects(vec![item.clone()], 30, item.added);
        let first = KeptWorld::take(&sim, "Swamp", None);
        assert!(first.gone.contains(&item.name.to_ascii_lowercase()));
        item.place = Place::World { position: [33.,45.,12.], angles: [0.;3] };
        sim.world.items.restore_objects(vec![item.clone()], 30, item.added);
        let later = KeptWorld::take(&sim, "Swamp", Some(&first));
        assert!(!later.gone.contains(&item.name.to_ascii_lowercase()));
        saved.kept.insert("swamp".into(), later);
        let json = serde_json::to_value(&saved).unwrap();
        let regional = &json["kept"]["swamp"]["items"]["objects"];
        assert_eq!(regional.as_array().unwrap().len(), 1);
        assert_eq!(regional[0]["name"], item.name);
        saved.validate().unwrap();
        let mut old = json.clone();
        old["kept"]["swamp"].as_object_mut().unwrap().remove("items");
        let old: Snapshot = serde_json::from_value(old).unwrap();
        assert!(old.kept["swamp"].items.is_none());
        old.validate().unwrap();
        let mut bad = json;
        let entries = bad["kept"]["swamp"]["items"]["objects"].as_array_mut().unwrap();
        entries.push(entries[0].clone());
        let bad: Snapshot = serde_json::from_value(bad).unwrap();
        assert!(bad.validate().unwrap_err().contains("invalid regional items for swamp"));
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn departed_dead_city_keeps_spirit_fields_without_replaying_entry_on_return() {
        let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
        let world = load::World::open(&game, "DeadCity").unwrap();
        let mut log = Log::create("kept-spirit-data").unwrap();
        let ground = load::load_ground(&game, &world, &mut log).unwrap();
        let objects = load::load_objects(&game, &world, &ground.terrain, &mut log, None, None);
        let mut traps: Vec<_> = objects.traps.into_iter().filter(|t| t.settings.spirit.is_some()).collect();
        assert_eq!(traps.len(), 4);
        let mut departing = active();
        departing.world.persistent_stat_objects = objects.persistent_stat_objects;
        departing.add_traps(traps.clone());
        let original = &mut departing.world.traps[0];
        let name = original.settings.name.clone();
        original.settings.spirit.as_mut().unwrap().spirit_state = 1;
        original.settings.spirit.as_mut().unwrap().trap_status = 1;
        original.inside.push("Connor".into());
        original.settings.mask = 0;
        let mut transient = original.clone();
        transient.settings.name = "FixtureOutsidePersistManager".into();
        departing.add_traps([transient]);

        let kept = KeptWorld::take(&departing, "DeadCity", None);
        let mut slot = fixture();
        slot.kept.insert("deadcity".into(), kept);
        let json = serde_json::to_value(&slot).unwrap();
        let records = &json["kept"]["deadcity"]["traps"]["states"];
        assert_eq!(records.as_array().unwrap().len(), 4);
        let saved: Snapshot = serde_json::from_value(json.clone()).unwrap();
        saved.validate().unwrap();
        saved.kept["deadcity"].traps.as_ref().unwrap().restore_for_arrival(&mut traps).unwrap();
        let mut returning = Active::new(Default::default(), vec![], vec![], ["Connor".into()], objects.scripts);
        returning.set_connor_stats(kq8_core::stats::ConnorStats {
            level: 2, stats: kq8_core::stats::Stats([20, 10, 0, 0, 0]),
        });
        returning.add_traps(traps);
        returning.start();
        returning.update();
        let restored = returning.world.traps.iter().find(|t| t.settings.name == name).unwrap();
        assert_eq!(restored.inside, ["Connor"]);
        assert_eq!(restored.settings.mask, 0);
        let spirit = restored.settings.spirit.unwrap();
        assert_eq!((spirit.spirit_state, spirit.trap_status, spirit.damage), (1, 1, 0.2));
        assert_eq!(returning.world.connor_stats.unwrap().stats.0[0], 20);
        let mut legacy = json.clone();
        legacy["kept"]["deadcity"].as_object_mut().unwrap().remove("traps");
        let legacy: Snapshot = serde_json::from_value(legacy).unwrap();
        legacy.validate().unwrap();
        assert!(legacy.kept["deadcity"].traps.is_none());
        let mut invalid = json;
        let records = invalid["kept"]["deadcity"]["traps"]["states"].as_array_mut().unwrap();
        records.push(records[0].clone());
        let invalid: Snapshot = serde_json::from_value(invalid).unwrap();
        assert!(invalid.validate().unwrap_err().contains("invalid trap state for deadcity"));
    }

    fn choice_fixture() -> Snapshot {
        let mut json = serde_json::to_value(fixture()).unwrap();
        json["choice"] = serde_json::json!({"state":{"list":{"script":"rooftrap.cs","points":[["0",[220000.,183200.]],["1",[230000.,183200.]]]},
            "circle":[["0",[220000.,183200.]],["1",[236478.,183277.]]],"made":false},"calls":[],
            "pose":{"movement":"left","phase":0.4,"bank":0.,"fresh":false,"talk_play":null,"wanted":null,"auto":null,
                "waiting":null,"playing":null,"hand_mode":4,"hands_off":false,"turn_by_script":false,"turn_press":false,
                "blocked":[false,false],"moving":{"state":"Normal","on_ground":true,"use_gravity":true,"keep_on_ground":true,
                    "fall_timer":0.,"air_updates":0,"pull":[0.,0.,-1.],"submerged":false}}});
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn tile_checkpoint_validates_letter_bounds_and_movement_ownership() {
        let legacy = serde_json::to_value(choice_fixture()).unwrap();
        let mut tile = serde_json::json!({
            "expected":0, "can_exit":false,
            "lines":{"teleported":false,"had_right":false,"sent_back_said":false},
            "stage":"Keys","remembered":0,"lit":null,
            "mode":"Free","pose":legacy["choice"]["pose"],"steps":[325.,325.]
        });
        let valid = |value:&serde_json::Value,has_special| {
            serde_json::from_value::<connor_view::SavedTile>(value.clone()).unwrap().valid(has_special)
        };
        assert!(valid(&tile,false));
        assert!(!valid(&tile,true));
        tile["expected"]=5.into();
        assert!(!valid(&tile,false));
        tile["mode"]="Special".into();
        tile["stage"]=serde_json::json!({"Up":{"right":true}});
        assert!(valid(&tile,true));
        tile["expected"]=0.into();
        tile["stage"]=serde_json::json!({"Jumping":{"from":{"row":-1,"col":4},"to":{"row":0,"col":4}}});
        assert!(valid(&tile,true));
        tile["stage"]["Jumping"]["to"]["row"]=i32::MAX.into();
        assert!(!valid(&tile,true));
    }

    #[test]
    fn rope_climb_rejects_a_mismatched_stage_model_or_controller_pose() {
        use connor_view::SavedClimb;
        let mut pose = serde_json::to_value(choice_fixture()).unwrap()["choice"]["pose"].clone();
        pose["movement"] = "conrope.anm/climb".into();
        let mut json = serde_json::json!({
            "state": {"from":"Bottom", "stage":"Hanging", "place":[1,2,3], "dir":[0,0,0],
                "cam":[null,null], "top_limit":0, "bottom_limit":0, "hand_before":"None",
                "weapon_before":null, "first_to_third":false, "aim":[1,2,3], "range":[4,8],
                "saved_steps":[325,325], "done":false},
            "pose":pose, "steps":[0,0], "direction":0, "was_climbing":false, "model":null, "ended":[]
        });
        let valid = |value: &serde_json::Value| serde_json::from_value::<SavedClimb>(value.clone()).unwrap().valid();
        assert!(valid(&json));
        json["pose"]["movement"] = "stop".into();
        assert!(!valid(&json));
        json["state"]["stage"] = "RopeModel".into();
        json["model"] = serde_json::json!(["conrope.anm/rope",0.1,0.2]);
        assert!(valid(&json));
        json["model"][1] = 0.5.into();
        assert!(!valid(&json));
        json["model"][1] = 0.1.into();
        json["state"]["from"] = "Top".into();
        assert!(!valid(&json));
    }

    #[test]
    fn choice_snapshot_keeps_a_free_pose_and_old_living_slots_remain_readable() {
        let saved = choice_fixture(); saved.validate().unwrap();
        let json = serde_json::to_value(&saved).unwrap();
        let restored: Snapshot = serde_json::from_value(json.clone()).unwrap();
        restored.validate().unwrap();
        assert_eq!(serde_json::to_value(restored.choice.unwrap()).unwrap(), json["choice"]);
        let mut legacy = json; legacy.as_object_mut().unwrap().remove("choice");
        let legacy: Snapshot = serde_json::from_value(legacy).unwrap();
        legacy.validate().unwrap(); assert!(legacy.choice.is_none());
    }

    #[test]
    fn choice_snapshot_rejects_lost_pose_and_invalid_pending_callback() {
        for bad in 0..3 {
            let mut json = serde_json::to_value(choice_fixture()).unwrap();
            match bad {
                0 => json["choice"]["pose"] = serde_json::Value::Null,
                1 => json["choice"]["calls"] = serde_json::json!([["Connor","DoChoice","1"]]),
                _ => json["choice"]["state"]["circle"][0][0] = "99".into(),
            }
            let invalid: Snapshot = serde_json::from_value(json).unwrap();
            assert!(invalid.validate().unwrap_err().contains("Choice"));
        }
    }

    fn combat_death_fixture() -> Snapshot {
        let mut json = serde_json::to_value(fixture()).unwrap();
        json["special"] = serde_json::json!({
            "movement":"firedie", "phase":0.4, "bank":0., "fresh":false,
            "steps":[{"Play":["dropweapon","dropweapon"]},{"Play":["firedie","firedie"]},"Kill"],
            "at":1, "auto":null, "wanted":null, "transition_waiting":null, "transition_playing":null, "death":"firedie"
        });
        json["connor_dying"] = true.into();
        let mut saved: Snapshot = serde_json::from_value(json).unwrap();
        saved.journey.stats = Some(kq8_core::stats::ConnorStats { stats: kq8_core::stats::Stats([-5,10,0,0,0]), level: 1 });
        saved
    }

    #[test]
    fn saved_combat_death_restores_hit_immunity_and_legacy_death_lists() {
        for legacy in [false,true] {
            let mut json = serde_json::to_value(combat_death_fixture()).unwrap();
            if legacy { json.as_object_mut().unwrap().remove("connor_dying"); }
            let mut saved: Snapshot = serde_json::from_value(json).unwrap();
            saved.validate().unwrap();
            let mut restored = active(); restored.set_connor_stats(saved.journey.stats.unwrap());
            restored.world.known.insert("connor".into());
            saved.restore_objects_and_vars(&mut restored);
            assert!(restored.world.connor_dying);
            restored.console_command("sendEvent 0 Connor KQAttackEvent later 0 1000 1", "test");
            restored.update();
            assert_eq!(restored.world.connor_stats.unwrap().stats.0[0], -5);
            assert!(restored.world.requests.iter().all(|r| !matches!(r, kq8_core::active::Request::ConnorDies { .. })));
            assert!(restored.world.fight_log.iter().any(|line| line.contains("hit by later: ignored, he is dying")));
        }
        let mut living = fixture();
        living.journey.stats = Some(kq8_core::stats::ConnorStats { stats: kq8_core::stats::Stats([0,10,0,0,0]), level: 1 });
        living.validate().unwrap();
        let mut restored = active(); restored.set_connor_stats(living.journey.stats.unwrap());
        living.restore_objects_and_vars(&mut restored);
        assert!(!restored.world.connor_dying, "zero health by itself remains alive");
    }

    #[test]
    fn a_combat_death_flag_requires_a_matching_terminal_death_list() {
        for change in 0..4 {
            let mut json = serde_json::to_value(combat_death_fixture()).unwrap();
            match change {
                0 => json["connor_dying"] = false.into(),
                1 => json["special"] = serde_json::Value::Null,
                2 => json["special"]["steps"].as_array_mut().unwrap().truncate(2),
                _ => json["special"]["death"] = "another_movement".into(),
            }
            let invalid: Snapshot = serde_json::from_value(json).unwrap();
            assert!(invalid.validate().unwrap_err().contains("combat-death"));
        }
    }

    #[test]
    fn ordinary_copy_save_validates_identity_and_legacy_slots_default_to_no_copies() {
        use kq8_core::{collide::Placement, object_checkpoint::ObjectCopy};
        let mut save = fixture();
        save.kept.get_mut("swamp").unwrap().object_copies.push(ObjectCopy {
            file: "boardpx.kq".into(), name: "OnceProbe".into(), placement: Placement::at([1., 2., 3.]) });
        save.validate().unwrap();
        let bytes = serde_json::to_vec(&save).unwrap();
        let mut loaded: Snapshot = serde_json::from_slice(&bytes).unwrap();
        loaded.validate().unwrap();
        assert_eq!(loaded.kept["swamp"].object_copies[0].name, "OnceProbe");
        let copy = loaded.kept["swamp"].object_copies[0].clone();
        loaded.kept.get_mut("swamp").unwrap().object_copies.push(copy);
        assert!(loaded.validate().unwrap_err().contains("ordinary-object"));
        let mut json = serde_json::to_value(&save).unwrap();
        json["kept"]["swamp"].as_object_mut().unwrap().remove("object_copies");
        let old: Snapshot = serde_json::from_value(json).unwrap();
        old.validate().unwrap();
        assert!(old.kept["swamp"].object_copies.is_empty());
    }

    #[test]
    fn live_copy_replacement_clears_a_previous_world_removal_before_another_save() {
        use kq8_core::{active::{Template, TemplateKind}, collide::Placement};
        let mut sim = active();
        sim.world.set_templates([("a.kq".into(), Template { name: "Copy".into(), script: None,
            placement: Placement::at([1., 2., 3.]), kind: TemplateKind::Still { bounds: None, collision_type: 0 }, dithered: 0, class: "KQObject".into() })]);
        let mut prior = KeptWorld::take(&sim, "Daventry", None);
        prior.gone.insert("copy".into());
        sim.console_command("loadKQ a.kq", "test");
        let kept = KeptWorld::take_for_disk_save(&sim, "Daventry", Some(&prior));
        assert!(!kept.gone.contains("copy"));
        assert_eq!(kept.object_copies.len(), 1);
        sim.console_command("deleteObject Copy", "test");
        let later = KeptWorld::take(&sim, "Daventry", Some(&kept));
        assert!(later.gone.contains("copy"));
        assert!(later.object_copies.is_empty(), "a removed copy must not be carried forward from an older snapshot");
    }

    #[test]
    fn only_the_completed_conversation_away_accepts_a_pending_weapon_list_change() {
        let mut sim = active();
        sim.world.conversation = Some(kq8_core::conversation::Conversation { partner: "Partner".into(), partner_at: [0.0; 3], info: None,
            lines: kq8_core::talk::Speak { issuer: "Partner".into(), first: kq8_data::msg::Tuple::new(1000, 1, 0, 0, 1), last: 1, nofade: false },
            keep_weapon: false, resume_first_person: false, case: Some(kq8_core::conversation::Case::There), stage: kq8_core::conversation::Stage::Out,
            walk_from: [0.0; 3], approach_updates: 0, at: "test".into(), cuts_stopped: false });
        let mut json = serde_json::to_value(fixture()).unwrap();
        json["conversation"] = serde_json::to_value(sim.world.saved_conversation()).unwrap();
        json["weapon_state"] = serde_json::json!({"weapon_set":"conAxe.anm","weapon_target":"conAxe.anm","pending_set":null,
            "readying":false,"target_mode":"Hand","ready_item":"INVITEM_SmallAx","set_mode":"Hand","held_walk":null});
        json["talk_pose"] = serde_json::json!({"movement":"away","phase":1.0,"bank":0.0,"fresh":false,"talk_play":"out",
            "wanted":null,"auto":null,"waiting":null,"playing":null,"hand_mode":4,"hands_off":false,"turn_by_script":false,
            "turn_press":false,"blocked":[false,false],"moving":{"state":"Normal","on_ground":true,"use_gravity":true,
                "keep_on_ground":true,"fall_timer":0.0,"air_updates":0,"pull":[0.0,0.0,-1.0],"submerged":false}});
        let saved: Snapshot = serde_json::from_value(json.clone()).unwrap();
        assert!(saved.conversation_weapon_change_pending());
        saved.validate().unwrap();
        for change in 0..5 {
            let mut invalid = json.clone();
            match change {
                0 => invalid["conversation"]["active"]["stage"] = "Away".into(),
                1 => invalid["conversation"]["active"]["approach_updates"] = 1.into(),
                2 => invalid["talk_pose"]["phase"] = 0.5.into(),
                3 => invalid["talk_pose"]["hand_mode"] = 0.into(),
                _ => invalid["talk_pose"]["talk_play"] = "stop".into(),
            }
            let invalid: Snapshot = serde_json::from_value(invalid).unwrap();
            assert!(!invalid.conversation_weapon_change_pending());
            assert!(invalid.validate().is_err(), "unrelated weapon mismatch was accepted: {change}");
        }
    }

    #[test]
    fn shadow_switch_restores_without_overriding_an_old_slots_world_default() {
        let mut snapshot = fixture();
        snapshot.shadows_allowed = Some(false);
        let mut snapshot: Snapshot = serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
        let mut fresh = active();
        assert!(fresh.world.shadows_allowed);
        snapshot.restore_objects_and_vars(&mut fresh);
        assert!(!fresh.world.shadows_allowed);
        let mut json = serde_json::to_value(fixture()).unwrap();
        json.as_object_mut().unwrap().remove("shadows_allowed");
        for allowed in [false, true] {
            let mut old: Snapshot = serde_json::from_value(json.clone()).unwrap();
            fresh.world.shadows_allowed = allowed;
            old.restore_objects_and_vars(&mut fresh);
            assert_eq!(fresh.world.shadows_allowed, allowed);
        }
    }

    #[test]
    fn scripted_footstep_types_round_trip_and_old_slots_keep_loaded_defaults() {
        let mut snapshot = fixture();
        snapshot.footstep_types = Some(BTreeMap::from([("temple_e".into(), 3)]));
        let mut restored: Snapshot = serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
        restored.validate().unwrap();
        let mut fresh = active();
        fresh.world.footstep_types.insert("temple_e".into(), 0);
        restored.restore_objects_and_vars(&mut fresh);
        assert_eq!(fresh.world.footstep_types.get("temple_e"), Some(&3));
        let mut old = serde_json::to_value(fixture()).unwrap();
        old.as_object_mut().unwrap().remove("footstep_types");
        let mut old: Snapshot = serde_json::from_value(old).unwrap();
        old.restore_objects_and_vars(&mut fresh);
        assert_eq!(fresh.world.footstep_types.get("temple_e"), Some(&3));
        snapshot.footstep_types = Some(BTreeMap::new());
        snapshot.restore_objects_and_vars(&mut fresh);
        assert!(fresh.world.footstep_types.is_empty());
        snapshot.footstep_types = Some(BTreeMap::from([("Temple_E".into(), 3)]));
        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn secondary_tile_pending_effect_snapshot_dispatch_matrix() {
        use connor_view::SecondaryTileCall as Call;
        use kq8_core::{active::Clickable,collide::Placement,stats::{ConnorStats,Stats}};
        let bind=|name:&str,file:&str| {
            let mut object=Clickable::plain(name,"KQObject",[-1.;3],[1.;3],Placement::at([0.;3]),32);
            object.script=Some(file.into());object
        };
        let active=|| {
            let mut a=Active::new(Default::default(),vec![],vec![bind("Rtswdoor","door.cs")],
                ["Connor".into(),"Rtswdoor".into(),"tile24".into(),"tile43".into(),"tile03".into(),"fiendoor".into()],
                [("door.cs".into(),b"sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 7\n".to_vec()),
                 ("tiletps2.cs".into(),b"set ExitCallback $1\nsendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 11\n".to_vec())]);
            a.set_connor_stats(ConnorStats {level:15,stats:Stats([150,0,0,0,0])});
            a.world.connor_projectile=Some("original.kq".into());
            let keys=kq8_data::magic_map::MapKeys::from_section(&kq8_data::kq_text::parse(b"[barren]\nclassType=KQMap\nnorth=100\nsouth=0\neast=100\nwest=0\nfullMap=full.pbm\nblankMap=blank.pbm\nmapName=#42\n")[0]).unwrap().unwrap();
            a.world.map=Some(kq8_core::magic_map::MapState::new("barren.kq",keys,None,None,None,None));a
        };
        let pose=serde_json::to_value(choice_fixture()).unwrap()["choice"]["pose"].clone();
        let owner=|kind:&str|serde_json::from_value(serde_json::json!({
            "kind":kind,"can_exit":false,"stage":"Keys","remembered":null,"back":false,"highlighted":null,
            "sequence":{"previous":null,"valid":false,"revealed":0,"lower_final":false},"solved":false,"saved_projectile":null,
            "pose":pose.clone(),"mode":"Free","rock":null})).unwrap();
        let cases=[
            (Some("Barren"),vec![Call::Cue("tile24".into(),2400,Some("lvtile19.bmp".into())),Call::Sound("T19.aud".into()),Call::Complete,
                Call::Cue("tile43".into(),2403,None),Call::Cue("tile03".into(),2400,Some("lvtile01.bmp".into()))],50000),
            (Some("Barren"),vec![Call::Script("tiletps2.cs".into())],11),
            // Exit has already removed the owner; cleanup still must survive.
            (None,vec![Call::Line("KQMusic::pop mlava.aud".into()),Call::Cue("tile03".into(),2402,None)],0),
            (Some("Swamp"),vec![Call::RestoreProjectile(Some("bow.kq".into())),Call::Click("Rtswdoor".into()),Call::Cue("fiendoor".into(),1,None)],7),
            (None,vec![Call::RestoreProjectile(None)],0),
        ];
        for (kind,calls,xp) in cases {
            let mut saved=fixture();saved.secondary_tile=kind.map(|k|owner(k));saved.pending_secondary_calls=calls.clone();saved.validate().unwrap();
            let mut restored:Snapshot=serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();restored.validate().unwrap();
            // Rebinding must replace fresh-entry outputs, not replay them.
            let mut live=vec![Call::Line("KQMusic::push mlava.aud".into())];restored.restore_secondary_calls(&mut live);
            assert!(restored.pending_secondary_calls.is_empty());assert_eq!(live,calls);
            let mut a=active();Call::dispatch(&mut live,&mut a);assert!(live.is_empty());
            let mut queue=a.world.queue.clone();let pending:Vec<_>=std::iter::from_fn(||queue.take_due(f64::INFINITY)).collect();
            let cue_targets:Vec<_>=pending.iter().filter(|e|e.kind=="KQCueEvent").map(|e|e.target.as_str()).collect();
            let expected:Vec<_>=calls.iter().filter_map(|c|if let Call::Cue(n,_,_)=c {Some(n.as_str())} else {None}).collect();
            assert_eq!(cue_targets,expected,"bridge commands retain their order");
            let complete=calls.contains(&Call::Complete);
            if complete {
                let reward=pending.iter().position(|e|e.target=="Connor" && e.kind=="KQStatsChangeRequestEvent").unwrap();
                assert_eq!(reward,1,"first reveal precedes reward, then bridge cues");
                assert!(a.world.solved_tile3);assert_eq!(a.scripts.vars.get("Barren::Tile::Complete"),Some("True"));
                let icons=&a.world.map.as_ref().unwrap().icons;assert_eq!(icons.len(),1);assert_eq!((icons[0].x,icons[0].y),(155,177));
                assert_eq!(a.world.sounds.iter().filter(|s|s.name=="T19.aud").count(),1);
            }
            if calls.iter().any(|c|matches!(c,Call::Script(_))) {assert_eq!(a.scripts.vars.get("ExitCallback"),Some("StartExit"));}
            for call in &calls {if let Call::RestoreProjectile(file)=call {assert_eq!(&a.world.connor_projectile,file);}}
            let before=(a.world.queue.len(),a.world.sounds.len(),a.world.map.as_ref().unwrap().icons.len());
            Call::dispatch(&mut live,&mut a);assert_eq!((a.world.queue.len(),a.world.sounds.len(),a.world.map.as_ref().unwrap().icons.len()),before);
            for _ in 0..5 {a.update();}
            assert_eq!(a.world.connor_stats.unwrap().stats.0[4],xp,"reward and callbacks dispatch once");
            assert_eq!(a.world.music.len(),usize::from(calls.iter().any(|c|matches!(c,Call::Line(_)))));
            let mut after=fixture();after.pending_secondary_calls=live;
            let json=serde_json::to_value(after).unwrap();assert!(json.get("pending_secondary_calls").is_none(),"consumed outputs are not saved again");
            let old:Snapshot=serde_json::from_value(json).unwrap();assert!(old.pending_secondary_calls.is_empty());old.validate().unwrap();
        }
        let valid=[Call::Line("KQMusic::push mlava.aud".into()),Call::Script("tiletpn2.cs".into())];assert!(Call::valid_queue(&valid));
        let invalid=[Call::Line("deleteObject Connor".into()),Call::Script("other.cs".into()),Call::Click("other".into()),Call::Sound("T4.aud".into()),
            Call::Cue("tile50".into(),2400,Some("lvtile03.bmp".into())),Call::Cue("tile03".into(),2400,Some("unknown.bmp".into())),
            Call::Cue("tile03".into(),2403,Some("lvtile01.bmp".into())),Call::Cue("fiendoor".into(),2400,None),Call::RestoreProjectile(Some("bad\nname".into()))];
        for call in invalid {let mut s=fixture();s.pending_secondary_calls=vec![call];assert!(s.validate().unwrap_err().contains("pending secondary"));}
        for calls in [vec![Call::Complete,Call::Complete],vec![Call::RestoreProjectile(None);257]] {let mut s=fixture();s.pending_secondary_calls=calls;assert!(s.validate().is_err());}
        assert!(Call::valid_queue(&vec![Call::RestoreProjectile(None);256]));
    }

    fn fixture() -> Snapshot {
        let mut sim = active();
        sim.scripts.vars.set("Daventry::Waterfall", "1".into());
        sim.scripts.vars.set("KQGame::Chapter", "3".into());
        sim.world.items.start_no_names_at(27);
        let item = ItemObject {
            name: "NoName10".into(),
            kind: "INVITEM_Crystal".into(),
            quantity: 2,
            third: -1,
            place: Place::Carried("Connor".into()),
            carried_pose: Default::default(),
            pending_delete: false,
            hidden: false,
            script: None,
            do_not_point_at: true,
            added: 44,
            permanent_startup: false,
        };
        sim.world.items.objects.push(item.clone());
        sim.world.items.objects.push(ItemObject {
            name: "NoName11".into(),
            place: Place::World {
                position: [33., 45., 12.],
                angles: [0.; 3],
            },
            ..item
        });
        sim.world.connor_flags.insert("BeenToCastle".into(), true);
        sim.world.updates = 800;
        sim.world.queue.send(sim.world.now(), 1.4, "bridge", "KQCueEvent", &["2".into()], "switch.cs:7");
        let potion = kq8_core::potions::Running::new("NoName12", "INVITEM_Strength", 60, 800);
        sim.world.potions.push(kq8_core::potions::Running {
            next_tick: 809,
            place: Some(2),
            ..potion
        });
        let mut kept = BTreeMap::new();
        kept.insert("daventry".into(), KeptWorld::take(&sim, "Daventry", None));
        kept.insert(
            "swamp".into(),
            KeptWorld {
                traps: None,
                items: None,
                terrain_changes: Default::default(),
                prop_placements: Default::default(),
                interactions: Default::default(),
                gone: ["usedkey".to_string()].into_iter().collect(),
                came_in: vec!["waterfall.kq".into()],
                object_copies: Vec::new(),
                vars: vec![("Swamp::Gate".into(), "opened".into())],
                states: vec![kq8_core::active::KeptState {
                    name: "Gate".into(),
                    movement: "Open".into(),
                    phase: 0.75,
                    waiting: None,
                    placement: kq8_core::collide::Placement::at([1., 2., 3.]),
                    collision_type: Some(32),
                }],
                spawned_monsters: Vec::new(),
                characters: Vec::new(),
                active_list: None,
            },
        );
        Snapshot {
            version: VERSION,
            world: "Daventry".into(),
            world_name_start: 4,
            label: "After the waterfall".into(),
            position: [43., 56., 8.],
            heading: 0.75,
            actor_tilt: None,
            thumbnail: Some(vec![127; THUMB_BYTES]),
            items: sim.world.items.objects.clone(),
            inventory_order: 47,
            script_events: Some(sim.world.saved_events()),
            swing: None,
            shot: None,
            shot_timing: None,
            hit_timing: None,
            special: None,
            connor_dying: None,
            combat_death: None,
            pending_combat_death: None,
            weapon_state: None,
            combat_cues: None,
            fight_rng: None,
            bolts: Some(kq8_core::fight::SavedBolts { bolts: Vec::new(), bolts_shot: 0 }),
            drains: Some(Vec::new()),
            shadows_allowed: None,
            footstep_types: None,
            traps: None,
            trap_checked_at: None,
            rope_place: None,
            rope_climb: None,
            tile: None,
            secondary_tile: None,
            pending_secondary_calls: Vec::new(),
            rope_camera_locked: None,
            speech: None,
            speech_sources: None,
            conversation: None,
            talk_pose: None,
            pending_state: None,
            choice: None,
            camera: None,
            view_flags: Default::default(),
            panels: None,
            journey: take_journey(&sim, "Daventry"),
            kept,
        }
    }

    #[test]
    fn completed_slot_survives_a_new_process_and_a_failed_overwrite() {
        // Use the test binary's output directory: the two independent
        // processes must see the same path even in isolated Windows temp
        // directories. Each test run gets its own slots.
        let binary = std::env::current_exe().unwrap();
        let dir = binary.parent().unwrap().join(format!(
            "kq8-slottest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut snapshot = fixture();
        write_slot(&dir, 2, &snapshot).unwrap();
        snapshot.label = "Second checkpoint".into();
        write_slot(&dir, 2, &snapshot).unwrap();
        let path = slot_path(&dir, 2).unwrap();
        let before = fs::read(&path).unwrap();
        snapshot.journey.potions[0].0.place = Some(999);
        assert!(write_slot(&dir, 2, &snapshot).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
        fs::write(slot_path(&dir, 3).unwrap(), b"{").unwrap();
        let mut newer: serde_json::Value = serde_json::from_slice(&before).unwrap();
        let mut previous_v2 = newer.clone();
        for world in previous_v2["kept"].as_object_mut().unwrap().values_mut() {
            world.as_object_mut().unwrap().remove("spawned_monsters");
            world.as_object_mut().unwrap().remove("characters");
            world.as_object_mut().unwrap().remove("active_list");
        }
        previous_v2.as_object_mut().unwrap().remove("swing");
        previous_v2.as_object_mut().unwrap().remove("fight_rng");
        previous_v2.as_object_mut().unwrap().remove("bolts");
        previous_v2.as_object_mut().unwrap().remove("drains");
        previous_v2.as_object_mut().unwrap().remove("special");
        previous_v2.as_object_mut().unwrap().remove("shot_timing");
        assert!(serde_json::from_value::<Snapshot>(previous_v2).unwrap().validate().is_ok());
        // Version-1 slots predate the door/chest fields. All previously
        // supported state still loads; these objects use asset defaults.
        for world in newer["kept"].as_object_mut().unwrap().values_mut() {
            world.as_object_mut().unwrap().remove("interactions");
            world.as_object_mut().unwrap().remove("prop_placements");
        }
        newer["version"] = 1.into();
        newer.as_object_mut().unwrap().remove("script_events");
        fs::write(slot_path(&dir, 5).unwrap(), serde_json::to_vec(&newer).unwrap()).unwrap();
        let mut old = read_slot(&dir, 5).unwrap();
        assert!(old.kept.values().all(|w| w.prop_placements.is_empty()));
        assert_eq!(old.label, "Second checkpoint");
        let mut initial = active();
        initial.world.queue.send(0.0, 2.0, "gate", "KQDoorEvent", &[], "Init");
        old.restore_objects_and_vars(&mut initial);
        assert_eq!(initial.world.queue.take_due(2.0).unwrap().target, "gate", "version 1 retains its old initialization behavior");
        newer["version"] = 2.into();
        assert!(serde_json::from_value::<Snapshot>(newer.clone()).unwrap().validate().is_err(), "version 2 requires the queue");
        newer["version"] = 3.into();
        fs::write(
            slot_path(&dir, 4).unwrap(),
            serde_json::to_vec(&newer).unwrap(),
        )
        .unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "world_build::save_slots::tests::read_slot_in_child_process",
                "--ignored",
            ])
            .env("KQ8_SAVE_DIR", &dir)
            .env("KQ8_SAVE_TEST_CHILD", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !dir.join("slot-2.json").exists(),
            "child must have run and deleted the slot"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    #[ignore = "spawned by completed_slot_survives_a_new_process_and_a_failed_overwrite"]
    fn read_slot_in_child_process() {
        if std::env::var_os("KQ8_SAVE_TEST_CHILD").is_none() {
            return;
        }
        assert_eq!(scan_slots().unwrap().1, Some(3));
        assert_eq!(
            list_slots().unwrap(),
            [
                (2, "Second checkpoint".into()),
                (3, "Unreadable save 3".into()),
                (4, "Unsupported save 4".into()),
                (5, "Second checkpoint".into()),
            ]
        );
        assert!(read_slot(&directory(), 4)
            .err()
            .unwrap()
            .contains("unsupported save version 3"));
        assert!(read_slot(&directory(), 3).is_err());
        assert_eq!(thumbnail(2).unwrap(), Some(vec![127; THUMB_BYTES]));
        let mut save = read_slot(&directory(), 2).unwrap();
        assert_eq!(save.world_name_start, 4);
        let swamp = &save.kept["swamp"];
        assert!(swamp.gone.contains("usedkey"));
        assert_eq!(swamp.vars[0].1, "opened");
        assert_eq!(swamp.came_in, ["waterfall.kq"]);
        assert_eq!(
            (swamp.states[0].movement.as_str(), swamp.states[0].phase),
            ("Open", 0.75)
        );
        let visited_again = KeptWorld::take(&active(), "Swamp", Some(swamp));
        assert!(visited_again.gone.contains("usedkey"));
        assert_eq!(visited_again.came_in, ["waterfall.kq"]);
        let mut fresh = active();
        fresh.scripts.vars.set("KQGame::Chapter", "0".into());
        fresh.world.updates = 2;
        save.journey.give(&mut fresh.world, &mut fresh.scripts.vars);
        save.restore_objects_and_vars(&mut fresh);
        let now = fresh.world.now();
        assert!(fresh.world.queue.take_due(now + 1.39).is_none());
        let delayed = fresh.world.queue.take_due(now + 1.4).unwrap();
        assert_eq!((delayed.target.as_str(), delayed.data[0].as_str()), ("bridge", "2"));
        assert_eq!(fresh.scripts.vars.get("KQGame::Chapter"), Some("3"));
        assert_eq!(fresh.scripts.vars.get("Daventry::Waterfall"), Some("1"));
        assert_eq!(fresh.world.connor_at, Some([43., 56., 8.]));
        assert_eq!(fresh.world.connor_heading, 0.75);
        assert!(fresh.world.connor_flags["BeenToCastle"]);
        assert_eq!(fresh.world.potions[0].next_tick, 11);
        assert_eq!(fresh.world.items.objects.len(), 2);
        assert_eq!(fresh.world.items.next_name(), "NoName27");
        assert!(fresh.world.items.objects[1].in_world());
        fresh.world.items.mark_added("NoName11");
        assert_eq!(fresh.world.items.objects[1].added, 48);
        delete_slot(2).unwrap();
        assert!(read_slot(&directory(), 2).is_err());
    }
}
