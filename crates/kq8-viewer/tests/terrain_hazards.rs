//! Q206: one regional-data matrix and one representative native liquid
//! dispatch/audio/death fixture. It is not an earned campaign route or
//! an original-executable/runtime comparison.
use std::{fs, path::Path, process::Command};
use kq8_core::terrain_hazards::{amount, Settings};
use kq8_data::{game_dir::WORLDS, kq_text, terrain::Grid, Bitmap, GameDir, Terrain, TerrainSettings, VolArchive};

#[test]
#[ignore = "requires read-only KQ8_GAME_DIR original assets"]
fn original_regional_liquid_settings_share_the_confirmed_amount_rules() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    for world in WORLDS {
        let vol = VolArchive::open(&game.volume_path(world, "kq").unwrap()).unwrap();
        let mut bytes = vol.unpack(vol.find("WORLD.CS").unwrap()).unwrap().data;
        if let Some(patch) = game.patch_files(world).unwrap().get("world.cs") {
            bytes = fs::read(patch).unwrap();
        }
        let text = String::from_utf8_lossy(&bytes);
        let mut vars = kq8_data::world_script::Vars::default();
        for line in text.lines() {
            let words: Vec<_> = line.split("//").next().unwrap_or("").split_whitespace().collect();
            if let ["set", name, value, ..] = words.as_slice() {
                if ["KQWorld::PoisonTime", "KQWorld::FastPoisonTime", "KQWorld::SwampTime"].contains(name) {
                    vars.set(name, (*value).to_string());
                }
            }
        }
        let settings = Settings::from_vars(&vars);
        assert_eq!(vars.get("KQWorld::PoisonTime"), Some("5"), "{world}");
        assert_eq!(vars.get("KQWorld::FastPoisonTime"), Some("0"), "{world}");
        assert_eq!(amount(settings.poison, Some(12), 0.5), 1, "{world}");
        assert_eq!(amount(settings.fast_poison, Some(12), 0.5), 1000, "{world}");
        if ["Daventry", "Swamp"].contains(&world) { assert_eq!(settings.swamp, 5.0, "{world}"); }
    }
}

