//! A bow spriggan's ordinary bolt can hit a goblin on its way to Connor.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and a Windows graphics device"]
fn original_bow_spriggan_bolts_injure_another_monster_in_flight() {
    let game =
        fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../kq8-artifacts/monster-friendly-fire/{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
        .current_dir(&dir)
        .arg("--game")
        .arg(&game)
        .args([
            "--windowed",
            "--sound",
            "off",
            "--god",
            "--world",
            "Daventry",
            "--at",
            "289284,188000,0",
            "--key",
            "popup:autohelp,0",
            "--command",
            "move goblin22 0 289284 190500 13770 0 0 0,5",
            "--command",
            "move sprgnbow 0 289284 193500 13770 0 0 0,5",
            // The first three bolts intercept the goblin while Connor is
            // still. Walk after them so the next shot must lead him.
            "--hold",
            "1,0,0,125,155",
            "--updates",
            "165",
            "--frames",
            "2200",
        ])
        .output()
        .unwrap();
    fs::write(dir.join("stdout.log"), &output.stdout).unwrap();
    fs::write(dir.join("stderr.log"), &output.stderr).unwrap();
    let logs = dir.join("kq8-artifacts/logs");
    let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
    let characters = fs::read_to_string(logs.join("characters.log")).unwrap();
    let fight = fs::read_to_string(logs.join("fight.log")).unwrap();
    let connor = fs::read_to_string(logs.join("connor.log")).unwrap();
    assert!(
        output.status.success() && viewer.contains("RESULT: PASS"),
        "inspect {}",
        dir.display()
    );
    assert!(
        characters.contains("sprgnbow: status: Wait, active")
            && fight.contains("sprgnbow: attack (rule 7, standing)")
            && fight.contains("sprgnbow: Do (Attack at phase"),
        "the original bow spriggan did not shoot through normal AI; inspect {}",
        dir.display()
    );
    let touches: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("sprgnbow: bolt") && line.contains("touches goblin22"))
        .collect();
    assert!(
        touches.len() >= 3,
        "the first three bolts did not strike the goblin once each; inspect {}",
        dir.display()
    );
    assert!(
        touches
            .iter()
            .take(3)
            .all(|line| line.contains("update 2: touches goblin22")
                && !line.contains("0 travelled")
                && line.contains("Projectile")),
        "the bolts did not collide in flight; inspect {}",
        dir.display()
    );
    let hits: Vec<_> = fight
        .lines()
        .filter(|line| line.contains("goblin22: hit by sprgnbow, Projectile:"))
        .collect();
    assert_eq!(
        hits.len(),
        touches.len(),
        "projectile damage did not follow each touch; inspect {}",
        dir.display()
    );
    assert!(
        hits[0].contains("health 15 -> 13")
            && hits[1].contains("health 13 -> 11")
            && hits[2].contains("health 11 -> 9")
            && !fight.contains("Connor: hit by sprgnbow"),
        "the intercept did not protect Connor or reduce the goblin's health; inspect {}",
        dir.display()
    );
    let moving_shot = fight
        .lines()
        .find(|line| {
            line.contains("sprgnbow: Do (Attack") && line.contains("shoots bolt4 (Prscbolt.kq")
        })
        .unwrap_or_else(|| {
            panic!(
                "no fourth bow shot at moving Connor; inspect {}",
                dir.display()
            )
        });
    let speed = moving_shot
        .split_once("led for his speed (")
        .unwrap()
        .1
        .split_once(')')
        .unwrap()
        .0
        .split(", ")
        .map(|n| n.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    assert!(
        speed.iter().any(|v| v.abs() > 10.0),
        "the bow shot saw no Connor movement: {moving_shot}"
    );
    let shot_update: u64 = moving_shot
        .strip_prefix("update ")
        .unwrap()
        .split_once(':')
        .unwrap()
        .0
        .parse()
        .unwrap();
    let at_shot = connor
        .lines()
        .find(|line| line.starts_with(&format!("{shot_update} ")))
        .unwrap_or_else(|| {
            panic!(
                "Connor movement at shot not logged; inspect {}",
                dir.display()
            )
        });
    let fields = at_shot.split_whitespace().collect::<Vec<_>>();
    let position = fields[2..5]
        .iter()
        .map(|n| n.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    let flight: f32 = moving_shot
        .split_once(": meeting in ")
        .unwrap()
        .1
        .split_once(" s;")
        .unwrap()
        .0
        .parse()
        .unwrap();
    let target = moving_shot
        .rsplit_once(" at (")
        .unwrap()
        .1
        .split_once("): heading")
        .unwrap()
        .0
        .split(", ")
        .map(|n| n.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    assert!(
        (target[0] - position[0]).hypot(target[1] - position[1]) > 300.0,
        "the fourth aim is indistinguishable from Connor's current position: {moving_shot}"
    );
    assert!(
        target
            .iter()
            .zip(position.iter().zip(speed.iter()))
            .enumerate()
            .all(|(axis, (aim, (here, v)))| {
                let raise = if axis == 2 { 975.0 } else { 0.0 };
                (aim - (here + raise + v * flight)).abs() < 160.0
            }),
        "the fourth bolt did not use Connor's predicted position: {moving_shot}; {at_shot}"
    );
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(), options);
}
