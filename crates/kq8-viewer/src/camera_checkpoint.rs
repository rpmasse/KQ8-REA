use super::*;
type Pose = ([f32; 3], f32, f32);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedCamera {
    cam: Camera,
    previous: Pose,
    current: Pose,
    glide_from: Option<Pose>,
    roll_before: f32,
    mouse_was_held: bool,
    following: bool,
    #[serde(default)]
    spots: Option<Vec<[f32; 3]>>,
    flight: Option<Flight>,
    flight_fresh: bool,
    view: View,
    places: bool,
    aims: bool,
    last_connor: Option<([f32; 3], f32)>,
    pending_start: bool,
    pending_follow: bool,
    around: Option<[f32; 3]>,
    skip_next_move: bool,
    move_skipped: bool,
    after_flight: Option<bool>,
    follow_due: Option<bool>,
    #[serde(default)]
    shake: kq8_core::camera_shake::Shake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first_person: Option<FirstPersonView>,
}

impl SavedCamera {
    pub fn valid(&self) -> bool {
        let pose = |p: Pose| p.0.into_iter().chain([p.1, p.2]).all(f32::is_finite);
        self.cam.valid_checkpoint() && self.shake.valid() && self.first_person.as_ref().is_none_or(FirstPersonView::valid)
            && pose(self.previous) && pose(self.current) && self.glide_from.is_none_or(pose)
            && [self.roll_before, self.view.tilt, self.view.heading, self.view.roll].into_iter().all(f32::is_finite)
            && self.last_connor.is_none_or(|p| p.0.into_iter().chain([p.1]).all(f32::is_finite))
            && self.around.is_none_or(|p| p.into_iter().all(f32::is_finite))
            && self.spots.as_ref().is_none_or(|s| s.len() <= 4096 && s.iter().flatten().all(|v| v.is_finite()))
            && self.flight.as_ref().is_none_or(|f| f.n > 0 && f.k <= f.n
                && f.spot.iter().chain(&f.aim).chain(&f.step).all(|v| v.is_finite()) && f.roll_step.is_finite())
    }
}

impl GameCam {
    pub fn saved(&self) -> SavedCamera {
        SavedCamera { cam: self.cam.clone(), first_person: self.first_person.clone(),
            previous: self.previous, current: self.current, glide_from: self.glide_from,
            roll_before: self.roll_before, mouse_was_held: self.mouse_was_held, following: self.following, flight: self.flight.clone(), flight_fresh: self.flight_fresh,
            spots: Some(self.spots.0.clone()),
            view: self.view, places: self.places, aims: self.aims, last_connor: self.last_connor,
            pending_start: self.pending.contains(&"start (stand-in)"), pending_follow: self.pending.iter().any(|p| *p != "start (stand-in)"),
            around: self.around, skip_next_move: self.skip_next_move, move_skipped: self.move_skipped,
            after_flight: self.after_flight, follow_due: self.follow_due, shake:self.shake.clone() }
    }

