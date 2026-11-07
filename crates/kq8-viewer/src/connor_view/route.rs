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
    /// Optional walking or running pace, selected with the bound Run key.
    run: Option<bool>,
    click: Option<String>,
    /// Aim the normal world-click ray at the target. Any nearer object wins.
    aim: Option<String>,
    /// Press a normal inventory number key once when this point is reached.
    inventory_slot: Option<usize>,
    /// Continue walking toward the point until the original script records this value.
    until_var: Option<ScriptValue>,
    /// At this landmark, turn in place and press the bound Jump key.
    jump_toward: Option<[f32; 2]>,
    /// Turn toward the neighboring tile at xy and press the bound Jump key.
    #[serde(default)]
    tile_jump: bool,
    /// Stop here and use ordinary hand-weapon attacks against this character.
    fight: Option<String>,
    /// Select the ranged HUD slot and use visible world clicks for this fight.
    #[serde(default)]
    ranged: bool,
    /// Continue to the next opponent without pressing the Away key.
    #[serde(default)]
    keep_weapon: bool,
    /// Choose a health item and when to use its normal inventory key.
    /// Older routes use slot 1 when five points of health are missing.
    heal: Option<FightHeal>,
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

#[derive(Deserialize)]
struct ScriptValue {
    name: String,
    value: String,
}

