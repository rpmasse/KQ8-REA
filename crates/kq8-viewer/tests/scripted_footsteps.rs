//! Temple 1's actual water trigger changes the remembered building's footsteps.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn temple1_water_steps_survive_reload_then_restore_dirt_on_exit() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/scripted-footsteps/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &result.stderr).unwrap();
        fs::write(dir.join("stdout.log"), &result.stdout).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(result.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("connor.log")).unwrap(), fs::read_to_string(logs.join("traps.log")).unwrap())
    };
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap_or_else(|e| panic!("{e}; inspect {}", root.display()))).unwrap()
    };
    let quit = "down:down:down:enter:release:left:enter:release";
    let save = format!("menu:down:down:enter:release:enter:release:escape:type=Water:enter:release:{quit},35");
    let (_, first_steps, first_commands) = run("save", &["--world", "temple1", "--frames", "1200",
        // newWorld.cs places the original sloshtrp at (97961, 57770, 6010).
        "--command", "move Connor 0 97961 57770 6500 0 0 0,5", "--hold", "1,0,0,15,25", "--key", &save]);
    assert_eq!(read(1)["footstep_types"]["temple_e"], 3, "water trigger did not set the building type; inspect {}", root.display());
    assert!(first_commands.contains("KQObject::setFootStepSound temple_e 3"));
    assert!(first_steps.lines().any(|l| l.contains("footstep on temple_e") && l.contains("flags 0x8 -> water")),
        "actual steps did not use the water set; inspect {}", root.display());
    let finish = format!("menu:down:down:down:enter:release:enter:release:escape:type=Dirt:enter:release:{quit},85");
    let (viewer, after, commands) = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1800",
        "--hold", "1,0,0,5,15", "--command", "move Connor 0 102000 57770 6500 0 0 0,35",
        "--hold", "1,0,0,55,70", "--key", &finish]);
    assert!(viewer.contains("save restored in temple1 without replaying Begin"));
    assert!(!commands.contains("KQObject::setFootStepSound temple_e 3"), "the occupied trigger replayed On after loading");
    assert_eq!(read(2)["footstep_types"]["temple_e"], 0);
    let water = after.lines().position(|l| l.contains("footstep on temple_e") && l.contains("flags 0x8 -> water")).expect("restored water steps missing");
    let dirt = after.lines().position(|l| l.contains("footstep on temple_e") && l.contains("flags 0x1 -> dirt")).expect("exit did not restore dirt steps");
    assert!(water < dirt);
    assert_eq!(commands.matches("KQObject::setFootStepSound temple_e 0").count(), 1);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
