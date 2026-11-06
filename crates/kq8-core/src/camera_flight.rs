//! Authored camera spots and flight motion (`camera.md`, flight rules 1–7,
//! Q193). Motion does not own another aiming or range/height controller:
//! after each step the caller runs `Camera::full_update` exactly once.
use crate::camera::{Camera, Followed, HeadingState, DEFAULT_TILT};

pub const FLIGHT_SPEED: f32 = 5000.0;
const UPDATES: f32 = 15.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spots(pub Vec<[f32; 3]>);

impl Spots {
    pub fn add(&mut self, spot: [f32; 3], newlist: bool) {
        if newlist {
            self.0.clear();
        }
        self.0.push(spot);
    }

    /// The nearest spot in three dimensions; first added wins a tie.
    pub fn nearest(&self, from: [f32; 3]) -> Option<[f32; 3]> {
        let mut best: Option<([f32; 3], f32)> = None;
        for &spot in &self.0 {
            let d = distance(spot, from);
            if best.is_none_or(|(_, old)| d < old) {
                best = Some((spot, d));
            }
        }
        best.map(|(spot, _)| spot)
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()
}

pub fn flight_updates(seconds: Option<f32>, distance: f32) -> u32 {
    let seconds = seconds.unwrap_or(0.0).max(0.0);
    let duration = if seconds > 0.0 {
        seconds
    } else {
        distance / FLIGHT_SPEED
    };
    (duration * UPDATES).floor() as u32 + 1
}

/// Drawn angles. `locked` remains readable in older Rust checkpoints;
/// `Camera::heading_state` is the live controller, including on flights.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct View {
    pub tilt: f32,
    pub heading: f32,
    pub roll: f32,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Flight {
    pub spot: [f32; 3],
    pub aim: [f32; 3],
    pub step: [f32; 3],
    pub roll_step: f32,
    pub n: u32,
    pub k: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightStep {
    pub k: u32,
    pub n: u32,
}

impl Flight {
    pub fn new(
        position: [f32; 3],
        roll: f32,
        spot: [f32; 3],
        aim: [f32; 3],
        seconds: Option<f32>,
        end_roll: Option<f32>,
    ) -> Flight {
        let n = flight_updates(seconds, distance(position, spot));
        let step = std::array::from_fn(|k| (spot[k] - position[k]) / n as f32);
        let roll_step = (end_roll.unwrap_or(0.0) - roll) / n as f32;
        Flight {
            spot,
            aim,
            step,
            roll_step,
            n,
            k: 0,
        }
    }

    /// Setup does not draw, place, or reset heading-controller history.
    pub fn begin(&self, camera: &mut Camera) {
        camera.avoid = false;
        camera.turn_back = true;
        camera.reset_shake_angles();
        camera.seed_actor(Followed {
            base: self.aim,
            facing: 0.0,
        });
    }

    pub fn done(&self) -> bool {
        self.k >= self.n
    }

    /// Add one position/roll step, clearing shared yaw. The last step
    /// enables placing and resets heading to idle BEFORE the full update.
    /// No aim, snap-to-spot, height or range pass is hidden in this method.
    pub fn advance(
        &mut self,
        camera: &mut Camera,
        roll: &mut f32,
        places: &mut bool,
    ) -> FlightStep {
        self.k += 1;
        if self.done() {
            camera.turn_back = false;
            camera.heading_state = HeadingState::Idle;
            *places = true;
        }
        camera.position = std::array::from_fn(|k| camera.position[k] + self.step[k]);
        *roll += self.roll_step;
        camera.clear_heading_offset();
        FlightStep {
            k: self.k,
            n: self.n,
        }
    }
}

/// Cut setup before the same ordinary full update used for following.
/// Placing/aiming/turn-back/heading state and an active flight are retained.
pub fn cut(camera: &mut Camera, view: &mut View, spot: [f32; 3], aim: [f32; 3], roll: f32) {
    camera.position = spot;
    camera.avoid = false;
    camera.tilt = DEFAULT_TILT;
    camera.clear_heading_offset();
    camera.seed_actor(Followed {
        base: aim,
        facing: 0.0,
    });
    camera.heading = crate::camera::view_angles([aim[0] - spot[0], aim[1] - spot[1], 0.0]).1;
    view.roll = roll;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{HeightThen, Input, Lift, Mode};
    use crate::connor_mover::GroundLookup;

    struct Floor(f32);
    impl GroundLookup for Floor {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((self.0, [0.0, 0.0, 1.0]))
        }
    }
    fn full(camera: &mut Camera, places: bool, aims: bool, floor: f32) -> crate::camera::Report {
        let actor = camera.followed(Followed {
            base: [0.0; 3],
            facing: 0.0,
        });
        camera.full_update(
            actor,
            false,
            &Input::default(),
            &Floor(floor),
            None,
            places,
            aims,
        )
    }

    #[test]
    fn nearest_spot_and_original_flight_lengths() {
        let mut spots = Spots::default();
        spots.add([0.0; 3], true);
        spots.add([100.0, 0.0, 0.0], false);
        assert_eq!(spots.nearest([80.0, 0.0, 0.0]), Some([100.0, 0.0, 0.0]));
        assert_eq!(spots.nearest([50.0, 0.0, 0.0]), Some([0.0; 3]));
        spots.add([7.0; 3], true);
        assert_eq!(spots.0, [[7.0; 3]]);
        assert_eq!(Spots::default().nearest([0.0; 3]), None);
        assert_eq!(flight_updates(Some(2.0), 1.0), 31);
        assert_eq!(flight_updates(Some(3.0), 1.0), 46);
        assert_eq!(flight_updates(None, 10_000.0), 31);
        assert_eq!(flight_updates(Some(-1.0), 0.0), 1);
    }

    #[test]
    fn flight_setup_and_completion_use_the_retained_heading_controller() {
        let mut camera = Camera::new([0.0, 0.0, 3000.0]);
        camera.heading = 0.3;
        camera.heading_state = HeadingState::Locked;
        let mut f = Flight::new(
            camera.position,
            0.1,
            [1000.0, 0.0, 3000.0],
            [0.0, 10000.0, 1600.0],
            Some(0.1),
            Some(0.4),
        );
        let pose = (camera.position, camera.heading, camera.view_tilt);
        f.begin(&mut camera);
        assert_eq!((camera.position, camera.heading, camera.view_tilt), pose);
        assert!(camera.turn_back);
        assert_eq!(camera.heading_state, HeadingState::Locked);
        let (mut places, mut roll) = (false, 0.1);
        camera.shake_angles([0.02, 0.03]);
        f.advance(&mut camera, &mut roll, &mut places);
        assert!(!places);
        assert!(!f.done());
        full(&mut camera, places, true, 0.0);
        assert_eq!(camera.heading_state, HeadingState::Locked);
        assert_eq!(camera.tilt, DEFAULT_TILT + 0.02);
        // The last full update is idle and may keep a remaining error.
        camera.heading = 0.2;
        f.advance(&mut camera, &mut roll, &mut places);
        assert!(places && f.done() && !camera.turn_back);
        assert_eq!(camera.heading_state, HeadingState::Idle);
        full(&mut camera, places, true, 0.0);
        assert_eq!(camera.heading, 0.2, "no second forced exact-yaw pass");
        assert!((roll - 0.4).abs() < 1e-6);
    }

    #[test]
    fn saved_motion_continues_after_a_scripted_move_without_aiming_or_snapping() {
        let mut camera = Camera::new([139260.0, 249970.0, 19550.0]);
        let spot = [139760.0, 249700.0, 17550.0];
        let mut f = Flight::new(
            camera.position,
            0.0,
            spot,
            [141400.0, 249400.0, 13000.0],
            Some(0.6),
            None,
        );
        let (mut places, mut roll) = (false, 0.0);
        for _ in 0..7 {
            f.advance(&mut camera, &mut roll, &mut places);
        }
        camera.position = [137200.0, 250000.0, 16500.0];
        camera.heading = 0.7;
        camera.view_tilt = -0.3;
        let mut restored: Flight =
            serde_json::from_slice(&serde_json::to_vec(&f).unwrap()).unwrap();
        let (mut other, mut other_roll, mut other_places) = (camera.clone(), roll, places);
        while !f.done() {
            f.advance(&mut camera, &mut roll, &mut places);
            restored.advance(&mut other, &mut other_roll, &mut other_places);
            assert_eq!(
                (camera.position, roll, places),
                (other.position, other_roll, other_places)
            );
        }
        assert_eq!(camera.position, [137350.0, 249919.0, 15900.0]);
        assert_eq!((camera.heading, camera.view_tilt), (0.7, -0.3));
        assert_ne!(camera.position, spot);
    }

    #[test]
    fn final_range_and_height_are_the_one_normal_update_including_fixed_distance() {
        let aim = [186960.0, 228000.0, 10000.0];
        let mut camera = Camera::new([192100.0, 232500.0, 9255.0]);
        camera.set_range(1000.0, 2500.0);
        let mut f = Flight::new(
            camera.position,
            0.0,
            [190000.0, 230000.0, 10600.0],
            aim,
            Some(2.0),
            None,
        );
        f.begin(&mut camera);
        let (mut places, mut roll) = (false, 0.0);
        let mut report = None;
        while !f.done() {
            f.advance(&mut camera, &mut roll, &mut places);
            report = Some(full(&mut camera, places, false, 7650.0));
        }
        let h = report.unwrap().height.unwrap();
        assert_eq!((h.then, h.wanted, h.sum), (HeightThen::Kept, 11400.0, 0.0));
        for (actual, expected) in camera
            .position
            .into_iter()
            .zip([188999.0, 229341.5, 10863.4])
        {
            assert!((actual - expected).abs() < 0.3);
        }
        // F11 uses its distance in the full update, with no normal settle afterwards.
        camera.mode = Mode::Fixed { distance: 1800.0 };
        camera.position = [0.0, 4000.0, 1400.0];
        let mut f = Flight::new(
            camera.position,
            0.0,
            [0.0, 5000.0, 1400.0],
            [0.0; 3],
            Some(0.01),
            None,
        );
        f.begin(&mut camera);
        f.advance(&mut camera, &mut roll, &mut places);
        full(&mut camera, places, false, 0.0);
        assert_eq!(camera.position, [0.0, 1800.0, 1400.0]);
    }

    #[test]
    fn cuts_keep_controller_history_and_apply_height_once_through_the_camera() {
        let mut camera = Camera::new([0.0; 3]);
        camera.heading_state = HeadingState::Locked;
        camera.turn_back = true;
        let mut view = View {
            tilt: 0.5,
            heading: 3.0,
            roll: 0.0,
            locked: false,
        };
        cut(&mut camera, &mut view, [0.0, 2000.0, 500.0], [0.0; 3], 0.1);
        assert!(camera.turn_back);
        assert_eq!(camera.heading_state, HeadingState::Locked);
        let report = full(&mut camera, true, true, 0.0);
        assert_eq!(report.height.unwrap().lift, Lift::ToGround);
        assert_eq!(camera.position[2], 1400.0);
        assert_eq!(view.roll, 0.1);
        cut(&mut camera, &mut view, [0.0; 3], [1000.0, 0.0, 0.0], 0.2);
        full(&mut camera, false, false, 0.0);
        assert_eq!(camera.position, [0.0; 3]);
        assert!((camera.heading + std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }
}
