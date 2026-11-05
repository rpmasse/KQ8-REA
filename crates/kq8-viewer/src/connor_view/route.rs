//! Unattended walking through landmarks. Only press/release Connor's bound
//! movement keys. Ground, buildings, falls, actors and scripts stay in play.
//! The optional named interaction uses the existing --click dispatcher.
use super::{ConnorSim, Shared};
use crate::door_view::{ClickAt, DoorSim, SwingAt};
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
    /// At this landmark, turn in place and press the bound Jump key.
    jump_toward: Option<[f32; 2]>,
    /// Stop here and use ordinary hand-weapon attacks against this character.
    fight: Option<String>,
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
    #[serde(skip)]
    jumped: Option<u64>,
    #[serde(skip)]
    fight_start: Option<u64>,
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
                    || l.jump_toward
                        .is_some_and(|point| !point.iter().all(|v| v.is_finite()))
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
    doors: Option<Res<DoorSim>>,
    mut swings: ResMut<SwingAt>,
    shared: Res<Shared>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(sim) = sim else { return };
    if (replay.landmarks.is_empty() && replay.jumped.is_none())
        || sim.updates < replay.start
        || replay.last_tick == Some(sim.updates)
    {
        return;
    }
    replay.last_tick = Some(sim.updates);
    let Some(doors) = doors else { return };
    let world = &doors.active.world;
    let health = world.connor_stats.map(|c| c.stats.0[0]);
    if health.is_some_and(|hp| hp < 0) {
        shared
            .log
            .lock()
            .unwrap()
            .summary(format!("walk-route: Connor died on update {}", sim.updates));
        replay.keys(&mut keys, &[]);
        exit.write(AppExit::error());
        return;
    }
    if let Some(tick) = replay.jumped {
        replay.keys(&mut keys, &[]);
        // The standing jump arc lasts 20 simulation updates. It can then
        // continue into Fall before finding a floor or the top of a prop.
        if sim.updates < tick + 20 || !sim.mover.on_ground {
            return;
        }
        replay.jumped = None;
        let p = sim.mover.position;
        shared.log.lock().unwrap().summary(format!(
            "walk-route landed on update {} at ({:.0}, {:.0}, {:.0})",
            sim.updates, p[0], p[1], p[2]
        ));
    }
    if replay.landmarks.is_empty() {
        return;
    }
    if sim.hands_off {
        replay.progress = None;
        replay.keys(&mut keys, &[]);
        return;
    }
    let landmark = replay.landmarks.front().unwrap();
    let pos = sim.mover.position;
    let distance = (landmark.xy[0] - pos[0]).hypot(landmark.xy[1] - pos[1]);
    if distance <= landmark.radius {
        if let Some(name) = landmark.fight.clone() {
            let Some(fighter) = world
                .character_index(&name)
                .and_then(|i| world.anims[i].character.as_ref()?.fight.as_ref())
            else {
                shared
                    .log
                    .lock()
                    .unwrap()
                    .summary(format!("walk-route: no fighter named {name}"));
                exit.write(AppExit::error());
                return;
            };
            let tick = sim.updates;
            let began = *replay.fight_start.get_or_insert(tick);
            if tick - began > 450 {
                shared
                    .log
                    .lock()
                    .unwrap()
                    .summary(format!("walk-route: timed out fighting {name}"));
                exit.write(AppExit::error());
                return;
            }
            if fighter.dying.is_none() && !fighter.stats.dead() {
                let mut wanted = Vec::new();
                if tick % 8 == 0
                    && world
                        .connor_stats
                        .is_some_and(|c| c.stats.0[0] <= c.level * 10 - 5)
                {
                    // Same first consumable key as in normal play.
                    wanted.push(KeyCode::Digit1);
                }
                if world.weapon_ready.is_none() {
                    if (tick - began) % 20 == 0 {
                        wanted.extend(sim.controls.draw);
                    }
                } else if tick % 4 == 0 {
                    // Only dispatch after the normal Draw key equips the weapon.
                    swings.0.push((name, tick, true));
                }
                replay.keys(&mut keys, &wanted);
                return;
            }
            // Put away the hand weapon before normal interaction resumes.
            if world.weapon_ready.is_some() {
                let wanted: Vec<_> = sim
                    .controls
                    .away
                    .filter(|_| tick % 12 == 0)
                    .into_iter()
                    .collect();
                replay.keys(&mut keys, &wanted);
                return;
            }
        }
        let landmark = replay.landmarks.front().unwrap();
        if let Some(target) = landmark.jump_toward {
            let error = wrap(heading_towards(pos, target) - sim.walker.heading);
            if let Some(key) = turn_key(&sim, error) {
                replay.keys(&mut keys, &[key]);
                return;
            }
            if !sim.mover.on_ground {
                replay.keys(&mut keys, &[]);
                return;
            }
        }
        let point = replay.landmarks.pop_front().unwrap();
        replay.fight_start = None;
        replay.progress = None;
        replay.keys(&mut keys, &[]);
        shared.log.lock().unwrap().summary(format!(
            "walk-route at {} on update {}: ({:.0}, {:.0}, {:.0}), distance {distance:.0}",
            point.name, sim.updates, pos[0], pos[1], pos[2]
        ));
        if let Some(name) = point.click {
            clicks.0.push((name, 0));
        }
        if point.jump_toward.is_some() {
            if let Some(key) = sim.controls.jump {
                replay.keys(&mut keys, &[key]);
                replay.jumped = Some(sim.updates);
            } else {
                shared
                    .log
                    .lock()
                    .unwrap()
                    .summary("walk-route: Jump has no key binding");
                exit.write(AppExit::error());
            }
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
    let turn = turn_key(&sim, error);
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

fn turn_key(sim: &ConnorSim, error: f32) -> Option<KeyCode> {
    if error > 0.13 {
        sim.controls.left
    } else if error < -0.13 {
        sim.controls.right
    } else {
        None
    }
}
