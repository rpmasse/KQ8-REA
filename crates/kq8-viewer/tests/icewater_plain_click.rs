//! The installed Ice World door reveals a plain-click target for a ready bow.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn installed_icewater_receives_a_plain_click_instead_of_a_crossbow_shot() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/icewater-plain-click/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "IceWorld",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--no-ai",
            "--at",
            "270710,485550,0",
            "--command",
            "move Connor 0 270710 485550 31660 0 0 0,4",
            "--click",
            "ildorlok,10",
            "--command",
            "KQMonster::inventory Connor INVITEM_SmCrossbow 1,170",
            "--command",
            "KQConner::setDoMode 1,175",
            "--aim",
            "icewater,190",
            "--updates",
            "220",
            "--frames",
            "3500",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "viewer failed; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("click ildorlok: Connor at")
            && doors.contains("script ildorlok.cs ildorlok ConnorAction do do")
            && doors.contains("loadKQ icewater.kq: icewater comes into the world")
            && doors.contains("icewater can be clicked now"),
        "the installed door did not reveal icewater; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("icewater.cs line 97: handsOff 0")
            && doors.contains("taken, none (None) -> ranged (Some(\"INVITEM_SmCrossbow\"))"),
        "the cutscene did not return control with the crossbow ready; inspect {}",
        dir.display()
    );
    assert!(
        doors
            .lines()
            .any(|line| line.contains("test --aim icewater")
                && line.contains("hits icewater first")
                && line.contains("talk pick meets icewater (type 0xC020): the plain click")),
        "the installed icewater object did not win the talk pick; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("script icewater.cs icewater ConnorAction do do")
            && doors.contains("icewater.cs line 9: KQMonster::speak Connor to Connor"),
        "the plain click did not call icewater's authored script; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("shoots bolt") && !fight.contains("Connor: do (conscbow.anm/attack"),
        "the icewater click incorrectly fired the crossbow; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
