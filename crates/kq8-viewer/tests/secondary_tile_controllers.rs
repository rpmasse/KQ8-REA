//! Two representative native controller fixtures using original geometry,
//! motion, collision and scripts. These are isolated setups, not earned
//! campaign routes or a comparison with the original executable.
use std::{fs,path::{Path,PathBuf},process::Command};
use serde_json::Value;

struct Fixture {game:PathBuf,root:PathBuf,options:Vec<u8>}
impl Fixture {
    fn new(name:&str)->Self {
        let game=fs::canonicalize(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let options=fs::read(game.join("Options.cs")).unwrap();
        let nonce=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../kq8-artifacts/secondary-tiles/{name}-{nonce}"));
        Self {game,root,options}
    }
    fn run(&self,part:&str,args:&[&str])->PathBuf {
        let dir=self.root.join(part);fs::create_dir_all(&dir).unwrap();
        let output=Command::new(env!("CARGO_BIN_EXE_kq8-viewer"))
            .current_dir(&dir).env("KQ8_SAVE_DIR",self.root.join("slots"))
            .arg("--game").arg(&self.game).args(["--sound","off","--no-ai","--frames","3000"]).args(args).output().unwrap();
        fs::write(dir.join("stderr.log"),output.stderr).unwrap();
        let logs=dir.join("kq8-artifacts/logs");
        assert!(output.status.success() && read(&logs,"viewer").contains("RESULT: PASS"),"inspect {}",dir.display());
        assert_eq!(fs::read(self.game.join("Options.cs")).unwrap(),self.options,"original options unchanged");
        println!("{part} evidence: {}",dir.display());logs
    }
    fn slot(&self,n:u32)->Value {serde_json::from_slice(&fs::read(self.root.join(format!("slots/slot-{n}.json"))).unwrap()).unwrap()}
}
fn read(logs:&Path,name:&str)->String {fs::read_to_string(logs.join(format!("{name}.log"))).unwrap()}
fn mid_jump(saved:&Value,kind:&str) {
    assert_eq!(saved["secondary_tile"]["kind"],kind);
    assert_eq!(saved["secondary_tile"]["stage"]["Jumping"]["motion"],"tilejump1");
    assert!(saved["secondary_tile"]["pose"]["phase"].as_f64().is_some_and(|p|p>0. && p<1.),"must save an actual in-progress original motion: {}",saved["secondary_tile"]);
}

#[test]
#[ignore="requires read-only GOG data and graphics device; isolated Swamp controller fixture"]
fn original_swamp_controller_jump_rock_and_death_after_reload() {
    let f=Fixture::new("swamp");
    let before=f.run("save",&["--world","Swamp","--item","Rock","--updates","95",
        // This loaded door's original callbacks enter Tile2, then after
        // 1.2 seconds place Connor, turn on power and release his hands.
        "--command","sendEvent 0 fiendoor KQCueEvent 13 0,1",
        // Actual safe tile02 collision top from the installed gastrap1 model.
        "--command","move Connor 0 50000 96600 16035 0 0 1.5707963,35",
        "--command","KQConner::setLevel 5,6",
        "--command","sendEvent 0 Connor KQStatsChangeRequestEvent 50 0 0 100 0,7",
        "--hold","1,0,0,45,46",
        "--key","menu:down:down:enter:release:enter:release:escape:type=SwampJump:enter:release:down:down:down:enter:release:left:enter:release,58"]);
    let first=f.slot(1);mid_jump(&first,"Swamp");
    let entry=format!("{}\n{}",read(&before,"traps"),read(&before,"doors"));
    assert!(entry.lines().any(|line|line.contains("script fiendoor.cs ") && line.contains(" Cue 14")),"entry must use the original delayed door callback; inspect {}",f.root.display());
    let after=f.run("restore",&["--menu-input","down,enter,release,enter,release","--updates","165",
        "--click","tile13,50",
        "--key","menu:down:down:down:enter:release:enter:release:escape:type=SwampRock:enter:release:escape,110",
        // One ordinary lethal attack tests the controller's Die exception.
        "--command","sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 -100 0,125",
        "--command","sendEvent 0 Connor KQAttackEvent tile12 0 100000 1,126"]);
    let connor=read(&after,"connor");let fight=read(&after,"fight");
    assert!(connor.contains("secondary tile Swamp: restored Jumping") && connor.contains("secondary tile restored grid before busy"));
    assert_eq!(connor.matches("secondary tile Swamp: landed tilejump1").count(),1,"saved motion must end once; inspect {}",f.root.display());
    assert!(!read(&after,"doors").contains("enters Swamp tile controller"),"restore must not replay entry");
    assert!(fight.contains("ThrowRock: started=true") && fight.contains("shoots rock"),"real throw/cue must run; inspect {}",f.root.display());
    assert!(read(&after,"doors").contains("Swamp gas "),"a powered ordinary tile impact/contact must spawn its original gas; inspect {}",f.root.display());
    let second=f.slot(2);
    assert_eq!(second["secondary_tile"]["kind"],"Swamp");
    assert_eq!(second["secondary_tile"]["stage"],"Keys","Rock must return to the tile owner");
    assert_eq!(second["secondary_tile"]["pose"]["hands_off"],false);
    assert!(second["secondary_tile"]["saved_projectile"].is_null());
    assert!(connor.contains("dies (Normal)"),"ordinary Die must leave the tile owner; inspect {}",f.root.display());
}

#[test]
#[ignore="requires read-only GOG data and graphics device; isolated Barren controller fixture"]
fn original_barren_platform_jump_and_exit_after_reload() {
    let f=Fixture::new("barren");
    let before=f.run("save",&["--world","Barren","--updates","95",
        // The original On callback disables this entrance before its Cue2
        // placement. Match that precondition to isolate one platform.
        "--command","KQObject::collision tiletps -4,0",
        // Hold one representative original platform through its class cues.
        "--command","sendEvent 0 tile03 KQCueEvent 2403,1",
        "--command","sendEvent 0 tile03 KQCueEvent 2400 0 lvtile01.bmp,2",
        "--command","move Connor 0 92640 149800 600 0 0 3.1415927,5",
        "--command","KQMonster::setState Connor Tile3,6",
        "--hold","1,0,0,45,46",
        "--key","menu:down:down:enter:release:enter:release:escape:type=BarrenJump:enter:release:down:down:down:enter:release:left:enter:release,58"]);
    let first=f.slot(1);mid_jump(&first,"Barren");
    assert!(read(&before,"connor").contains("secondary tile Barren: request tilejump1"));
    let lava=&first["kept"]["barren"]["interactions"]["lava_tiles"]["tile03"];
    assert_eq!(lava["tile"]["hold"],true);assert_eq!(lava["tile"]["activation_count"],1);
    assert_eq!(lava["animation"]["movement"],"stopup1");
    let after=f.run("restore",&["--menu-input","down,enter,release,enter,release","--updates","165",
        "--hold","-1,0,0,60,61"]);
    let connor=read(&after,"connor");let doors=read(&after,"doors");
    assert!(connor.contains("secondary tile Barren: restored Jumping") && connor.contains("secondary tile restored grid before busy"));
    assert_eq!(connor.matches("secondary tile Barren: landed tilejump1").count(),1);
    assert!(connor.contains("secondary tile Barren: landed tilebackjump"),"backward original motion must reach the south approach; inspect {}",f.root.display());
    assert!(doors.contains("script tiletps2.cs StartExit"),"the original exit callback must run; inspect {}",f.root.display());
    assert!(!doors.contains("enters Barren tile controller"),"restore must not replay entry");
}
