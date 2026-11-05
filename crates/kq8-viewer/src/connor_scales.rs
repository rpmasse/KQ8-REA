//! Connor on the Scales of Justice (tile-puzzle.md, rules 23–27).
//! Crossing the dish and touching it are separate: the latter runs the
//! installed object's collision script, which decides the puzzle outcome.

use super::{ConnorSim, TileCall};
use kq8_core::{
    connor::{Intent, Mode},
    connor_auto::{AutoKind, AutoMove, wrap},
    weapons::WeaponMode,
};

const START: [f32; 3] = [217450., 49140., 2900.];

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Ready,
    Onto,
    Off,
    Leave,
}

pub struct Scales {
    step: Step,
    forward_held: bool,
}

fn floor_heading(heading: f32) -> f32 {
    [-1.57_f32, 2.207, 0.933]
        .into_iter()
        .min_by(|a, b| wrap(heading - a).abs().total_cmp(&wrap(heading - b).abs()))
        .unwrap()
}

impl ConnorSim {
    pub fn enter_scales(&mut self) {
        self.walker.auto = None;
        self.intent.forward = 0;
        if self.set_mode != WeaponMode::None {
            self.ready_set(None, WeaponMode::None, None, false, false, true);
        } else {
            self.walker.stand_now();
        }
        self.scales = Some(Scales {
            step: Step::Ready,
            forward_held: false,
        });
    }

    fn scales_camera(&mut self, at: [f32; 3], fixed_heading: Option<f32>) {
        let aim = kq8_core::camera::Camera::aim_for(self.mover.position);
        let (pitch, toward) =
            kq8_core::camera::view_angles(std::array::from_fn(|i| aim[i] - at[i]));
        let heading = fixed_heading.unwrap_or(toward);
        self.tile_calls.push(TileCall::Line(
            "Scales state",
            "KQCamera::follow none".into(),
        ));
        self.tile_calls.push(TileCall::Line(
            "Scales state",
            format!(
                "move KQCamera 0 {} {} {} {pitch} 0 {heading}",
                at[0], at[1], at[2]
            ),
        ));
    }

    pub fn scales_step(&mut self, intent: &mut Intent) {
        let Some(state) = &mut self.scales else {
            return;
        };
        let forward = std::mem::take(&mut intent.forward) > 0;
        let pressed = forward && !state.forward_held;
        state.forward_held = forward;
        let step = state.step;
        if step != Step::Ready {
            intent.turn = 0;
        }
        if self.readying || self.walker.mode != Mode::Free {
            return;
        }
        let [x, y, z] = self.mover.position;
        let next = match step {
            Step::Ready if pressed && x < 219000. => {
                let facing = floor_heading(self.walker.heading);
                self.put(START, facing);
                if facing < 0. {
                    self.scales_camera([216360., 52250., 7260.], None);
                    self.walker.auto = Some(AutoMove::new(
                        AutoKind::Walk { run: false },
                        [209500., 49065.],
                        None,
                    ));
                    Step::Leave
                } else {
                    let camera_y = if facing > 1.57 { 53496. } else { 44790. };
                    let camera = [216115., camera_y, 5560.];
                    self.scales_camera(camera, None);
                    self.mover.start_tile_jump();
                    self.start_special(
                        "AnimList=conscale.anm,AnimListOn,onscale1,onscale2",
                        "Scales",
                    );
                    Step::Onto
                }
            }
            Step::Ready if pressed && y != 50000. => {
                self.put([x, y, z], if y > 50000. { -0.933 } else { -2.207 });
                self.mover.start_tile_jump();
                self.start_special(
                    "AnimList=conscale.anm,AnimListOn,offscale1,offscale2",
                    "Scales",
                );
                Step::Off
            }
            Step::Onto => {
                self.mover.end_tile_jump();
                self.put(
                    [x, y, z + 200.],
                    wrap(self.walker.heading + std::f32::consts::PI),
                );
                self.scales_camera([216500., 49140., 4500.], Some(-1.57));
                Step::Ready
            }
            Step::Off => {
                self.mover.end_tile_jump();
                self.scales_camera([214421., 49340., 4190.], None);
                self.tile_calls.push(TileCall::Line(
                    "Scales state",
                    "KQCamera::follow Connor".into(),
                ));
                Step::Ready
            }
            Step::Leave if self.walker.auto.is_none() => {
                self.tile_calls.push(TileCall::Line(
                    "Scales state",
                    "KQCamera::follow Connor".into(),
                ));
                self.scales = None;
                self.log.detail(format!(
                    "connor update {}: leaves Scales at ({x:.0}, {y:.0}, {z:.0})",
                    self.updates
                ));
                return;
            }
            _ => return,
        };
        self.scales.as_mut().unwrap().step = next;
        let action = match next {
            Step::Onto => "boarding",
            Step::Off => "stepping off",
            Step::Leave => "leaving",
            Step::Ready => "ready",
        };
        let [x, y, z] = self.mover.position;
        self.log.detail(format!(
            "connor update {}: Scales {action} at ({x:.0}, {y:.0}, {z:.0}), heading {:.3}",
            self.updates, self.walker.heading
        ));
    }
}
