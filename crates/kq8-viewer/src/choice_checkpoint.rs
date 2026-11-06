//! Continuation of the original Choice controller without repeating its entry script.
use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedChoice {
    state: kq8_core::choice::Choice,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pose: Option<SavedControllerPose>,
    calls: Vec<Vec<String>>,
}

impl SavedChoice {
    pub fn valid(&self) -> bool {
        self.state.valid() && self.pose.as_ref().is_none_or(SavedControllerPose::valid)
            && self.calls.len() <= 64
            && self.calls.iter().all(|args| match args.as_slice() {
                [who, kind] => who.eq_ignore_ascii_case("Connor") && kind == "StartChoice",
                [who, kind, number] => who.eq_ignore_ascii_case("Connor") && kind == "DoChoice" && self.state.made
                    && self.state.list.points.iter().any(|(n, _)| n == number),
                _ => false,
            })
            && self.calls.iter().filter(|args| args.get(1).is_some_and(|kind| kind == "DoChoice")).count() <= 1
    }

    pub fn script(&self) -> &str { &self.state.list.script }

    /// Free pivots own their pose; a swap or an accepted post-choice action
    /// uses the existing action-specific checkpoint instead.
    pub fn compatible(&self, special: bool, swapping: bool, other_pose: bool) -> bool {
        match &self.pose {
            Some(_) => !special && !swapping && !other_pose,
            None => (special && !other_pose && (self.state.made || swapping)) || (self.state.made && other_pose && !special),
        }
    }
}

impl ConnorSim {
    /// Characters' Choice rule 9: an ended movement repeats the script's
    /// entry callback without re-entering or measuring the directions again.
    pub(super) fn choice_movement_ended(&mut self, movement: &str) {
        let Some(choice) = &self.choice else { return; };
        let script = choice.list.script.clone();
        self.choice_calls.push(vec!["Connor".into(), "StartChoice".into()]);
        self.log.detail(format!("connor update {}: Choice movement {movement} ended: {script} gets StartChoice; measured directions and decision kept (characters.md rule 9)", self.updates));
    }

    pub fn saved_choice(&self) -> Option<SavedChoice> {
        let state = self.choice.clone()?;
        let saved = SavedChoice { state, pose: (self.walker.mode == Mode::Free).then(|| self.capture_controller_pose()), calls: self.choice_calls.clone() };
        saved.valid().then_some(saved)
    }

    pub fn restore_choice(&mut self, saved: &SavedChoice) -> Result<(), String> {
        if !saved.valid() { return Err("invalid Choice checkpoint".into()); }
        if let Some(pose) = &saved.pose { self.restore_controller_pose(pose, Mode::Free, "Choice")?; }
        self.choice = Some(saved.state.clone());
        self.choice_calls.clone_from(&saved.calls);
        self.log.detail(format!("Choice restored: {}, made {}, cached {} points, {} pending call(s)", saved.script(), saved.state.made,
            saved.state.circle.as_ref().map_or(0, Vec::len), saved.calls.len()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callbacks_keep_authored_numbers_and_supported_pose_ownership() {
        let mut state = kq8_core::choice::Choice::new(kq8_core::choice::ChoiceList::parse("choice.cs,0,268329,480983,2,270551,488629").unwrap());
        state.circle = Some(kq8_core::choice::on_circle([270000.,482000.], &state.list.points));
        let mut saved = SavedChoice { state, pose: None, calls: vec![vec!["Connor".into(),"StartChoice".into()]] };
        assert!(saved.valid() && saved.compatible(true, true, false));
        assert!(!saved.compatible(true, false, false) && !saved.compatible(false, false, false));
        saved.state.made = true;
        // An ended movement may notify even after a decision, while the
        // script has left Connor in Choice (characters' rules 9 and 12).
        assert!(saved.valid());
        saved.calls = vec![vec!["Connor".into(),"DoChoice".into(),"2".into()]];
        let restored: SavedChoice = serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert!(restored.valid() && restored.compatible(true, false, false) && restored.compatible(false, false, true));
        assert_eq!(restored.calls, saved.calls);
        for bad in 0..4 {
            let mut invalid = saved.clone();
            match bad {
                0 => invalid.calls[0][2] = "1".into(), // a list index is not the authored choice number
                1 => invalid.calls.push(invalid.calls[0].clone()),
                2 => invalid.calls[0][0] = "another actor".into(),
                _ => invalid.state.made = false,
            }
            assert!(!invalid.valid());
        }
    }
}
