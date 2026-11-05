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
}

impl Snapshot {
    fn validate(&self) -> Result<(), String> {
        if !(1..=VERSION).contains(&self.version) {
            return Err(format!(
                "unsupported save version {} (supported: 1 through {VERSION})",
                self.version
            ));
        }
        if self.version >= 2 && !self.script_events.as_ref().is_some_and(|e| e.valid()) {
            return Err("save has a missing or invalid script event queue".into());
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
        if self.version >= 2 {
            if let Some(events) = self.script_events.take() {
                active.world.restore_events(events);
            }
        }
    }

    pub(super) fn restore_current(mut self, world: &mut bevy::prelude::World) {
        if let Some(mut sim) = world.get_resource_mut::<crate::door_view::DoorSim>() {
            self.restore_objects_and_vars(&mut sim.active);
        }
        if let Some(mut connor) = world.get_resource_mut::<connor_view::ConnorSim>() {
            connor.put(self.position, self.heading);
        }
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
    let current = KeptWorld::take(&sim.active, &change.current, kept.get(&key));
    kept.insert(key, current);
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
                vars: vec![("Swamp::Gate".into(), "opened".into())],
                states: vec![kq8_core::active::KeptState {
                    name: "Gate".into(),
                    movement: "Open".into(),
                    phase: 0.75,
                    waiting: None,
                    placement: kq8_core::collide::Placement::at([1., 2., 3.]),
                    collision_type: Some(32),
                }],
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
