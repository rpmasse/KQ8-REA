//! A live Temple 4 fight pushes the protected Lucreto into the vortex.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn temple_sword_staggers_lucreto_into_the_final_trap() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/live-lucreto-finale/{nonce}"));
    fs::create_dir_all(&root).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
    command.current_dir(&root).arg("--game").arg(&game).args([
        "--windowed",
        "--sound",
        "off",
        "--world",
        "temple4",
        "--command",
        "sendEvent 0 altar KQCueEvent 6 7,5",
        "--command",
        "move Connor 0 101180 96100 5938 0 0 0,20",
        "--command",
        "KQMonster::inventory Connor INVITEM_Invulnerable 1,20",
        "--command",
        "move Connor 0 101180 95000 5938 0 0 0,170",
        "--command",
        "move Connor 0 101180 95000 5938 0 0 0,295",
        "--command",
        "KQMonster::inventory Connor INVITEM_Invulnerable 1,320",
        "--key",
        "5,25",
        "--key",
        "5,330",
        "--command",
        "KQConner::setDoMode 0,35",
    ]);
    for update in (60..=600).step_by(20) {
        command.args(["--aim", &format!("LucEnd,{update}")]);
    }
    let output = command
        .args(["--updates", "730", "--frames", "4300"])
        .output()
        .unwrap();
    fs::write(root.join("stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("stderr.log"), &output.stderr).unwrap();
    let logs = root.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let traps = fs::read_to_string(logs.join("traps.log")).unwrap();
    let items = fs::read_to_string(logs.join("items.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        root.display()
    );
    assert!(
        items.contains("INVITEM_Invulnerable, slot used): the shield: protection 82 -> 100"),
        "the ordinary shield item did not apply; inspect {}",
        root.display()
    );
    let close_staggers = fight
        .lines()
        .filter(|line| {
            line.contains("LucEnd: hit by Connor, Normal:")
                && line.contains("flinches: 'reactback' starts at once")
        })
        .count();
    assert!(
        fight.contains("Connor: hit by LucEnd, Normal:")
            && close_staggers >= 2
            && !fight.contains("god mode is on")
            && !fight.contains("Connor: dies"),
        "the damage-enabled sword fight did not survive; inspect {}",
        root.display()
    );
    let trapped = traps
        .find("trap vortexTrap: On LucEnd")
        .expect("LucEnd entered the vortex");
    let first = traps
        .find("loadAnim lsuck1: playing lu_suck1.kqs")
        .expect("first finale movement");
    let second = traps
        .find("loadAnim lsuck2: playing lu_suck2.kqs")
        .expect("second finale movement");
    let goodbye = traps
        .find("loadAnim vortexbye: playing vrtexbye.kqs")
        .expect("vortex exit");
    let end = traps
        .find("loadAnim imdone: playing c_imdone.kqs")
        .expect("last finale movement");
    assert!(
        trapped < first && first < second && second < goodbye && goodbye < end,
        "the scripted finale did not advance in order; inspect {}",
        root.display()
    );
    assert!(
        viewer.contains("ending: killGame disabled gameplay"),
        "the final movement did not hand off to the ending; inspect {}",
        root.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
