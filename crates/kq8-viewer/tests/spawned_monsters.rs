//! A Dead City script-created monster survives a disk save and fresh process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn scripted_monster_survives_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/spawn-save-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &out.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(out.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    run("save", &["--world", "DeadCity", "--command", "KQMonster::add testRat rat,8",
        "--frames", "230",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Spawned:enter:release:escape,30"]);
    let slot = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = slot(1);
    let monster = &first["kept"]["deadcity"]["spawned_monsters"][0];
    assert_eq!(monster["command"][0], "testRat");
    assert_eq!(monster["command"][1], "rat");
    assert!(monster["character"]["fight"]["stats"][0].as_i64().unwrap() > 0);
    let viewer = run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "420",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=SpawnedAgain:enter:release:escape,24"]);
    assert!(viewer.contains("save restored in DeadCity without replaying Begin"));
    assert!(viewer.contains("copy testRat of rat.kq: drawn"), "inspect {}", root.join("restore").display());
    let again = slot(2);
    assert_eq!(again["kept"]["deadcity"]["spawned_monsters"][0]["command"], monster["command"]);
    assert_eq!(again["kept"]["deadcity"]["spawned_monsters"][0]["character"]["fight"]["stats"], monster["character"]["fight"]["stats"]);
}