    pub fn restore(&mut self, saved: &SavedCamera) -> Result<(), String> {
        if !saved.valid() { return Err("invalid camera checkpoint".into()); }
        self.cam = saved.cam.clone(); self.previous = saved.previous; self.current = saved.current;
        self.first_person.clone_from(&saved.first_person);
        self.actor = None;
        self.glide_from = saved.glide_from; self.roll_before = saved.roll_before; self.following = saved.following;
        self.flight = saved.flight.clone(); self.flight_fresh = saved.flight_fresh; self.view = saved.view;
        if let Some(spots) = &saved.spots { self.spots = Spots(spots.clone()); }
        self.places = saved.places; self.aims = saved.aims; self.last_connor = saved.last_connor;
        self.around = saved.around; self.skip_next_move = saved.skip_next_move; self.move_skipped = saved.move_skipped;
        self.after_flight = saved.after_flight; self.follow_due = saved.follow_due;
        self.shake.clone_from(&saved.shake);
        self.pending.clear();
        if saved.pending_start { self.pending.push("start (stand-in)"); }
        if saved.pending_follow { self.pending.push("disk follow continuation"); }
        self.mouse = Vec2::ZERO; self.mouse_was_held = saved.mouse_was_held; self.pressed_mode = false; self.pressed_reset = false;
        self.script_moves.clear();
        self.log.detail(format!("camera checkpoint restored at ({:.3}, {:.3}, {:.3}), following {}, around {:?}, flight {:?}, {} destination spots",
            self.cam.position[0], self.cam.position[1], self.cam.position[2], self.following, self.around, self.flight.as_ref().map(|f| (f.k, f.n)), self.spots.0.len()));
        if self.shake.active() {
            self.log.detail(format!("camera shake restored after sample {}, amplitudes {:?}; next sync continues",self.shake.count(),self.shake.amplitude()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> GameCam {
        GameCam { cam: Camera::new([0.0; 3]), first_person: None, actor: None,
            previous: ([0.0; 3], 0.0, 0.0), current: ([0.0; 3], 0.0, 0.0),
            glide_from: None, roll_before: 0.0,
            keys: Keys { first_person: None, change_mode: None, tilt_up: None, tilt_down: None, reset: None, shift: None, mouse: None },
            mouse: Vec2::ZERO, mouse_was_held: false, pressed_mode: false, pressed_reset: false, updates: 0,
            log: Log::create("camera-checkpoint-unit").unwrap(), following: false, spots: Spots::default(),
            flight: None, flight_fresh: false, view: View { tilt: 0.0, heading: 0.0, roll: 0.0, locked: false },
            places: false, aims: false, last_connor: None, pending: vec![], script_moves: vec![], around: None,
            skip_next_move: false, move_skipped: false, after_flight: None, follow_due: None, shake:Default::default() }
    }

    #[test]
    fn a_shake_checkpoint_keeps_the_current_ramp_and_fixed_view_without_restarting() {
        let mut original=camera();
        original.move_to(0,[100.,200.,3000.],[-0.3,0.2,0.7]);
        original.start_shake(kq8_core::camera_shake::Parameters {seconds:4.,maximum:[0.02,0.03],rate:[4.,5.]});
        for _ in 0..29 { original.shake_sync(); }
        let saved:SavedCamera=serde_json::from_slice(&serde_json::to_vec(&original.saved()).unwrap()).unwrap();
        let mut restored=camera(); restored.restore(&saved).unwrap();
        assert_eq!(restored.current,original.current); assert_eq!(restored.view,original.view);
        assert_eq!(restored.shake.count(),29);
        assert_eq!(restored.shake_sync(),original.shake_sync());
        assert_eq!(restored.current,original.current);
        assert_eq!(restored.shake.count(),30);
        let mut legacy=serde_json::to_value(&saved).unwrap(); legacy.as_object_mut().unwrap().remove("shake");
        restored.restore(&serde_json::from_value(legacy).unwrap()).unwrap();
        assert!(!restored.shake.active());
        let mut bad=serde_json::to_value(&saved).unwrap(); bad["shake"]["count"]=999.into();
        let before=restored.current;
        assert!(restored.restore(&serde_json::from_value(bad).unwrap()).is_err());
        assert_eq!(restored.current,before);
    }

    #[test]
    fn fixed_shake_accumulates_and_replacement_or_stop_never_restores_a_base_pose() {
        let mut g=camera();
        g.move_to(0,[100.,200.,3000.],[-0.3,0.2,0.7]);
        let parameters=kq8_core::camera_shake::Parameters {seconds:4.,maximum:[0.02,0.03],rate:[4.,5.]};
        g.start_shake(parameters);
        let first=g.current;
        assert!(!g.shake_sync());
        assert!((g.current.1-first.1-0.0006143527).abs()<1e-7);
        assert!((g.current.2-first.2-0.0009944087).abs()<1e-7);
        for _ in 1..17 { g.shake_sync(); }
        let prior=g.current;
        g.start_shake(parameters); g.shake_sync();
        assert_eq!(g.shake.count(),1);
        assert!((g.current.1-prior.1-0.0006143527).abs()<1e-7);
        assert_eq!(g.cam.position,first.0); assert_eq!(g.view.roll,0.2);
        for _ in 1..60 { g.shake_sync(); }
        let last=g.current;
        assert!(!g.shake_sync()); assert_eq!(g.current,last);
        assert!(!g.shake.active()); assert_eq!(g.view.roll,0.2);
    }

    #[test]
    fn destination_order_empty_list_and_legacy_fallback_survive_restore() {
        let mut g = camera();
        g.add_spot([-100.0, 0.0, 3000.0], true);
        g.add_spot([100.0, 0.0, 3000.0], false);
        let saved: SavedCamera = serde_json::from_slice(&serde_json::to_vec(&g.saved()).unwrap()).unwrap();
        g.add_spot([999.0; 3], true);
        g.restore(&saved).unwrap();
        assert_eq!(g.spots.0, [[-100.0, 0.0, 3000.0], [100.0, 0.0, 3000.0]]);
        assert_eq!(g.spots.nearest([0.0, 0.0, 3000.0]), Some([-100.0, 0.0, 3000.0]));
        let mut old = serde_json::to_value(&saved).unwrap();
        old.as_object_mut().unwrap().remove("spots");
        let old: SavedCamera = serde_json::from_value(old).unwrap();
        g.add_spot([999.0; 3], true);
        g.restore(&old).unwrap();
        assert_eq!(g.spots.0, [[999.0; 3]], "older slots retain the loaded world's destination fallback");
        let empty = camera().saved();
        g.restore(&empty).unwrap();
        assert!(g.spots.0.is_empty(), "an explicitly saved empty list must clear startup destinations");
        let mut invalid = saved;
        invalid.spots = Some(vec![[f32::NAN, 0.0, 0.0]]);
        assert!(g.restore(&invalid).is_err());
        assert!(g.spots.0.is_empty(), "invalid coordinates must fail before modifying the live camera");
    }
}
