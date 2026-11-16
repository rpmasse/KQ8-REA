//! Clarity changes Snowmane's live protection and attack strength during combat.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn clarity_changes_live_snowmane_combat() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/snowmane-live-clarity/{nonce}"));
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
            "--world",
            "IceWorld",
            "--at",
            "362147,488500,0",
        ])
        .args([
            "--key",
            "popup:autohelp,0",
            "--command",
            "KQConner::setLevel 1000,1",
            "--command",
            "sendEvent 0 Connor KQStatsChangeRequestEvent 10000 0 0 0 0,2",
            "--command",
            "KQMonster::inventory Connor INVITEM_FlameSword 1,20",
            "--command",
            "KQMonster::inventory Connor INVITEM_Clarity 1,20",
            "--command",
            "KQConner::setDoMode 0,25",
            "--swing",
            "snowmane,50",
            "--key",
            "7,80",
            "--swing",
            "snowmane,115",
            "--updates",
            "160",
            "--frames",
            "12000",
        ])
        .output()
        .unwrap();
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
        viewer.contains("draw distance: [snowmane] scripted groups: drawn false -> true"),
        "Reveal did not make nearby Snowmane drawable; inspect {}",
        dir.display()
    );
    assert!(
        doors.contains("script snowmane.cs snowmane Reveal True")
            && doors.contains("live combat stats")
            && !doors.contains("ERROR unknown command 'KQObject::setGroupsActive3D"),
        "Clarity did not complete Snowmane's original Reveal script; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("snowmane: hit by Connor, Fire:"))
        .collect();
    assert_eq!(
        hits.len(),
        2,
        "expected two real hits; inspect {}",
        dir.display()
    );
    let update = |line: &str| {
        line.strip_prefix("update ")
            .and_then(|tail| tail.split_once(':'))
            .and_then(|(number, _)| number.parse::<u32>().ok())
            .unwrap()
    };
    assert!(
        update(hits[0]) < 80
            && hits[0].contains("D 350")
            && hits[0].contains("P 30 -> P'")
            && update(hits[1]) > 80
            && hits[1].contains("D 350")
            && hits[1].contains("P 0 -> P' 0"),
        "the two hits did not use the original pre- and post-Reveal stats; inspect {}",
        dir.display()
    );
    let health = |line: &str| {
        let (before, after) = line
            .split_once("; health ")
            .and_then(|(_, tail)| tail.split(';').next())
            .and_then(|s| s.split_once(" -> "))
            .unwrap();
        (
            before.parse::<i64>().unwrap(),
            after.parse::<i64>().unwrap(),
        )
    };
    let (first_before, first_after) = health(hits[0]);
    let (second_before, second_after) = health(hits[1]);
    assert!(
        first_before == 3000
            && first_after < first_before
            && second_before == first_after
            && second_after < second_before
            && second_after > 0,
        "Snowmane did not retain both live hits; inspect {}",
        dir.display()
    );
    let incoming: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("Connor: hit by snowmane, Normal:"))
        .collect();
    assert!(
        incoming.iter().any(|line| update(line) < 80 && line.contains("D 350,"))
            && incoming.iter().any(|line| update(line) > 80 && line.contains("D 200,")),
        "Snowmane did not strike with its pre- and post-Reveal hand damage; inspect {}",
        dir.display()
    );
    for line in incoming {
        let (before, after) = line
            .rsplit_once("health ")
            .and_then(|(_, tail)| tail.split_once(" -> "))
            .unwrap();
        assert!(
            after.parse::<i64>().unwrap() < before.parse::<i64>().unwrap(),
            "Snowmane did not lower Connor's health: {line}"
        );
    }
    assert!(
        !fight.contains("the AI is switched off") && !fight.contains("god mode is on"),
        "test switches obscured live combat; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
