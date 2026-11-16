//! The moving-shot delay and scheduled run return survive a new process.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn moving_crossbow_cooldown_resumes_after_disk_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/moving-shot-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--windowed", "--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        viewer
    };
    let before = run("save", &[
        "--world", "DeadCity", "--item", "INVITEM_SmCrossbow",
        "--command", "KQConner::setDoMode 1,25",
        "--hold", "1,1,1,45,100", "--aim-point", "182000,55000,1800,50",
        "--frames", "1200",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Moving:enter:release:escape,52",
    ]);
    assert!(before.contains("Save screen -> Save(1, \"Moving\")"), "inspect {}", root.display());
    let shot_log = fs::read_to_string(root.join("save/kq8-artifacts/logs/fight.log")).unwrap();
    let shot = shot_log.lines().find(|line| line.contains("shoots bolt1 (Prscbolt.kq"))
        .expect("the moving crossbow must release its first bolt");
    assert!(shot.contains("its part 'arrow' in frame") && shot.contains("stoprun"),
        "shot must use the original arrow part in the moving pose: {shot}");
    let shot_update: usize = shot.strip_prefix("update ").unwrap().split_once(':').unwrap().0.parse().unwrap();
    let triplet = |after: &str| -> [f64; 3] {
        let values: Vec<_> = after.split_once(')').unwrap().0.split(',')
            .map(|v| v.trim().parse::<f64>().unwrap()).collect();
        [values[0], values[1], values[2]]
    };
    let offset = triplet(shot.split_once("start offset (").unwrap().1);
    let from = triplet(shot.split_once(" from (").unwrap().1);
    let connor_log = fs::read_to_string(root.join("save/kq8-artifacts/logs/connor.log")).unwrap();
    let columns: Vec<_> = connor_log.lines().next().unwrap().split_whitespace().collect();
    let row = connor_log.lines().find(|line| line.starts_with(&format!("{shot_update} "))).unwrap();
    let fields: Vec<_> = row.split_whitespace().collect();
    let number = |name: &str| -> f64 {
        fields[columns.iter().position(|column| *column == name).unwrap()].parse().unwrap()
    };
    let feet = [number("x"), number("y"), number("z")];
    let (heading, bank) = (number("heading"), number("bank"));
    assert!(bank > 0.1, "the shot must occur during a real lean: {row}");
    // Independent row-vector calculation for the documented Y lean, then Z heading.
    let leaned_x = bank.cos() * offset[0] + bank.sin() * offset[2];
    let leaned_z = -bank.sin() * offset[0] + bank.cos() * offset[2];
    let projected = |x: f64, z: f64| -> [f64; 3] {
        [feet[0] + heading.cos() * x - heading.sin() * offset[1],
         feet[1] + heading.sin() * x + heading.cos() * offset[1],
         feet[2] + z]
    };
    let full_turn = projected(leaned_x, leaned_z);
    let heading_only = projected(offset[0], offset[2]);
    assert!(from.iter().zip(full_turn).all(|(actual, expected)| (actual - expected).abs() < 3.0),
        "bolt start {from:?} did not follow Connor's full turn {full_turn:?}; inspect {}", root.display());
    assert!(from.iter().zip(heading_only).any(|(actual, unleaned)| (actual - unleaned).abs() > 50.0),
        "the shot did not distinguish lean from heading alone; inspect {}", root.display());
    let read = |n| -> serde_json::Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let first = read(1);
    let timing = &first["shot_timing"];
    assert_eq!(timing["run"], false);
    // Menu input is sampled per frame, so the save may capture a different
    // fixed tick. Check the original 0.5 s / 1 s deadlines and their gap,
    // then test both sides of the retained cooldown after loading.
    let next_shot = timing["next_shot_in"].as_f64().unwrap();
    let run_back = timing["run_back_in"].as_f64().unwrap();
    assert!(next_shot > 0. && next_shot <= 0.5);
    assert!((run_back - next_shot - 0.5).abs() < 1e-6);
    assert_eq!(first["bolts"]["bolts_shot"], 1);

    let after = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--updates", "75", "--frames", "10000",
        "--hold", "1,0,0,0,65",
        "--aim-point", "182000,55000,1800,2",
        "--aim-point", "182000,55000,1800,8",
        "--key", "physical-hold:control:50,35",
        "--aim-point", "182000,55000,1800,40",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=AfterMoving:enter:release:escape,30",
    ]);
    assert!(after.contains("save restored in DeadCity without replaying Begin")
        && after.contains("Save screen -> Save(2, \"AfterMoving\")"), "inspect {}", root.display());
    let fight = fs::read_to_string(root.join("restore/kq8-artifacts/logs/fight.log")).unwrap();
    assert_eq!(fight.matches("moving: the last shot was less than 0.5 s ago").count(), 1,
        "the shot during the saved cooldown was accepted; inspect {}", root.display());
    assert_eq!(fight.matches("shoots bolt2 (Prscbolt.kq").count(), 1,
        "the later moving shot did not fire once; inspect {}", root.display());
    assert!(!fight.contains("shoots bolt3 ("),
        "Ctrl-click fired a third bolt; inspect {}", root.display());
    let doors = fs::read_to_string(root.join("restore/kq8-artifacts/logs/doors.log")).unwrap();
    assert!(doors.lines().any(|line| line.starts_with("update 40: test --aim-point")
        && line.contains("(Ctrl held)") && !line.contains("the aim pick")),
        "the ranged Ctrl-click did not take the plain interaction path; inspect {}", root.display());
    let update = |line: &str| -> u64 { line.strip_prefix("update ").unwrap().split(':').next().unwrap().parse().unwrap() };
    let shot_tick = update(fight.lines().find(|line| line.contains("shoots bolt2 (Prscbolt.kq")).unwrap());
    let return_tick = update(fight.lines().find(|line| line.contains("Connor: 1 s after the last moving shot: the run is switched on again")).unwrap());
    assert!((15..=16).contains(&(return_tick - shot_tick)),
        "the run must return one second (15 game ticks) after the actual second shot; inspect {}", root.display());
    let second = read(2);
    assert_eq!(second["bolts"]["bolts_shot"], 2);
    assert_eq!(second["shot_timing"]["run"], true);
    assert!(second["shot_timing"]["run_back_in"].is_null());
}
