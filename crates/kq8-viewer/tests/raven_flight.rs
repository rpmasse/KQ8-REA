//! Two independent processes run the shipped grave raven's flight.
//! Set up the encounter with the same loadKQ and Special movement requests
//! used by rvngrvtp.cs and ravengrv.cs, bypassing their approach and dialogue.
//! Compare the resumed audio cues, final script and quest state to continuing
//! to play immediately after saving in the first process.
use serde_json::Value;
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn grave_raven_continues_with_its_next_cue_and_completes_once() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/raven-tests/{nonce}"));
    let slots = root.join("slots");
    let run = |name: &str, args: &[&str]| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", &slots)
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let main = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && main.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (
            main,
            fs::read_to_string(logs.join("doors.log"))
                .unwrap()
                .to_ascii_lowercase(),
        )
    };
    let read_slot = |n| -> Value {
        serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap()
    };
    let (main, before) = run(
        "first",
        &[
            "--world",
            "Daventry",
            "--frames",
            "560",
            "--console",
            "move Connor 0 222000 177000 9400 0 0 0,2",
            "--console",
            "loadKQ ravengrv.kq,4",
            "--console",
            "move Raveng 1 0 0 0 0 0 1.5,12",
            "--console",
            "KQMonster::setState Raveng Special fromgrave,13",
            "--key",
            "menu:down:down:enter:release:enter:release:escape:type=Flight:enter:release:escape,23",
            "--key",
            "menu:up:enter:release:left:enter:release,70",
        ],
    );
    assert!(
        main.contains("Save screen -> Save(1, \"Flight\")"),
        "inspect {}",
        root.display()
    );
    let slot = read_slot(1);
    let raven = &slot["kept"]["daventry"]["interactions"]["special_actors"]["raveng"];
    assert_eq!(raven["playing"], "fromgrave", "inspect {}", root.display());
    let phase = raven["animation"]["phase"].as_f64().unwrap();
    let placement = &raven["animation"]["placement"];
    assert_eq!(placement["position"], raven["mover"]["position"]);
    assert_eq!(placement["position"][0], 221758.0);
    assert_eq!(placement["position"][1], 178380.0);
    assert!((placement["rotation"][0][1].as_f64().unwrap() - 1.5_f64.sin()).abs() < 1e-6);
    // First cry played, second and third are still ahead (raven.anm).
    assert!((0.021739..0.326087).contains(&phase), "saved phase {phase}");
    let (main, after) = run("restored", &[
        "--menu-input", "down,enter,release,enter,release", "--frames", "550",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=Finished:enter:release:escape,85",
        "--key", "menu:up:enter:release:left:enter:release,94",
    ]);
    assert!(main.contains("save restored in Daventry without replaying Begin"));
    // The log prefixes come from the gameplay tick and the actual callbacks.
    let ticks = |log: &str, marker: &str| -> Vec<u64> {
        log.lines()
            .filter_map(|line| {
                let (tick, _) = line.split_once(marker)?;
                tick.strip_prefix("update ")?.parse().ok()
            })
            .collect()
    };
    let sound = ": object raveng: cue 'playsound raven.aud' fires";
    let end = ": script ravengrv.cs raveng endloop fromgrave";
    let reference_cues = ticks(&before, sound);
    let actual_cues = ticks(&after, sound);
    assert_eq!(reference_cues.len(), 3, "inspect {}", root.display());
    assert_eq!(actual_cues.len(), 2, "inspect {}", root.display());
    // The saved phase determines the exact tick of the next cry at 15 Hz.
    // The 46-frame ravntomb.KQS plays in three seconds.
    // Active handles the first animation interval at update 0, then
    // increments the update counter. The fifth advance is logged as 4.
    let next = ((0.326087 - phase) * 45.).ceil() as u64 - 1;
    assert_eq!(
        actual_cues[0],
        next,
        "saved phase {phase}; inspect {}",
        root.display()
    );
    let reference_end = ticks(&before, end);
    let actual_end = ticks(&after, end);
    assert_eq!((reference_end.len(), actual_end.len()), (1, 1));
    assert_eq!(
        reference_cues[2] - reference_cues[1],
        actual_cues[1] - actual_cues[0]
    );
    assert_eq!(
        reference_end[0] - reference_cues[1],
        actual_end[0] - actual_cues[0]
    );
    let finished = read_slot(2);
    let daventry = &finished["kept"]["daventry"];
    assert!(daventry["gone"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n == "raveng"));
    assert!(daventry["vars"].as_array().unwrap().iter().any(|v| {
        v[0].as_str()
            .is_some_and(|k| k.eq_ignore_ascii_case("Daventry::Raven::Tomb"))
            && v[1] == "done"
    }));
}
