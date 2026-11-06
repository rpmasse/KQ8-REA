//! The original fuse-box script switches authored building lighting and restores it at Init.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; isolated fuse-box setup"]
fn original_gnome_fuse_switch_and_reload_use_authored_alternate_maps() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/alternate-interior/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part); fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        // This authored baseline must switch even when point lighting is off.
        fs::write(dir.join("kq8-artifacts/Options.cs"), "useDynamicLighting 0\n").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound", "off", "--no-ai", "--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (viewer, fs::read_to_string(logs.join("doors.log")).unwrap())
    };
    let read = |n| -> serde_json::Value { serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap() };
    let check_on = |viewer: &str| {
        for name in ["gnome13", "gnome14"] {
            let lines: Vec<_> = viewer.lines().filter(|line| line.contains(&format!("interior lighting {name}: alternate 1 -> {name}l.dil"))).collect();
            assert_eq!(lines.len(), 1, "inspect {}", root.display());
            assert!(lines[0].contains("dynamic lighting false"));
            let pixels: usize = lines[0].split("; ").nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap();
            assert!(pixels > 0, "the authored stored image must change");
        }
    };
    let quest_on = |save: &serde_json::Value| {
        assert!(save["kept"]["gnome"]["vars"].as_array().unwrap().iter().any(|entry| {
            entry[0].as_str().is_some_and(|name| name.eq_ignore_ascii_case("Gnome::Dragon::lights")) && entry[1] == "on"
        }));
    };
    let (first, first_script) = run("install-fuse", &["--world", "Gnome",
        "--command", "move Connor 0 99200 127000 15525 0 0 1.57,10",
        "--item", "INVITEM_ShardFuse", "--click", "fusebox,15", "--frames", "1700",
        "--key", "menu:down:down:enter:release:enter:release:escape:type=FuseLights:enter:release:down:down:down:enter:release:left:enter:release,60"]);
    check_on(&first);
    assert!(first_script.contains("script fusebox.cs fusebox ConnorAction"));
    assert_eq!(first_script.matches("sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 50000").count(), 1);
    quest_on(&read(1));
    assert!(!read(1)["journey"]["items"].as_array().unwrap().iter().any(|item| item["kind"] == "INVITEM_ShardFuse" && item["quantity"].as_i64().unwrap() > 0));
    let (second, second_script) = run("reload-and-override-permanent", &["--menu-input", "down,enter,release,enter,release", "--frames", "1900",
        "--console", "useAlternateLightmap gnome13 0,10", "--console", "useAlternateLightmap gnome13 0,12",
        "--console", "useAlternateLightmap gnome14 1,15",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=PermanentOverride:enter:release:down:down:down:enter:release:left:enter:release,35"]);
    check_on(&second);
    assert_eq!(second.matches("interior lighting gnome13: alternate 0 -> gnome13.dil").count(), 1);
    assert!(!second.contains("interior lighting gnome14: alternate 0"));
    assert!(!second_script.contains("KQStatsChangeRequestEvent 0 0 0 0 50000"));
    quest_on(&read(2));
    // Permanent scenery reloads fresh; the real quest Init chooses the map,
    // not the previous process's temporary console override.
    let (third, third_script) = run("quest-init-restores-lighting", &["--menu-input", "down,enter,release,down,enter,release", "--frames", "1900",
        "--key", "menu:down:down:down:enter:release:enter:release:escape:type=RestoredFuseLights:enter:release:down:down:down:enter:release:left:enter:release,25"]);
    check_on(&third); assert!(!third_script.contains("KQStatsChangeRequestEvent 0 0 0 0 50000"));
    quest_on(&read(3));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device; isolated temple lighting commands"]
fn temple4_authored_alternate_map_switches_with_dynamic_lighting_on() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original_options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/temple-alternate-interior/{nonce}"));
    fs::create_dir_all(root.join("kq8-artifacts")).unwrap();
    fs::write(root.join("kq8-artifacts/Options.cs"), "useDynamicLighting 1\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer")).current_dir(&root).arg("--game").arg(&game)
        .args(["--world", "temple4", "--sound", "off", "--no-ai", "--god", "--frames", "150",
            "--console", "useAlternateLightmap temple 1,4", "--console", "useAlternateLightmap temple 1,6",
            "--console", "useAlternateLightmap temple 0,10"]).output().unwrap();
    fs::write(root.join("stderr.log"), output.stderr).unwrap();
    let viewer = fs::read_to_string(root.join("kq8-artifacts/logs/viewer.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", root.display());
    for (mode, file) in [(1,"tmpl401r.dil"), (0,"tmpl401.dil")] {
        let lines: Vec<_> = viewer.lines().filter(|line| line.contains(&format!("interior lighting temple: alternate {mode} -> {file}"))).collect();
        assert_eq!(lines.len(), 1, "inspect {}", root.display());
        assert!(lines[0].contains("dynamic lighting true") && !lines[0].contains("; 0 changed pixels"));
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original_options);
}
