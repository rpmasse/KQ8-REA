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
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .arg("--game")
            .arg(&game)
            .args([
                "--world",
                world,
                "--windowed",
                "--renderer",
                "dx12",
                "--sound",
                "off",
                "--at",
                place,
                "--updates",
                updates,
            ])
            .output()
            .unwrap();
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
