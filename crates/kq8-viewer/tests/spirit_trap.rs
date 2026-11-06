//! Original spirit boxes, queued cues, and state-owned reaction continuation.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Fixture {
    game: PathBuf,
    root: PathBuf,
    slots: PathBuf,
    original_options: Vec<u8>,
}
struct Run {
    viewer: String,
    traps: String,
    fight: String,
    connor: String,
    events: String,
}
impl Fixture {
    fn new(name: &str) -> Self {
        let game =
            fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../kq8-artifacts/{name}/{nonce}"));
        let slots = root.join("slots");
        let original_options = fs::read(game.join("Options.cs")).unwrap();
        Self {
            game,
            root,
            slots,
            original_options,
        }
    }
    fn run(&self, name: &str, args: &[&str]) -> Run {
        let dir = self.root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &self.slots)
            .arg("--game")
            .arg(&self.game)
            .args(["--sound", "off", "--no-ai"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let read = |name| fs::read_to_string(logs.join(name)).unwrap();
        let viewer = read("viewer.log");
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        Run {
            viewer,
            traps: read("traps.log"),
            fight: read("fight.log"),
            connor: read("connor.log"),
            events: read("doors.log"),
        }
    }
    fn slot(&self, n: u32) -> Value {
        serde_json::from_slice(
            &fs::read(self.slots.join(format!("slot-{n}.json")))
                .unwrap_or_else(|e| panic!("{e}; inspect {}", self.root.display())),
        )
        .unwrap()
    }
    fn options_unchanged(&self) {
        assert_eq!(
            fs::read(self.game.join("Options.cs")).unwrap(),
            self.original_options
        );
    }
}
fn spirit(slot: &Value) -> &Value {
    slot["traps"]["states"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "spirtp1")
        .unwrap()
}
fn row_pose(log: &str, update: u64) -> (&str, &str, f64) {
    let row = log
        .lines()
        .find(|line| line.starts_with(&format!("{update} ")))
        .unwrap();
    let fields: Vec<_> = row.split_whitespace().collect();
    let at = fields
        .iter()
        .position(|s| matches!(*s, "free" | "special" | "talk"))
        .unwrap();
    (fields[at], fields[at + 1], fields[at + 3].parse().unwrap())
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn occupied_original_spirit_trap_restores_without_entry_and_repeats_queued_drains() {
    let f = Fixture::new("spirit-trap-checkpoint");
    let first=f.run("save",&["--world","DeadCity","--frames","1400",
        // Isolate this controller from the separately checked animated Cue sources.
        "--command","deleteObject spiritst,1","--command","deleteObject spirits2,1",
        "--command","KQConner::setLevel 3,2","--command","sendEvent 0 Connor KQStatsChangeRequestEvent 1000 0 0 0 0,3",
        "--command","move Connor 0 62000 140800 1810 0 0 0,5","--hold","1,0,0,6,7",
        "--key","menu:down:down:enter:release:enter:release:escape:type=Spirit:enter:release:down:down:down:enter:release:left:enter:release,20"]);
    let saved = f.slot(1);
    let trap = spirit(&saved);
    assert_eq!(
        saved["journey"]["stats"]["stats"][0],
        17,
        "inspect {}",
        f.root.display()
    );
    assert!(saved["journey"]["stats"]["stats"][3]
        .as_i64()
        .is_some_and(|n| (0..100).contains(&n)));
    assert_eq!(trap["inside"], serde_json::json!(["Connor"]));
    assert_eq!(
        trap["spirit"],
        serde_json::json!({"damage":0.2,"spirit_state":2,"trap_status":1})
    );
    assert_eq!(
        first
            .traps
            .matches("spirit trap spirtp1: On Connor:")
            .count(),
        1
    );
    assert!(
        first.traps.contains("stored difference -12;")
            && first.fight.contains("hit by spirtp1, Normal:")
    );
    let after=f.run("restore",&["--menu-input","down,enter,release,enter,release","--frames","1800",
        "--command","sendEvent 0 spirtp1 KQCueEvent 1 0,10","--command","sendEvent 0 spirtp1 KQCueEvent 2 0,12",
        "--command","sendEvent 0 spirtp1 KQCueEvent 1 0,25",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=SpiritAfter:enter:release:down:down:down:enter:release:left:enter:release,40"]);
    assert!(after
        .viewer
        .contains("save restored in DeadCity without replaying Begin"));
    assert!(!after.traps.contains("spirit trap spirtp1: On Connor:"));
    assert_eq!(
        after
            .traps
            .matches("spirit trap spirtp1: queues direct stage:")
            .count(),
        2
    );
    assert!(after
        .traps
        .lines()
        .any(|line| line.contains("spirit trap spirtp1: Cue 2 ")
            && line.contains("spiritState 1 -> 1")));
    let final_slot = f.slot(2);
    assert_eq!(final_slot["journey"]["stats"]["stats"][0], 3);
    assert_eq!(spirit(&final_slot)["spirit"]["spirit_state"], 1);
    assert_eq!(spirit(&final_slot)["spirit"]["trap_status"], 1);
    assert_eq!(spirit(&final_slot)["inside"], trap["inside"]);
    f.options_unchanged();
}

fn reaction_case(pending: bool) {
    let f = Fixture::new(if pending {
        "forced-reaction-pending"
    } else {
        "forced-reaction-playing"
    });
    let mut args = vec![
        "--world",
        "DeadCity",
        "--frames",
        "1400",
        "--command",
        "KQConner::setLevel 8,1",
        "--command",
        "sendEvent 0 Connor KQStatsChangeRequestEvent 1000 0 0 100 0,2",
        "--command",
        "KQMonster::setReactPercent Connor 2,3",
        "--command",
        "sendEvent 0 Connor KQAttackEvent hazard 0 1 0,5",
    ];
    if pending {
        args.extend(["--command", "sendEvent 0 Connor KQAnimPlayEvent portal,9"]);
    }
    args.extend(["--command","sendEvent 0 Connor KQAttackEvent hazard 0 1 1,10",
        "--command","sendEvent 0 Connor KQAttackEvent hazard 0 1 1,11",
        "--key","menu:down:down:enter:release:enter:release:escape:type=Reaction:enter:release:down:down:down:enter:release:left:enter:release,15"]);
    let first = f.run("save", &args);
    let saved = f.slot(1);
    assert_eq!(saved["journey"]["stats"]["stats"][0], 80);
    assert!(saved["special"].is_null());
    assert_eq!(saved["hit_timing"]["reaction"], "react");
    let pose = &saved["hit_timing"]["reaction_pose"];
    assert_eq!(pose["reaction"], "react");
    assert_eq!(pose["movement"], if pending { "portal" } else { "react" });
    if pending {
        assert_eq!(pose["wanted"], "react");
    }
    let phase = pose["phase"].as_f64().unwrap();
    assert!(phase > 0.0 && phase < 1.0);
    assert!(
        first.fight.contains("no flinch (the roll is not above")
            && first.fight.contains("forced true, ordinary owner free")
    );
    if pending {
        assert!(first.fight.contains("reaction queued: 'react'"));
    }
    let after = f.run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--updates",
            "65",
            "--frames",
            "1600",
            "--command",
            "sendEvent 0 Connor KQAttackEvent hazard 0 1 1,2",
            "--command",
            "KQMonster::setReactPercent Connor 0,15",
            "--command",
            "sendEvent 0 Connor KQAttackEvent hazard 0 1 0,20",
            "--command",
            "sendEvent 0 Connor KQAttackEvent hazard 0 1 1,21",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 -100 0,48",
            "--command",
            "KQMonster::setState Connor special stop,49",
            "--command",
            "sendEvent 0 Connor KQAttackEvent hazard 0 1 1,50",
        ],
    );
    assert!(after
        .viewer
        .contains("save restored in DeadCity without replaying Begin"));
    let (owner, movement, next_phase) = row_pose(&after.connor, 1);
    assert_eq!(
        (owner, movement),
        ("free", if pending { "portal" } else { "react" })
    );
    let step = if pending { 1.0 / 20.0 } else { 1.0 / 12.0 };
    assert!(
        (next_phase - (phase + step)).abs() < 0.002,
        "pose restarted; inspect {}",
        f.root.display()
    );
    assert!(after
        .connor
        .contains("reaction react ended; next ordinary selection strictly after"));
    assert!(after.fight.contains("forced true, ordinary owner free"));
    assert!(after
        .fight
        .contains("special uses base hurt handling, without reaction selection"));
    assert!(
        after.fight.contains("health 80 -> 79"),
        "scripted Special still takes damage; inspect {}",
        f.root.display()
    );
    assert!(
        after.fight.contains("no flinch (his last flinch ended")
            || pending && after.fight.contains("'react' can't be cut short")
    );
    f.options_unchanged();
}
#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn forced_reaction_keeps_the_playing_pose_and_bypasses_selection_gates() {
    reaction_case(false);
}
#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn forced_reaction_waits_behind_original_portal_and_resumes_the_pending_request() {
    reaction_case(true);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_spirit_cycle_delivers_repeated_cues_to_the_live_trap() {
    let f = Fixture::new("spirit-trap-cycle");
    let run = f.run(
        "cycle",
        &[
            "--world",
            "DeadCity",
            "--updates",
            "90",
            "--frames",
            "1600",
            "--command",
            "KQConner::setLevel 20,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 1000 0 0 100 0,2",
            "--command",
            "move Connor 0 62000 140800 1810 0 0 0,5",
            "--hold",
            "1,0,0,6,7",
            "--command",
            "sendEvent 0 spiritst KQAnimPlayEvent cycle,8",
        ],
    );
    let ones = run
        .traps
        .lines()
        .filter(|line| line.contains("spirit trap spirtp1: Cue 1 from a cue of spiritst:"))
        .count();
    let twos = run
        .traps
        .lines()
        .filter(|line| line.contains("spirit trap spirtp1: Cue 2 from a cue of spiritst:"))
        .count();
    assert!(
        ones >= 2 && twos >= 2,
        "authored cycle cues missing: {ones}/{twos}; inspect {}",
        f.root.display()
    );
    assert!(run
        .traps
        .lines()
        .any(|line| line.contains("Cue 2 from a cue of spiritst:")
            && line.contains("spiritState 1 -> 1")));
    assert!(
        run.fight.contains("hit by spirtp1, Normal:")
            && run.fight.contains("forced true, ordinary owner")
    );
    assert!(!run.fight.contains("his dying state, kind"));
    f.options_unchanged();
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn simultaneous_spirit_cues_preserve_later_negative_stats_after_one_dying_entry() {
    let f = Fixture::new("spirit-trap-simultaneous");
    let run = f.run(
        "death",
        &[
            "--world",
            "DeadCity",
            "--updates",
            "55",
            "--frames",
            "900",
            "--command",
            "deleteObject spiritst,1",
            "--command",
            "deleteObject spirits2,1",
            "--command",
            "KQConner::setLevel 1,2",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 1000 0 0 0 0,3",
            "--command",
            "move Connor 0 62000 140800 1810 0 0 0,5",
            "--hold",
            "1,0,0,6,7",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent -4 0 0 0 0,15",
            "--command",
            "sendEvent 0 spirtp1 KQCueEvent 1 0,20",
            "--command",
            "sendEvent 0 spirtp1 KQCueEvent 1 0,20",
        ],
    );
    assert_eq!(
        run.traps.matches("stored difference -1;").count(),
        2,
        "same snapshot deltas missing; inspect {}",
        f.root.display()
    );
    assert_eq!(
        run.fight
            .matches("health -1 is below 0: his dying state")
            .count(),
        1
    );
    assert!(run.fight.contains("ignored, he is dying"));
    assert!(
        run.events.lines().any(
            |line| line.contains("KQStatsChangeRequestEvent -1 to Connor")
                && line.contains("Connor [0,")
                && line.contains("-> [-1,")
        ),
        "later 0 -> -1 direct stage is missing from the event log; inspect {}",
        f.root.display()
    );
    assert_eq!(
        run.viewer.matches("applied: Connor dies (Normal):").count(),
        1
    );
    f.options_unchanged();
}

fn controller_reaction_case(talk: bool) {
    let f = Fixture::new(if talk {
        "forced-reaction-talk"
    } else {
        "forced-reaction-choice"
    });
    let mut args = vec![
        "--world",
        "Daventry",
        "--frames",
        "1600",
        "--command",
        "KQConner::setLevel 8,1",
        "--command",
        "sendEvent 0 Connor KQStatsChangeRequestEvent 1000 0 0 100 0,2",
    ];
    if talk {
        args.extend(["--command","set Daventry::Wizard::Clicked true,3",
            "--command","move Connor 0 192171 198091 6700 0 0 0,5",
            "--command","KQConner::conversation Wizard 1000 4 14 27 1 2 0,12",
            "--command","sendEvent 0 Connor KQAttackEvent hazard 0 1 1,55",
            "--key","menu:down:down:enter:release:enter:release:escape:type=TalkReact:enter:release:down:down:down:enter:release:left:enter:release,59"]);
    } else {
        args.extend(["--command","move Connor 0 228700 183200 12350 0 0 -1.57,5",
            "--command","sendEvent 0 Connor KQAttackEvent hazard 0 1 1,34",
            "--key","menu:down:down:enter:release:enter:release:escape:type=ChoiceReact:enter:release:down:down:down:enter:release:left:enter:release,38"]);
    }
    let first = f.run("save", &args);
    let saved = f.slot(1);
    assert!(saved["special"].is_null() && saved["hit_timing"]["reaction_pose"].is_null());
    assert_eq!(saved["hit_timing"]["reaction"], "react");
    let pose = if talk {
        &saved["talk_pose"]
    } else {
        &saved["choice"]["pose"]
    };
    assert_eq!(pose["movement"], "react");
    assert_eq!(pose["reaction"], "react");
    let phase = pose["phase"].as_f64().unwrap();
    assert!(phase > 0.0 && phase < 1.0);
    assert!(first.fight.contains(if talk {
        "forced true, ordinary owner talk"
    } else {
        "forced true, ordinary owner free"
    }));
    if talk {
        assert!(saved["conversation"]["active"].is_object());
        assert_eq!(
            pose["wanted"], "gestureg",
            "the original lip code must wait behind react"
        );
        assert_eq!(pose["reaction_playing"], "react");
        assert_eq!(
            pose["animation_input_lock"], false,
            "the lip request unlocks before playback starts"
        );
        assert_eq!(saved["hit_timing"]["animation_input_lock"], false);
        assert_eq!(saved["hit_timing"]["flinch_in"], 0.0);
        assert!(first
            .connor
            .contains("lip-sync code 14: requests 'gestureg'; animation input lock true -> false"));
    } else {
        assert_eq!(saved["choice"]["state"]["made"], false);
    }
    let after = f.run(
        "restore",
        &[
            "--menu-input",
            "down,enter,release,enter,release",
            "--updates",
            "20",
            "--frames",
            "1300",
        ],
    );
    assert!(after
        .viewer
        .contains("save restored in Daventry without replaying Begin"));
    let (owner, movement, next) = row_pose(&after.connor, 1);
    assert_eq!(
        (owner, movement),
        (if talk { "talk" } else { "free" }, "react")
    );
    assert!(
        (next - (phase + 1.0 / 12.0)).abs() < 0.002,
        "controller reaction restarted; inspect {}",
        f.root.display()
    );
    if talk {
        assert!(
            !after.viewer.contains("Wizard: TalkExit:"),
            "a hit must not complete the conversation"
        );
    } else {
        assert_eq!(
            after
                .connor
                .matches("Choice movement react ended: rooftrap.cs gets StartChoice")
                .count(),
            1
        );
        assert!(!after.connor.contains("Choice points measured from"));
    }
    f.options_unchanged();
}
#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn forced_reaction_keeps_original_roof_choice_and_its_saved_pose() {
    controller_reaction_case(false);
}
#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn forced_reaction_keeps_wizard_talk_and_its_saved_pose() {
    controller_reaction_case(true);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn forced_reaction_during_an_axe_swing_preserves_do_and_the_weapon_bank_after_loading() {
    let f = Fixture::new("forced-reaction-melee");
    let first=f.run("save",&["--world","DeadCity","--item","INVITEM_SmallAx","--frames","1600",
        "--command","KQConner::setLevel 8,1","--command","sendEvent 0 Connor KQStatsChangeRequestEvent 1000 0 0 100 0,2",
        "--command","move skeleton1 0 183300 45000 864 0 0 0,5",
        "--swing","skeleton1,20","--swing","skeleton1,50",
        "--command","sendEvent 0 Connor KQAttackEvent hazard 0 1 1,51",
        "--key","menu:down:down:enter:release:enter:release:escape:type=SwingReact:enter:release:down:down:down:enter:release:left:enter:release,54"]);
    let saved = f.slot(1);
    let target_health = |slot: &Value| {
        slot["kept"]["deadcity"]["characters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "skeleton1")
            .unwrap()["character"]["fight"]["stats"][0]
            .as_i64()
            .unwrap()
    };
    assert_eq!(target_health(&saved), 60);
    assert!(saved["swing"]["phase"]
        .as_f64()
        .is_some_and(|phase| phase < 0.5));
    assert_eq!(saved["hit_timing"]["reaction"], "conaxe.anm/react");
    assert!(saved["hit_timing"]["reaction_pose"].is_null() && saved["special"].is_null());
    assert!(first.fight.contains("reaction queued: 'conaxe.anm/react'"));
    let after=f.run("restore",&["--menu-input","down,enter,release,enter,release","--frames","1400",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=AfterSwingReact:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    assert!(after
        .viewer
        .contains("save restored in DeadCity without replaying Begin"));
    assert_eq!(
        after
            .fight
            .matches("Connor: Do (conaxe.anm/attack): hits skeleton1")
            .count(),
        1
    );
    assert!(after
        .connor
        .contains("reaction conaxe.anm/react ended; next ordinary selection strictly after"));
    let last = f.slot(2);
    assert!((1..60).contains(&target_health(&last)));
    assert!(
        last["hit_timing"]["reaction"].is_null()
            && last["special"].is_null()
            && last["swing"].is_null()
    );
    f.options_unchanged();
}
