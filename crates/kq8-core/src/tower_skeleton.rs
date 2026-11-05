//! Dead City tower encounter, fighting.md section 7a and rule 23.
use crate::active::{Request, WorldState};
use crate::character::state_number;
use crate::collide::{BallStack, Placement};
use crate::connor_mover::MoverWorld;
use crate::fight::HitKind;
use kq8_data::{characters::TowerKeys, msg::Tuple};

#[derive(Debug, Clone)]
pub struct Tower {
    pub keys: TowerKeys,
    pub move_time: f64,
    pub stagger_ended: bool,
    pub flinch_pause: f64,
    greeted: bool,
    fall: Option<Fall>,
}

#[derive(Debug, Clone)]
struct Fall {
    origin: Placement,
    landing: Option<[f32; 3]>,
    screamed: bool,
    ended: bool,
    mail: Option<String>,
}

impl Tower {
    pub fn new(keys: TowerKeys) -> Self {
        Self { move_time: keys.move_time, keys, stagger_ended: false, flinch_pause: crate::fight::FLINCH_PAUSE, greeted: false, fall: None }
    }
}

fn floor_below(scene: &dyn MoverWorld, from: [f32; 3], depth: f32) -> Option<[f32; 3]> {
    let [x, y, z] = from;
    let to = [x, y, z - depth];
    let point = BallStack { radius: 0., heights: vec![0.] };
    let hit = scene.sweep(&point, from, to).map(|h| h.touch[2]);
    let terrain = scene.ground(x, y).map(|(height, _)| height).filter(|&height| height >= to[2] && height <= z);
    hit.into_iter().chain(terrain).max_by(f32::total_cmp).map(|h| [x, y, h])
}

impl WorldState {
    fn tower_say(&mut self, actor: &str, verb: u8, case: u8, seq: u8) {
        let first = Tuple::new(2000, 16, verb, case, seq);
        let text = self.talk.queue(&crate::talk::Speak { issuer: actor.into(), first, last: seq, nofade: false }, self.now());
        self.fight_note(actor, format!("tower line {first}: {text}"));
    }

    fn tower_camera(&mut self, spot: [f32; 3], aim: [f32; 3]) {
        let (tilt, heading) = crate::camera::view_angles(std::array::from_fn(|k| aim[k] - spot[k]));
        self.requests.push(Request::Move { who: "KQCamera".into(), flags: 0, position: spot, turn: [tilt, 0., heading], at: "tower fall".into() });
    }

    fn tower_controls(&mut self, on: bool) {
        self.requests.push(Request::CameraFollow { connor: on, at: "tower skeleton".into() });
        self.requests.push(Request::HandsOff { on: !on, at: "tower skeleton".into() });
    }

    pub(crate) fn tower_notified(&mut self, actor: &str, data: &[String]) {
        let Some(i) = self.character_index(actor) else { return };
        let Some(t) = self.anims[i].character.as_mut().and_then(|c| c.tower.as_mut()) else { return };
        if data.first().is_none_or(|s| !s.eq_ignore_ascii_case("Connor")) { return; }
        match data.get(2).and_then(|v| v.parse().ok()) {
            Some(state_number::ROPE) if !t.greeted => {
                t.greeted = true;
                self.tower_controls(false);
                self.requests.push(Request::CameraAround { point: Some(self.anims[i].player.base.position), at: "tower greeting".into() });
                self.tower_say(actor, 0, 18, 1);
            }
            Some(state_number::SWAP_WEAPONS) if self.weapon_mode == crate::weapons::WeaponMode::Ranged => self.tower_say(actor, 0, 102, 1),
            _ => {}
        }
    }

    /// These are native scene lines. The ordinary speech callback's request
    /// for Stop would otherwise interrupt the fall animations.
    pub(crate) fn tower_speech_ended(&mut self, actor: &str, line: Tuple) -> bool {
        let Some(i) = self.character_index(actor) else { return false };
        if self.anims[i].character.as_ref().and_then(|c| c.tower.as_ref()).is_none() { return false; }
        if line == Tuple::new(2000, 16, 0, 18, 1) {
            self.requests.push(Request::CameraAround { point: None, at: "tower greeting ends".into() });
            self.tower_controls(true);
            self.music.push(crate::music::MusicRequest::Push("mtower.aud".into()));
            // The class consumes 201, but neither the tower trap nor the
            // shipped movement list sends it. Use the end of the greeting
            // as the activation boundary until the original timing is known.
            self.queue.send(self.now(), 0., actor, "KQCueEvent", &["201".into()], "tower greeting ended");
        }
        line.file == 2000 && line.noun == 16 && matches!((line.verb, line.case), (0, 18 | 102) | (12, 22))
    }

    pub(crate) fn tower_fall_start(&mut self, i: usize) {
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let tower = a.character.as_mut().unwrap().tower.as_mut().unwrap();
        tower.fall = Some(Fall { origin: a.player.base, landing: None, screamed: false, ended: false, mail: None });
        self.requests.push(Request::ConnorWait { at: "tower fall".into() });
        self.tower_controls(false);
        self.tower_say(&name, 12, 22, 1);
    }

