//! A melee `do` cue awaiting the next world tick survives a disk save.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn pending_axe_hit_applies_once_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/connor-melee-cue/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--item", "INVITEM_SmallAx",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--swing", "skeleton1,20", "--swing", "skeleton1,50", "--frames", "120",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Hit:enter:release:escape,57",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Hit\")"), "inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    assert_eq!(first["swing"]["movement"], "conaxe.anm/attack");
    assert!((first["swing"]["phase"].as_f64().unwrap() - 7.0 / 12.0).abs() < 0.001);
    assert_eq!(first["combat_cues"]["melee"][0], "conaxe.anm/attack");
    let skeleton = |slot: &serde_json::Value| -> i64 {
        slot["kept"]["deadcity"]["characters"].as_array().unwrap().iter()
            .find(|a| a["name"] == "skeleton1").unwrap()["character"]["fight"]["stats"][0].as_i64().unwrap()
    };
    assert_eq!(skeleton(&first), 60, "the hit happened before the saved world state");

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "700",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterHit:enter:release:escape,25",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterHit\")"), "inspect {}", root.display());
    let second = read(2);
    assert!((1..60).contains(&skeleton(&second)), "the pending hit did not reach skeleton1");
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("Connor: Do (conaxe.anm/attack): hits skeleton1").count(), 1,
        "the pending hit was lost or applied twice; inspect {}", root.display());
}
