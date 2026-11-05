//! Exercise the original Options entry at startup, then again from a live
//! paused region. The two processes share only their recreation options.
use kq8_core::options_menu::Settings;
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn original_options_controls_reach_the_world_mixer_and_caption_switch() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
    let original_name = kq8_data::GameDir::open(&game)
        .unwrap()
        .find("Options.cs")
        .unwrap();
    let original = fs::read(&original_name).unwrap();
    let expected_captions = !Settings::from_script(&original).subtitles;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/menu-tests/options-{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let run = |name: &str, args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .env("KQ8_SAVE_DIR", dir.join("saves"))
            .arg("--game")
            .arg(&game)
            .args(["--sound", "off"])
            .args(args)
            .output()
            .unwrap();
        fs::write(dir.join(format!("{name}.stderr.log")), &output.stderr).unwrap();
        let log = fs::read_to_string(dir.join("kq8-artifacts/logs/viewer.log")).unwrap();
        fs::write(dir.join(format!("{name}.viewer.log")), &log).unwrap();
        assert!(
            output.status.success() && log.contains("RESULT: PASS"),
            "inspect {name} in {}",
            dir.display()
        );
        log
    };
    let startup=run("startup",&["--menu-input","down,enter,release,click=330x198,drag=525x198,mouse-up,click=344x228,mouse-up,click=330x258,mouse-up,click=481x285,click=315x451","--frames","58"]);
    assert!(
        startup.contains("options: back to main")
            && startup.contains("theme gain 0.00")
            && startup.contains("theme gain 1.00")
    );
    let prefs_file = dir.join("kq8-artifacts/Options.cs");
    let saved = Settings::from_script(&fs::read(&prefs_file).unwrap());
    assert_eq!(saved.volume, [100, 10, 0]);
    assert_eq!(saved.subtitles, expected_captions);

    let world=run("world",&["--world","Daventry","--key",
        "menu:down:down:down:enter:release:click=330x198:mouse-up:click=481x285:click=315x451:escape,12",
        "--command","KQMonster::speak Wizard Connor 1000 4 0 8 1 1,24",
        "--key","menu:down:down:down:enter:release:click=481x285:click=315x451:escape,42","--frames","240"]);
    for (gains, captions) in [
        ("[0.1, 0.1, 0.1, 0.0, 1.0, 1.0]", expected_captions),
        ("[0.1, 0.1, 0.1, 0.0, 0.0, 0.0]", !expected_captions),
    ] {
        assert!(
            world.contains(&format!(
                "Daventry world mixer sliders {gains}, active captions Some(\"{captions}\")"
            )),
            "inspect world.viewer.log in {}",
            dir.display()
        );
    }
    // The requested update can be followed by other fixed updates in the
    // same render frame before the menu opens. Compare the actual pause
    // and resume clocks and make sure the two pauses bracket the line.
    let marker = "menu: leaving at update Some(";
    let pauses: Vec<_> = world
        .match_indices(marker)
        .map(|(pos, _)| {
            let (resume, entry) = world[pos + marker.len()..]
                .lines()
                .next()
                .unwrap()
                .split_once("), entered at ")
                .unwrap();
            let tick = resume.parse::<u64>().unwrap();
            assert_eq!(
                entry.parse::<u64>().unwrap(),
                tick,
                "world clock moved in a menu"
            );
            (pos, tick)
        })
        .collect();
    assert_eq!(pauses.len(), 2);
    assert!((12..24).contains(&pauses[0].1));
    assert!((42..148).contains(&pauses[1].1));
    let caption_art = world
        .lines()
        .find(|s| s.starts_with("caption: IDDLG_CAPTION"))
        .expect("original caption renderer loaded")
        .to_ascii_lowercase();
    assert!(caption_art.contains("8gui/console.pft") && caption_art.contains("8gui/stontran.pbm"));
    let (before, after) = world.split_at(pauses[1].0);
    if !expected_captions {
        assert!(before.contains("caption picture shown:"));
        assert!(after.contains("caption picture hidden"));
    }
    let talk = fs::read_to_string(dir.join("kq8-artifacts/logs/talk.log")).unwrap();
    let line = talk
        .lines()
        .find(|l| l.contains("1000 4 0 8 1 starts:"))
        .expect("original Wizard speech started");
    assert!(line.contains("; caption \""));
    assert_eq!(line.contains("(captions off)"), expected_captions);
    assert!(
        !line.contains("; 0 mouth change(s)"),
        "the regional lip file should animate the speaker"
    );
    let audio = fs::read_to_string(dir.join("kq8-artifacts/logs/sound.log"))
        .unwrap()
        .to_ascii_lowercase();
    for (source, group, loudness) in [
        ("daventry/sound/forest.aud", "group 5", "loudness 1.000"),
        ("daventry/english/a0rs0400.081", "group 3", "loudness 0.000"),
    ] {
        assert!(
            audio.lines().any(|l| l.contains("start #")
                && l.contains(source)
                && l.contains(group)
                && l.contains(loudness)),
            "expected {source} in {group}"
        );
    }
    assert!(
        audio.lines().any(|l| l.contains("start #")
            && l.contains("daventry/sound/")
            && l.contains("group 0")),
        "expected a regional sound effect"
    );
    assert_eq!(fs::read(original_name).unwrap(), original);
}
