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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    speech: Option<kq8_core::talk::SavedTalk>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    speech_sources: Option<std::collections::BTreeMap<u64, Option<[f32; 3]>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conversation: Option<kq8_core::conversation::SavedConversation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    talk_pose: Option<connor_view::SavedTalkPose>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    camera: Option<crate::game_camera::SavedCamera>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    panels: Option<kq8_core::panels::Panels>,
}

impl Snapshot {
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
        if self.traps.as_ref().is_some_and(|s| !s.valid()) { return Err("save has invalid trap state".into()); }
        if self.footstep_types.as_ref().is_some_and(|m| m.len() > 4096 || m.keys().any(|n|
            n.is_empty() || n.len() > 1024 || n.chars().any(char::is_control) || n.to_ascii_lowercase() != *n)) {
            return Err("save has invalid scripted footstep types".into());
        }
        if self.conversation.as_ref().is_some_and(|s| !s.valid()) || self.talk_pose.as_ref().is_some_and(|s| !s.valid())
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
        if self.special.as_ref().is_some_and(|s| !s.valid())
            || (self.special.is_some() && (self.shot.is_some() || self.swing.is_some())) {
            return Err("save has an invalid Special movement".into());
        }
        if self.connor_dying.is_some_and(|dying| dying != self.special.as_ref().is_some_and(|s| s.combat_death())) {
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
            || self.inventory_order >= u64::MAX / 2
            || self.items.iter().any(|o| o.added >= u64::MAX / 2)
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
        // Older slots already retain the explicit hit-death list marker.
        // Health zero alone is alive and must never imply this state.
        active.world.connor_dying = self.connor_dying.unwrap_or_else(|| self.special.as_ref().is_some_and(|s| s.combat_death()));
        if let Some(on) = self.shadows_allowed { active.world.shadows_allowed = on; }
        if let Some(types) = &self.footstep_types { active.world.footstep_types.clone_from(types); }
        if self.version >= 2 {
            if let Some(events) = self.script_events.take() {
                active.world.restore_events(events);
            }
        }
    }

    pub(super) fn restore_current(mut self, world: &mut bevy::prelude::World) -> Result<(), String> {
        if let Some(sources) = &self.speech_sources {
            if let Some(mut speech) = world.get_resource_mut::<crate::talk_view::Speech>() { speech.sources.clone_from(sources); }
        }
        if let Some(mut sim) = world.get_resource_mut::<crate::door_view::DoorSim>() {
            self.restore_objects_and_vars(&mut sim.active);
            if let Some(speech) = &self.speech {
                let now = sim.active.world.now();
                let captions = sim.active.scripts.vars.get("Options::Captions") == Some("true");
                sim.active.world.talk.restore(speech, now, captions)?;
            }
            if self.weapon_state.as_ref().is_some_and(|state| !state.matches_list(crate::door_view::weapon_list(&sim.active.world).as_deref()))
                && !self.conversation_weapon_change_pending() {
                return Err("saved weapon movement list does not match the restored world".into());
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
            }
            if let Some(drains) = &self.drains {
                sim.active.world.restore_drains(drains, self.special.is_some())?;
            }
            if let Some(conversation) = &self.conversation { sim.active.world.restore_conversation(conversation)?; }
        }
        if let Some(mut connor) = world.get_resource_mut::<connor_view::ConnorSim>() {
            connor.put(self.position, self.heading);
            if let Some(swing) = &self.swing {
                connor.restore_swing(swing)?;
            }
            if let Some(shot) = &self.shot {
                connor.restore_shot(shot)?;
            }
            if let Some(timing) = &self.shot_timing {
                connor.restore_shot_timing(timing)?;
            }
            if let Some(timing) = &self.hit_timing {
                connor.restore_hit_timing(timing)?;
            }
            if let Some(special) = &self.special {
                connor.restore_special(special)?;
            }
            if let Some(weapon) = &self.weapon_state {
                connor.restore_weapon_state(weapon, self.special.as_ref())?;
            }
            if let Some(cues) = &self.combat_cues {
                connor.restore_combat_cues(cues)?;
            }
            if let Some(pose) = &self.talk_pose { connor.restore_talk_pose(pose)?; }
        } else if self.swing.is_some() || self.shot.is_some() || self.special.is_some() || self.weapon_state.is_some() || self.combat_cues.is_some() {
            return Err("Connor is unavailable while restoring his attack".into());
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
    let weapon_state = connor.saved_weapon_state(special.as_ref()).ok_or("Connor's weapon state cannot be saved")?;
    let combat_cues = connor.saved_combat_cues().ok_or("Connor's pending combat cues cannot be saved")?;
    let snapshot = Snapshot {
        version: VERSION,
        world: change.current.clone(),
        label: label.trim().into(),
        thumbnail,
        position: connor.mover.position,
        heading: connor.walker.heading,
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
        speech: Some(sim.active.world.talk.saved(sim.active.world.now())),
        speech_sources: world.get_resource::<crate::talk_view::Speech>().map(|s| s.sources.clone()),
        conversation: Some(sim.active.world.saved_conversation()),
        talk_pose: connor.saved_talk_pose(),
        camera: world.get_resource::<crate::game_camera::GameCam>().map(|c| c.saved()),
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
            keep_weapon: false, case: Some(kq8_core::conversation::Case::There), stage: kq8_core::conversation::Stage::Out,
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
            hidden: false,
            script: None,
            do_not_point_at: true,
            added: 44,
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
            weapon_state: None,
            combat_cues: None,
            fight_rng: None,
            bolts: Some(kq8_core::fight::SavedBolts { bolts: Vec::new(), bolts_shot: 0 }),
            drains: Some(Vec::new()),
            shadows_allowed: None,
            footstep_types: None,
            traps: None,
            speech: None,
            speech_sources: None,
            conversation: None,
            talk_pose: None,
            camera: None,
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
