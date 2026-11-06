//! Unattended walking through landmarks. Only press/release Connor's bound
//! movement keys. Ground, buildings, falls, actors and scripts stay in play.
//! Named interactions can use the existing --click dispatcher or --aim ray.
use super::{ConnorSim, Shared};
use crate::door_view::{AimAt, ClickAt, DoorSim, SwingAt};
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
    /// Aim the normal world-click ray at the target. Any nearer object wins.
    aim: Option<String>,
    /// At this landmark, turn in place and press the bound Jump key.
    jump_toward: Option<[f32; 2]>,
    /// Stop here and use ordinary hand-weapon attacks against this character.
    fight: Option<String>,
    /// Use the same normal weapon input against a breakable's live health.
    break_object: Option<String>,
    /// Omit to preserve the earlier replay's Shift+click; false is a plain click.
    big_swing: Option<bool>,
    /// Bait a stationary opponent's attack, step outside reach and return.
    #[serde(default)]
    retreat: bool,
    /// Walk toward a one-shot trigger until its script takes over; wait for
    /// the original scene to delete its controller and return player control.
    scene: Option<Scene>,
    /// Accept a doorway landmark only when moving in its outward direction.
    direction: Option<[f32; 2]>,
}

#[derive(Deserialize)]
struct Scene {
    trigger: String,
    controller: String,
}

