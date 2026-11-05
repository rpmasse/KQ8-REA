//! Unattended walking through landmarks. Only press/release Connor's bound
//! movement keys. Ground, buildings, falls, actors and scripts stay in play.
//! The optional named interaction uses the existing --click dispatcher.
use super::{ConnorSim, Shared};
use crate::door_view::ClickAt;
use bevy::prelude::*;
use kq8_core::connor_auto::{heading_towards, wrap};
use serde::Deserialize;
use std::collections::VecDeque;
use std::path::Path;

#[derive(Deserialize)]
struct Landmark {
    name: String,
    xy: [f32; 2],
    radius: f32,
    click: Option<String>,
}

#[derive(Resource, Deserialize, Default)]
pub struct Replay {
    start: u64,
    landmarks: VecDeque<Landmark>,
    #[serde(skip)]
    held: Vec<KeyCode>,
    #[serde(skip)]
    last_tick: Option<u64>,
    #[serde(skip)]
    progress: Option<(f32, u64)>,
}

impl Replay {
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let replay: Self =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if replay.landmarks.is_empty()
            || replay.landmarks.iter().any(|l| {
                l.name.is_empty()
                    || !l.xy.iter().all(|v| v.is_finite())
                    || !l.radius.is_finite()
                    || l.radius <= 0.
            })
        {
            return Err("a walking route needs named points with a positive arrival radius".into());
        }
        Ok(replay)
    }

    fn keys(&mut self, input: &mut ButtonInput<KeyCode>, wanted: &[KeyCode]) {
        for key in self.held.drain(..) {
            if !wanted.contains(&key) {
                input.release(key);
            }
        }
        for &key in wanted {
            input.press(key);
        }
        self.held.extend_from_slice(wanted);
    }
}

pub fn steer(
    mut replay: ResMut<Replay>,
    sim: Option<Res<ConnorSim>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut clicks: ResMut<ClickAt>,
    shared: Res<Shared>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(sim) = sim else { return };
    if replay.landmarks.is_empty()
        || sim.updates < replay.start
        || replay.last_tick == Some(sim.updates)
    {
        return;
    }
    replay.last_tick = Some(sim.updates);
    if sim.hands_off {
        replay.progress = None;
        replay.keys(&mut keys, &[]);
        return;
    }
    let landmark = replay.landmarks.front().unwrap();
    let pos = sim.mover.position;
    let distance = (landmark.xy[0] - pos[0]).hypot(landmark.xy[1] - pos[1]);
    if distance <= landmark.radius {
        let point = replay.landmarks.pop_front().unwrap();
        replay.progress = None;
        replay.keys(&mut keys, &[]);
        shared.log.lock().unwrap().summary(format!(
            "walk-route at {} on update {}: ({:.0}, {:.0}, {:.0}), distance {distance:.0}",
            point.name, sim.updates, pos[0], pos[1], pos[2]
        ));
        if let Some(name) = point.click {
            clicks.0.push((name, 0));
        }
        return;
    }
    let error = wrap(heading_towards(pos, landmark.xy) - sim.walker.heading);
    if replay.progress.is_some_and(|(best, tick)| {
        distance >= best - 100. && sim.updates.saturating_sub(tick) >= 180
    }) {
        shared.log.lock().unwrap().summary(format!("walk-route blocked toward {} on update {} at ({:.0}, {:.0}, {:.0}), distance {distance:.0}", landmark.name, sim.updates, pos[0], pos[1], pos[2]));
        replay.keys(&mut keys, &[]);
        exit.write(AppExit::error());
        return;
    }
    if sim.updates % 60 == 0 {
        shared.log.lock().unwrap().summary(format!("walk-route toward {} on update {}: ({:.0}, {:.0}, {:.0}), distance {distance:.0}, heading error {error:.3}", landmark.name, sim.updates, pos[0], pos[1], pos[2]));
    }
    let turn = if error > 0.13 {
        sim.controls.left
    } else if error < -0.13 {
        sim.controls.right
    } else {
        None
    };
    let forward = sim.controls.forward.filter(|_| error.abs() < 0.55);
    let wanted: Vec<_> = [turn, forward].into_iter().flatten().collect();
    if replay
        .progress
        .is_none_or(|(best, _)| distance < best - 100.)
    {
        replay.progress = Some((distance, sim.updates));
    }
    replay.keys(&mut keys, &wanted);
}
