//! Original monster scripts must change live fight stats, including through saves.
use std::{fs, path::{Path, PathBuf}, process::Command};

struct Fixture { game: PathBuf, root: PathBuf, slots: PathBuf, options: Vec<u8> }
impl Fixture {
    fn new(label: &str) -> Self {
        let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/monster-stats/{label}-{nonce}"));
        let options = fs::read(game.join("Options.cs")).unwrap();
        Self { game, slots: root.join("slots"), root, options }
    }
    fn run(&self, label: &str, args: &[&str]) -> String {
        let dir = self.root.join(label);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &self.slots).arg("--game").arg(&self.game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        fs::read_to_string(logs.join("doors.log")).unwrap()
    }
    fn slot(&self, n: usize) -> serde_json::Value {
        serde_json::from_slice(&fs::read(self.slots.join(format!("slot-{n}.json")))
            .unwrap_or_else(|e| panic!("{e}; inspect {}", self.root.display()))).unwrap()
    }
    fn unchanged(&self) { assert_eq!(fs::read(self.game.join("Options.cs")).unwrap(), self.options); }
}
fn actor<'a>(s: &'a serde_json::Value, world: &str, name: &str) -> &'a serde_json::Value {
    s["kept"][world]["characters"].as_array().unwrap().iter()
        .find(|a| a["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(name))).unwrap()
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_snowmane_reveal_changes_combat_stats_across_disk_reload() {
    let f = Fixture::new("snowmane");
    let first_log = f.run("reveal", &["--world", "IceWorld", "--frames", "1700", "--item", "INVITEM_Clarity",
        "--key", "7,20",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=Revealed:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    let first = f.slot(1);
    let stats = &actor(&first, "iceworld", "snowmane")["character"]["fight"]["stats"];
    assert_eq!(stats[1], 200, "Reveal did not remove the original +150 short damage; inspect {}", f.root.display());
    assert_eq!(stats[3], 0, "Reveal did not remove 30 protection");
    assert!(first_log.contains("live combat stats"));
    assert_eq!(first["journey"]["potions"].as_array().unwrap().len(), 1);
    f.run("hide-after-load", &["--menu-input", "down,enter,release,enter,release", "--frames", "6000",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Hidden:enter:release:down:down:down:enter:release:left:enter:release,925"]);
    let second = f.slot(2);
    let after = &actor(&second, "iceworld", "snowmane")["character"]["fight"]["stats"];
    assert_eq!(after[1], 350);
    assert_eq!(after[3], 30);
    for i in [0, 2, 4] { assert_eq!(after[i], stats[i]); }
    assert!(second["journey"]["potions"].as_array().unwrap().is_empty());
    f.unchanged();
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn monster_stats_death_continues_once_after_a_fresh_process() {
    let f = Fixture::new("skeleton-death");
    f.run("save", &["--world", "DeadCity", "--frames", "1600",
        "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
        "--command", "sendEvent 0 skeleton1 KQStatsChangeRequestEvent -100000 0 0 0 0,80",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=StatsDeath:enter:release:down:down:down:enter:release:left:enter:release,90"]);
    let first = f.slot(1);
    assert!(!actor(&first, "deadcity", "skeleton1")["character"]["fight"]["dying"].is_null());
    let xp = first["journey"]["stats"]["stats"][4].as_i64().unwrap();
    f.run("restore", &["--menu-input", "down,enter,release,enter,release", "--frames", "1800",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=StatsFinished:enter:release:down:down:down:enter:release:left:enter:release,25"]);
    let after = f.slot(2);
    assert!(after["kept"]["deadcity"]["gone"].as_array().unwrap().iter().any(|n| n == "skeleton1"));
    assert_eq!(after["journey"]["stats"]["stats"][4].as_i64().unwrap() - xp, 200);
    f.unchanged();
}
