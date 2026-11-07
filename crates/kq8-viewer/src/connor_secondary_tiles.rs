//! Original Swamp/Barren controllers, driven by authored motion and the mover.
//! behaviour/secondary-tile-puzzles.md rules 1–15, 26–35, 44, 46–50.
use super::{ConnorSim, RockThrow, SavedControllerPose, find};
use kq8_core::{connor::{Intent,Mode,choose_movement},connor_auto::{AutoKind,AutoMove},secondary_tiles::{Kind,Grid,Cell,Choice,Where,Sequence},weapons::WeaponMode};
use serde::{Serialize,Deserialize};

#[path="secondary_tile_checkpoint.rs"]
mod checkpoint;
pub use checkpoint::SavedSecondaryTile;

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum Stage { Keys, Turning(Choice), Pending {choice:Choice,motion:String}, Jumping {choice:Choice,motion:String}, Throwing }
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum Call { Line(String), Cue(String,i32,Option<String>), Click(String), Complete, Script(String), RestoreProjectile(Option<String>), Sound(String) }
impl Call {
    /// Only the output families emitted by these two controllers are saved.
    /// Pending effects can outlive the controller that produced them.
    pub fn valid_queue(calls:&[Self])->bool {
        let token=|s:&str|!s.is_empty() && s.len()<=1024 && !s.chars().any(|c|c.is_control() || c.is_whitespace());
        calls.len()<=256 && calls.iter().filter(|c|matches!(c,Self::Complete)).count()<=1 && calls.iter().all(|call|match call {
            Self::Line(line)=>matches!(line.as_str(),"KQMusic::push mlava.aud"|"KQMusic::pop mlava.aud"),
            Self::Cue(name,cue,bitmap)=>{
                if name.eq_ignore_ascii_case("fiendoor") {return *cue==1 && bitmap.is_none();}
                let bytes=name.as_bytes();
                bytes.len()==6 && bytes[..4].eq_ignore_ascii_case(b"tile") && (b'0'..=b'4').contains(&bytes[4]) && (b'0'..=b'6').contains(&bytes[5])
                    && match cue {2400=>bitmap.as_deref().is_some_and(|b|kq8_core::secondary_tiles::LAVA_BITMAPS.iter().any(|known|b.eq_ignore_ascii_case(known))),2401..=2403=>bitmap.is_none(),_=>false}
            }
            Self::Click(name)=>name.eq_ignore_ascii_case("Rtswdoor"),
            Self::Complete=>true,
            Self::Script(name)=>["tiletps2.cs","tiletpn2.cs"].iter().any(|known|name.eq_ignore_ascii_case(known)),
            Self::RestoreProjectile(file)=>file.as_deref().is_none_or(token),
            Self::Sound(name)=>kq8_core::secondary_tiles::PRIMES.iter().any(|(_,p)|name.eq_ignore_ascii_case(&format!("T{p}.aud"))),
        })
    }

    /// The production bridge drains each retained effect exactly once.
    pub fn dispatch(calls:&mut Vec<Self>,active:&mut kq8_core::active::Active) {
        for call in std::mem::take(calls) {match call {
            Self::Line(line)=>{active.test_command(&line,"secondary tile");}
            Self::Cue(name,cue,bmp)=>{let mut data=vec![cue.to_string(),"0".into()];if let Some(bmp)=bmp {data.push(bmp);}
                let w=&mut active.world;w.queue.send(w.now(),0.,&name,"KQCueEvent",&data,"secondary tile");}
            Self::Click(name)=>{active.call_named(&name,&["ConnorAction","Connor"]);}
            Self::Sound(name)=>{active.world.sounds.push(kq8_core::sound::Start::plain_one_shot(&name,"Tile3 prime".into()));}
            Self::RestoreProjectile(file)=>{active.world.connor_projectile=file;}
            Self::Complete=>{
                active.world.solved_tile3=true;active.scripts.vars.set("Barren::Tile::Complete","True".into());
                active.test_command("sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 50000","Tile3 completion");
                active.test_command("addMapIcon ba_Order.bmp 155 177","Tile3 completion");
            }
            Self::Script(name)=>{if active.world.connor_stats.is_some_and(|s|s.stats.0[0]>0) {active.run_script(&name,&["StartExit".into()]);}}
        }}
    }
}
#[derive(Debug,Clone)]
pub struct Run {
    pub kind:Kind,
    grid:Option<Grid>,
    pub can_exit:bool,
    stage:Stage,
    remembered:Option<String>,
    back:bool,
    pub highlighted:Option<Cell>,
    /// Original highlight index survives saves; per-object unlit does not.
    pub lit:Option<Cell>,
    sequence:Sequence,
    pub solved:bool,
    saved_projectile:Option<String>,
}
impl Run {
    pub fn lit_name(&self)->Option<String> {self.lit.map(|c|self.kind.name(c))}
    fn light(&mut self,next:Option<Cell>) {if self.highlighted!=next {self.highlighted=next;self.lit=next;}}
}

