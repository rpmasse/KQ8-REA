//! Q202: one grouped data/numeric/dispatch matrix. No original runtime replay.
use super::*;
use crate::active::{Active, Clickable, Item, Request};
use crate::collide::{BoxCollider, ColliderKind, Placement};
use crate::connor_mover::GroundLookup;

struct Ground { floor: f32, end: Option<[f32; 3]> }
impl GroundLookup for Ground {
    fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> { Some((self.floor, [0., 0., 1.])) }
    fn drawn_line_hit(&self, _: [f32; 3], _: [f32; 3]) -> Option<[f32; 3]> { self.end }
    fn water(&self, _: f32, _: f32) -> Option<(f32, u32)> { Some((1000., 1)) }
}
fn identity() -> [[f32; 3]; 3] { kq8_data::objects::rotation([0.; 3]) }
fn read_world(game: &kq8_data::GameDir, world: &str, name: &str) -> Vec<u8> {
    if let Some(path) = game.patch_files(world).unwrap().get(&name.to_ascii_lowercase()) { return std::fs::read(path).unwrap(); }
    for v in kq8_data::game_dir::VOLUMES {
        if let Some(path) = game.volume_path(world, v) {
            let archive = kq8_data::VolArchive::open(&path).unwrap();
            if let Some(entry) = archive.find(name) { return archive.unpack(entry).unwrap().data; }
        }
    }
    for rel in [format!("GAME/Common/{name}"), format!("GAME/Kq/{name}")] {
        if let Some(path) = game.find(&rel) { return std::fs::read(path).unwrap(); }
    }
    panic!("missing asset {world}/{name}")
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; grouped Q202 asset/numeric/dispatch matrix"]
fn authored_rock_numeric_dispatch_and_asset_matrix() {
    let game = kq8_data::GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let bytes = read_world(&game, "Daventry", "prrock.kq");
    let p = kq8_data::characters::ProjectileKeys::from_section(&kq8_data::kq_text::parse(&bytes)[0]).unwrap();
    assert_eq!((p.speed, p.max_distance, p.parabola), (2500., 2500., Some((500., 2500.))));
    assert_eq!((p.mesh_name.as_deref(), p.script.as_deref(), p.ouch_sound.as_deref()), (Some("rock"), Some("rockHit.cs"), Some("rock.aud")));
    let malformed=kq8_data::characters::ProjectileKeys::from_section(&kq8_data::kq_text::parse(b"[Rock]\nclassType=KQProjectile\nparabola=1\npeakHeightOffset=-5\nupVelocity=2500\n")[0]).unwrap();
    assert_eq!(malformed.parabola,Some((-5.,2500.)),"malformed arcs reach explicit safeguards, never silently become straight bolts");
    let bank = kq8_data::characters::AnimList::parse(&std::fs::read(game.find("GAME/Kq/Conner.anm").unwrap()).unwrap()).unwrap();
    let movement = &bank.movements[bank.index("throw").unwrap()].1;
    assert!(movement.shape.eq_ignore_ascii_case("c_rktoss.kqs"));
    assert!(movement.cues.iter().any(|(t,s)| *t == 0.777778 && s.trim() == "do"));
    assert!(movement.cues.iter().any(|(t,s)| *t == 0.111111 && s.contains("tossrock.aud")));
    let shape = kq8_data::shape::Shape::parse(&read_world(&game, "Daventry", &movement.shape)).unwrap();
    let sequence = shape.loops.iter().find(|l| shape.names[l.name as usize].eq_ignore_ascii_case(&movement.sequence)).unwrap();
    let frame_count = sequence.sequence_count as usize;
    let motion = crate::connor::MovementLoop { duration:sequence.duration, frames:frame_count, speed:movement.speed,
        start:movement.start, repeat:movement.repeat, cues:movement.cues.clone(), ..Default::default() };
    let mut phase = movement.start;
    let frame = loop {
        let next = phase + movement.speed / (15. * sequence.duration);
        if motion.cues_between(phase, next).iter().any(|s| s.trim() == "do") { break motion.frame_at(next); }
        assert!(next < 1., "authored do cue must be delivered"); phase = next;
    };
    let parts = shape.part_transforms(sequence.first_sequence as usize + frame);
    let part = parts.iter().find(|(n,_)| n.eq_ignore_ascii_case("rock")).unwrap().1;
    assert!(part.p.iter().chain(part.m.iter().flatten()).all(|v| v.is_finite()));
    println!("Q202 asset do frame {frame}/{frame_count}, duration {}, local translation {:?}, authored rotation {:?}", sequence.duration, part.p, part.m);

    let mut arc = RockArc::new([0.,0.,1000.], [2500.,0.,99999.], 500., 2500.,2500.,2500.,1000.,identity());
    assert!((arc.height(2500./3.) - 1000.).abs() < 0.001);
    assert!((arc.height(1500.) - 874.7727).abs() < 0.002);
    assert!((arc.height(2500.) + 500.).abs() < 0.001);
    assert_eq!(arc.height(0.), 500.);
    let down_aim = RockArc::new(arc.launch, [2500.,0.,-99999.],500.,2500.,2500.,2500.,1000.,identity());
    assert_eq!(arc, down_aim, "aim height does not change the arc");
    let mut at = arc.launch;
    let mut samples = Vec::new();
    while !arc.falling { let before=at; at=arc.advance(at,1./15.); samples.push((arc.progress,at)); if arc.falling { assert_eq!(at,before); } }
    let stopped=at; at=arc.advance(at,1./15.);
    assert_eq!(&at[..2], &stopped[..2]); assert!((at[2]-(stopped[2]-8000./15.)).abs()<0.001);
    println!("Q202 f32 curve samples {samples:?}; subsequent descent is initialized safeguard, not original numeric proof");
    let mut rates=RockArc::new([0.,0.,2000.],[900.,0.,0.],500.,2250.,1500.,2500.,600.,identity());
    let mut at=rates.launch;
    at=rates.advance(at,1./15.);assert!((rates.progress-150.).abs()<0.001);
    // Place exactly on the f32 boundary: the crossing update still pays
    // the up rate; only its successor pays the later rate. Repeated f32
    // additions can already pass300 by an ULP, so do not pretend they equal it.
    rates.progress=300.;
    for expected in [450.,550.] {at=rates.advance(at,1./15.);assert!((rates.progress-expected).abs()<0.001);}
    rates.progress=rates.range; assert!(!rates.falling); let before=at; assert_eq!(rates.advance(at,1./15.),before); assert!(rates.falling);
    for (point,normals,steep) in [([1.,2.,3.],vec![[2.,0.,0.]],true),([1.,2.,3.],vec![[0.,0.,2.]],false),
        ([1.,2.,3.],vec![[0.,0.,2.],[0.,0.,-4.]],true),([0.;3],vec![[1.,0.,0.]],false),([1.,2.,3.],vec![[0.;3]],false)] {
        let mut a=down_aim.clone(); assert_eq!(a.steep_contact(point,&normals),steep);
    }
    for (aim,h,peak) in [([0.,0.,1000.],600.,500.),([100.,0.,0.],-1000.,500.),([100.,0.,0.],0.,-500.)] {
        let mut a=RockArc::new([0.,0.,1000.],aim,peak,2500.,2500.,2500.,h,identity());
        assert!(a.advance(a.launch,1./15.).iter().all(|x| x.is_finite()));
    }
    let saved=serde_json::to_string(&arc).unwrap(); assert_eq!(serde_json::from_str::<RockArc>(&saved).unwrap(),arc);
    assert!(serde_json::from_str::<RockArc>(r#"{"velocity":[1,2,3],"gravity":1000}"#).is_err(),"old approximate arcs are explicitly incompatible");

    let types=kq8_data::items::ItemTypes::parse(&std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap()).0;
    let fixture=|| {
        let mut a=Active::new(Default::default(),vec![],vec![],["Connor".into()],
            [("rockHit.cs".into(),read_world(&game,"Daventry","rockHit.cs")),
             ("offer.cs".into(),b"if test $2 == ConnorAction\n set received $3 $4\n KQMonster::setState Connor ThrowRock\nendif\n".to_vec())]);
        a.world.items=crate::item::Items::new(types.clone(),[("rok.kqs".into(),[-5.;3],[5.;3])]);
        a.world.projectiles.insert("prrock.kq".into(),p.clone());
        a.world.connor_stats=Some(crate::stats::ConnorStats{stats:crate::stats::Stats([100,0,37,0,0]),level:1});
        a.world.connor_at=Some([0.;3]);
        a.console_command("give Rock 3","fixture"); a
    };
    let mut a=fixture();
    for first_person in [false,true] {
        a.world.requests.clear();a.world.first_person=first_person;
        a.world.direct_rock_click([100.,0.,50000.]);
        assert_eq!(a.world.requests.len(),if first_person{1}else{2});
        assert!(matches!(a.world.requests.last(),Some(Request::ConnorThrowRock{aim:Some([100.,0.,50000.])})));
        if !first_person {assert!(matches!(a.world.requests[0],Request::ClickTurn{..}));}
    }
    a.world.first_person=false;a.world.requests.clear();
    let mut target=Clickable::plain("target","KQObject",[-10.;3],[10.;3],Placement{position:[1000.,0.,100.],rotation:identity()},0x41);
    target.script=Some("offer.cs".into());target.accepts_inventory=Some("Rock".into());
    a.world.clickables.push(target.clone());
    let ground=Ground{floor:0.,end:Some([500.,0.,100.])};
    assert_eq!(a.world.rock_aim([0.,0.,100.],[1.,0.,0.],Some(&ground),&[],&[]).0,[500.,0.,100.]);
    let (hit,object)=a.world.rock_aim([0.,0.,100.],[1.,0.,0.],None,&[],&[]);
    assert_eq!(hit,[990.,0.,100.]);assert_eq!(object.unwrap().name,"target");
    a.world.clickables[0].collision_type=0;
    assert!(a.world.rock_aim([0.,0.,100.],[1.,0.,0.],None,&[],&[]).1.is_none());
    a.world.clickables[0].collision_type=0x41;
    assert!(a.world.rock_offer("target",[-1510.,0.,100.],"INVITEM_Rock").is_none(),"exact reach2500 excluded");
    assert_eq!(a.world.rock_offer("target",[-1509.,0.,100.],"INVITEM_Rock").unwrap().0,[1000.,0.,100.]);
    let held=Item{object:a.world.items.find("Connor","Rock").unwrap().name.clone(),kind:"INVITEM_Rock".into()};
    for suppress in [false,true] {
        a.world.requests.clear();a.world.clickables[0].do_not_point_at=suppress;
        a.click_rock("target",[0.;3],&held);
        assert_eq!(a.world.requests.iter().filter(|r|matches!(r,Request::ConnorThrowRock{..})).count(),1);
        assert_eq!(a.world.requests.iter().filter(|r|matches!(r,Request::ClickTurn{..})).count(),usize::from(!suppress));
        assert_eq!(a.scripts.vars.get("received"),Some(format!("{} INVITEM_Rock",held.object).as_str()));
    }
    for ordinary in [false,true] {
        let mut a=fixture(); let rotation=kq8_data::objects::rotation([0.3,0.2,1.]);
        let feet=[100.,200.,0.];
        a.world.shoot_rock("prrock.kq",feet,rotation,Some((part,"actual cue part".into())),[2400.,200.,10000.],ordinary,"throw");
        let b=&a.world.bolts[0]; let expected=std::array::from_fn(|k|feet[k]+(0..3).map(|i|part.p[i]*rotation[i][k]).sum::<f32>());
        assert_eq!(b.position,expected);assert_eq!(b.rotation(),part.m);assert_eq!(b.damage,37);
        assert_eq!(b.kind,if ordinary{HitKind::Normal}else{bolt_kind(p.weapon_type)});
        assert_eq!(a.world.items.count("Connor","INVITEM_Rock"),3);
    }
    // Full production flight -> original Die script -> item pose; surface1000
    // must not replace floor0. No registration matrix is replayed here.
    let mut a=fixture();
    let orient=kq8_data::objects::rotation([0.2,-0.1,0.7]);
    a.world.shoot_rock("prrock.kq",[10.,20.,0.],identity(),Some((kq8_data::shape::Transform{flags:3,p:[0.,0.,1000.],m:orient},"numeric part".into())),[2510.,20.,10000.],true,"throw");
    let mut final_pose=None;
    for _ in 0..40 {
        a.world.bolts_step(&ground,&[],&[]);
        if let Some((_,_,"Die",place))=a.world.projectile_calls.last() {final_pose=Some(place.clone());}
        a.update(); if a.world.bolts.is_empty(){break;}
        assert_eq!(a.world.items.count("Connor","INVITEM_Rock"),3);
    }
    let final_pose=final_pose.expect("flight dies at floor");
    assert_eq!(final_pose.position[2],0.);assert_eq!(final_pose.rotation,orient);
    assert_eq!(a.world.items.count("Connor","INVITEM_Rock"),2);
    let dropped=a.world.items.find("world","Rock").unwrap().placement().unwrap();
    assert_eq!(dropped.position,final_pose.position);
    for (x,y) in dropped.rotation.iter().flatten().zip(orient.iter().flatten()){assert!((x-y).abs()<0.0001);}
    assert!(a.world.sounds.iter().any(|s|s.name.eq_ignore_ascii_case("rock.aud")));

    // Real box sweeps retain wall/upward normals and immunity dispatch.
    for (wall,immune) in [(true,false),(false,false),(false,true)] {
        let mut a=fixture();
        a.world.shoot_rock("prrock.kq",[10.,20.,0.],identity(),Some((kq8_data::shape::Transform{flags:3,p:[0.,0.,1000.],m:orient},"part".into())),[2510.,20.,0.],true,"throw");
        let obstacle=BoxCollider{name:"obstacle".into(),min:if wall{[100.,-100.,0.]}else{[-1000.,-1000.,-100.]},
            max:if wall{[110.,100.,2000.]}else{[1000.,1000.,800.]},placement:Placement{position:[0.;3],rotation:identity()},
            kind:ColliderKind{collision_type:if immune{0x4001}else{1},..Default::default()}};
        a.world.bolts_step(&Ground{floor:-10000.,end:None},&[],&[obstacle]);
        if wall {assert_eq!(a.world.bolts.len(),1);assert!(a.world.bolts[0].arc.as_ref().unwrap().falling);assert!(a.world.projectile_calls.is_empty());}
        else {assert!(a.world.bolts.is_empty());assert!(a.world.projectile_calls.iter().any(|(_,_,e,_)|*e=="Die"));
            assert_eq!(a.world.projectile_calls.iter().any(|(_,_,e,_)|*e=="ProjImmune"),immune);}
    }
    println!("Q202 PASS: assets/cue part, curve/rates/range/safeguards, clipped mask/reach/dispatch, launch kind/pose, floor/recovery, supplied wall/top normals/immunity");
}
