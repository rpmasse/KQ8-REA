//! Original-data first-person attack matrix for Connor's ordinary weapons.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn first_person_weapons_use_their_authored_attack_lists() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/first-person-weapon-matrix/{nonce}"
    ));

    let run = |item: &str, mode: &str| {
        let dir = root.join(item);
        fs::create_dir_all(&dir).unwrap();
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let ready = format!("KQConner::setDoMode {mode},35");
        let mut command = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        command.current_dir(&dir).arg("--game").arg(&game).args([
            "--world",
            "DeadCity",
            "--windowed",
            "--renderer",
            "dx12",
            "--sound",
            "off",
            "--no-ai",
            "--god",
            "--command",
            "move skeleton1 0 183300 45000 864 0 0 0,5",
        ]);
        command.args(["--command", &give]);
        command.args(["--command", &ready, "--key", "physical:f2,80"]);
        if mode == "0" {
            command.args(["--swing", "skeleton1,95"]);
        } else {
            command.args(["--aim", "skeleton1,95"]);
        }
        let output = command.args(["--updates", "160"]).output().unwrap();
        fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "{item}: inspect {}",
            dir.display()
        );
        assert!(
            viewer.contains("CameraFirstPerson: first person true"),
            "{item}: first person did not activate; inspect {}",
            dir.display()
        );
        (fs::read_to_string(logs.join("fight.log")).unwrap(), dir)
    };

    for (item, list, kind) in [
        ("Fists", "confisfp.anm", "Normal"),
        ("BroadSword", "conbsfp.anm", "Normal"),
        ("SmallAx", "conaxefp.anm", "Normal"),
        ("Mace", "conmcefp.anm", "Normal"),
        ("SkelKingSword", "conkngfp.anm", "Normal"),
        ("LakeSword", "conswdfp.anm", "Normal"),
        ("Dagger", "condagfp.anm", "Normal"),
        ("WarHammer", "conwhfp.anm", "Normal"),
        ("LongSword", "conlngfp.anm", "Normal"),
        ("BattleAx", "conbaxfp.anm", "Normal"),
        ("Pike", "conbpkfp.anm", "Normal"),
        ("TempleSword", "contmsfp.anm", "Normal"),
        ("FlameSword", "confsfp.anm", "Fire"),
    ] {
        let (fight, dir) = run(item, "0");
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&format!("Connor: Do ({list}/attack"))
                    && line.contains("hits skeleton1")),
            "{item}: no authored first-person melee hit; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("skeleton1: hit by Connor, {kind}:"))
                .count(),
            1,
            "{item}: wrong melee damage; inspect {}",
            dir.display()
        );
    }

    for (item, list, projectile, kind) in [
        ("SmCrossBow", "consbowfp.anm", "Prscbolt.kq", "Projectile"),
        ("SmCpndCBow", "conscbfp.anm", "Prsccblt.kq", "Projectile"),
        ("CrossBow", "concbwfp.anm", "Prcbolt.kq", "Projectile"),
        ("FlameBow", "confbwfp.anm", "Prfarrow.kq", "Fire"),
        ("ShortBow", "coscbwfp.anm", "Prarrow.kq", "Projectile"),
        ("CpndCrossBow", "conccbfp.anm", "Prccbolt.kq", "Projectile"),
        ("LongBow", "conlnbfp.anm", "Prarrow.kq", "Projectile"),
        ("IceCrossBow", "coicebfp.anm", "PRicycle.kq", "Ice"),
    ] {
        let (fight, dir) = run(item, "1");
        assert!(
            fight
                .lines()
                .any(|line| line.contains(&format!("Connor: do ({list}/attack"))
                    && line.contains(&format!("shoots bolt1 ({projectile},"))),
            "{item}: no authored first-person projectile; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches(&format!("skeleton1: hit by Connor, {kind}:"))
                .count(),
            1,
            "{item}: wrong ranged damage; inspect {}",
            dir.display()
        );
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