#[derive(Resource, Deserialize, Default)]
pub struct Replay {
    start: u64,
    landmarks: VecDeque<Landmark>,
    /// Diagnostic boundary. The full campaign continues by default.
    stop_on_attack: Option<String>,
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
    #[serde(skip)]
    exchange: Exchange,
    #[serde(skip)]
    live_trigger_at: Option<u64>,
    #[serde(skip)]
    previous_xy: Option<[f32; 2]>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum Exchange {
    #[default]
    Bait,
    Withdraw,
    Return,
    Swing,
}

impl Replay {
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let replay: Self =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if replay.landmarks.is_empty()
            || replay.landmarks.iter().any(|l| {
                l.name.is_empty()
                    || (l.aim.is_some() && l.click.is_some())
                    || (l.fight.is_some() && l.break_object.is_some())
                    || (l.retreat && l.fight.is_none())
                    || l.scene.as_ref().is_some_and(|s| s.trigger.is_empty() || s.controller.is_empty())
                    || l.direction.is_some_and(|d| !d.iter().all(|v| v.is_finite()) || d == [0., 0.])
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
    mut aims: ResMut<AimAt>,
    doors: Option<Res<DoorSim>>,
    gui: Option<Res<crate::gui_view::Gui>>,
    mut swings: ResMut<SwingAt>,
    shared: Res<Shared>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(sim) = sim else { return };
    if aims.landmark.is_some() {
        replay.keys(&mut keys, &[]);
        return;
    }
    // Gameplay ignores key presses while a popup owns input. Release the
    // diagnostic route's held keys so resuming creates a new press edge.
    if gui.is_some_and(|g| g.popup.is_some()) {
        if !replay.held.is_empty() {
            shared.log.lock().unwrap().summary(format!(
                "walk-route released {:?} for popup at update {}", replay.held, sim.updates
            ));
            replay.keys(&mut keys, &[]);
        }
        replay.last_tick = None;
        return;
    }
    if (replay.landmarks.is_empty() && replay.jumped.is_none())
        || sim.updates < replay.start
        || replay.last_tick == Some(sim.updates)
    {
        return;
    }
    replay.last_tick = Some(sim.updates);
    let Some(doors) = doors else { return };
    let world = &doors.active.world;
    let pos = sim.mover.position;
    let previous = replay.previous_xy.replace([pos[0], pos[1]]);
    if world.connor_death_entered || sim.walker.mode == kq8_core::connor::Mode::Dead {
        shared.log.lock().unwrap().summary(format!("walk-route: Connor entered Die on update {}", sim.updates));
        replay.keys(&mut keys, &[]);
        exit.write(AppExit::error());
        return;
    }
    if let Some(name) = replay.stop_on_attack.as_deref() {
        if world.character_index(name).is_some_and(|i| world.anims[i].player.movement_name().eq_ignore_ascii_case("Attack")) {
            shared.log.lock().unwrap().summary(format!(
                "walk-route observed {name} attacking on update {} at ({:.0}, {:.0}, {:.0})",
                sim.updates, pos[0], pos[1], pos[2]
            ));
            replay.keys(&mut keys, &[]);
            exit.write(AppExit::Success);
            return;
        }
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
    if let Some(scene) = replay.landmarks.front().and_then(|l| l.scene.as_ref()) {
        let live = world.trap_index(&scene.trigger).is_some_and(|i| !world.traps[i].gone);
        if replay.live_trigger_at.is_none() {
            if !live || world.is_deleted(&scene.controller) {
                shared.log.lock().unwrap().summary(format!("walk-route: expected live {} and {}", scene.trigger, scene.controller));
                exit.write(AppExit::error());
                return;
            }
            replay.live_trigger_at = Some(sim.updates);
        } else if sim.updates - replay.live_trigger_at.unwrap() > 1200 {
            shared.log.lock().unwrap().summary(format!("walk-route: timed out waiting for {}", scene.controller));
            exit.write(AppExit::error());
            return;
        } else if world.is_deleted(&scene.trigger) {
            let finished = world.is_deleted(&scene.controller) && !sim.hands_off;
            replay.keys(&mut keys, &[]);
            if finished {
                let point = replay.landmarks.pop_front().unwrap();
                replay.live_trigger_at = None;
                replay.progress = None;
                shared.log.lock().unwrap().summary(format!(
                    "walk-route scene complete at {} on update {}: ({:.0}, {:.0}, {:.0}), control returned",
                    point.name, sim.updates, pos[0], pos[1], pos[2]
                ));
            }
            return;
        }
    }
    if sim.hands_off {
        replay.progress = None;
        replay.keys(&mut keys, &[]);
        return;
    }
    let landmark = replay.landmarks.front().unwrap();
    let distance = (landmark.xy[0] - pos[0]).hypot(landmark.xy[1] - pos[1]);
    let moving_out = landmark.direction.is_none_or(|d| previous.is_some_and(|p|
        (pos[0] - p[0]) * d[0] + (pos[1] - p[1]) * d[1] > 0.));
    if (landmark.scene.is_none() && distance <= landmark.radius && moving_out) || (landmark.retreat && replay.fight_start.is_some()) {
        let retreat = landmark.retreat;
        if let Some(name) = landmark.fight.clone().or_else(|| landmark.break_object.clone()) {
            let big_swing = landmark.big_swing.unwrap_or(true);
            let alive = if landmark.break_object.is_some() {
                world.stats.get(&name.to_ascii_lowercase()).map(|stats| !stats.dead())
            } else {
                world.character_index(&name).and_then(|i| world.anims[i].character.as_ref()?.fight.as_ref())
                    .map(|fighter| fighter.dying.is_none() && !fighter.stats.dead())
            };
            let Some(alive) = alive else {
                shared
                    .log
                    .lock()
                    .unwrap()
                    .summary(format!("walk-route: no combat target with live stats named {name}"));
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
            if alive {
                if retreat {
                    let guard = world.character_index(&name).and_then(|i| {
                        let target = &world.anims[i];
                        Some((&target.player, target.character.as_ref()?.ai.as_ref()?.settings.attack_dist))
                    });
                    let Some((player, enemy_reach)) = guard else {
                        shared.log.lock().unwrap().summary(format!("walk-route: no active guard settings for {name}"));
                        exit.write(AppExit::error());
                        return;
                    };
                    let GuardInput { movement, swing } = replay.guard_input(
                        &sim, player, enemy_reach, world.weapon_ready.is_some(),
                    );
                    if swing {
                        swings.0.push((name, tick, false));
                    }
                    replay.keys(&mut keys, &movement);
                    return;
                }
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
                    swings.0.push((name, tick, big_swing));
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
        replay.exchange = Exchange::Bait;
        replay.progress = None;
        replay.keys(&mut keys, &[]);
        shared.log.lock().unwrap().summary(format!(
            "walk-route at {} on update {}: ({:.0}, {:.0}, {:.0}), distance {distance:.0}",
            point.name, sim.updates, pos[0], pos[1], pos[2]
        ));
        if let Some(name) = point.click {
            clicks.0.push((name, 0));
        }
        if let Some(name) = point.aim {
            aims.landmark = Some((name, sim.updates));
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

struct GuardInput {
    movement: Vec<KeyCode>,
    swing: bool,
}

impl Replay {
    /// Read the guard's visible spear animation, then use the same movement
    /// keys and normal swing that a player can use. The guard stands still.
    fn guard_input(&mut self, sim: &ConnorSim, enemy: &kq8_core::named_anim::NamedAnimPlayer,
        reach: f32, armed: bool) -> GuardInput {
        let mut input = GuardInput { movement: Vec::new(), swing: false };
        let pos = sim.mover.position;
        let target = enemy.base.position;
        let distance = ((pos[0] - target[0]).powi(2) + (pos[1] - target[1]).powi(2) + (pos[2] - target[2]).powi(2)).sqrt();
        let error = wrap(heading_towards(pos, [target[0], target[1]]) - sim.walker.heading);
        let attack = enemy.movement_name().eq_ignore_ascii_case("Attack");
        let windup = attack && enemy.phase < 0.5;
        if !armed {
            if sim.updates % 20 == 0 { input.movement.extend(sim.controls.draw); }
            return input;
        }
        if self.exchange == Exchange::Swing && sim.swing.is_none() {
            self.exchange = Exchange::Bait;
        }
        if windup && self.exchange != Exchange::Swing {
            self.exchange = Exchange::Withdraw;
        } else if self.exchange == Exchange::Withdraw && !windup {
            self.exchange = Exchange::Return;
        }
        if self.exchange == Exchange::Swing {
            return input;
        }
        input.movement.extend(turn_key(sim, error));
        if error.abs() > 0.5 {
            return input;
        }
        match self.exchange {
            Exchange::Bait => {
                if distance > reach - 40. { input.movement.extend(sim.controls.forward); }
            }
            Exchange::Withdraw => {
                // The spear reaches farther if Back is held at its hit.
                // Release Back as soon as the ordinary reach is cleared.
                if distance < reach + 60. { input.movement.extend(sim.controls.back); }
            }
            Exchange::Return => {
                // Aim just inside Connor's original 2350-unit hand reach.
                if distance > 2280. { input.movement.extend(sim.controls.forward); }
                else if sim.swing.is_some() { self.exchange = Exchange::Swing; }
                else { input.swing = true; }
            }
            Exchange::Swing => {}
        }
        input
    }
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
