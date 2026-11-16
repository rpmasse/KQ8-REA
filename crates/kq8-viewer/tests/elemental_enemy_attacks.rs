//! Original-data enemy Fire and Ice projectiles damage Connor without god mode.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn barren_flame_bow_and_iceworld_orc_bow_damage_connor() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/elemental-enemy-attacks/{nonce}"
    ));

    let run = |world: &str, place: &str, updates: &str| {
        let dir = root.join(world);
        fs::create_dir_all(&dir).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        command.current_dir(&dir).arg("--game").arg(&game).args([
                "--world",
                world,
                "--windowed",
                "--renderer",
                "dx12",
                "--sound",
                "off",
                "--at",
                place,
            ]);
        if world == "Barren" {
            // This spawn can land in a continuous Fire terrain volume. Keep Connor
            // alive long enough to observe both bow starts and their contacts.
            command.args([
                "--command", "KQConner::setLevel 10000,1",
                "--command", "sendEvent 0 Connor KQStatsChangeRequestEvent 100000 0 0 0 0,2",
            ]);
        }
        let output = command.args(["--updates", updates]).output().unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        (
            fs::read_to_string(logs.join("fight.log")).unwrap(),
            fs::read_to_string(logs.join("connor.log")).unwrap(),
        )
    };

    let check_health_loss = |fight: &str, shooter: &str, kind: &str, damage: i32| {
        let marker = format!("Connor: hit by {shooter}, {kind}: D {damage},");
        let line = fight
            .lines()
            .find(|line| line.contains(&marker))
            .unwrap_or_else(|| panic!("no {kind} hit by {shooter}; inspect {}", root.display()));
        let transition = line.rsplit_once("health ").unwrap().1;
        let (before, after) = transition.split_once(" -> ").unwrap();
        let before: i32 = before.parse().unwrap();
        let after: i32 = after.parse().unwrap();
        assert!(
            before > after,
            "{shooter} did not lower Connor's health: {line}"
        );
    };

    let (barren, _) = run("Barren", "22495,34877,0", "100");
    for name in ["eledwarf1", "walkdwarf2"] {
        let shots: Vec<_> = barren
            .lines()
            .filter(|line| line.contains(&format!("{name}: Do (Attack")))
            .collect();
        assert!(shots.len() >= 2, "{name} did not fire twice; inspect {}", root.display());
        assert!(shots[0].contains("start offset (-247, 171, 804): its part 'arrow' in frame 3 of 'Attack'")
            && shots[0].contains("shoots bolt"),
            "{name} did not launch from the installed arrow part at Do; inspect {}", root.display());
        assert!(shots[1].contains("start offset (-247, 171, 804): kept from an earlier shot")
            && shots[1].contains("shoots bolt"),
            "{name} did not retain the arrow start; inspect {}", root.display());
    }
    assert!(barren.lines().any(|line| {
        line.contains("eledwarf1: bolt")
            && line.contains("touches Connor")
            && line.contains("a hit of 135, Fire")
    }));
    check_health_loss(&barren, "eledwarf1", "Fire", 135);
    assert!(barren.lines().any(|line| {
        line.contains("walkdwarf2: bolt")
            && line.contains("touches building sublayer")
            && line.contains("no damage")
    }));
    assert!(!barren.contains("Connor: hit by walkdwarf2, Fire:"));

    let (iceworld, connor) = run("IceWorld", "268329,480983,0", "200");
    let guard_shots: Vec<_> = iceworld
        .lines()
        .filter(|line| line.contains("WLGuard1B: Do (Attack"))
        .collect();
    assert!(guard_shots.len() >= 2, "the orc bow guard did not fire twice; inspect {}", root.display());
    assert!(guard_shots[0].contains("start offset (-3, -1048, 1032): its part 'arrow' in frame 3 of 'Attack'")
        && guard_shots[0].contains("shoots bolt"),
        "the orc bow guard did not launch from its installed arrow part at Do; inspect {}", root.display());
    assert!(guard_shots[1].contains("start offset (-3, -1048, 1032): kept from an earlier shot")
        && guard_shots[1].contains("shoots bolt"),
        "the orc bow guard did not retain its arrow start; inspect {}", root.display());
    assert!(iceworld.lines().any(|line| {
        line.contains("WLGuard1B: bolt")
            && line.contains("touches Connor")
            && line.contains("a hit of 155, Ice")
    }));
    check_health_loss(&iceworld, "WLGuard1B", "Ice", 155);
    assert_eq!(
        iceworld
            .matches("dies (Ice): first dropweapon, final icedie")
            .count(),
        1
    );
    assert!(connor.contains("icedie finished: Die state holds"));
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
