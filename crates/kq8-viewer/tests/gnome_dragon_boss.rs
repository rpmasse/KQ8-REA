//! The installed Gnome Dragon's light prerequisite, live combat and reward.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn gnome_dragon_fights_dies_and_leaves_the_black_diamond() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/gnome-dragon-boss/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--god",
            "--world",
            "Gnome",
            "--at",
            "100117,136500,0",
            // Connor starts on the authored chamber floor. The original
            // Dragon stays at its placed Gnome position throughout.
            "--command",
            "move Connor 0 100117 136500 14901 0 0 0,4",
            // A strong installed weapon keeps this boss test bounded. The
            // ordinary attack, damage roll, death and drop paths still run.
            "--command",
            "KQMonster::inventory Connor INVITEM_TempleSword 1,20",
            "--command",
            "KQConner::setDoMode 0,35",
            // Fusebox Init first sets darkness. Its authored Cue then
            // restores the Dragon's attack collision before Connor fights.
            "--command",
            "sendEvent 0 fusebox KQCueEvent 10 0 on,65",
            "--swing",
            "Dragon,100",
            "--swing",
            "Dragon,130",
            "--swing",
            "Dragon,160",
            "--swing",
            "Dragon,190",
            "--swing",
            "Dragon,220",
            "--swing",
            "Dragon,250",
            "--swing",
            "Dragon,280",
            "--updates",
            "390",
            "--frames",
            "5000",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("collisionType Dragon -64: 0x4002 -> 0x4002")
            && doors.contains("collisionType Dragon +64: 0x4002 -> 0x4042"),
        "the original fuse-box lighting prerequisite failed; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("Dragon: attack (rule 7, standing)")
            && fight.contains("Connor: hit by Dragon, Normal, damage 85"),
        "the Dragon did not attack through ordinary AI; inspect {}",
        dir.display()
    );
    assert!(
        talk.contains("I cannot see my opponent!")
            && doors.contains("script dragworm.cs Dragon EndLoop Attack"),
        "the dark-chamber attack callback did not speak; inspect {}",
        dir.display()
    );
    assert!(
        fight.matches("Dragon: hit by Connor, Normal:").count() >= 4
            && fight.contains("Dragon: dies (rule 30): kind Normal"),
        "ordinary sword hits did not defeat the Dragon; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("Dragon: its death ended (rule 32): Connor gets its experience 25000:")
            .count(),
        1,
        "the Dragon's death or experience completed more than once; inspect {}",
        dir.display()
    );
    assert!(
        items.contains("Dragon: kept weapon (rule 31a):")
            && items.contains("(BlackDiamond x1) laid at"),
        "the Black Diamond did not leave the Die animation; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("script dragworm.cs Dragon Die Normal")
            && doors.contains("addMapIcon gn_dworm.bmp 235 91")
            && doors.contains("script Some(\"blackdmd.cs\")")
            && doors.contains("deleteObject dragblck: placed object deleted"),
        "the authored defeat script did not finish; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
