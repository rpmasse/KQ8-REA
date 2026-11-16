//! Easy and Hard combat factors in a live original-data ranged encounter.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn combat_difficulty_scales_a_live_bow_skeleton() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../kq8-artifacts/live-combat-difficulty/{nonce}"
    ));

    for (level, factor, health, damage, hits_to_win, frames, aims, roll) in [
        (
            "1",
            "0.66",
            33,
            6,
            2,
            "180",
            &["skeleton2,60", "skeleton2,100"][..],
            "85-125",
        ),
        (
            "3",
            "1.5",
            75,
            15,
            5,
            "260",
            &[
                "skeleton2,60",
                "skeleton2,100",
                "skeleton2,140",
                "skeleton2,180",
                "skeleton2,220",
            ][..],
            "75-125",
        ),
    ] {
        let dir = root.join(level);
        fs::create_dir_all(&dir).unwrap();
        let mut run = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"));
        run.current_dir(&dir).arg("--game").arg(&game).args([
            "--windowed",
            "--sound",
            "off",
            "--combat-level",
            level,
            "--world",
            "DeadCity",
            "--command",
            "move skeleton2 0 183300 45000 864 0 0 0,5",
            "--command",
            "KQConner::setDoMode 1,35",
        ]);
        for aim in aims {
            run.args(["--aim", aim]);
        }
        let output = run.args(["--frames", frames]).output().unwrap();
        fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
        let doors = fs::read_to_string(logs.join("doors.log")).unwrap();
        assert!(
            output.status.success() && viewer.contains("RESULT: PASS"),
            "level {level}: inspect {}",
            dir.display()
        );
        let skeleton = fight
            .lines()
            .find(|line| line.contains("skeleton2: fights (rule 2):"))
            .expect("the original bow skeleton must load");
        assert!(
            skeleton.contains(&format!("combat level x {factor} (rule 3): [50, 10, 10, 10, 150] -> [{health}, {damage}, {damage}, 10, 150]")),
            "level {level}: incorrect scaled stats: {skeleton}"
        );
        assert_eq!(
            doors
                .matches("aim pick meets character skeleton2 (type 0x42)")
                .count(),
            hits_to_win,
            "level {level}: camera aims missed; inspect {}",
            dir.display()
        );
        let player_shots: Vec<_> = fight
            .lines()
            .filter(|line| {
                line.contains("Connor: do (conscbow.anm/attack") && line.contains("Prscbolt.kq")
            })
            .collect();
        let player_hits: Vec<_> = fight
            .lines()
            .filter(|line| line.contains("skeleton2: hit by Connor, Projectile:"))
            .collect();
        assert_eq!(player_shots.len(), hits_to_win);
        assert_eq!(player_hits.len(), hits_to_win);
        assert!(player_hits
            .iter()
            .all(|line| line.contains(&format!("roll {roll} %"))));
        assert!(player_hits.last().unwrap().contains("below 0: it dies"));
        let incoming: Vec<_> = fight
            .lines()
            .filter(|line| line.contains("Connor: hit by skeleton2, Projectile:"))
            .collect();
        assert!(
            incoming.len() >= 2
                && incoming
                    .iter()
                    .all(|line| line.contains(&format!("D {damage},")))
                && incoming.iter().all(|line| {
                    line.split_once("; health ")
                        .and_then(|(_, pair)| pair.split_once(" -> "))
                        .and_then(|(before, after)| {
                            Some((before.parse::<i32>().ok()?, after.parse::<i32>().ok()?))
                        })
                        .is_some_and(|(before, after)| before > after && after > 0)
                })
                && !fight.contains("god mode is on")
                && !fight.contains("Connor: dies"),
            "level {level}: live arrows did not damage a surviving Connor; inspect {}",
            dir.display()
        );
        assert_eq!(
            fight
                .matches("skeleton2: dies (rule 30): kind Projectile")
                .count(),
            1
        );
        assert_eq!(
            fight
                .matches("skeleton2: its death ended (rule 32): Connor gets its experience 150:")
                .count(),
            1
        );
        assert!(fight.contains("a corpse skbdead.kqs where it stood"));
    }
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
