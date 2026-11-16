//! Two original-data ranged picks: a Cottage occlusion and an open-sky line.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn a_building_stops_the_first_bolt_and_a_sky_click_uses_the_far_end() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/ranged-building-sky/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--world",
            "Daventry",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--no-ai",
            "--at",
            "289284,195000,0",
            "--command",
            "KQMonster::inventory Connor INVITEM_SmCrossbow 1,20",
            "--command",
            "KQConner::setDoMode 1,35",
            "--aim",
            "goblin22,60",
            // Keep the second line clear of the Cottage and the ground.
            "--command",
            "KQCamera::follow none,89",
            "--command",
            "move KQCamera 0 300000 210000 25000 0 0 0,90",
            "--aim-point",
            "300000,210000,40000,110",
            "--updates",
            "160",
            "--frames",
            "7000",
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
        "inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("taken, none (None) -> ranged (Some(\"INVITEM_SmCrossbow\"))"),
        "Small Crossbow was not readied; inspect {}",
        dir.display()
    );
    assert!(
        doors
            .lines()
            .any(|line| line.contains("test --aim goblin22")
                && line.contains("hits building Cottage first")
                && line.contains("the aim pick meets building Cottage (type 0x28)")),
        "the Cottage did not occlude goblin22; inspect {}",
        dir.display()
    );
    assert!(
        fight.lines().any(|line| line.contains("bolt1 update")
            && line.contains("touches building Cottage")
            && line.contains("no damage")),
        "the first bolt did not stop harmlessly at the Cottage; inspect {}",
        dir.display()
    );
    assert!(
        !fight.contains("goblin22: hit by Connor"),
        "the occluded goblin took damage; inspect {}",
        dir.display()
    );
    assert!(
        doors.lines().any(|line| line.contains("test --aim-point")
            && line.contains("the aim pick meets nothing: the line's far end (the sky)")),
        "the second click did not use the open-sky fallback; inspect {}",
        dir.display()
    );
    assert!(
        fight
            .lines()
            .any(|line| line.contains("shoots bolt2 (Prscbolt.kq")
                && line.contains("pitch 32.7 deg")),
        "the sky click did not release a bolt at the steepness limit; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
