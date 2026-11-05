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
            && self.flight.as_ref().is_none_or(|f| f.n > 0 && f.k <= f.n
                && f.spot.iter().chain(&f.aim).chain(&f.step).all(|v| v.is_finite()) && f.roll_step.is_finite())
    }
}

impl GameCam {
    pub fn saved(&self) -> SavedCamera {
        SavedCamera { cam: self.cam.clone(), previous: self.previous, current: self.current, glide_from: self.glide_from,
            roll_before: self.roll_before, mouse_was_held: self.mouse_was_held, following: self.following, flight: self.flight.clone(), flight_fresh: self.flight_fresh,
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
        self.places = saved.places; self.aims = saved.aims; self.last_connor = saved.last_connor;
        self.around = saved.around; self.skip_next_move = saved.skip_next_move; self.move_skipped = saved.move_skipped;
        self.after_flight = saved.after_flight; self.follow_due = saved.follow_due;
        self.pending.clear();
        if saved.pending_start { self.pending.push("start (stand-in)"); }
        if saved.pending_follow { self.pending.push("disk follow continuation"); }
        self.mouse = Vec2::ZERO; self.mouse_was_held = saved.mouse_was_held; self.pressed_mode = false; self.pressed_reset = false;
        self.script_moves.clear();
        self.log.detail(format!("camera checkpoint restored at ({:.3}, {:.3}, {:.3}), following {}, around {:?}, flight {:?}",
            self.cam.position[0], self.cam.position[1], self.cam.position[2], self.following, self.around, self.flight.as_ref().map(|f| (f.k, f.n))));
        Ok(())
    }
}
