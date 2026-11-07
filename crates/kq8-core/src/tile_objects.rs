//! The two original floor-object classes. Implemented from
//! behaviour/secondary-tile-puzzles.md rules 16–25 and 36–45.
//! Geometry, playback and damage stay in their ordinary engine paths.
use crate::named_anim::NamedAnimPlayer;
use serde::{Deserialize, Serialize};

pub const SWAMP_CLASS: &str = "KQSwampTile";
pub const LAVA_CLASS: &str = "KQLavaTile";
pub const GAS_SHAPE: &str = "swmpgas.kqs";
pub const REVEAL_BITMAPS: [&str; 4] = ["I_gastrp.bmp", "I_wtowr7.bmp", "I_trprng.bmp", "I_trunk.bmp"];

/// Shared KQObject firing fields (object-projectile-cues.md rules 1–6).
#[derive(Debug,Clone)]
pub struct ObjectProjectile {pub file:String,pub offset:[f32;3],pub aim:[f32;3],pub damage:i64}
impl ObjectProjectile {
    pub fn from_section(s:&kq8_data::kq_text::Section)->Option<Self> {
        let file=s.get("projectileFile")?.trim().to_string();if file.is_empty() {return None;}
        let n=|key:&str,default:f64|s.number(key).and_then(Result::ok).unwrap_or(default);
        Some(Self {file,offset:[n("shootOffsetX",0.) as f32,n("shootOffsetY",0.) as f32,n("shootOffsetZ",0.) as f32],
            aim:[n("relAimPointX",0.) as f32,n("relAimPointY",0.) as f32,n("relAimPointZ",0.) as f32],damage:n("objProjDamage",20.) as i64})
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SwampPower { pub power: bool, pub switch: bool }
impl SwampPower {
    pub fn from_vars(vars: &kq8_data::world_script::Vars) -> Self {
        let on=|name| vars.get(name).is_some_and(|v| v.eq_ignore_ascii_case("true") || crate::script::number_value(v)!=0.0);
        Self {power:on("Swamp::Tile::PowerOn"),switch:on("Swamp::Tile::SwitchOn")}
    }
    pub fn active(self)->bool {self.power && self.switch}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwampTile {
    pub has_script: bool,
    pub height_num: i32,
    pub damage_factor: f32,
    /// Not serialized in the original: restoration resets this to zero
    /// while the world's clock resumes (secondary-tile-puzzles rule 51).
    #[serde(skip)]
    pub last_contact: f64,
    pub revealed: bool,
}

impl SwampTile {
    pub fn from_section(section: &kq8_data::kq_text::Section) -> Option<Self> {
        section.get("classType").filter(|c| c.eq_ignore_ascii_case(SWAMP_CLASS))?;
        Some(Self {
            has_script: section.get("script").is_some_and(|s| !s.trim().is_empty()),
            height_num: section.number("heightNum").and_then(Result::ok).unwrap_or(0.0) as i32,
            damage_factor: section.number("damFactor").and_then(Result::ok).unwrap_or(0.48) as f32,
            last_contact: 0.0, revealed: false,
        })
    }

    pub fn height(&self) -> f32 {
        match self.height_num { 1 => 750.0, 2 | 12 => 1000.0, 3 | 13 => 1250.0, _ => 0.0 }
    }

    pub fn contact(&mut self, now: f64, powered: bool, has_script: bool, level: i64) -> Option<i64> {
        let previous = std::mem::replace(&mut self.last_contact, now);
        (powered && !has_script && now - previous > 2.0)
            .then(|| (-level as f64 * self.damage_factor as f64 * 10.0).trunc() as i64)
    }

    pub fn rock_gas(&self, damage: i64, powered: bool, has_script: bool) -> bool {
        damage == 0 && powered && !has_script
    }

    pub fn reveal_pairs(&self) -> Vec<(String, String)> {
        if !self.revealed { return Vec::new(); }
        let pairs = match self.height_num {
            1..=3 => [("gastrap.bmp", "I_gastrp.bmp"), ("wtower5.bmp", "I_wtowr7.bmp")],
            12..=13 => [("trapring.bmp", "I_trprng.bmp"), ("trunk.bmp", "I_trunk.bmp")],
            _ => return Vec::new(),
        };
        pairs.into_iter().map(|(a,b)| (a.into(),b.into())).collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LavaTile {
    /// None = ordinary cycling, Some(true) = up, Some(false) = down.
    pub hold: Option<bool>,
    pub activation_count: i64,
    pub bitmap: Option<String>,
    pub reversed: bool,
    pub marked: bool,
    pub cycling: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LavaEffects {
    pub hidden: Option<bool>,
    /// Named animation events, handled by the ordinary queue.
    pub events: Vec<(f64, &'static str)>,
}

impl LavaTile {
    fn up(&self) -> &'static str { if self.marked { "up1" } else { "up" } }
    fn down(&self) -> &'static str { if self.marked { "down1" } else { "down" } }
    fn bottom(&self) -> &'static str { if self.marked { "stop1" } else { "stop" } }

    fn reverse(&mut self, player: &mut NamedAnimPlayer) {
        player.random_speed = Some(-player.speed());
        player.playing = true;
        self.reversed = true;
    }

    fn lower(&mut self, player: &mut NamedAnimPlayer) {
        let motion = player.movement_name().to_ascii_lowercase();
        match motion.as_str() {
            "stopup" | "stopup1" => { player.start(self.down()); }
            "up" | "up1" => self.reverse(player),
            _ => {}
        }
    }

    /// Returns None for a cue outside the class's private protocol.
    pub fn cue(&mut self, name: &str, cue: i64, bitmap: Option<&str>, player: &mut NamedAnimPlayer) -> Option<LavaEffects> {
        let mut out = LavaEffects::default();
        match cue {
            2400 => {
                self.hold = Some(true);
                if let Some(bitmap) = bitmap {
                    self.bitmap = Some(bitmap.into());
                    self.marked = true;
                    let old = player.movement_name().to_ascii_lowercase();
                    let phase = player.phase;
                    match old.as_str() {
                        "stop" => { player.start("up1"); }
                        "stopup" => { player.start("stopup1"); }
                        "up" => {
                            player.cut_short = true;
                            player.start("up1"); player.phase = phase;
                            player.base.position[2] += 50.0;
                        }
                        "down" => {
                            player.cut_short = true;
                            player.start("down1"); player.phase = phase;
                            self.reverse(player);
                        }
                        _ => {}
                    }
                }
            }
            2401 => { self.hold = Some(false); self.lower(player); }
            2402 => {
                self.activation_count -= 1;
                if self.activation_count == 0 { self.cycling = false; self.lower(player); }
            }
            2403 => {
                self.activation_count += 1;
                if self.activation_count == 1 || name.eq_ignore_ascii_case("tile43") {
                    out.hidden = Some(false);
                    player.start(self.up());
                    self.cycling = true;
                }
            }
            _ => return None,
        }
        Some(out)
    }

    pub fn accepts_animation_event(&self) -> bool { self.hold.is_none() && self.cycling }

    pub fn ended(&mut self, movement: &str, player: &mut NamedAnimPlayer) -> LavaEffects {
        let mut out = LavaEffects::default();
        match movement.to_ascii_lowercase().as_str() {
            "up" | "up1" if self.reversed => {
                self.reversed = false;
                player.random_speed = Some(player.speed().abs());
                player.start(self.bottom());
                player.base.position[2] -= 50.0;
                out.hidden = Some(true);
            }
            "down1" if self.reversed => {
                self.reversed = false;
                player.random_speed = Some(player.speed().abs());
                player.start("stopup1");
                player.base.position[2] += 50.0;
            }
            "up" => { player.start("stopup"); out.events.push((7.0,"down")); }
            "up1" => { player.start("stopup1"); }
            "down" => {
                player.start("stop");
                if self.hold == Some(false) || !self.cycling { out.hidden = Some(true); }
                else { out.events.push((0.1,"up")); }
            }
            "down1" => {
                player.start("stop1");
                if self.hold == Some(false) || !self.cycling { out.hidden = Some(true); }
            }
            _ => {}
        }
        out
    }

    pub fn valid(&self) -> bool {
        self.activation_count.unsigned_abs() < 1_000_000
            && self.bitmap.as_ref().is_none_or(|s| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control))
    }
}

impl crate::active::WorldState {
    /// Class overrides take precedence while active; an empty result leaves
    /// the ordinary changeBmp override intact.
    pub fn tile_bitmap_pairs(&self, name: &str) -> Vec<(String,String)> {
        let key=name.to_ascii_lowercase();
        if let Some(tile)=self.swamp_tiles.get(&key) { return tile.reveal_pairs(); }
        self.lava_tiles.get(&key).and_then(|t|t.bitmap.as_ref())
            .map(|bmp|vec![("lvtile01.bmp".into(),bmp.clone())]).unwrap_or_default()
    }
    pub(crate) fn swamp_reveal(&mut self,name:&str,on:bool) {
        if let Some(tile)=self.swamp_tiles.get_mut(&name.to_ascii_lowercase()).filter(|t|t.has_script) {tile.revealed=on;}
    }
    fn swamp_gas(&mut self,name:&str) {
        let Some(tile)=self.swamp_tiles.get(&name.to_ascii_lowercase()) else {return;};
        let height=tile.height();
        let Some(mut placement)=self.place_of(name) else {self.note(format!("Swamp gas {name}: missing placement"));return;};
        placement.position[2]+=height;
        let Some(mut movement)=self.script_shapes.get(GAS_SHAPE).cloned() else {self.note(format!("Swamp gas {name}: missing {GAS_SHAPE}"));return;};
        movement.name=GAS_SHAPE.into();movement.start=0.;movement.speed=1.;movement.repeat=false;movement.cut_short=false;
        let mut serial=self.anims.len();
        let fresh=loop {let n=format!("SwampGas.{serial}");if !self.known.contains(&n.to_ascii_lowercase()) {break n;}serial+=1;};
        let mut player=NamedAnimPlayer::new(&fresh,vec![movement],0,placement);
        player.once=true;player.delete_when_done=true;
        self.known.insert(fresh.to_ascii_lowercase());
        self.anims.push(crate::active::AnimObject {player,script:None,gone:false,character:None,translucency:None});
        self.requests.push(crate::active::Request::Spawn {file:GAS_SHAPE.into(),name:fresh.clone(),placement,at:format!("KQSwampTile {name}")});
        self.note(format!("Swamp gas {name}: {fresh} at {:?}, one-shot speed 1",placement.position));
    }
    pub(crate) fn swamp_contact(&mut self,name:&str)->bool {
        let now=self.now();let level=self.connor_stats.map_or(1,|s|s.level);
        let powered=self.swamp_power.active();
        let Some(tile)=self.swamp_tiles.get_mut(&name.to_ascii_lowercase()) else {return false;};
        let Some(delta)=tile.contact(now,powered,tile.has_script,level) else {return true;};
        self.swamp_gas(name);
        let from=format!("KQSwampTile {name} contact");
        self.queue.send(now,0.,crate::talk::CONNOR,"KQStatsChangeRequestEvent",&[delta.to_string()],&from);
        self.queue.send(now,0.,crate::talk::CONNOR,"KQAttackEvent",&[name.into(),"2".into(),"1".into(),"0".into()],&from);
        self.note(format!("{from}: queued health {delta}, then Fire attack 1"));true
    }
    pub(crate) fn swamp_attack(&mut self,name:&str,damage:i64)->bool {
        let Some(tile)=self.swamp_tiles.get(&name.to_ascii_lowercase()) else {return false;};
        if tile.rock_gas(damage,self.swamp_power.active(),tile.has_script) {self.swamp_gas(name);}
        true
    }
    fn lava_effects(&mut self,name:&str,effects:LavaEffects) {
        if let Some(hidden)=effects.hidden {self.plain_hidden.insert(name.to_ascii_lowercase(),hidden);}
        let now=self.now();
        for (delay,motion) in effects.events {self.queue.send(now,delay,name,"KQAnimPlayEvent",&[motion.into()],"KQLavaTile cycle");}
    }
    pub(crate) fn lava_cue(&mut self,name:&str,data:&[String])->bool {
        let key=name.to_ascii_lowercase();let Some(i)=self.anim_index(name) else {return false;};
        let Some(tile)=self.lava_tiles.get_mut(&key) else {return false;};
        let cue=data.first().map_or(0,|n|crate::script::number_value(n) as i64);
        // sendEvent KQCueEvent carries number, integer, optional string.
        let bitmap=data.get(2).map(String::as_str);
        let Some(effects)=tile.cue(name,cue,bitmap,&mut self.anims[i].player) else {return false;};
        self.lava_effects(name,effects);true
    }
    pub(crate) fn lava_ended(&mut self,i:usize,motion:&str) {
        let name=self.anims[i].player.name.clone();
        let Some(tile)=self.lava_tiles.get_mut(&name.to_ascii_lowercase()) else {return;};
        let effects=tile.ended(motion,&mut self.anims[i].player);self.lava_effects(&name,effects);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collide::Placement, named_anim::tests::movement};

    fn platform() -> NamedAnimPlayer {
        let names = ["stop","down","up","stopup","stop1","down1","up1","stopup1"];
        let movements = names.into_iter().map(|name| {
            let still = name.starts_with("stop");
            let mut m = movement(name,if still {0.999} else {0.0},if still {0.0} else {1.0},still,still,vec![]);
            m.play_time = if name.starts_with("down") { 50.0/15.0 } else {2.0};
            m
        }).collect();
        NamedAnimPlayer::new("tile00",movements,0,Placement::at([0.0,0.0,-700.0]))
    }

    #[test]
    fn secondary_tiles_grouped_swamp_clock_precision_and_reveal() {
        let section=kq8_data::kq_text::parse(b"[tile]\nclassType=KQSwampTile\nheightNum=2\n").remove(0);
        for (level,damage) in [(1,-4),(5,-23),(43,-206)] {
            let mut tile=SwampTile::from_section(&section).unwrap();
            assert_eq!(tile.contact(2.0,true,false,level),None);
            assert_eq!(tile.contact(4.0,true,false,level),None);
            assert_eq!(tile.contact(6.01,true,false,level),Some(damage));
            assert_eq!(tile.contact(10.0,false,false,level),None);
            assert_eq!(tile.contact(11.0,true,false,level),None,"gated contact still resets clock");
            assert_eq!(tile.contact(14.0,true,true,level),None,"any script makes it safe");
            assert!(tile.rock_gas(0,true,false));
            assert!(!tile.rock_gas(1,true,false) && !tile.rock_gas(0,false,false) && !tile.rock_gas(0,true,true));
        }
        for (height,z,count) in [(0,0.0,0),(1,750.0,2),(2,1000.0,2),(3,1250.0,2),(12,1000.0,2),(13,1250.0,2),(14,0.0,0)] {
            let mut tile=SwampTile::from_section(&section).unwrap(); tile.height_num=height; tile.revealed=true;
            assert_eq!(tile.height(),z); assert_eq!(tile.reveal_pairs().len(),count);
            tile.revealed=false; assert!(tile.reveal_pairs().is_empty());
        }
    }

    #[test]
    fn secondary_tiles_grouped_lava_cues_reversal_and_stale_events() {
        let mut p=platform(); let mut tile=LavaTile::default();
        tile.cue("tile00",2402,None,&mut p);
        assert_eq!(tile.activation_count,-1);
        tile.cue("tile00",2403,None,&mut p);
        assert_eq!((tile.activation_count,tile.cycling),(0,false));
        assert_eq!(tile.cue("tile43",2403,None,&mut p).unwrap().hidden,Some(false));
        assert!(tile.accepts_animation_event());
        assert_eq!(tile.ended("up",&mut p).events,[(7.0,"down")]);
        assert_eq!(tile.ended("down",&mut p).events,[(0.1,"up")]);
        for from in ["up","down"] {
            let mut p=platform(); p.start(from); p.phase=0.4;
            let mut tile=LavaTile { cycling:true,..Default::default() };
            tile.cue("tile00",2400,Some("lvtile03.bmp"),&mut p);
            assert_eq!(p.phase,0.4); assert!(tile.marked && !tile.accepts_animation_event());
            assert_eq!(p.movement_name(),if from=="up" {"up1"} else {"down1"});
            assert_eq!(p.base.position[2],if from=="up" {-650.0} else {-700.0});
            if from=="down" {
                assert!(p.speed()<0.0 && tile.reversed);
                tile.ended("down1",&mut p); assert_eq!((p.movement_name(),p.base.position[2]),("stopup1",-650.0));
            } else {
                tile.cue("tile00",2401,None,&mut p); assert!(p.speed()<0.0);
                assert_eq!(tile.ended("up1",&mut p).hidden,Some(true));
                assert_eq!((p.movement_name(),p.base.position[2]),("stop1",-700.0));
            }
        }
        let mut p=platform(); let mut tile=LavaTile::default();
        tile.cue("tile00",2400,None,&mut p);
        assert_eq!((tile.hold,tile.marked,p.movement_name()),(Some(true),false,"stop"));
    }

    fn active() -> crate::active::Active {
        use crate::{active::{Active,Clickable},stats::{ConnorStats,Stats}};
        let click=Clickable::plain("tile00",SWAMP_CLASS,[-10.;3],[10.;3],Placement::at([10.,20.,30.]),16417);
        let mut a=Active::new(Default::default(),vec![],vec![click],["Connor".into(),"tile00".into()],Vec::<(String,Vec<u8>)>::new());
        a.set_connor_stats(ConnorStats {level:5,stats:Stats([50,0,0,100,0])});
        a.world.swamp_tiles.insert("tile00".into(),SwampTile::from_section(&kq8_data::kq_text::parse(b"[tile]\nclassType=KQSwampTile\nheightNum=2\n")[0]).unwrap());
        let mut gas=movement(GAS_SHAPE,0.,1.,false,false,vec![]);gas.play_time=0.2;
        a.world.script_shapes.insert(GAS_SHAPE.into(),gas);
        a.scripts.vars.set("Swamp::Tile::PowerOn","True".into());a.scripts.vars.set("Swamp::Tile::SwitchOn","True".into());a.world.updates=45;a
    }

    #[test]
    fn secondary_tiles_grouped_dispatch_flags_damage_and_continuation() {
        use crate::{active::{AnimObject,Request},kept_interactions::KeptInteractions,character::state_number};
        let mut a=active();assert!(a.connor_contact("tile00",[0.,0.,1.]));
        let mut queue=a.world.queue.clone();let first=queue.take_due(a.world.now()).unwrap();let second=queue.take_due(a.world.now()).unwrap();
        assert_eq!((first.kind.as_str(),first.data),("KQStatsChangeRequestEvent",vec!["-23".to_string()]));
        assert_eq!((second.kind.as_str(),second.data),("KQAttackEvent",vec!["tile00".into(),"2".into(),"1".into(),"0".into()]));
        let gas=&a.world.anims[0].player;assert_eq!(gas.base.position,[10.,20.,1030.]);assert!(gas.once && gas.delete_when_done);
        a.update();assert_eq!(a.world.connor_stats.unwrap().stats.0[0],27,"direct loss ignores protection; ordinary attack follows armor");
        for _ in 0..6 {a.update();}assert!(a.world.anims[0].gone,"gas is removed after authored end");
        for state in [state_number::TILE2,state_number::TILE3] {
            let mut a=active();a.world.connor_state=Some(state);a.world.connor_stats.as_mut().unwrap().stats.0[0]=1;
            a.connor_contact("tile00",[0.,0.,1.]);a.update();assert!(a.world.connor_dying);assert!(a.world.requests.iter().any(|r|matches!(r,Request::ConnorDies{..})));
        }
        let mut a=active();a.world.swamp_power=SwampPower::from_vars(&a.scripts.vars);a.world.swamp_tiles.get_mut("tile00").unwrap().last_contact=2.9;
        let before=a.world.queue.len();assert!(a.world.swamp_attack("tile00",0));assert_eq!(a.world.queue.len(),before,"Rock gas is not contact damage");
        assert_eq!(a.world.anims.len(),1);a.world.swamp_attack("tile00",2);assert_eq!(a.world.anims.len(),1);
        a.world.lava_tiles.insert("tile43".into(),LavaTile {activation_count:-1,reversed:true,cycling:true,..Default::default()});
        let mut p=platform();p.name="tile43".into();p.start("down1");p.phase=0.4;p.random_speed=Some(-1.);
        a.add_anims([AnimObject {player:p,script:None,gone:false,character:None,translucency:None}]);
        let saved:KeptInteractions=serde_json::from_slice(&serde_json::to_vec(&KeptInteractions::take(&a.world)).unwrap()).unwrap();assert!(saved.valid_animations());
        a.world.swamp_tiles.get_mut("tile00").unwrap().last_contact=999.;a.world.lava_tiles.get_mut("tile43").unwrap().activation_count=17;
        saved.restore_animations(&mut a.world);assert_eq!(a.world.swamp_tiles["tile00"].last_contact,0.);
        assert_eq!(a.world.lava_tiles["tile43"].activation_count,-1);assert!(a.world.lava_tiles["tile43"].reversed);
        let p=&a.world.anims[a.world.anim_index("tile43").unwrap()].player;assert_eq!(p.phase,0.4);assert!(p.speed()>0.,"original reversed-speed save limitation is preserved");
        a.world.solved_tile3=true;let j=crate::journey::Journey::take(&a.world,&a.scripts.vars,"Barren");
        let j:crate::journey::Journey=serde_json::from_slice(&serde_json::to_vec(&j).unwrap()).unwrap();let mut b=active();j.give(&mut b.world,&mut b.scripts.vars);assert!(b.world.solved_tile3);
    }

    #[test]
    fn secondary_tiles_grouped_script_order_and_object_projectiles() {
        use crate::{active::{Active,Clickable},script::ScriptHost};
        let script=b"set Swamp::Tile::PowerOn False\nset Swamp::Tile::SwitchOn True\nnext.cs\n";
        let nested=b"set Swamp::Tile::PowerOn True\nsendEvent 0 tile00 KQAttackEvent Connor 0 0 0\n";
        let mut a=Active::new(Default::default(),vec![],vec![Clickable::plain("tile00",SWAMP_CLASS,[-1.;3],[1.;3],Placement::at([0.;3]),16417)],vec!["tile00".into()],[("outer.cs".into(),script.to_vec()),("next.cs".into(),nested.to_vec())]);
        a.world.swamp_tiles=active().world.swamp_tiles;a.world.script_shapes=active().world.script_shapes;
        a.run_script("outer.cs",&[]);assert!(a.world.swamp_power.active());a.update();assert_eq!(a.world.anims.len(),1);
        a.scripts.vars.set("Swamp::Tile::PowerOn","False".into());a.connor_contact("tile00",[0.;3]);assert!(!a.world.swamp_power.active(),"external variable replacement synchronized before contact");
        a.scripts.vars.set("Swamp::Tile::PowerOn","False".into());
        a.console_command("getLoc tile00 Swamp::Tile::PowerOn","host assignment");assert!(!a.world.swamp_power.active());
        a.console_command("KQObject::setScript tile00 next.cs","late safe attachment");
        assert!(a.world.swamp_tiles["tile00"].has_script);
        assert!(a.world.swamp_power.active(),"the attached script's Init executes synchronously");
        a.world.swamp_reveal("tile00",true);assert_eq!(a.world.tile_bitmap_pairs("tile00").len(),2);
        let before=a.world.anims.len();a.update();assert_eq!(a.world.anims.len(),before,"new safe script suppresses its queued zero-damage gas");
        a.console_command("KQObject::setScript tile00 none","remove safe attachment");
        assert!(!a.world.swamp_tiles["tile00"].has_script);
        a.world.swamp_attack("tile00",0);assert_eq!(a.world.anims.len(),before+1,"removing the script re-enables the class hazard");
        a.console_command("KQObject::setScript tile00 next.cs","restore safe attachment");
        assert!(a.world.swamp_tiles["tile00"].has_script);
        let shooter=ObjectProjectile {file:"fire.kq".into(),offset:[10.,20.,30.],aim:[1000.,0.,0.],damage:37};
        let p=kq8_data::characters::ProjectileKeys::from_section(&kq8_data::kq_text::parse(b"[bolt]\nclassType=KQProjectile\nshapeName=fire.kqs\nvelocity=9000\nmaxDistance=21000\nweaponType=2\n")[0]).unwrap();
        a.world.projectiles.insert("fire.kq".into(),p);a.world.object_projectiles.insert("tile00".into(),shooter);
        a.world.clickables[0].placement.rotation=kq8_data::objects::rotation([0.,0.,std::f32::consts::FRAC_PI_2]);
        let expected=a.world.clickables[0].placement.to_world([10.,20.,30.]);
        for (value,damage) in [(-1,37),(0,37),(1,1),(99999,99999),(100000,37)] {
            a.world.object_projectile_cue("tile00",&["500".into(),value.to_string()]);let bolt=a.world.bolts.last().unwrap();
            assert_eq!(bolt.position,expected);assert!((bolt.direction[0]-1.).abs()<1e-6 && bolt.direction[1].abs()<1e-6 && bolt.direction[2].abs()<1e-6);assert_eq!(bolt.damage,damage);assert_eq!((bolt.speed,bolt.max),(9000.,21000.));assert_eq!(bolt.kind,crate::fight::HitKind::Fire);
        }
        a.world.variables_changed(&a.scripts.vars);
    }
}
