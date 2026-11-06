//! Local graphics preferences override the GOG defaults on a fresh launch.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn live_brightness_rebuilds_world_and_open_menu_then_persists() {
    live_palette_case("D3D-display","Direct3D");
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn glide_palette_rules_rebuild_live_and_persist() {
    live_palette_case("Glide","Glide");
}

fn live_palette_case(mode:&str,output_name:&str) {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/live-palette/{nonce}"));
    let run = |name:&str,args:&[&str]| {
        let dir = root.join(name); fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        if name == "sweep" { fs::write(dir.join("kq8-artifacts/Options.cs"),format!("assignGModeName {mode}\nsetBrightness 50\n")).unwrap(); }
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer")).current_dir(&dir).env("KQ8_SAVE_DIR",root.join("slots")).arg("--game").arg(&game)
            .args(["--sound","off","--no-ai","--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"),output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && log.contains("RESULT: PASS"),"inspect {}",dir.display());
        log
    };
    let first = run("sweep",&["--world","DeadCity","--updates","60","--frames","1300",
        "--key","menu:down:down:down:enter:release:enter:release:click=335x388:release:release:release:click=525x388:release:release:release:click=335x388:release:release:release:click=310x450:release:click=315x451:release:escape,12",
        "--key","menu:down:down:enter:release:enter:release:escape:type=Palette:enter:release:escape,30"]);
    assert!(root.join("slots/slot-1.json").is_file(),"the post-brightness game state must be saved");
    let changes:Vec<_> = first.lines().filter(|line| line.contains("palette: live brightness ") && line.contains("(Graphics slider)")).collect();
    assert_eq!(changes.len(),3,"inspect {}",root.display());
    let mut stable = Vec::new();
    for (i,(line,value)) in changes.iter().zip([0,100,0]).enumerate() {
        assert!(line.contains(&format!("live brightness {value} ")) && line.contains("menu open true") && line.contains(&format!("output {output_name}")));
        assert!(line.contains(&format!("serial: {}",i+1)));
        let n = |key:&str| line.split_once(key).unwrap().1.split(|c:char| !c.is_ascii_digit()).next().unwrap().parse::<u64>().unwrap();
        assert!(n("images: ") > 20 && n("changed: ") > 20);
        let pairs:Vec<_> = line.split_once("stable cache Some(((").unwrap().1.split_once(")));").unwrap().0
            .split(|c:char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).map(|s| s.parse::<u64>().unwrap()).collect();
        assert_eq!(pairs.len(),4); assert!(pairs[0]>20 && pairs[0]==pairs[2]); assert_ne!(pairs[1],pairs[3]);
        stable.push((pairs[2],pairs[3]));
    }
    assert_eq!(stable[0],stable[2],"the same cached world images must return to identical 0-brightness bytes");
    let menus:Vec<_> = first.lines().filter(|line| line.contains("palette: menu live brightness ") && line.contains("(Graphics slider)")).collect();
    assert_eq!(menus.len(),3);
    let after = |line:&str| line.split_once(", after Some(").unwrap().1.split(')').next().unwrap().parse::<u64>().unwrap();
    assert_ne!(after(menus[0]),after(menus[1])); assert_eq!(after(menus[0]),after(menus[2]),"static menu art must be reversible while Graphics stays open");
    let saved = fs::read(root.join("sweep/kq8-artifacts/Options.cs")).unwrap();
    assert_eq!(kq8_core::options_menu::Settings::from_script(&saved).graphics.brightness,0);
    fs::create_dir_all(root.join("restore/kq8-artifacts")).unwrap();
    fs::write(root.join("restore/kq8-artifacts/Options.cs"),saved).unwrap();
    let restored = run("restore",&["--menu-input","down,enter,release,enter,release","--updates","20","--frames","1300",
        "--key","menu:down:down:down:down:enter:release:enter:release:click=310x450:release:click=315x451:release:escape,5"]);
    assert!(restored.contains("brightness 0 palette factor 0.300") && restored.contains("gui: fresh palette brightness 0"));
    assert!(restored.contains("save restored in DeadCity without replaying Begin") && restored.contains("graphics: opened original Graphics.gui"));
    let setup = restored.lines().find(|line| line.contains("palette: menu live brightness 0,") && line.contains("(Graphics setup)")).unwrap();
    assert_eq!(after(setup),after(menus[0]),"reopening Graphics after disk load must use the retained brightness and source menu palette");
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(),original);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR, KQ8_FFMPEG and a graphics device"]
fn brightness_is_reapplied_after_original_film_returns_to_menu_and_world() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let decoder = fs::canonicalize(std::env::var_os("KQ8_FFMPEG").expect("set KQ8_FFMPEG")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/live-palette-film/{nonce}"));
    for world in [false,true] {
        let dir = root.join(if world {"world"} else {"menu"}); fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        let mode = if world {"D3D-display"} else {"Glide"};
        fs::write(dir.join("kq8-artifacts/Options.cs"),format!("setBrightness 100\nassignGModeName {mode}\n")).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        command.current_dir(&dir).env("KQ8_FFMPEG",&decoder).arg("--game").arg(&game).args(["--sound","off","--no-ai","--god"]);
        if world {
            command.args(["--world","DeadCity","--updates","20","--frames","5000","--command","playAvi w32mdl_1.dll,5"]);
        } else {
            command.args(["--menu-input","click=314x304,escape,click=314x365,left,enter,release","--frames","1300"]);
        }
        let output = command.output().unwrap(); fs::write(dir.join("stderr.log"),output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && log.contains("RESULT: PASS"),"inspect {}",dir.display());
        assert!(log.contains(if world {"w32mdl_1.dll first picture"} else {"W32opn_1.dll first picture"}),"an actual decoded picture must precede restoration");
        assert!(!log.contains("unavailable:") && !log.contains("decoding failed"));
        let after = log.split_once(if world {"returning to Gameplay"} else {"returning to Menu"}).unwrap().1;
        assert!(after.contains("palette: live brightness 100 ") && after.contains("(return from film)"));
        assert!(after.contains(if world {"output Direct3D"} else {"output Glide"}));
        if world { assert!(log.contains("finished after 255 intervals, 237 decoded pictures")); }
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(),original);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn fresh_brightness_preferences_reach_world_and_interface_palettes() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/palette-brightness/{nonce}"));
    for (value, factor) in [(0, "0.300"), (50, "1.000"), (100, "1.700")] {
        let dir = root.join(value.to_string());
        fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        fs::write(dir.join("kq8-artifacts/Options.cs"), format!("setBrightness {value}\n")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game)
            .args(["--world", "DeadCity", "--frames", "80", "--sound", "off", "--no-ai", "--god"])
            .output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let viewer = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        let lines: Vec<_> = viewer.lines().filter(|l| l.contains(&format!("brightness {value} palette factor {factor}"))).collect();
        assert!(!lines.is_empty(), "brightness did not reach decoded world palettes; inspect {}", dir.display());
        if value == 50 { assert!(lines.iter().all(|l| l.contains("0 entries changed"))); }
        else { assert!(lines.iter().any(|l| !l.contains(", 0 entries changed"))); }
        assert!(viewer.contains(&format!("gui: fresh palette brightness {value}; display gamma not applied")));
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn graphics_menu_switches_live_shadow_categories_off_and_on() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/shadow-menu/{nonce}"));
    fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
    fs::write(dir.join("kq8-artifacts/Options.cs"), b"setShadowMask 1\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir).arg("--game").arg(&game)
        .args(["--world", "DeadCity", "--frames", "350", "--sound", "off", "--no-ai", "--god",
            "--command", "move skeleton1 0 183300 45000 864 0 0 0,5",
            "--key", "menu:down:down:down:enter:release:enter:release:click=350x295:click=350x325:click=350x355:click=310x450:click=315x451:escape,12",
            "--key", "menu:down:down:down:enter:release:enter:release:click=350x265:click=350x295:click=350x325:click=350x355:click=310x450:click=315x451:escape,40",
            "--key", "menu:down:down:down:enter:release:enter:release:click=350x265:click=350x295:click=350x325:click=350x355:click=310x450:click=315x451:escape,65"])
        .output().unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
    let rows: Vec<_> = viewer.lines().filter(|l| l.starts_with("shadow renderer: mask ")).collect();
    for mask in [3, 7, 15, 14, 12, 8, 0, 1, 3, 7, 15] {
        assert!(viewer.contains(&format!("Graphics menu: shadow mask {mask} applied live")), "mask {mask} missing; inspect {}", dir.display());
    }
    assert!(rows.iter().any(|l| l.contains("mask 0, Connor 0 pieces, 0 character shadows shown")), "disabled shadows survived; inspect {}", dir.display());
    let full: Vec<_> = rows.iter().filter(|l| l.contains("mask 15,")).collect();
    assert_eq!(full.len(), 2, "inspect {}", dir.display());
    for row in full {
        assert!(!row.contains("Connor 0 pieces") && !row.contains(", 0 character shadows shown"), "eligible shadows were not restored: {row}; inspect {}", dir.display());
    }
    let shadows = fs::read_to_string(logs.join("shadows.log")).unwrap();
    let zero = shadows.lines().find(|l| l.ends_with("; mask 0")).unwrap();
    assert!(zero.contains("0 plain objects' shadows shown, 0 pieces"), "inspect {}", dir.display());
    let enabled: Vec<_> = shadows.lines().filter(|l| l.ends_with("; mask 15")).collect();
    assert_eq!(enabled.len(), 2, "inspect {}", dir.display());
    let rebuilds = |line: &str| line.split("worked out ").nth(1).unwrap().split_whitespace().next().unwrap().parse::<usize>().unwrap();
    assert!(rebuilds(enabled[0]) > 0 && rebuilds(enabled[1]) > rebuilds(enabled[0]),
        "re-enabling objects reused their disabled cache; inspect {}", dir.display());
    let saved = kq8_core::options_menu::Settings::from_script(&fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap());
    assert_eq!(saved.graphics.shadow_mask, 15);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn local_resolution_lighting_and_shadow_settings_reach_world_loading() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/graphics-preferences/{nonce}"));
    for (name, local, screen, dynamic, mask) in [
        ("installed", None, "640x480", "useDynamicLighting 0", "world mask 1"),
        ("local", Some(b"set800mode 1\nuseDynamicLighting 1\nsetShadowMask 15\n".as_slice()),
            "800x600", "useDynamicLighting 1", "world mask 15"),
    ] {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("kq8-artifacts")).unwrap();
        if let Some(script) = local { fs::write(dir.join("kq8-artifacts/Options.cs"), script).unwrap(); }
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game)
            .args(["--world", "DeadCity", "--frames", "80", "--sound", "off", "--no-ai", "--god"])
            .output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        assert!(viewer.contains(&format!("screen {screen}")) && viewer.contains(dynamic),
            "resolution or terrain light map ignored the options; inspect {}", logs.display());
        let shadows = fs::read_to_string(logs.join("shadows.log")).unwrap();
        assert!(shadows.contains(mask), "shadows ignored the options; inspect {}", logs.display());
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original, "the GOG installation was modified");
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn graphics_menu_saves_choices_for_a_fresh_world_process() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let original = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/graphics-menu/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game).args(["--sound", "off"]).args(args)
            .output().unwrap();
        fs::write(dir.join(format!("{name}.stderr.log")), &output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        fs::write(dir.join(format!("{name}.viewer.log")), &log).unwrap();
        assert!(output.status.success() && log.contains("RESULT: PASS"), "inspect {}", dir.display());
        log
    };
    let menu = run("menu", &["--menu-input", "down,enter,release,enter,click=350x190,click=350x232,click=350x263,click=525x387,mouse-up,click=310x450,click=315x451", "--frames", "70"]);
    assert!(menu.contains("graphics: opened original Graphics.gui") && menu.contains("graphics: returned to Options; saved local preferences"), "inspect {}", dir.display());
    let saved = kq8_core::options_menu::Settings::from_script(&fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap());
    assert!(saved.graphics.high_resolution && saved.graphics.dynamic_lighting);
    assert_eq!((saved.graphics.shadow_mask, saved.graphics.brightness), (0, 100));
    assert_eq!(saved.graphics.mode, "D3D-display");
    let world = run("world", &["--world", "DeadCity", "--frames", "80", "--no-ai", "--god"]);
    assert!(world.contains("screen 800x600") && world.contains("useDynamicLighting 1"), "inspect {}", dir.display());
    assert!(fs::read_to_string(dir.join("kq8-artifacts/logs/shadows.log")).unwrap().contains("world mask 0"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), original);
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn graphics_menu_switches_dynamic_lighting_in_a_paused_world() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/graphics-live/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).arg("--game").arg(&game).args(["--sound", "off", "--no-ai", "--god"]).args(args)
            .output().unwrap();
        fs::write(dir.join(format!("{name}.stderr.log")), &output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        fs::write(dir.join(format!("{name}.viewer.log")), &log).unwrap();
        assert!(output.status.success() && log.contains("RESULT: PASS"), "inspect {}", dir.display());
        log
    };
    let live = run("live", &["--world", "DeadCity", "--frames", "180", "--key",
        "menu:down:down:down:enter:release:enter:release:click=350x232:click=310x450:click=315x451:escape,12"]);
    assert!(live.contains("Options.cs useDynamicLighting 0"), "inspect {}", dir.display());
    assert_eq!(live.matches("Graphics menu: dynamic lighting on applied live (ground shows its shadow map").count(), 1,
        "the live light map should switch once; inspect {}", dir.display());
    let (before, after) = live.split_once("Graphics menu: dynamic lighting on applied live").unwrap();
    fn sample(log: &str) -> Option<&str> { log.lines().find(|line| line.starts_with("ground light x1: Connor")) }
    assert_ne!(sample(before), sample(after), "the displayed ground light did not change; inspect {}", dir.display());
    assert!(after.contains("ground point lights: 8 patch(es)"), "live point lights did not start; inspect {}", dir.display());
    let saved = kq8_core::options_menu::Settings::from_script(&fs::read(dir.join("kq8-artifacts/Options.cs")).unwrap());
    assert!(saved.graphics.dynamic_lighting);
    let fresh = run("fresh", &["--world", "DeadCity", "--frames", "80"]);
    assert!(fresh.contains("Options.cs useDynamicLighting 1"), "fresh launch ignored the saved choice; inspect {}", dir.display());
}