#[derive(Clone, Copy, Deserialize)]
struct FightHeal {
    slot: usize,
    at_health: i64,
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
    /// Start update and number of corpses already present at that point.
    fight_start: Option<(u64, usize)>,
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
                    || (l.ranged && (l.fight.is_none() || l.retreat || l.big_swing.is_some()))
                    || ((l.keep_weapon || l.heal.is_some()) && l.fight.is_none())
                    || l.heal.is_some_and(|h| !(1..=4).contains(&h.slot) || h.at_health <= 0)
                    || l.inventory_slot.is_some_and(|n| !(1..=crate::gui_view::INVENTORY_DIGITS.len()).contains(&n)
                        || l.scene.is_some() || l.jump_toward.is_some())
                    || l.until_var.as_ref().is_some_and(|v| v.name.trim().is_empty() || l.scene.is_some()
                        || l.fight.is_some() || l.break_object.is_some() || l.jump_toward.is_some())
                    || l.scene.as_ref().is_some_and(|s| s.trigger.is_empty() || s.controller.is_empty())
                    || l.direction.is_some_and(|d| !d.iter().all(|v| v.is_finite()) || d == [0., 0.])
                    || !l.xy.iter().all(|v| v.is_finite())
                    || l.jump_toward
                        .is_some_and(|point| !point.iter().all(|v| v.is_finite()))
                    || (l.tile_jump && (l.jump_toward.is_some() || l.scene.is_some() || l.until_var.is_some()
                        || l.fight.is_some() || l.break_object.is_some() || l.aim.is_some() || l.click.is_some()
                        || l.inventory_slot.is_some() || l.run.is_some() || l.direction.is_some()
                        || !matches!(kq8_core::tile_puzzle::where_is(l.xy[0], l.xy[1]), kq8_core::tile_puzzle::Where::Spot(_))))
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
    mut gui_keys: ResMut<crate::gui_view::KeyAt>,
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
    if (replay.landmarks.is_empty() && replay.jumped.is_none() && replay.held.is_empty())
        || sim.updates < replay.start
        || replay.last_tick == Some(sim.updates)
    {
        return;
    }
    replay.last_tick = Some(sim.updates);
    if replay.landmarks.is_empty() && replay.jumped.is_none() {
        replay.keys(&mut keys, &[]);
        return;
    }
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
    if replay.landmarks.front().is_some_and(|l| l.tile_jump) {
        if let Err(problem) = steer_tile(&mut replay, &sim, &mut keys, &shared) {
            replay.keys(&mut keys, &[]);
            shared.log.lock().unwrap().summary(format!("walk-route: {problem}"));
            exit.write(AppExit::error());
        }
        return;
    }
    let landmark = replay.landmarks.front().unwrap();
    let distance = (landmark.xy[0] - pos[0]).hypot(landmark.xy[1] - pos[1]);
    let moving_out = landmark.direction.is_none_or(|d| previous.is_some_and(|p|
        (pos[0] - p[0]) * d[0] + (pos[1] - p[1]) * d[1] > 0.));
    let arrived = match &landmark.until_var {
        Some(target) => doors.active.scripts.vars.get(&target.name) == Some(target.value.as_str()),
        None => distance <= landmark.radius && moving_out,
    };
    if (landmark.scene.is_none() && arrived)
        || ((landmark.retreat || landmark.ranged) && replay.fight_start.is_some()) {
        let retreat = landmark.retreat;
        let ranged = landmark.ranged;
        if let Some(name) = landmark.fight.clone().or_else(|| landmark.break_object.clone()) {
            let keep_weapon = landmark.keep_weapon;
            let heal = landmark.heal;
            let big_swing = landmark.big_swing.unwrap_or(true);
            let alive = if landmark.break_object.is_some() {
                world.stats.get(&name.to_ascii_lowercase()).map(|stats| !stats.dead())
            } else {
                world.character_index(&name).and_then(|i| world.anims[i].character.as_ref()?.fight.as_ref())
                    .map(|fighter| fighter.dying.is_none() && !fighter.stats.dead())
                    .or_else(|| {
                        let (_, before) = replay.fight_start?;
                        world.corpses.get(before..)?.iter()
                            .any(|corpse| corpse.name.eq_ignore_ascii_case(&name)).then_some(false)
                    })
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
            let (began, _) = *replay.fight_start.get_or_insert((tick, world.corpses.len()));
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
                    && world.connor_stats.is_some_and(|c| {
                        let full = c.level * 10;
                        let threshold = heal.map_or(full - 5, |h| h.at_health);
                        c.stats.0[0] < full && c.stats.0[0] <= threshold
                    })
                {
                    let slot = heal.map_or(1, |h| h.slot);
                    wanted.push(crate::gui_view::INVENTORY_DIGITS[slot - 1].0);
                }
                if ranged {
                    // First-person shots do not turn Connor toward the click.
                    // Face the live target using his ordinary turn keys first.
                    if sim.first_person {
                        if let Some(target) = world.character_index(&name).map(|i| world.anims[i].player.base.position) {
                            let error = wrap(heading_towards(pos, [target[0], target[1]]) - sim.walker.heading);
                            if let Some(key) = turn_key(&sim, error) {
                                wanted.push(key);
                                replay.keys(&mut keys, &wanted);
                                return;
                            }
                        }
                    }
                    if world.weapon_mode != kq8_core::weapons::WeaponMode::Ranged {
                        if (tick - began) % 20 == 0 {
                            gui_keys.0.push(("left:IDCTL_INVBAR_LONG".into(), tick));
                            shared.log.lock().unwrap().summary(format!(
                                "walk-route selects the ranged weapon slot for {name} on update {tick}"
                            ));
                        }
                    } else if tick % 12 == 0 {
                        // Use the normal on-screen ray, including nearer geometry.
                        // The shot controller decides whether this click is accepted.
                        aims.landmark = Some((name, tick));
                    }
                } else if world.weapon_ready.is_none() {
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
            // Put away the hand weapon before normal interaction resumes,
            // unless this route is moving directly to another opponent.
            if world.weapon_ready.is_some() && !keep_weapon {
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
        if let Some(target) = point.until_var {
            shared.log.lock().unwrap().summary(format!(
                "walk-route observed {} = {} at {}", target.name, target.value, point.name
            ));
        }
        if let Some(name) = point.click {
            clicks.0.push((name, 0));
        }
        if let Some(name) = point.aim {
            aims.landmark = Some((name, sim.updates));
        }
        if let Some(slot) = point.inventory_slot {
            replay.keys(&mut keys, &[crate::gui_view::INVENTORY_DIGITS[slot - 1].0]);
            shared.log.lock().unwrap().summary(format!(
                "walk-route pressed inventory key {slot} at {} on update {}", point.name, sim.updates
            ));
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
    // Scripted turns can finish with turn intent zero while this key is
    // still held. Original Turning rule 2 needs a fresh key-down event.
    let rearm_turn = sim.intent.turn == 0
        && turn.is_some_and(|key| replay.held.contains(&key));
    if rearm_turn {
        shared.log.lock().unwrap().summary(format!(
            "walk-route releases a held turn key after its turn intent was cleared on update {}",
            sim.updates
        ));
    }
    let forward = sim.controls.forward.filter(|_| error.abs() < 0.55);
    let mut wanted: Vec<_> = [turn.filter(|_| !rearm_turn), forward].into_iter().flatten().collect();
    if landmark.run.is_some_and(|run| run != sim.intent.run) && !sim.hands_off {
        if let Some(key) = sim.controls.run {
            if !replay.held.contains(&key) {
                wanted.push(key);
            }
        } else {
            shared.log.lock().unwrap().summary("walk-route: Run has no key binding");
            exit.write(AppExit::error());
            return;
        }
    }
    if replay
        .progress
        .is_none_or(|(best, _)| distance < best - 100.)
    {
        replay.progress = Some((distance, sim.updates));
    }
    replay.keys(&mut keys, &wanted);
}

fn steer_tile(
    replay: &mut Replay,
    sim: &ConnorSim,
    keys: &mut ButtonInput<KeyCode>,
    shared: &Shared,
) -> Result<(), String> {
    use kq8_core::tile_puzzle::{jump_target, where_is, Where};
    let point = replay.landmarks.front().unwrap();
    let xy = point.xy;
    let Where::Spot(target) = where_is(xy[0], xy[1]) else {
        return Err("tile route target outside the floor".into());
    };
    let tile = sim
        .tile
        .as_ref()
        .ok_or_else(|| format!("Tile state missing at {}", point.name))?;
    if tile.jumping_to() == Some(target) {
        let point = replay.landmarks.pop_front().unwrap();
        replay.keys(keys, &[]);
        shared.log.lock().unwrap().summary(format!(
            "walk-route: Tile accepted {} toward {target} on update {}",
            point.name, sim.updates
        ));
        return Ok(());
    }
    if !tile.accepts_jump_key() {
        replay.keys(keys, &[]);
        return Ok(());
    }
    let pos = sim.mover.position;
    let Where::Spot(from) = where_is(pos[0], pos[1]) else {
        return Err(format!("outside the tile floor at {}", point.name));
    };
    if from == target || (from.row - target.row).abs() > 1 || (from.col - target.col).abs() > 1 {
        return Err(format!(
            "{} requests {target} from {from}, which is not a neighboring tile",
            point.name
        ));
    }
    let key = if jump_target(from, sim.walker.heading, false) == Some(target) {
        sim.controls.jump.ok_or("Jump has no key binding")?
    } else {
        let error = wrap(heading_towards(pos, xy) - sim.walker.heading);
        turn_key(sim, error).ok_or("no bound turn toward the requested tile")?
    };
    replay.keys(keys, &[key]);
    Ok(())
}

struct GuardInput {
    movement: Vec<KeyCode>,
    swing: bool,
}

impl Replay {
    /// Read the enemy's attack animation, then use ordinary movement
    /// keys to retreat and return for a normal swing.
    fn guard_input(&mut self, sim: &ConnorSim, enemy: &kq8_core::named_anim::NamedAnimPlayer,
        reach: f32, armed: bool) -> GuardInput {
        let mut input = GuardInput { movement: Vec::new(), swing: false };
        // Run is a persistent toggle. A preceding road can leave it on
        // even when combat begins at an already-reached waypoint.
        if sim.intent.run {
            input.movement.extend(sim.controls.run);
            return input;
        }
        let pos = sim.mover.position;
        let target = enemy.base.position;
        let distance = ((pos[0] - target[0]).powi(2) + (pos[1] - target[1]).powi(2) + (pos[2] - target[2]).powi(2)).sqrt();
        let error = wrap(heading_towards(pos, [target[0], target[1]]) - sim.walker.heading);
        let attack = enemy.movement_name().eq_ignore_ascii_case("Attack");
        // The hit fires when the animation crosses Do. At exactly the
        // cue phase it still belongs to the following update.
        let windup = attack && enemy.movement().is_some_and(|movement| {
            movement.cues.iter().any(|cue| {
                cue.action.eq_ignore_ascii_case("do") && enemy.phase <= cue.phase
            })
        });
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
