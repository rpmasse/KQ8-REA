//! Original-data Fire and Ice shots follow their distinct death and corpse paths.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a graphics device"]
fn flame_bow_and_ice_crossbow_use_their_original_projectiles_and_deaths() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/elemental-ranged-tests/{nonce}"
    ));

    for (item, projectile, emitter, particles, kind, target, movement, corpse) in [
        (
            "FlameBow",
            "Prfarrow.kq",
            "firebits.kq",
            4,
            "Fire",
            "skeleton2",
            "firedie",
            None,
        ),
        (
            "IceCrossbow",
            "PRicycle.kq",
            "icebits.kq",
            8,
            "Ice",
            "skeleton1",
            "icedie",
            Some("skmided.kqs"),
        ),
    ] {
        let run = root.join(item);
        fs::create_dir_all(&run).unwrap();
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let move_target = format!("move {target} 0 183300 45000 864 0 0 0,5");
        let aim = format!("{target},60");
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&run)
            .arg("--game")
            .arg(&game)
            .args([
                "--windowed",
                "--sound",
                "off",
                "--no-ai",
                "--god",
                "--world",
                "DeadCity",
            ])
            .args([
                "--command",
                &give,
                "--command",
                &move_target,
                "--command",
                "KQConner::setDoMode 1,35",
                "--aim",
                &aim,
                "--updates",
                "180",
            ])
            .output()
            .unwrap();
        fs::write(run.join("stderr.log"), &output.stderr).unwrap();
        let logs = run.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            run.display()
        );

        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        let effects = fs::read_to_string(logs.join("effects.log")).unwrap();
        assert!(effects.lines().any(|line| line.contains("[bolt1:")
            && line.contains(&format!("from {emitter} with {particles} particles"))),
            "{item} did not create its original effect; inspect {}", run.display());
        assert!(effects.contains("[bolt1:") && effects.contains("bolt bolt1 ended: dying")
            && effects.contains("all particles expired: removed"),
            "{item} effect did not finish after impact; inspect {}", run.display());
        let shot = format!("shoots bolt1 ({projectile}, 6000 units/s = 400 per update, range 12000, mask 0x1F, {kind})");
        assert_eq!(
            fight.matches(&shot).count(),
            1,
            "wrong weapon projectile; inspect {}",
            run.display()
        );
        assert!(
            fight.contains("bolt1 update "),
            "bolt did not move; inspect {}",
            run.display()
        );
        assert_eq!(
            fight.matches(&format!("touches {target}")).count(),
            1,
            "bolt missed; inspect {}",
            run.display()
        );
        assert_eq!(
            fight
                .matches(&format!("{target}: hit by Connor, {kind}:"))
                .count(),
            1,
            "wrong damage kind; inspect {}",
            run.display()
        );
        assert_eq!(
            fight
                .matches(&format!(
                    "{target}: dies (rule 30): kind {kind}, '{movement}' starts"
                ))
                .count(),
            1,
            "wrong death movement; inspect {}",
            run.display()
        );
        let death = fight.lines().find(|line| line.contains(&format!("{target}: dies (rule 30): kind {kind}"))).unwrap();
        assert!(death.contains("gravity off; not solid (collision type and mask 0); its state never changes again"),
            "{kind} death did not clear gravity and collision; inspect {}", run.display());
        assert_eq!(
            fight
                .matches(&format!("{target}: its death ended (rule 32):"))
                .count(),
            1,
            "death did not finish exactly once; inspect {}",
            run.display()
        );
        match corpse {
            Some(shape) => {
                assert!(
                    fight.contains(&format!("a corpse {shape} where it stood")),
                    "inspect {}",
                    run.display()
                );
                assert!(
                    viewer.contains(&format!("corpse of {target}: {shape} drawn")),
                    "inspect {}",
                    run.display()
                );
            }
            None => assert!(
                fight.contains("no corpse (Fire: burnt away)"),
                "inspect {}",
                run.display()
            ),
        }
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn fire_arrow_effect_follows_flight_and_finishes_after_range_removal() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let run = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/elemental-range-tests/{nonce}"));
    fs::create_dir_all(&run).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&run).arg("--game").arg(&game)
        .args(["--windowed", "--renderer", "dx12", "--sound", "off", "--no-ai", "--god",
            "--world", "DeadCity", "--command", "KQMonster::inventory Connor INVITEM_FlameBow 1,20",
            "--command", "KQConner::setDoMode 1,35", "--aim-point", "182000,55000,1800,60",
            "--updates", "150"])
        .output().unwrap();
    fs::write(run.join("stdout.log"), &output.stdout).unwrap();
    fs::write(run.join("stderr.log"), &output.stderr).unwrap();
    let logs = run.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    assert!(output.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", run.display());
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let effects = fs::read_to_string(logs.join("effects.log")).unwrap();
    assert!(doors.contains("the aim pick meets the drawn ground at")
        && doors.contains("shot click at the drawn ground"),
        "the Fire-arrow click did not use the drawn-ground aim fallback; inspect {}", run.display());
    assert!(fight.contains("shoots bolt1 (Prfarrow.kq")
        && fight.contains("bolt1 update 30:") && fight.contains("more than its range 12000"),
        "Fire bolt did not reach range removal; inspect {}", run.display());
    assert_eq!(effects.matches("started firebits from firebits.kq with 4 particles").count(), 1);
    let follow = effects.lines().find(|line| line.contains("follows bolt bolt1:")
        && line.contains("particles drawn") && line.contains("update 90 ")).unwrap();
    let bolt = fight.lines().find(|line| line.contains("update 90:")
        && line.contains("bolt1 update ")).unwrap();
    let triple = |line: &str, start: &str, end: char| -> [f32; 3] {
        let values = line.split_once(start).unwrap().1.split_once(end).unwrap().0;
        let numbers: Vec<f32> = values.split(',').map(|s| s.trim().parse().unwrap()).collect();
        numbers.try_into().unwrap()
    };
    let at = triple(bolt, "at (", ')');
    let loc = triple(follow, "loc [", ']');
    assert!(at.iter().zip(loc).all(|(a, b)| (*a - b).abs() < 1.0),
        "emitter origin did not follow the bolt at update 90: {at:?} vs {loc:?}");
    assert!(effects.contains("bolt bolt1 ended: dying")
        && effects.contains("[bolt1:firebits] all particles expired: removed"),
        "the particle tail did not finish; inspect {}", run.display());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
