//! Original-data integration, from secondary-tile-puzzles.md rules 9,
//! 16, 23, 26, 34, 36, 43 and object-projectile-cues.md rules 7–10.
//! Reads the installed data through the production loader; no original
//! executable, GPU pixels or manual gameplay is involved.
use super::*;
use kq8_core::active::{Active, Request};

fn active(objects: &Objects) -> Active {
    let mut active = Active::new(objects.vars.clone(), vec![], objects.clickables.clone(), objects.known.clone(), objects.scripts.clone());
    active.add_traps(objects.traps.clone());
    active
}

fn script(active: &mut Active, file: &str, args: &[&str]) {
    assert!(active.run_script(file, &args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()));
}

fn pending(active: &Active) -> Vec<kq8_core::events::Pending> {
    let mut queue = active.world.queue.clone();
    std::iter::from_fn(|| queue.take_due(f64::INFINITY)).collect()
}

fn pictures(objects: &Objects, wanted: &[&str]) {
    for name in wanted {
        let (_, look) = objects.swap_looks.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("missing replacement recipe {name}"));
        match look {
            Look::Texture { width, height, indexed, .. } => {
                assert!(*width > 0 && *height > 0 && indexed.is_some(), "original indexed image {name}");
            }
            _ => panic!("replacement {name} was not decoded"),
        }
    }
}

fn scripted_entry(active: &Active, position: [f32; 3], heading: f32) {
    assert!(active.world.requests.iter().any(|r| matches!(r, Request::Move { who, position: p, turn, .. }
        if who.eq_ignore_ascii_case("Connor") && *p == position && *turn == [0.0, 0.0, heading])),
        "expected Connor move {position:?} heading {heading}: {}", active.world.trap_log.join("\n"));
    assert!(active.world.requests.iter().any(|r| matches!(r, Request::Move { who, .. } if who.eq_ignore_ascii_case("KQCamera"))));
    assert!(active.world.requests.iter().any(|r| matches!(r, Request::ConnorSecondaryTile { kind: kq8_core::secondary_tiles::Kind::Barren, .. })));
    assert!(active.world.requests.iter().any(|r| matches!(r, Request::HandsOff { on: false, .. })));
}

