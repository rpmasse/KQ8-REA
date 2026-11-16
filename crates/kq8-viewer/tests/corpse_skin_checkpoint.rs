//! Installed Daventry bossprig keeps its authored bitmap on the corpse.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn bossprig_corpse_skin_survives_script_changes_and_a_fresh_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/corpse-skin-checkpoint/{nonce}"));
    let slots = root.join("slots");
    let run = |label: &str, args: &[&str]| {
        let dir = root.join(label);
        fs::create_dir_all(&dir).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots)
            .arg("--game").arg(&game)
            .args(["--windowed", "--sound", "off", "--no-ai", "--god"])
            .args(args).output().unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        let fight = fs::read_to_string(dir.join("kq8-artifacts/logs/fight.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fight)
    };

    let (first, fight) = run("authored", &[
        "--world", "Daventry", "--at", "277744,199295,13633",
        "--command", "move bossprig 0 277744 199295 13633 0 0 0,5",
        "--console", "kill bossprig,10", "--key", "popup:autohelp,0",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=BossSkin:enter:release:escape,110",
        "--updates", "140", "--frames", "900",
    ]);
    assert!(fight.contains("bossprig: console kill: dies as from a normal hit (ours)")
        && fight.contains("bossprig: its death ended (rule 32)"),
        "ordinary Bossprig death did not finish; inspect {}", root.display());
    assert!(first.contains("corpse of bossprig: spdeath.kqs drawn")
        && first.contains("skin pairs [(\"test2.bmp\", \"test3.bmp\")], 1 material(s) changed"),
        "authored skin was not drawn on its corpse; inspect {}", root.display());
    let slot: serde_json::Value = serde_json::from_slice(&fs::read(slots.join("slot-1.json")).unwrap()).unwrap();
    assert_eq!(slot["kept"]["daventry"]["corpses"][0]["skins"], serde_json::json!([["test2.bmp", "test3.bmp"]]));

    let (restored, _) = run("restore", &[
        "--menu-input", "down,enter,release,enter,release", "--updates", "30", "--frames", "600",
    ]);
    assert!(restored.contains("save restored in Daventry without replaying Begin")
        && restored.contains("corpse of bossprig: spdeath.kqs drawn")
        && restored.contains("skin pairs [(\"test2.bmp\", \"test3.bmp\")], 1 material(s) changed"),
        "restored corpse lost its skin; inspect {}", root.display());

    let (changed, _) = run("scripted", &[
        "--world", "Daventry", "--at", "217361,284013,24650", "--console", "KQObject::changeBmp bossprig test3.bmp test2.bmp,5",
        "--command", "KQMonster::setState bossprig Die now,10", "--updates", "80", "--frames", "600",
    ]);
    assert!(changed.contains("skin pairs [(\"test2.bmp\", \"test3.bmp\"), (\"test3.bmp\", \"test2.bmp\")], 0 material(s) changed"),
        "script replacement was not kept; inspect {}", root.display());

    let (normal, _) = run("normal", &[
        "--world", "Daventry", "--at", "217361,284013,24650", "--console", "KQObject::changeBmp bossprig test3.bmp test2.bmp,5",
        "--console", "KQObject::changeBmp bossprig normal,7",
        "--command", "KQMonster::setState bossprig Die now,10", "--updates", "80", "--frames", "600",
    ]);
    assert!(normal.contains("skin pairs [(\"test2.bmp\", \"test3.bmp\")], 1 material(s) changed"),
        "normal did not restore authored skin; inspect {}", root.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
