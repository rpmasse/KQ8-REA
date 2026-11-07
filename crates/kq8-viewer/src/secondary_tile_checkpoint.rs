use super::*;

#[derive(Debug,Clone,Serialize,Deserialize)]
struct Rock {file:String,aim:[f32;3],throw:String,saved_list:Option<String>,saved_mode:WeaponMode}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SavedSecondaryTile {
    kind:Kind,can_exit:bool,stage:Stage,remembered:Option<String>,back:bool,highlighted:Option<Cell>,sequence:Sequence,solved:bool,saved_projectile:Option<String>,
    pose:SavedControllerPose,mode:Mode,rock:Option<Rock>,
}
impl SavedSecondaryTile {
    pub fn valid(&self)->bool {
        let safe=|s:&str|!s.is_empty() && s.len()<1024 && !s.chars().any(char::is_control);
        let choice=|c:&Choice|c.point.into_iter().all(f32::is_finite) && c.cell.is_none_or(|c|(-2..=7).contains(&c.col) && (-1..=6).contains(&c.row));
        let stage=match &self.stage {Stage::Keys=>true,Stage::Turning(c)=>choice(c),Stage::Pending{choice:c,motion}|Stage::Jumping{choice:c,motion}=>choice(c) && safe(motion),Stage::Throwing=>self.kind==Kind::Swamp && self.rock.is_some()};
        stage && self.pose.valid() && matches!(self.mode,Mode::Free|Mode::Special|Mode::Throw)
            && self.remembered.as_deref().is_none_or(safe) && self.highlighted.is_none_or(|c|self.kind.actual(c))
            && self.sequence.revealed<128 && self.sequence.previous.is_none_or(|p|kq8_core::secondary_tiles::PRIMES.iter().any(|(_,q)|*q==p))
            && self.saved_projectile.as_deref().is_none_or(safe)
            && self.rock.as_ref().is_none_or(|r|safe(&r.file) && safe(&r.throw) && r.aim.into_iter().all(f32::is_finite))
    }
}
impl ConnorSim {
    pub fn saved_secondary_tile(&self)->Option<SavedSecondaryTile> {
        let t=self.secondary_tile.as_ref()?;
        Some(SavedSecondaryTile {kind:t.kind,can_exit:t.can_exit,stage:t.stage.clone(),remembered:t.remembered.clone(),back:t.back,highlighted:t.highlighted,
            sequence:t.sequence.clone(),solved:t.solved,saved_projectile:t.saved_projectile.clone(),pose:self.capture_controller_pose(),mode:self.walker.mode,
            rock:self.rock.as_ref().map(|r|Rock {file:r.file.clone(),aim:r.aim,throw:r.throw.into(),saved_list:r.saved_list.clone(),saved_mode:r.saved_mode})})
    }
    pub fn restore_secondary_tile(&mut self,s:&SavedSecondaryTile)->Result<(),String> {
        if !s.valid() {return Err("invalid secondary tile continuation".into());}
        let rock=s.rock.as_ref().map(|r|Ok::<_,String>(RockThrow {file:r.file.clone(),aim:r.aim,throw:self.saved_movement_name(&r.throw).ok_or("saved Rock throw motion missing")?,out:None,saved_list:r.saved_list.clone(),saved_mode:r.saved_mode})).transpose()?;
        self.restore_controller_pose(&s.pose,s.mode,"secondary tile")?;
        self.secondary_tile=Some(Run {kind:s.kind,grid:None,can_exit:s.can_exit,stage:s.stage.clone(),remembered:s.remembered.clone(),back:s.back,highlighted:s.highlighted,
            lit:None,sequence:s.sequence.clone(),solved:s.solved,saved_projectile:s.saved_projectile.clone()});
        self.rock=rock;self.walker.controller_owned_input=true;self.secondary_calls.clear();
        self.log.detail(format!("secondary tile {:?}: restored {:?}; grid pending; no entry/exit replay; highlight {:?} retained, unlit off",s.kind,s.stage,s.highlighted));
        Ok(())
    }
}