impl ConnorSim {
    pub fn enter_secondary_tile(&mut self,kind:Kind,world:&kq8_core::active::WorldState,at:&str)->String {
        let (grid,missing)=Grid::read(kind,|n|world.place_of(n).map(|p|p.position));
        self.walker.auto=None;self.intent.forward=0;
        let armed=self.set_mode!=WeaponMode::None;
        if armed {
            let away=self.drawn("away").map(|m|m.name);
            self.switch_set(None,WeaponMode::None);self.weapon_target=None;self.target_mode=WeaponMode::None;self.ready_item=None;
            if let Some(away)=away {self.walker.start_special(vec![kq8_core::connor::SpecialStep::Play(away,"away".into())]);}
        }
        self.secondary_tile=Some(Run {kind,grid:Some(grid),can_exit:false,stage:Stage::Keys,remembered:None,back:false,highlighted:None,lit:None,sequence:Sequence::default(),solved:world.solved_tile3,saved_projectile:None});
        self.walker.controller_owned_input=true;self.walker.lean_allowed=false;
        if !armed && kind==Kind::Swamp {self.secondary_request("stop");self.secondary_tile.as_mut().unwrap().remembered=Some("stop".into());self.secondary_highlight(false);}
        if kind==Kind::Barren {self.secondary_calls.push(Call::Line("KQMusic::push mlava.aud".into()));}
        format!("{at}: enters {kind:?} tile controller; {} missing named tiles; exit off; armed entry {armed}",missing.len())
    }
    fn secondary_request(&mut self,name:&str) {
        let Some(motion)=self.drawn(name).map(|m|m.name) else {self.log.detail(format!("secondary tile: missing motion {name}"));return;};
        let movements=&self.movements;let set=self.weapon_set.as_deref();
        let notes=self.walker.script_request(motion,&|n|find(movements,set,n).map(|m|&m.movement_loop));
        for note in notes {self.log.detail(format!("secondary tile request {name}: {note}"));}
    }
    fn secondary_highlight(&mut self,back:bool) {
        let Some(t)=&mut self.secondary_tile else {return;};let Some(grid)=&t.grid else {return;};
        if !matches!(t.kind.where_is(self.mover.position),Where::Cell(_)) {return;}
        let choice=grid.choose(self.mover.position,self.walker.heading,back,t.solved,true);
        t.light(choice.and_then(|c|c.cell).filter(|c|grid.found(*c)));
    }
    /// Restoration rebuilds before the busy check and does not replay entry.
    pub fn secondary_rebuild(&mut self,world:&kq8_core::active::WorldState) {
        let Some(t)=&mut self.secondary_tile else {return;};if t.grid.is_some() {return;}
        let (grid,missing)=Grid::read(t.kind,|n|world.place_of(n).map(|p|p.position));t.grid=Some(grid);
        t.remembered=Some(kq8_core::connor::plain_movement_name(self.walker.movement).into());
        self.walker.controller_owned_input=true;self.log.detail(format!("secondary tile restored grid before busy: {} missing",missing.len()));
    }
    pub fn secondary_holds(&self)->bool {self.secondary_tile.as_ref().is_some_and(|t|!t.can_exit)}
    pub fn leave_secondary_tile(&mut self) {
        let Some(t)=self.secondary_tile.take() else {return;};
        self.mover.end_tile_jump();self.walker.controller_owned_input=false;self.walker.auto=None;
        if matches!(t.stage,Stage::Throwing) {self.rock=None;self.hands_off=false;self.secondary_calls.push(Call::RestoreProjectile(t.saved_projectile));}
        if t.kind==Kind::Barren {
            self.secondary_calls.push(Call::Line("KQMusic::pop mlava.aud".into()));
            if !t.solved {for col in 0..7 {for row in 0..5 {self.secondary_calls.push(Call::Cue(Kind::Barren.name(Cell::new(col,row)),2402,None));}}}
        }
    }
    pub fn secondary_exit(&mut self,on:bool)->String {
        if let Some(t)=&mut self.secondary_tile {t.can_exit=on;self.mover.end_tile_jump();if on {self.leave_secondary_tile();self.walker.stand_now();}}
        format!("secondary tileExit {}: physics restored",u8::from(on))
    }
    pub fn secondary_step(&mut self,intent:&mut Intent,jump:bool) {
        let Some(t)=&self.secondary_tile else {return;};let stage=t.stage.clone();let kind=t.kind;
        self.walker.controller_owned_input=true;self.walker.lean_allowed=false;
        let wanted=choose_movement(*intent,self.mover.submerged);intent.forward=0;
        match stage {
            Stage::Keys=>{
                if self.walker.mode!=Mode::Free {intent.turn=0;return;}
                if matches!(wanted,"waterwalk"|"waterrun") {return;}
                let changed=self.secondary_tile.as_ref().unwrap().remembered.as_deref()!=Some(wanted);
                let choose=(changed && matches!(wanted,"walk"|"run"|"back")) || jump;
                if choose {
                    if !jump {self.secondary_tile.as_mut().unwrap().back=wanted=="back";}
                    let t=self.secondary_tile.as_ref().unwrap();let back=t.back;
                    let choice=t.grid.as_ref().and_then(|g|g.choose(self.mover.position,self.walker.heading,back,t.solved,false));
                    if let Some(choice)=choice {
                        if back {let lit=choice.cell.filter(|c|t.grid.as_ref().unwrap().found(*c));self.secondary_tile.as_mut().unwrap().light(lit);}
                        intent.turn=0;
                        if choice.ordinary {self.secondary_begin_jump(choice);}
                        else {let face=choice.turn_point(self.mover.position,back);self.walker.auto=Some(AutoMove::new(AutoKind::Point,[face[0],face[1]],None));self.secondary_tile.as_mut().unwrap().stage=Stage::Turning(choice);}
                    } else {self.log.detail(format!("{kind:?} tile: no established destination at {:?}",self.mover.position));}
                } else if changed {self.secondary_request(wanted);self.secondary_highlight(false);}
                self.secondary_tile.as_mut().unwrap().remembered=Some(wanted.into());
                if intent.turn!=0 {self.secondary_highlight(false);}
            }
            Stage::Turning(choice)=>{intent.turn=0;if self.walker.auto.is_none() {self.secondary_begin_jump(choice);}}
            Stage::Pending {choice,motion}=>{intent.turn=0;if kq8_core::connor::plain_movement_name(self.walker.movement).eq_ignore_ascii_case(&motion) {
                self.mover.start_tile_jump();
                self.log.detail(format!("secondary tile {kind:?}: accepted {motion} at update {}; physics off; position {:?}",self.updates,self.mover.position));
                self.secondary_tile.as_mut().unwrap().stage=Stage::Jumping {choice,motion};
            }}
            Stage::Jumping {..}|Stage::Throwing=>{intent.turn=0;}
        }
    }
    fn secondary_begin_jump(&mut self,choice:Choice) {
        let t=self.secondary_tile.as_ref().unwrap();let kind=t.kind;let motion=choice.motion(t.back).to_string();
        self.secondary_request(&motion);
        let started=kq8_core::connor::plain_movement_name(self.walker.movement).eq_ignore_ascii_case(&motion);
        self.log.detail(format!("secondary tile {kind:?}: request {motion} at update {}; target {:?}; actual accepted {started}; physics off {}",self.updates,choice.cell,kind==Kind::Swamp || started));
        let stage=if kind==Kind::Swamp || started {self.mover.start_tile_jump();Stage::Jumping {choice,motion}} else {Stage::Pending {choice,motion}};
        self.secondary_tile.as_mut().unwrap().stage=stage;
    }
    pub fn secondary_motion_ended(&mut self,motion:&str) {
        let Some(t)=&self.secondary_tile else {return;};let Stage::Jumping {choice,motion:expected}=t.stage.clone() else {return;};
        if !kq8_core::connor::plain_movement_name(motion).eq_ignore_ascii_case(&expected) {return;}
        self.mover.end_tile_jump();self.secondary_request("stop");
        let t=self.secondary_tile.as_mut().unwrap();t.stage=Stage::Keys;t.remembered=None;
        let kind=t.kind;let where_is=kind.where_is(self.mover.position);
        self.log.detail(format!("secondary tile {kind:?}: landed {motion} at update {}; position {:?}; cell {where_is:?}; physics restored",self.updates,self.mover.position));
        match (kind,where_is) {
            (Kind::Swamp,Where::Cell(c)) if c.row==3 && c.col<0=>self.secondary_calls.push(Call::Cue("fiendoor".into(),1,None)),
            (Kind::Swamp,Where::Cell(c)) if c.row==3 && c.col>=6=>self.secondary_calls.push(Call::Click("Rtswdoor".into())),
            (Kind::Barren,_)=>{
                let cell=match where_is {Where::Cell(c)=>Some(c),_=>None};let landing=t.sequence.land(cell,t.solved);
                let mut cues=landing.cues;
                // First reveal precedes the completion. Its bridge commands
                // follow the T19 sound, persistent flag and queued experience.
                if landing.complete && cues.first().is_some_and(|(name,cue,_)|name=="tile24" && *cue==2400) {
                    let (name,cue,bmp)=cues.remove(0);self.secondary_calls.push(Call::Cue(name,cue,bmp));
                }
                if let Some(sound)=landing.sound {self.secondary_calls.push(Call::Sound(sound));}
                if landing.complete {t.solved=true;self.secondary_calls.push(Call::Complete);}
                for (name,cue,bmp) in cues {self.secondary_calls.push(Call::Cue(name,cue,bmp));}
                if choice.exit {t.can_exit=true;self.secondary_calls.push(Call::Script((if where_is==Where::South {"tiletps2.cs"} else {"tiletpn2.cs"}).into()));}
            }
            _=>{}
        }
        self.secondary_highlight(false);
    }
    pub fn start_secondary_rock(&mut self,file:String,aim:[f32;3],previous:Option<String>)->bool {
        if !self.secondary_tile.as_ref().is_some_and(|t|t.kind==Kind::Swamp && matches!(t.stage,Stage::Keys)) || self.hands_off {return false;}
        let Some(away)=self.drawn("away").map(|m|m.name) else {return false;};let Some(throw)=find(&self.movements,None,"throw").map(|m|m.name) else {return false;};
        self.walker.auto=Some(AutoMove::new(AutoKind::Point,[aim[0],aim[1]],None));self.hands_off=true;
        let t=self.secondary_tile.as_mut().unwrap();t.stage=Stage::Throwing;t.saved_projectile=previous;
        self.rock=Some(RockThrow {file,aim,throw,out:None,saved_list:self.weapon_set.clone(),saved_mode:self.set_mode});self.walker.throw_play(away);true
    }
    pub fn secondary_rock_finished(&mut self)->bool {
        let Some(t)=self.secondary_tile.as_mut().filter(|t|matches!(t.stage,Stage::Throwing)) else {return false;};
        t.stage=Stage::Keys;t.remembered=None;self.hands_off=false;self.secondary_calls.push(Call::RestoreProjectile(t.saved_projectile.take()));
        self.walker.throw_leave();self.walker.controller_owned_input=true;true
    }
}
