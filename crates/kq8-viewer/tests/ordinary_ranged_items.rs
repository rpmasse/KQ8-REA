//! Original-data shot matrix for the five ordinary player bows and crossbows.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn ordinary_bows_and_crossbows_use_their_authored_projectiles() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
        .unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/ordinary-ranged-items/{nonce}"));

    for (item, projectile) in [
        ("SmCpndCBow", "Prsccblt.kq"),
        ("ShortBow", "Prarrow.kq"),
        ("LongBow", "Prarrow.kq"),
        ("CrossBow", "Prcbolt.kq"),
        ("CpndCrossbow", "Prccbolt.kq"),
    ] {
        let run = root.join(item);
        fs::create_dir_all(&run).unwrap();
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&run)
            .arg("--game")
            .arg(&game)
            .args(["--windowed", "--sound", "off", "--no-ai", "--god", "--world", "DeadCity"])
            .args([
                "--command",
                &give,
                "--command",
                "move skeleton1 0 183300 45000 864 0 0 0,5",
                "--command",
                "KQConner::setDoMode 1,35",
                "--aim",
                "skeleton1,60",
                "--frames",
                "130",
            ])
            .output()
            .unwrap();
        fs::write(run.join("stderr.log"), &output.stderr).unwrap();
        let logs = run.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            run.display()
        );
        assert_eq!(
            fight
                .matches(&format!("shoots bolt1 ({projectile},"))
                .count(),
            1,
            "wrong projectile for {item}; inspect {}",
            run.display()
        );
        assert!(fight.contains("bolt1 update "), "no flight; inspect {}", run.display());
        assert_eq!(
            fight.matches("touches skeleton1").count(),
            1,
            "no single contact for {item}; inspect {}",
            run.display()
        );
        assert_eq!(
            fight.matches("skeleton1: hit by Connor, Projectile:").count(),
            1,
            "no Projectile hit for {item}; inspect {}",
            run.display()
        );
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
