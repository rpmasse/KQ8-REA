//! Alt puts away each original first-person weapon through its authored movement list.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn first_person_weapons_put_away_into_the_base_list() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/first-person-weapon-away/{nonce}"
    ));

    for (item, mode, bank) in [
        ("Fists", "0", "confisfp.anm"),
        ("BroadSword", "0", "conbsfp.anm"),
        ("SmallAx", "0", "conaxefp.anm"),
        ("Mace", "0", "conmcefp.anm"),
        ("SkelKingSword", "0", "conkngfp.anm"),
        ("LakeSword", "0", "conswdfp.anm"),
        ("Dagger", "0", "condagfp.anm"),
        ("WarHammer", "0", "conwhfp.anm"),
        ("LongSword", "0", "conlngfp.anm"),
        ("BattleAx", "0", "conbaxfp.anm"),
        ("Pike", "0", "conbpkfp.anm"),
        ("TempleSword", "0", "contmsfp.anm"),
        ("FlameSword", "0", "confsfp.anm"),
        ("SmCrossBow", "1", "consbowfp.anm"),
        ("SmCpndCBow", "1", "conscbfp.anm"),
        ("CrossBow", "1", "concbwfp.anm"),
        ("FlameBow", "1", "confbwfp.anm"),
        ("ShortBow", "1", "coscbwfp.anm"),
        ("CpndCrossBow", "1", "conccbfp.anm"),
        ("LongBow", "1", "conlnbfp.anm"),
        ("IceCrossBow", "1", "coicebfp.anm"),
        ("AzrielHammer", "1", "conhmrfp.anm"),
    ] {
        let dir = root.join(item);
        fs::create_dir_all(&dir).unwrap();
        let give = format!("KQMonster::inventory Connor INVITEM_{item} 1,20");
        let ready = format!("KQConner::setDoMode {mode},35");
        let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir)
            .arg("--game")
            .arg(&game)
            .args([
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
                &give,
                "--command",
                &ready,
                "--key",
                "physical:f2,80",
                "--key",
                "alt,95",
                "--updates",
                "150",
            ])
            .output()
            .unwrap();
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

        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        assert!(
            fight
                .lines()
                .any(|line| line.starts_with("update 95: Connor: weapon request")
                    && line.contains("Alt: put away")
                    && line.contains("-> none (None)")),
            "{item}: Alt did not put the weapon away; inspect {}",
            dir.display()
        );
        let connor = fs::read_to_string(logs.join("connor.log"))
            .unwrap()
            .to_ascii_lowercase();
        let swap = connor
            .lines()
            .find(|line| line.starts_with("connor update 95: weapon: rule 6 case"))
            .unwrap_or_else(|| panic!("{item}: no put-away movement; inspect {}", dir.display()));
        if item == "Fists" {
            assert!(
                swap.contains(&format!("\"{bank}/undraw\""))
                    && !swap.contains("connerfp.anm/out")
                    && connor.contains(&format!(
                        "movement {bank}/undraw -> stop (drawn: connerfp.anm/stop)"
                    )),
                "{item}: wrong Fists transition; inspect {}",
                dir.display()
            );
        } else {
            assert!(
                swap.contains(&format!("\"{bank}/away\""))
                    && swap.contains("\"connerfp.anm/out\"")
                    && connor.contains(&format!("movement {bank}/away -> connerfp.anm/out"))
                    && connor
                        .contains("movement connerfp.anm/out -> stop (drawn: connerfp.anm/stop)"),
                "{item}: wrong put-away sequence; inspect {}",
                dir.display()
            );
        }
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
