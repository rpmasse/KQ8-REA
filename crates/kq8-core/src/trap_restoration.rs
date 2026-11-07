//! Pending saved-name contacts and elevator commands. Source:
//! behaviour/scripted-controller-commands.md rules 1, 5–10; traps.md Q203.
use super::Trap;
use crate::active::WorldState;
use std::collections::BTreeSet;

impl Trap {
    /// Restore names before collider registration; no contact or callback here.
    pub fn prepare_saved_occupants(&mut self, names: Vec<String>) -> Result<(), String> {
        let mut unique = BTreeSet::new();
        if names.len() > 4096 || names.iter().any(|name| name.is_empty() || name.len() > 1024
            || name.chars().any(char::is_control) || !unique.insert(name.to_ascii_lowercase())) {
            return Err("invalid saved trap occupant names".into());
        }
        if !self.inside.is_empty() {
            return Err("saved trap fields must be prepared before live contacts".into());
        }
        self.pending_occupants = names;
        Ok(())
    }

    /// Remove only an unresolved saved name. No live exit, callback or probe.
    pub fn cancel_saved_occupant(&mut self, name: &str) -> bool {
        let Some(index) = self.pending_occupants.iter().position(|saved| saved.eq_ignore_ascii_case(name)) else {
            return false;
        };
        self.pending_occupants.remove(index);
        true
    }
}

