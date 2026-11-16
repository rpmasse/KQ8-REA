//! The installed Daventry Henchman refuses bolts but takes ordinary axe damage.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn henchman_laughs_at_a_bolt_and_loses_health_to_an_axe() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/henchman-projectile-refusal/{nonce}"
    ));

    let run = |name: &str, connor_x: i32, weapon: &str, mode: &str, input: &str| {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let move_connor = format!("move Connor 0 {connor_x} 250000 15045 0 0 -1.57,5");
        let give = format!("KQMonster::inventory Connor INVITEM_{weapon} 1,20");
        let ready = format!("KQConner::setDoMode {mode},35");
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .arg("--game")
            .arg(&game)
            .args([
                "--windowed",
                "--sound",
                "off",
                "--god",
                "--world",
                "Daventry",
                "--command",
                &move_connor,
                "--command",
                "KQMonster::setHome Henchman 137200 250000 15045,5",
                "--command",
                "move Henchman 0 137200 250000 15045 0 0 1.57,5",
                "--command",
                &give,
                "--command",
                &ready,
                input,
                "Henchman,70",
                "--updates",
                "125",
            ])
            .output()
            .unwrap();
        fs::write(dir.join("stderr.log"), output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (
            fs::read_to_string(logs.join("fight.log")).unwrap(),
            fs::read_to_string(logs.join("doors.log")).unwrap(),
        )
    };

    // Both runs use the original actor and the same authored post-scene placement.
    // The closer hand-weapon start is within Connor's 2350-unit swing reach.
    let (bolt, script) = run("bolt", 139600, "SmCrossbow", "1", "--aim");
    assert!(
        bolt.contains("shot click at character Henchman (type 0x4042)")
            && bolt.contains("touches Henchman")
            && bolt.contains(
                "its type 0x4042 has the immune bit 0x4000: a hit of 0, its script gets ProjImmune"
            ),
        "the bolt must contact the original immune actor; inspect {}",
        root.display()
    );
    let bolt_hits: Vec<_> = bolt
        .lines()
        .filter(|line| line.contains("Henchman: hit by Connor, Projectile:"))
        .collect();
    assert_eq!(bolt_hits.len(), 1, "inspect {}", root.display());
    assert!(
        bolt_hits[0].contains("loss 0") && bolt_hits[0].contains("health 300 -> 300"),
        "projectile immunity must preserve health: {}",
        bolt_hits[0]
    );
    assert!(
        script.contains("script henchman.cs Henchman ProjImmune")
            && script.contains("KQMonster::setState special laugh")
            && script.contains("Henchman: Special: 'laugh' ended")
            && script.contains("script henchman.cs Henchman EndLoop laugh"),
        "the installed script must play and finish laugh; inspect {}",
        root.display()
    );

    let (axe, _) = run("axe", 139200, "SmallAx", "0", "--swing");
    assert!(
        axe.contains("Connor: Do (conaxe.anm/attack): hits Henchman"),
        "the ordinary axe swing did not contact; inspect {}",
        root.display()
    );
    let hand_hits: Vec<_> = axe
        .lines()
        .filter(|line| line.contains("Henchman: hit by Connor, Normal:"))
        .collect();
    assert_eq!(hand_hits.len(), 1, "inspect {}", root.display());
    let (before, after) = hand_hits[0]
        .split_once("; health ")
        .and_then(|(_, tail)| tail.split(';').next())
        .and_then(|health| health.split_once(" -> "))
        .map(|(before, after)| {
            (
                before.parse::<i64>().unwrap(),
                after.parse::<i64>().unwrap(),
            )
        })
        .unwrap();
    assert!(before == 300 && after < before, "{}", hand_hits[0]);
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