    pub(crate) fn tower_ground_step(&mut self, i: usize, scene: &dyn MoverWorld) -> bool {
        let now = self.now();
        let a = &mut self.anims[i];
        let p = a.player.base;
        let name = a.player.name.clone();
        let c = a.character.as_mut().unwrap();
        let t = c.tower.as_mut().unwrap();
        if let Some(f) = t.fall.as_mut() {
            if f.landing.is_none() {
                f.landing = floor_below(scene, f.origin.to_world([0., 2000., 0.]), 100_000.);
            }
            return true;
        }
        let stagger = (a.player.movement_name().eq_ignore_ascii_case("reactback") && !a.player.has_ended()) || t.stagger_ended;
        if stagger {
            t.move_time = now + t.keys.stun_pause;
            if floor_below(scene, p.to_world([0., 400., 0.]), 1000.).is_none() {
                self.fight_note(&name, "reactback: no support 400 behind, starts the fall");
                self.character_dies(i, HitKind::Normal, false);
            } else if std::mem::take(&mut t.stagger_ended) {
                c.set_state(&mut a.player, "Wait", &[], now);
            }
            return true;
        }
        false
    }

    pub(crate) fn tower_has_forward_ground(&self, i: usize, scene: &dyn MoverWorld) -> bool {
        floor_below(scene, self.anims[i].player.base.to_world([0., -1000., 0.]), 1000.).is_some()
    }

    pub(crate) fn tower_fall_end(&mut self, i: usize, movement: &str) -> bool {
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let c = a.character.as_mut().unwrap();
        let Some(fall) = c.tower.as_mut().and_then(|t| t.fall.as_mut()) else { return false };
        let p = fall.origin;
        let landing = fall.landing.unwrap_or_else(|| p.to_world([0., 2000., 0.]));
        let mut camera = None;
        let mut cue_delay = None;
        let next = match movement.to_ascii_lowercase().as_str() {
            "teeter" => {
                let spot = p.to_world([0., 2000., -5000.]);
                camera = Some((spot, std::array::from_fn(|k| p.position[k] + (spot[k] - p.position[k]) * 0.9)));
                cue_delay = Some(2.18);
                Some("die")
            }
            "die" => {
                let around = Placement { position: landing, ..p };
                camera = Some((around.to_world([3000., 0., 2000.]), landing));
                a.player.base.position = p.to_world([0., 600., 0.]);
                a.player.base.position[2] = landing[2] + 5000.;
                Some("die2")
            }
            "die2" => {
                a.player.base.position = landing;
                Some("fallend")
            }
            "fallend" => {
                fall.ended = true;
                cue_delay = Some(3.);
                None
            }
            _ => return true,
        };
        c.fight.as_mut().unwrap().dying = Some(next.map(String::from));
        if let Some(m) = next { a.player.start(m); }
        if let Some((spot, aim)) = camera { self.tower_camera(spot, aim); }
        if next == Some("fallend") {
            let start = self.items.objects.len();
            self.drop_items(i);
            self.lay_kept_drop(i);
            let mail = self.items.objects[start..].iter().find(|o| o.kind.eq_ignore_ascii_case("INVITEM_ChainMailShirt")).map(|o| o.name.clone());
            self.anims[i].character.as_mut().unwrap().tower.as_mut().unwrap().fall.as_mut().unwrap().mail = mail;
        }
        if let Some(delay) = cue_delay {
            self.queue.send(self.now(), delay, &name, "KQCueEvent", &["50".into()], "tower fall");
        }
        self.fight_note(&name, format!("tower {movement} ended, next {next:?}"));
        true
    }

    pub(crate) fn tower_cue(&mut self, actor: &str, cue: Option<&String>) -> bool {
        let Some(i) = self.character_index(actor) else { return false };
        let c = self.anims[i].character.as_mut().unwrap();
        let Some(t) = c.tower.as_mut() else { return false };
        match cue.map(String::as_str) {
            Some("201") => {
                if let Some(ai) = &mut c.ai { ai.settings.alarm_radius = t.keys.real_alarm; }
                self.fight_note(actor, "tower Cue 201: enabled its real alarm radius");
            }
            Some("50") => {
                let Some(fall) = t.fall.as_mut() else { return false };
                if !fall.screamed {
                    fall.screamed = true;
                    self.tower_say(actor, 12, 22, 2);
                } else if fall.ended {
                    let mail = fall.mail.take();
                    if let Some(p) = self.connor_at {
                        let placement = Placement { position: p, rotation: kq8_data::objects::rotation([0., 0., self.connor_heading]) };
                        self.tower_camera(placement.to_world([0., -5000., 1000.]), p);
                    }
                    self.tower_controls(true);
                    self.death_ended(i, self.now());
                    if let Some(item) = mail.and_then(|name| self.items.objects.iter_mut().find(|o| o.name == name)) { item.script = Some("chainmal.cs".into()); }
                }
            }
            _ => return false,
        }
        true
    }
}