impl WorldState {
    pub(crate) fn saved_trap_command(&mut self, words: &[String], at: &str, cancel: bool) -> String {
        let [command, name] = words else {
            self.note(format!("{at}: usage: KQConner::{} <trap name>", if cancel { "removeFromTrap" } else { "inTrap" }));
            return "0".into();
        };
        // These two commands use exact object names, unlike saved occupant names.
        let Some(index) = self.traps.iter().position(|t| !t.gone && t.settings.name == *name)
            .filter(|_| !self.is_gone(name)) else {
            self.note(format!("{at}: {command}: no live trap named '{name}'"));
            return "0".into();
        };
        if cancel {
            let removed = self.traps[index].cancel_saved_occupant("Connor");
            self.note(format!("{at}: removeFromTrap {name}: cancelled pending Connor {removed}; live occupants unchanged"));
            "0".into()
        } else {
            let inside = self.queue.receiver("Connor").is_some() && !self.is_gone("Connor")
                && self.traps[index].inside.iter().any(|n| n.eq_ignore_ascii_case("Connor"));
            self.note(format!("{at}: inTrap {name}: {}; no contact probe or event service", u8::from(inside)));
            u8::from(inside).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{active::Active, collide::BallStack, trap::{SavedTraps, TrapSettings, Visitor, VisitorShape}};

    fn trap() -> Trap {
        Trap::new(TrapSettings { name: "portalt6".into(), position: [0.; 3],
            min: [-10.; 3], max: [10.; 3], script: None, lookup_call: true,
            mask: 4 | 512, trap_type: 0, inner_percent: None, spirit: None })
    }

    fn visitor<'a>(name: &'a str, x: f32, shape: VisitorShape<'a>) -> Visitor<'a> {
        Visitor { name, collision_type: if name.eq_ignore_ascii_case("Connor") {4} else {512},
            from: [x, 0., 0.], position: [x, 0., 0.], min: [-1.; 3], max: [1.; 3], shape }
    }

    #[test]
    fn saved_trap_contact_and_command_matrix() {
        let balls = BallStack::from_box([-1.; 3], [1.; 3]);
        let shapes = [VisitorShape::Balls(&balls), VisitorShape::Box([[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]])];
        for shape in shapes {
            let mut t = trap();
            t.prepare_saved_occupants(vec!["cOnNoR".into(), "oldRock".into()]).unwrap();
            for (name, x, collision_type) in [("Connor",100.,4),("Connor",0.,8),("newRock",0.,512)] {
                let mut v = visitor(name,x,shape); v.collision_type = collision_type;
                assert!(t.check(&v).call.is_none());
                assert!(t.inside.is_empty());
                assert_eq!(t.pending_occupants.len(),2,"only eligible matching contacts consume a saved name");
            }
            assert!(t.stay_check(&visitor("Connor",0.,shape)).call.is_none());
            assert!(t.inside.is_empty(),"placing an existing collider is not fresh registration");
            assert!(t.check(&visitor("Connor",0.,shape)).call.is_none());
            assert_eq!(t.inside,["Connor"]);
            assert_eq!(t.pending_occupants,["oldRock"]);
            assert!(t.check(&visitor("CONNOR",0.,shape)).call.is_none());
            assert_eq!(t.pending_occupants,["oldRock"],"repeat contact does not consume another entry");
            assert!(!t.cancel_saved_occupant("connor"),"an attached identity is no longer a pending name");
            assert!(t.check(&visitor("OLDrock",0.,shape)).call.is_none());
            assert!(t.pending_occupants.is_empty());
            assert_eq!(t.check(&visitor("newRock",0.,shape)).call.unwrap(),["Cue","301","newRock"]);
            assert_eq!(t.check(&visitor("Connor",100.,shape)).call.unwrap(),["Cue","302","Connor"]);
            assert_eq!(t.inside,["OLDrock","newRock"]);
        }

        let mut a = Active::new(Default::default(),vec![],vec![],["Connor".into(),"notATrap".into()],
            [("recover.cs".into(),b"KQConner::inTrap portalt6\nset Test::before $Console::Return\nKQConner::removeFromTrap portalt6\nKQConner::inTrap portalt6\nset Test::after $Console::Return\n".to_vec())]);
        a.world.register_event_receiver("Connor");
        let mut t = trap(); t.prepare_saved_occupants(vec!["cOnNoR".into()]).unwrap();
        a.add_traps([t]);
        a.world.queue.send(a.world.now(),0.,"Connor","KQCueEvent",&["17".into()],"queued before query");
        let queued = a.world.queue.len();
        assert!(a.run_script("recover.cs",&[]));
        assert_eq!(a.scripts.vars.get("Test::before"),Some("0"));
        assert_eq!(a.scripts.vars.get("Test::after"),Some("0"));
        assert_eq!(a.world.queue.len(),queued,"trap commands must not service events");
        assert!(a.world.traps[0].inside.is_empty() && a.world.traps[0].pending_occupants.is_empty());
        let live = visitor("Connor",0.,shapes[0]);
        assert_eq!(a.world.traps[0].check(&live).call.unwrap(),["On","Connor"],"later new contact after cancellation follows ordinary entry");
        assert!(a.run_script("recover.cs",&[]));
        assert_eq!(a.scripts.vars.get("Test::before"),Some("1"));
        assert_eq!(a.scripts.vars.get("Test::after"),Some("1"),"cancellation must not remove a live Connor");
        for cmd in ["KQConner::inTrap PortalT6","KQConner::inTrap notATrap","KQConner::inTrap 123",
            "KQConner::inTrap","KQConner::inTrap portalt6 extra","KQConner::removeFromTrap portalt6 extra"] {
            assert_eq!(a.console_command(cmd,"trap matrix"),Some("0".into()));
        }
        assert_eq!(a.world.traps[0].inside,["Connor"]);
        assert_eq!(a.world.queue.len(),queued);

        // Completed restore saves live names; reconstruction needs contacts again.
        // The separate unresolved-name field preserves the unique union,
        // avoiding the original repeated-save duplicate-accumulation defect.
        let mut source = a.world.traps[0].clone(); source.pending_occupants=vec!["oldRock".into()];
        source.settings.mask=512;
        let saved: SavedTraps = serde_json::from_slice(&serde_json::to_vec(&SavedTraps::capture([&source])).unwrap()).unwrap();
        let mut restored = trap();
        assert!(saved.restore_before_registration(&mut restored).unwrap());
        assert!(restored.inside.is_empty());
        assert_eq!(restored.pending_occupants,["Connor","oldRock"]);
        assert!(restored.check(&live).call.is_none());
        assert!(restored.inside.is_empty(),"disabled Connor mask cannot restore occupancy from a name");
        assert!(restored.check(&visitor("oldRock",0.,shapes[0])).call.is_none());
        assert_eq!(restored.inside,["oldRock"]);
        assert_eq!(restored.pending_occupants,["Connor"]);
        let partial = SavedTraps::capture([&restored]);
        let mut resumed=trap(); partial.restore_before_registration(&mut resumed).unwrap();
        assert_eq!(resumed.pending_occupants,["oldRock","Connor"]);
        assert!(resumed.inside.is_empty());
        assert!(resumed.cancel_saved_occupant("cOnNoR"));
        assert!(resumed.check(&visitor("oldRock",0.,shapes[0])).call.is_none());
        assert!(resumed.pending_occupants.is_empty());
        let mut too_late=source.clone(); let before=too_late.clone();
        assert!(saved.restore_before_registration(&mut too_late).is_err());
        assert_eq!(too_late,before,"failed late field application is atomic");

        let mut invalid=serde_json::to_value(&saved).unwrap();
        invalid["states"][0]["pending_inside"]=serde_json::json!(["cOnNoR"]);
        let invalid:SavedTraps=serde_json::from_value(invalid).unwrap();
        assert!(!invalid.valid(),"live and pending names must be disjoint ignoring capitals");
        let before=restored.clone();
        assert!(invalid.restore_before_registration(&mut restored).is_err());
        assert_eq!(restored,before);
        for names in [vec!["Connor".into(),"cOnNoR".into()],vec!["".into()],vec!["bad\nname".into()]] {
            let mut fresh=trap();let before=fresh.clone();
            assert!(fresh.prepare_saved_occupants(names).is_err());assert_eq!(fresh,before);
        }
        let mut old=serde_json::to_value(&saved).unwrap();
        old["states"][0].as_object_mut().unwrap().remove("pending_inside");
        let old:SavedTraps=serde_json::from_value(old).unwrap();
        let mut fresh=trap();old.restore_before_registration(&mut fresh).unwrap();
        assert_eq!(fresh.pending_occupants,["Connor"],"older snapshots retain their available saved names");
        println!("saved trap matrix: sphere/box eligibility, silent ordered attachment, exact-name commands, pending cancellation, no queue service, checkpoints and invalid-data rejection passed; production loader integration not exercised");
    }
}
