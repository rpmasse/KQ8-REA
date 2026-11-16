//! Runtime additions use both placed and otherwise unused original monster types.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn every_available_type_can_create_a_drawn_live_actor() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/available-monster-add/{nonce}"));

    for (world, at, command, actor, kind, file) in [
        (
            "Daventry",
            "289284,190000,0",
            "KQMonster::add testGoblin goblin2 @ @ @ @ @ 289284 192527 -1 0 @ @ @ @ @ @ @ @ @,10",
            "testGoblin",
            "goblin2",
            "goblin2.kq",
        ),
        (
            "DeadCity",
            "182221,45000,864",
            "KQMonster::add testBigskel bigskel @ @ @ @ @ 183300 45000 864 0 @ @ @ @ @ @ @ @ @,10",
            "testBigskel",
            "bigskel",
            "bigskel.kq",
        ),
    ] {
        let dir = root.join(world);
        fs::create_dir_all(&dir).unwrap();
        let mut run = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        run.current_dir(&dir).arg("--game").arg(&game).args([
            "--windowed",
            "--sound",
            "off",
            "--no-ai",
            "--world",
            world,
            "--at",
            at,
            "--key",
            "popup:autohelp,0",
            "--command",
            command,
            "--updates",
            "45",
            "--frames",
            "3000",
        ]);
        if world == "DeadCity" {
            // This is a defined global type, but its file is absent here.
            run.args(["--command", "KQMonster::add testMissing goblin2,20"]);
        }
        let output = run.output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "inspect {}",
            dir.display()
        );
        assert!(
            doors.contains(&format!("KQMonster::add {actor} type {kind}"))
                && doors.contains("live character, AI and fight created"),
            "the valid {kind} type did not create a live actor; inspect {}",
            dir.display()
        );
        assert!(
            doors.contains(&format!("copy {actor} of {file}: drawn,")),
            "the {kind} model was not drawn; inspect {}",
            dir.display()
        );
        if world == "Daventry" {
            assert!(
                doors.contains(
                    "copy testGoblin of goblin2.kq: drawn, 10 movement(s), 10 part entities"
                ),
                "the placed goblin model changed; inspect {}",
                dir.display()
            );
        } else {
            assert!(
                doors.contains("Monster type undefined ('goblin2')")
                    && !doors.contains("copy testMissing"),
                "a type file absent from Dead City was accepted; inspect {}",
                dir.display()
            );
        }
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
