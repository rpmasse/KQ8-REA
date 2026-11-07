//! Q206: one regional-data matrix and one representative native liquid
//! dispatch/audio/death fixture. It is not an earned campaign route or
//! an original-executable/runtime comparison.
use std::{fs, path::Path, process::Command};
use kq8_core::terrain_hazards::{amount, Settings};
use kq8_data::{game_dir::WORLDS, GameDir, VolArchive};

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
