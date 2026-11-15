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

    for (item, projectile, kind, target, movement, corpse) in [
        (
            "FlameBow",
            "Prfarrow.kq",
            "Fire",
            "skeleton2",
            "firedie",
            None,
        ),
        (
            "IceCrossbow",
            "PRicycle.kq",
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
                "--frames",
                "130",
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
