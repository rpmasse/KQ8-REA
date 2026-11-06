//! Original Henchman attack, a nonreacting hit, disk continuation and retreat.
use std::{fs, path::Path, process::Command};

#[test]
#[ignore = "requires KQ8_GAME_DIR and graphics; isolated protected placement, not campaign acceptance"]
fn henchman_keeps_an_uninterruptible_attack_and_misses_a_retreat_after_reload() {
    let game = fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let options = fs::read(game.join("Options.cs")).unwrap();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/henchman-combat/{nonce}"));
    let slots = root.join("slots");
    let run = |part: &str, args: &[&str]| {
        let dir = root.join(part); fs::create_dir_all(&dir).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR", &slots).arg("--game").arg(&game)
            .args(["--sound","off","--god"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"),result.stderr).unwrap();
        let logs = dir.join("kq8-artifacts/logs");
        let viewer = fs::read_to_string(logs.join("viewer.log")).unwrap();
        assert!(result.status.success() && viewer.contains("RESULT: PASS"), "inspect {}", dir.display());
        (fs::read_to_string(logs.join("characters.log")).unwrap(),fs::read_to_string(logs.join("fight.log")).unwrap())
    };
    let setup = ["--world","Daventry",
        "--command","move Connor 0 139600 250000 15045 0 0 -1.57,5",
        "--command","KQMonster::setHome Henchman 137200 250000 15045,5",
        "--command","move Henchman 0 137200 250000 15045 0 0 1.57,5"];
    let mut args = setup.to_vec(); args.extend(["--frames","500"]);
    let baseline = run("observe-attack", &args);
    let starts: Vec<u64> = baseline.0.lines().filter(|line| line.to_ascii_lowercase().contains("henchman: attack (rule 7, standing)"))
        .map(|line| line.strip_prefix("update ").unwrap().split(':').next().unwrap().parse().unwrap()).collect();
    assert!(starts.len() >= 2, "the original guard must attack normally; inspect {}", root.display());
    for pair in starts.windows(2) { assert!((22..=25).contains(&(pair[1]-pair[0])), "1 s attack plus 0.5 s delay, sampled at 15 Hz: {starts:?}"); }
    let first_attack = starts[0];
    // Observe the real AI's first start rather than assume a first-swing
    // grace period after a script, which the original study did not find.
    let hit = format!("sendEvent 0 Henchman KQAttackEvent Connor 0 2 0,{}",first_attack+1);
    let quit = "down:down:down:enter:release:left:enter:release";
    let save = format!("menu:down:down:enter:release:enter:release:escape:type=HenchmanAttack:enter:release:{quit},{}",first_attack+2);
    let mut args = setup.to_vec(); args.extend(["--frames","1500","--command",&hit,"--key",&save]);
    let before = run("hit-and-save", &args);
    assert!(before.1.to_ascii_lowercase().lines().any(|line| line.contains("henchman: hit by connor") && line.contains("no flinch") && !line.contains("next attack due")), "inspect {}", root.display());
    let read = |n| -> serde_json::Value { serde_json::from_slice(&fs::read(slots.join(format!("slot-{n}.json"))).unwrap()).unwrap() };
    let actor = |slot: &serde_json::Value| slot["kept"]["daventry"]["characters"].as_array().unwrap().iter()
        .find(|actor| actor["name"].as_str().is_some_and(|name| name.eq_ignore_ascii_case("Henchman"))).unwrap().clone();
    let first = actor(&read(1));
    assert!(first["animation"]["movement"].as_str().is_some_and(|m| m.eq_ignore_ascii_case("attack")));
    let phase = first["animation"]["phase"].as_f64().unwrap(); assert!(phase > 0. && phase < 0.5);
    assert!(first["character"]["fight"]["attack_due"].is_null(), "a nonreacting Wait hit must keep never");
    assert!(first["character"]["fight"]["stats"][0].as_i64().is_some_and(|h| (297..300).contains(&h)));
    let save = format!("menu:down:down:down:enter:release:enter:release:escape:type=Retreated:enter:release:{quit},100");
    let after = run("restore-and-retreat", &["--menu-input","down,enter,release,enter,release","--frames","1800",
        // Backward extends his hit reach to 3125. Retreat briefly, then
        // release before Do so the ordinary 2500-unit test applies.
        "--hold","-1,0,0,1,5","--key",&save]);
    let do_lines: Vec<_> = after.1.lines().filter(|line| line.to_ascii_lowercase().contains("henchman: do (")).collect();
    assert_eq!(do_lines.len(),1, "the original attack must finish once, then the guard must stay out of reach; inspect {}",root.display());
    assert!(do_lines[0].contains("miss") && do_lines[0].contains("reach 2500") && !do_lines[0].contains("back key held"), "{}",do_lines[0]);
    assert!(after.0.to_ascii_lowercase().contains("henchman: 'attack' ended: next attack due at") && after.0.contains("attackDelay 0.5 s"));
    let second = actor(&read(2));
    assert_eq!(second["animation"]["placement"]["position"],first["animation"]["placement"]["position"]);
    assert_eq!(second["character"]["fight"]["stats"],first["character"]["fight"]["stats"]);
    assert!(!second["character"]["fight"]["attack_due"].is_null());
    assert_eq!(fs::read(game.join("Options.cs")).unwrap(),options);
}
