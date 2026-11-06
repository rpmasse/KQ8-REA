//! First-person eye and rotation from the original camera study (Q193).
use crate::collide::Placement;
use kq8_data::objects::{angles_of, rotation};

pub const EYE: [f32; 3] = [0.0, 50.0, 1200.0];
/// The original stores 3.14 for this heading offset.
pub const ENTRY_ANGLES: [f32; 3] = [-0.175, 0.0, 3.14];
pub const MIN_PITCH: f32 = -0.54;
pub const MAX_PITCH: f32 = 1.57;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModeFlags {
    pub active: bool,
    pub script_locked: bool,
    pub resume_after_script: bool,
    #[serde(default)]
    pub rope_locked: bool,
    #[serde(default)]
    pub resume_after_rope: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Support {
    correction: f32,
    unsupported_updates: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub eye: [f32; 3],
    /// The engine's pitch, roll and heading, in radians.
    pub angles: [f32; 3],
}

/// Parameters retained between full updates and immediate shake redraws.
/// Mouse values are the camera action's vertical value (positive tilts up).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FirstPerson {
    offsets: [f32; 3],
    last_mouse: Option<f32>,
    support: Support,
}

impl Default for FirstPerson {
    fn default() -> Self {
        Self {
            offsets: ENTRY_ANGLES,
            last_mouse: None,
            support: Support::default(),
        }
    }
}

impl FirstPerson {
    /// Page tilt is supplied only in the final sync input phase (Q193).
    /// Movement is measured between retained actor records, independently of F11.
    pub fn full_update(
        &mut self,
        position: [f32; 3],
        actor_angles: [f32; 3],
        support_normal: Option<[f32; 3]>,
        retained_horizontal: f32,
        mouse_vertical: Option<f32>,
        page: i8,
        home: bool,
    ) -> Pose {
        if home {
            self.offsets[0] = crate::camera::DEFAULT_TILT;
        }
        self.offsets[0] += f32::from(page) * crate::camera::TILT_PER_UPDATE;
        if let (Some(now), Some(before)) = (mouse_vertical, self.last_mouse) {
            self.offsets[0] += now - before;
        }
        self.last_mouse = mouse_vertical;
        if retained_horizontal > crate::camera::MOVE_RESETS_TILT
            && mouse_vertical.unwrap_or(0.0) == 0.0
        {
            self.offsets[0] = crate::camera::DEFAULT_TILT;
        }
        self.offsets[0] = self.offsets[0].clamp(MIN_PITCH, MAX_PITCH);
        self.support
            .full_update(position, actor_angles, self.offsets, support_normal)
    }

    pub fn shake_sample(
        &mut self,
        position: [f32; 3],
        actor_angles: [f32; 3],
        sample: [f32; 2],
    ) -> Pose {
        self.offsets = [
            ENTRY_ANGLES[0] + sample[0],
            0.0,
            ENTRY_ANGLES[2] + sample[1],
        ];
        shake_pose(position, actor_angles, self.offsets)
    }

    /// Shake termination resets parameters. The caller retains its visible pose.
    pub fn stop_shake(&mut self) {
        self.offsets = ENTRY_ANGLES;
    }

    pub fn valid(&self) -> bool {
        self.offsets.iter().all(|v| v.is_finite())
            && self.last_mouse.is_none_or(f32::is_finite)
            && self.support.valid()
    }
}

impl Support {
    /// Only full first-person updates spend the three-update support grace.
    pub fn full_update(
        &mut self,
        position: [f32; 3],
        actor_angles: [f32; 3],
        mut offsets: [f32; 3],
        support_normal: Option<[f32; 3]>,
    ) -> Pose {
        offsets[0] = offsets[0].clamp(MIN_PITCH, MAX_PITCH);
        if let Some(slope) = support_normal.and_then(|n| slope_for_heading(n, actor_angles[2])) {
            self.correction = slope.clamp(MIN_PITCH - offsets[0], MAX_PITCH - offsets[0]);
            self.unsupported_updates = 0;
        } else {
            self.unsupported_updates = self.unsupported_updates.saturating_add(1).min(4);
            if self.unsupported_updates == 4 {
                self.correction = 0.0;
            }
        }
        let slope = self
            .correction
            .clamp(MIN_PITCH - offsets[0], MAX_PITCH - offsets[0]);
        compose(position, actor_angles, offsets, slope)
    }

    pub fn valid(self) -> bool {
        self.correction.abs() <= MAX_PITCH - MIN_PITCH
            && self.unsupported_updates <= 4
            && (self.unsupported_updates < 4 || self.correction == 0.0)
    }
}

/// A direct shake redraw uses the actor's pitch and leaves remembered
/// support, the unsupported count and the ordinary pitch clamp untouched.
pub fn shake_pose(position: [f32; 3], actor_angles: [f32; 3], offsets: [f32; 3]) -> Pose {
    compose(position, actor_angles, offsets, actor_angles[0])
}

fn slope_for_heading(normal: [f32; 3], heading: f32) -> Option<f32> {
    let length = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !length.is_finite() || length == 0.0 || normal[2] <= 0.0 {
        return None;
    }
    let [nx, ny, nz] = normal.map(|v| v / length);
    let horizontal = nx.hypot(ny);
    if horizontal == 0.0 {
        return Some(0.0);
    }
    // Heading rotates model +Y; the model's facing direction is -Y.
    let (sin, cos) = heading.sin_cos();
    let angle = ((-sin * nx + cos * ny) / horizontal)
        .clamp(-1.0, 1.0)
        .acos();
    let pitch = (-(nx * nx + ny * ny) / nz).asin();
    pitch
        .is_finite()
        .then_some(pitch * (std::f32::consts::FRAC_PI_2 - angle) * 0.63661975)
}