fn placement_report(objects: &Objects, names: &[&str]) {
    for name in names {
        let (placed, model, _) = objects.placed.iter().find(|(p, _, unused)| !unused && p.name.eq_ignore_ascii_case(name)).unwrap();
        let shape = &objects.shapes[*model].1;
        println!("{} {name}: placed {:?}; model bounds {:?}..{:?}", objects.realm, placed.position, shape.min, shape.max);
        for collider in objects.objects.iter().filter(|c| c.name.eq_ignore_ascii_case(name)) {
            let mut top = f32::NEG_INFINITY;
            for x in [collider.min[0], collider.max[0]] { for y in [collider.min[1], collider.max[1]] { for z in [collider.min[2], collider.max[2]] {
                top = top.max(collider.placement.to_world([x, y, z])[2]);
            } } }
            println!("{} {name}: collision {:?}..{:?}; world top {top}; kind {:?}", objects.realm, collider.min, collider.max, collider.kind);
        }
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR pointing to the original GOG installation"]
fn original_secondary_tile_data_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let mut log = Log::create("secondary-tile-data").unwrap();
    {
        let world = World::open(&game, "Swamp").unwrap();
        let ground = load_ground(&game, &world, &mut log).unwrap();
        let objects = load_objects(&game, &world, &ground.terrain, &mut log, None, None);
        placement_report(&objects, &["tile03", "tile13", "tile02", "tile12"]);
        for trap in objects.traps.iter().filter(|t| t.settings.script.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("tiletrap.cs"))) {
            println!("Swamp Tile2 contact trap {}: position {:?}; local {:?}..{:?}; world {:?}; mask {}", trap.settings.name, trap.settings.position, trap.settings.min, trap.settings.max, trap.full_box(), trap.settings.mask);
        }
        for script in objects.building_clicks.iter().filter(|b| b.script.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("tiletrap.cs"))) {
            let bounds = objects.buildings.iter().find(|b| b.name.eq_ignore_ascii_case(&script.name)).map(|b| (b.world_min, b.world_max));
            println!("Swamp Tile2 scripted building {}: position {:?}; local {:?}; world {:?}; collision type {}", script.name, script.placement.position, script.bounds, bounds, script.collision_type);
        }
        for (file, bytes) in &objects.scripts {
            for (line, text) in String::from_utf8_lossy(bytes).lines().enumerate() {
                let words: Vec<_> = text.split('#').next().unwrap_or("").split_whitespace().collect();
                if words.len() == 3 && words[0].eq_ignore_ascii_case("set") && words[1].eq_ignore_ascii_case("Swamp::Tile::SwitchOn") && words[2].eq_ignore_ascii_case("true") {
                    println!("Swamp authored switch: {file}:{}: {}", line + 1, text.trim());
                }
            }
        }
        assert_eq!(objects.swamp_tiles.len(), 42);
        assert_eq!(objects.swamp_tiles.values().filter(|tile| tile.has_script).count(), 12);
        for col in 0..6 {
            for row in 0..7 {
                let name = format!("tile{col}{row}");
                let tile = &objects.swamp_tiles[&name];
                assert_eq!(tile.damage_factor, 0.48_f32);
                assert_eq!(tile.last_contact, 0.0);
                assert!(!tile.revealed);
                assert!(matches!(tile.height_num, 1..=3 | 12..=13));
                if tile.has_script {
                    let clickable = objects.clickables.iter().find(|c| c.name.eq_ignore_ascii_case(&name)).unwrap();
                    assert!(clickable.script.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("fiensafe.cs")), "{name}: safe callback not registered");
                }
                let (placed, model, unused) = objects.placed.iter().find(|(p, _, unused)| !unused && p.name.eq_ignore_ascii_case(&name)).unwrap();
                assert!(!unused);
                assert_eq!(placed.class.as_deref(), Some(kq8_core::tile_objects::SWAMP_CLASS));
                assert_eq!(placed.position[0], 50000.0 + col as f32 * 2000.0);
                assert_eq!(placed.position[1], 92600.0 + row as f32 * 2000.0);
                assert_eq!(placed.keep_on_ground, kq8_data::objects::KeepOnGround::Off);
                let texture_names: Vec<_> = objects.shapes[*model].1.groups.iter().filter_map(|g| match &g.look {Look::Texture {name, ..} => Some(name.rsplit(['/', '\\']).next().unwrap_or(name)), _ => None}).collect();
                let names = if tile.height_num < 10 { ["gastrap.bmp", "wtower5.bmp"] } else { ["trapring.bmp", "trunk.bmp"] };
                for original in names { assert!(texture_names.iter().any(|n| n.eq_ignore_ascii_case(original)), "{name}: missing original {original}: {texture_names:?}"); }
            }
        }
        pictures(&objects, &kq8_core::tile_objects::REVEAL_BITMAPS);
        for file in ["swmpgas.kqs", "endfrbl.kqs"] {
            let (index, movement) = objects.script_shapes.get(file).unwrap_or_else(|| panic!("missing one-shot recipe {file}"));
            let model = &objects.object_animations[*index];
            assert!(model.source.is_some() && model.frame_count > 1 && model.duration > 0.0);
            assert_eq!((movement.start, movement.speed, movement.repeat), (0.0, 1.0, false));
        }
        let shooters: Vec<_> = objects.object_projectiles.iter().filter(|(_, s)| s.file.eq_ignore_ascii_case("PRfire2.kq")).collect();
        assert_eq!(shooters.len(), 6);
        for (name, shooter) in shooters {
            assert_eq!((shooter.offset, shooter.aim, shooter.damage), ([0.0, 0.0, 1075.0], [1000.0, 0.0, 0.0], 20), "{name}");
            assert!(objects.placed.iter().any(|(p, _, unused)| !unused && p.name.eq_ignore_ascii_case(name) && p.class.as_deref() == Some("KQObject")));
        }
        let projectile = &objects.projectiles["prfire2.kq"];
        assert_eq!((projectile.speed, projectile.max_distance, projectile.weapon_type), (9000.0, 21000.0, Some(2)));
        assert_eq!(projectile.shape.as_deref(), Some("swmpfrbl.kqs"));
        assert_eq!(projectile.script.as_deref(), Some("swmpfrbl.cs"));
        assert!(objects.corpse_shapes.contains_key("swmpfrbl.kqs"));
        let shared = std::fs::read(game.find("GAME/Kq/swmpfrbl.cs").unwrap()).unwrap();
        let loaded = objects.scripts.iter().find(|(name, _)| name.eq_ignore_ascii_case("swmpfrbl.cs")).unwrap();
        assert_eq!(loaded.1, shared, "projectile impact script comes from the installed shared file");
        println!("Swamp original data: 42 tiles, 12 scripted; both original texture families and four Reveal images; gas/impact recipes and six configured fireball shooters");
    }
    {
        let world = World::open(&game, "Barren").unwrap();
        let ground = load_ground(&game, &world, &mut log).unwrap();
        let objects = load_objects(&game, &world, &ground.terrain, &mut log, None, None);
        placement_report(&objects, &["tile03"]);
        assert_eq!(objects.lava_tiles.len(), 35);
        for row in 0..5 {
            for col in 0..7 {
                let name = format!("tile{row}{col}");
                let tile = &objects.lava_tiles[&name];
                assert_eq!(*tile, kq8_core::tile_objects::LavaTile::default());
                let actor = objects.animated.iter().find(|a| objects.placed[a.placed].0.name.eq_ignore_ascii_case(&name)).unwrap();
                let placed = &objects.placed[actor.placed].0;
                assert_eq!(placed.position, [86640.0 + col as f32 * 2000.0, 151943.0 + row as f32 * 2000.0, -700.0]);
                assert!(placed.hidden);
                assert_eq!(placed.keep_on_ground, kq8_data::objects::KeepOnGround::Off);
                assert_eq!(actor.movements.len(), 8);
                assert!(actor.movements[actor.current].0.eq_ignore_ascii_case("stop"));
                for motion in ["stop", "up", "stopup", "down", "stop1", "up1", "stopup1", "down1"] {
                    let (_, _, index) = actor.movements.iter().find(|(n, _, _)| n.eq_ignore_ascii_case(motion)).unwrap();
                    let model = &objects.object_animations[*index];
                    assert!(model.source.is_some() && model.frame_count > 0);
                    if motion.starts_with("up") { assert_eq!(model.frame_count, 31); assert!((model.duration - 2.0).abs() < 0.0001); }
                    if motion.starts_with("down") { assert_eq!(model.frame_count, 51); assert!((model.duration - 3.3333).abs() < 0.0001); }
                }
            }
        }
        pictures(&objects, &kq8_core::secondary_tiles::LAVA_BITMAPS);
        for file in ["tiletps.cs", "tiletpn.cs", "tiletps2.cs", "tiletpn2.cs"] {
            let (label, source) = world.load(file).unwrap().unwrap();
            assert!(label.to_ascii_lowercase().contains("/patch/"), "{file}: {label}");
            let installed = std::fs::read(game.find(&format!("Barren/patch/{file}")).unwrap()).unwrap();
            assert_eq!(source, installed);
            assert_eq!(objects.scripts.iter().find(|(n, _)| n.eq_ignore_ascii_case(file)).unwrap().1, installed);
        }
        let mut south = active(&objects);
        south.scripts.vars.set("Barren::Tile::Complete", "false".into());
        south.scripts.vars.set("Barren::Tile::SouthBridge", "On".into());
        script(&mut south, "tiletps.cs", &["tiletps", "On"]);
        let cues: Vec<_> = pending(&south).into_iter().filter(|e| e.kind.eq_ignore_ascii_case("KQCueEvent") && e.data.first().is_some_and(|s| s == "2403")).collect();
        assert_eq!(cues.len(), 32);
        assert!((cues.first().unwrap().due - 0.7).abs() < 0.00001 && (cues.last().unwrap().due - 11.4).abs() < 0.00001);
        for omitted in ["tile42", "tile43", "tile44"] { assert!(!cues.iter().any(|e| e.target.eq_ignore_ascii_case(omitted))); }
        let text = String::from_utf8(world.load("tiletps.cs").unwrap().unwrap().1).unwrap();
        let expected: BTreeMap<_, _> = text.lines().filter_map(|line| {
            let words: Vec<_> = line.split('#').next()?.split_whitespace().collect();
            (words.len() == 6 && words[0].eq_ignore_ascii_case("sendEvent") && words[4] == "2403")
                .then(|| (words[2].to_ascii_lowercase(), words[1].parse::<f64>().unwrap()))
        }).collect();
        assert_eq!(cues.iter().map(|e| (e.target.to_ascii_lowercase(), e.due)).collect::<BTreeMap<_, _>>(), expected);
        assert!(south.world.requests.iter().any(|r| matches!(r, Request::ConnorAuto { auto, .. } if auto.target == [92640.0, 149100.0])));
        script(&mut south, "tiletps.cs", &["tiletps", "Cue", "1", "0"]);
        assert!(pending(&south).iter().any(|e| e.target.eq_ignore_ascii_case("tiletps") && e.data == ["2", "0"] && (e.due - 0.7).abs() < 0.00001));
        script(&mut south, "tiletps.cs", &["tiletps", "Cue", "2", "0"]);
        scripted_entry(&south, [92640.0, 149800.0, 600.0], 3.14);
        for (file, target, gate, position, heading) in [
            ("tiletps.cs", "tiletps", "SouthBridge", [92640.0, 149940.0, 600.0], 3.14),
            ("tiletpn.cs", "tiletpn", "NorthBridge", [92640.0, 162000.0, 600.0], 0.0),
        ] {
            let mut entry = active(&objects);
            entry.scripts.vars.set("Barren::Tile::Complete", "true".into());
            entry.scripts.vars.set(&format!("Barren::Tile::{gate}"), "On".into());
            script(&mut entry, file, &[target, "On"]);
            let events = pending(&entry);
            assert_eq!(events.len(), 1);
            assert_eq!((events[0].due, events[0].target.as_str(), events[0].data.clone()), (1.0, target, vec!["3".into(), "0".into()]));
            script(&mut entry, file, &[target, "Cue", "3", "0"]);
            scripted_entry(&entry, position, heading);
        }
        for (file, y) in [("tiletps2.cs", 145100.0), ("tiletpn2.cs", 166800.0)] {
            let mut exit = active(&objects);
            script(&mut exit, file, &["StartExit"]);
            assert!(exit.world.requests.iter().any(|r| matches!(r, Request::ConnorAuto { auto, .. }
                if auto.target == [92640.0, y] && matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { run: true }))));
        }
        println!("Barren original data: 35 platforms, eight authored motions and eight bitmap choices; installed patch precedence; 32 staggered starts; initial/completed/north entries and both exit destinations");
    }
}