#[test]
#[ignore = "requires GOG assets and a graphics device; isolated hazard fixture"]
fn original_liquid_dispatches_protected_hits_audio_cycle_and_ordinary_death() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/terrain-hazards/{nonce}"));
    fs::create_dir_all(&root).unwrap();
    // Temporary fixture commands: set full protection, move into the actual
    // deep river material, then remove protection. Original Time settings,
    // assets and the combat implementation remain unchanged. No save is
    // promoted to the earned campaign and no game-install file is changed.
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root).env("KQ8_SAVE_DIR", root.join("slots")).arg("--game").arg(&game)
        .args(["--world", "DeadCity", "--no-ai", "--sound", "off", "--frames", "1500",
            "--command", "KQConner::setLevel 12,1",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 120 0 0 100 0,2",
            "--command", "move Connor 0 32000 198000 0 0 0 0,5",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 -89 0,125"])
        .output().unwrap();
    fs::write(root.join("stderr.log"), output.stderr).unwrap();
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
    println!("Original Options.cs bytes unchanged; terrain evidence: {}", root.display());
    let logs = root.join("kq8-artifacts/logs");
    let read = |name| fs::read_to_string(logs.join(name)).unwrap();
    let viewer = read("viewer.log");
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", root.display());
    let fight = read("fight.log");
    assert!(read("doors.log").contains("terrain liquid flags 0x80"), "actual river material must emit the attack; inspect {}", root.display());
    assert!(fight.contains("hit by world, Normal"), "inspect {}", root.display());
    assert!(fight.contains("health 120 -> 120"), "full protection must reach ordinary zero-loss handling");
    for clip in ["poison1.aud", "poison2.aud", "poison3.aud"] {
        assert!(fight.contains(&format!("world-hit sound {clip}")), "missing {clip}; inspect {}", root.display());
        assert!(read("sound.log").contains(clip), "{clip} did not reach audio dispatch");
    }
    let connor = read("connor.log");
    assert!(connor.contains("combat ouch suppressed for world hit"));
    assert!(connor.contains("dies (Normal)"), "inspect {}", root.display());
    assert!(viewer.contains("die finished: Die state holds until"), "ordinary death must complete; inspect {}", root.display());
    println!("Grouped native terrain evidence: {}", root.display());
}

#[test]
#[ignore = "requires read-only KQ8_GAME_DIR original assets and a graphics device"]
fn original_barren_ontop_lava_uses_the_extra_mover_and_world_attack() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let dir = GameDir::open(&game).unwrap();
    let text = fs::read(dir.find("data/Barren/KQ/Terrain/TERRAIN.KQ").unwrap()).unwrap();
    let sections = kq_text::parse(&text);
    let settings = TerrainSettings::from_section(sections.iter()
        .find(|s| s.get("classType") == Some("KQTerrain")).unwrap()).unwrap();
    let archives: Vec<_> = ["kq", "resource", "8bit", "8gui"].into_iter()
        .map(|name| VolArchive::open(&dir.volume_path("Barren", name).unwrap()).unwrap()).collect();
    let load = |name: &str| {
        let loose = format!("data/Barren/KQ/Terrain/{name}");
        if let Some(path) = dir.find(&loose) { return fs::read(path).unwrap(); }
        archives.iter().find_map(|archive| archive.find(name).map(|entry| archive.unpack(entry).unwrap().data))
            .unwrap_or_else(|| panic!("Barren asset {name} missing"))
    };
    let grid = |name: &str| Grid::from_bitmap(&Bitmap::parse(&load(name)).unwrap()).unwrap();
    let env_name = settings.all.iter().find(|(key, _)| key.eq_ignore_ascii_case("envinfoName"))
        .map_or("envInfo.kq", |(_, value)| value.as_str());
    let terrain = Terrain::new(settings.clone(), grid(&settings.height_image)).unwrap()
        .with_ground_flags(&grid(&settings.material_image), &kq8_data::envinfo::parse(&load(env_name))).unwrap();
    let (nx, ny) = terrain.squares();
    let size = settings.square_size();
    let flags = kq8_data::envinfo::FAST_POISON | kq8_core::terrain_hazards::FIRE | kq8_core::terrain_hazards::ONTOP;
    let (x, y, surface) = (2..ny - 2).flat_map(|j| (2..nx - 2).map(move |i| (i, j)))
        .filter_map(|(i, j)| {
            let (x, y) = ((i as f32 + 0.5) * size, (j as f32 + 0.5) * size);
            (terrain.flags_at(x, y) & flags == flags).then(|| (x, y, terrain.height_at(x, y).unwrap().height))
        }).next().expect("original Barren has no lava/fire/ontop square");
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/terrain-hazards/ontop-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&root).env("KQ8_SAVE_DIR", root.join("slots")).arg("--game").arg(&game)
        .args(["--world", "Barren", "--windowed", "--no-ai", "--sound", "off", "--frames", "90",
            "--command", "KQConner::setLevel 12,1",
            "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 120 0 0 100 0,2"])
        .args(["--command", &format!("move Connor 0 {x:.0} {y:.0} {:.0} 0 0 0,5", surface + 50.0)])
        .output().unwrap();
    fs::write(root.join("stderr.log"), output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", root.display());
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(doors.contains("terrain ontop flags"), "inspect {}", root.display());
    let descended = connor.lines().filter(|line| line.contains("ontop extra mover (Q206)"))
        .any(|line| {
            let Some((_, movement)) = line.split_once(": z ") else { return false };
            let Some((before, rest)) = movement.split_once(" -> ") else { return false };
            let Some(after) = rest.split(',').next() else { return false };
            before.parse::<f32>().ok().zip(after.parse::<f32>().ok())
                .is_some_and(|(before, after)| (before - after - 15.0).abs() < 0.01)
        });
    assert!(descended, "the collision-free extra mover must lower Connor by 15; inspect {}", root.display());
    assert!(fight.contains("hit by world, Fire"), "inspect {}", root.display());
    println!("Barren ontop at ({x:.0},{y:.0}), surface {surface:.0}; evidence: {}", root.display());
}
