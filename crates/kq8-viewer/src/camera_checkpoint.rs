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
}

impl SavedCamera {
    pub fn valid(&self) -> bool {
        let pose = |p: Pose| p.0.into_iter().chain([p.1, p.2]).all(f32::is_finite);
        self.cam.valid_checkpoint() && pose(self.previous) && pose(self.current) && self.glide_from.is_none_or(pose)
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
        SavedCamera { cam: self.cam.clone(), previous: self.previous, current: self.current, glide_from: self.glide_from,
            roll_before: self.roll_before, mouse_was_held: self.mouse_was_held, following: self.following, flight: self.flight.clone(), flight_fresh: self.flight_fresh,
            spots: Some(self.spots.0.clone()),
            view: self.view, places: self.places, aims: self.aims, last_connor: self.last_connor,
            pending_start: self.pending.contains(&"start (stand-in)"), pending_follow: self.pending.iter().any(|p| *p != "start (stand-in)"),
            around: self.around, skip_next_move: self.skip_next_move, move_skipped: self.move_skipped,
            after_flight: self.after_flight, follow_due: self.follow_due }
    }

    pub fn restore(&mut self, saved: &SavedCamera) -> Result<(), String> {
        if !saved.valid() { return Err("invalid camera checkpoint".into()); }
        self.cam = saved.cam.clone(); self.previous = saved.previous; self.current = saved.current;
        self.glide_from = saved.glide_from; self.roll_before = saved.roll_before; self.following = saved.following;
        self.flight = saved.flight.clone(); self.flight_fresh = saved.flight_fresh; self.view = saved.view;
        if let Some(spots) = &saved.spots { self.spots = Spots(spots.clone()); }
        self.places = saved.places; self.aims = saved.aims; self.last_connor = saved.last_connor;
        self.around = saved.around; self.skip_next_move = saved.skip_next_move; self.move_skipped = saved.move_skipped;
        self.after_flight = saved.after_flight; self.follow_due = saved.follow_due;
        self.pending.clear();
        if saved.pending_start { self.pending.push("start (stand-in)"); }
        if saved.pending_follow { self.pending.push("disk follow continuation"); }
        self.mouse = Vec2::ZERO; self.mouse_was_held = saved.mouse_was_held; self.pressed_mode = false; self.pressed_reset = false;
        self.script_moves.clear();
        self.log.detail(format!("camera checkpoint restored at ({:.3}, {:.3}, {:.3}), following {}, around {:?}, flight {:?}, {} destination spots",
            self.cam.position[0], self.cam.position[1], self.cam.position[2], self.following, self.around, self.flight.as_ref().map(|f| (f.k, f.n)), self.spots.0.len()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> GameCam {
        GameCam { cam: Camera::new([0.0; 3]), previous: ([0.0; 3], 0.0, 0.0), current: ([0.0; 3], 0.0, 0.0),
            glide_from: None, roll_before: 0.0,
            keys: Keys { change_mode: None, tilt_up: None, tilt_down: None, reset: None, shift: None, mouse: None },
            mouse: Vec2::ZERO, mouse_was_held: false, pressed_mode: false, pressed_reset: false, updates: 0,
            log: Log::create("camera-checkpoint-unit").unwrap(), following: false, spots: Spots::default(),
            flight: None, flight_fresh: false, view: View { tilt: 0.0, heading: 0.0, roll: 0.0, locked: false },
            places: false, aims: false, last_connor: None, pending: vec![], script_moves: vec![], around: None,
            skip_next_move: false, move_skipped: false, after_flight: None, follow_due: None }
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