fn compose(position: [f32; 3], actor_angles: [f32; 3], offsets: [f32; 3], base_pitch: f32) -> Pose {
    let actor = Placement {
        position,
        rotation: rotation(actor_angles),
    };
    let base = Placement {
        position,
        rotation: rotation([base_pitch, 0.0, actor_angles[2]]),
    };
    let view = rotation(offsets).map(|axis| base.direction_to_world(axis));
    Pose {
        eye: actor.to_world(EYE),
        angles: angles_of(view),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) {
        assert!((a - b).abs() < 1.0e-5, "{a} != {b}");
    }

    #[test]
    fn first_person_drag_consumes_each_change_once_and_resets_only_on_a_zero_value() {
        let mut view = FirstPerson::default();
        let flat = Some([0., 0., 1.]);
        let update = |v: &mut FirstPerson, moved, mouse, page| {
            v.full_update([0.; 3], [0.; 3], flat, moved, mouse, page, false);
            v.offsets[0]
        };
        close(update(&mut view, 0., Some(0.3), 0), ENTRY_ANGLES[0]);
        close(update(&mut view, 0., Some(0.4), 0), ENTRY_ANGLES[0] + 0.1);
        close(update(&mut view, 20., Some(0.4), 0), ENTRY_ANGLES[0] + 0.1);
        close(
            update(&mut view, 20., Some(0.0), 0),
            crate::camera::DEFAULT_TILT,
        );
        close(
            update(&mut view, 10., None, 1),
            crate::camera::DEFAULT_TILT + crate::camera::TILT_PER_UPDATE,
        );
        close(
            update(&mut view, 10.01, None, 0),
            crate::camera::DEFAULT_TILT,
        );
    }

    #[test]
    fn direct_shake_keeps_support_and_the_following_full_update_clamps_it() {
        let mut view = FirstPerson::default();
        let normal = Some([0.0, 0.5, 0.8660254]);
        view.full_update([0.; 3], [0.; 3], normal, 0., None, 0, false);
        let support = view.support;
        let actor = [0.35, 0.07, 0.2];
        let direct = view.shake_sample([0.; 3], actor, [-1.0, 0.1]);
        assert_eq!(
            direct,
            shake_pose(
                [0.; 3],
                actor,
                [ENTRY_ANGLES[0] - 1.0, 0.0, ENTRY_ANGLES[2] + 0.1]
            )
        );
        assert_eq!(view.support, support);
        assert!(view.valid());
        let mut restored: FirstPerson =
            serde_json::from_slice(&serde_json::to_vec(&view).unwrap()).unwrap();
        let after = restored.full_update([0.; 3], actor, None, 0., None, 0, false);
        assert_ne!(after.angles, direct.angles);
        close(restored.offsets[0], MIN_PITCH);
        assert_eq!(restored.support.unsupported_updates, 1);
        restored.stop_shake();
        assert_eq!(restored.offsets, ENTRY_ANGLES);
        assert_eq!(restored.support.unsupported_updates, 1);
    }

    #[test]
    fn eye_uses_full_actor_rotation_and_is_independent_of_camera_tilt() {
        let mut support = Support::default();
        let base = [10., 20., 30.];
        let flat = Some([0., 0., 1.]);
        let upright = support.full_update(base, [0.; 3], ENTRY_ANGLES, flat);
        assert_eq!(upright.eye, [10., 70., 1230.]);
        for (got, expected) in upright.angles.into_iter().zip(ENTRY_ANGLES) {
            close(got, expected);
        }
        let actor = [0.2, -0.1, 1.0];
        let a = support.full_update(base, actor, ENTRY_ANGLES, flat);
        let b = support.full_update(base, actor, [0.5, 0.0, 3.14], flat);
        assert_eq!(a.eye, b.eye);
        assert_ne!(a.eye, upright.eye);
    }

    #[test]
    fn support_uses_the_squared_horizontal_normal_and_actor_heading() {
        let normal = [0.0, 0.5, 0.8660254];
        close(slope_for_heading(normal, 0.0).unwrap(), -0.2928428);
        close(
            slope_for_heading(normal, std::f32::consts::PI).unwrap(),
            0.2928428,
        );
        close(
            slope_for_heading(normal, std::f32::consts::FRAC_PI_2).unwrap(),
            0.0,
        );
        close(slope_for_heading([0.0, 0.0, 2.0], 1.0).unwrap(), 0.0);
    }

    #[test]
    fn only_the_fourth_unsupported_full_update_drops_the_limited_correction() {
        let mut support = Support::default();
        let offsets = [-0.5, 0.0, 3.14];
        let normal = [0.0, 0.5, 0.8660254];
        let original = support.full_update([0.; 3], [0.; 3], offsets, Some(normal));
        close(support.correction, MIN_PITCH + 0.5);
        let snapshot = serde_json::to_vec(&support).unwrap();
        let mut loaded: Support = serde_json::from_slice(&snapshot).unwrap();
        assert!(loaded.valid());
        for _ in 0..3 {
            assert_eq!(
                loaded.full_update([0.; 3], [0.; 3], offsets, None),
                original
            );
            let _ = shake_pose([0.; 3], [0.25, 0.0, 0.0], offsets);
        }
        let flat = loaded.full_update([0.; 3], [0.; 3], offsets, None);
        assert_ne!(flat.angles, original.angles);
        assert_eq!(loaded.correction, 0.0);
        assert_eq!(
            loaded.full_update([0.; 3], [0.; 3], offsets, Some(normal)),
            original
        );
    }
}
