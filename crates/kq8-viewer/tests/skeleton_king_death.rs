//! The original Dead City boss death script clears its own state listener.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn skeleton_king_death_clears_notify_and_finishes_rewards() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/skeleton-king-death/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
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
            "--at",
            "123016,182806,0",
            "--command",
            "KQMonster::stateChangeNotify Skelking Skelking,5",
            "--command",
            "sendEvent 0 Skelking KQAttackEvent Connor 0 100000 0,20",
            "--updates",
            "100",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
    let talk = fs::read_to_string(logs.join("talk.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        talk.contains("Skelking now hears Skelking's TalkComplete and StateChange")
            && doors.contains("script skelking.cs Skelking Die Normal")
            && talk
                .contains("KQMonster::clearstateChangeNotify: Skelking no longer hears Skelking")
            && !doors.contains("ERROR unknown command 'KQMonster::clearstateChangeNotify'"),
        "the authored death callback did not clear its listener; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight.matches("Skelking: kept weapon (rule 31a):").count(),
        1,
        "the original sword was not laid once; inspect {}",
        dir.display()
    );
    assert_eq!(
        fight
            .matches("Skelking: its death ended (rule 32): Connor gets its experience 20000")
            .count(),
        1,
        "the boss experience was not awarded once; inspect {}",
        dir.display()
    );
    assert!(
        fight.contains("a corpse sk_dead.kqs where it stood"),
        "the authored corpse did not appear; inspect {}",
        dir.display()
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
