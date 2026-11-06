//! Dead City Weeper drain, from `ranged-weapons.md` section 9.
use crate::active::{AnimObject, Request, TemplateKind, WorldState};
use crate::character::{state_number, State};
use crate::fight::HitKind;
use kq8_data::characters::DrainKeys;

#[derive(Debug, Clone)]
pub struct Drain {
    pub bolt: String,
    pub shooter: String,
    pub target: String,
    pub keys: DrainKeys,
    ticks: u32,
    pulses: u32,
    links: Vec<usize>,
}

/// A post-impact Weeper drain in a disk slot. Link names and positions are
/// saved instead of indices, which change when the world is read again.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SavedDrain {
    bolt: String,
    shooter: String,
    target: String,
    keys: DrainKeys,
    ticks: u32,
    pulses: u32,
    links: Vec<SavedDrainLink>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct SavedDrainLink {
    name: String,
    position: [f32; 3],
    end: bool,
}

impl SavedDrain {
    pub fn bolt_name(&self) -> &str { &self.bolt }

    pub fn valid(&self) -> bool {
        fn name(s: &str) -> bool {
            !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
                && !s.starts_with(['/', '\\']) && !s.contains(':')
                && !s.split(['/', '\\']).any(|part| matches!(part, "." | ".."))
        }
        if ![&self.bolt, &self.shooter, &self.target, &self.keys.segment,
            &self.keys.end, &self.keys.sound, &self.keys.fire_anim, &self.keys.hurt_anim,
            &self.keys.done_anim].into_iter().all(|s| name(s))
            || ![self.keys.duration, self.keys.shoot_height, self.keys.hit_height].into_iter().all(f32::is_finite)
            || !(0.0..=1000.0).contains(&self.keys.duration)
            || self.ticks > (self.keys.duration * crate::UPDATES_PER_SECOND as f32).ceil() as u32
            || self.pulses > (self.keys.duration * 4.0).ceil() as u32
            || self.links.len() > 512
        {
            return false;
        }
        let mut names = std::collections::BTreeSet::new();
        self.links.iter().all(|link| {
            name(&link.name) && names.insert(link.name.to_ascii_lowercase())
                && link.position.iter().all(|p| p.is_finite())
                && if link.end {
                    link.name.eq_ignore_ascii_case(&format!("WeepEnd.{}", self.shooter))
                } else {
                    link.name.to_ascii_lowercase().starts_with(&format!("weeplink.{}.", self.shooter.to_ascii_lowercase()))
                }
        }) && self.links.iter().filter(|link| link.end).count() <= 1
    }
}

impl WorldState {
    pub fn saved_drains(&self) -> Result<Vec<SavedDrain>, String> {
        self.weeper_drains.iter().map(|d| {
            let links = d.links.iter().map(|&i| {
                let a = self.anims.get(i).ok_or("Weeper drain has a missing link")?;
                Ok(SavedDrainLink {
                    name: a.player.name.clone(), position: a.player.base.position,
                    end: a.player.name.eq_ignore_ascii_case(&format!("WeepEnd.{}", d.shooter)),
                })
            }).collect::<Result<Vec<_>, String>>()?;
            let saved = SavedDrain {
                bolt: d.bolt.clone(), shooter: d.shooter.clone(), target: d.target.clone(),
                keys: d.keys.clone(), ticks: d.ticks, pulses: d.pulses, links,
            };
            if !saved.valid() { return Err("Weeper drain cannot form a valid checkpoint".into()); }
            Ok(saved)
        }).collect()
    }

    pub fn restore_drains(&mut self, saved: &[SavedDrain], connor_react_saved: bool) -> Result<(), String> {
        if saved.len() > 64 || saved.iter().any(|d| !d.valid()) {
            return Err("invalid Weeper drain checkpoint".into());
        }
        self.weeper_drains.clear();
        self.weeper_links.clear();
        for d in saved {
            if self.drain_actor_position(&d.shooter).is_none() || self.drain_actor_position(&d.target).is_none() {
                return Err(format!("Weeper drain {} has a missing actor", d.bolt));
            }
            let mut links = Vec::with_capacity(d.links.len());
            for link in &d.links {
                let file = if link.end { &d.keys.end } else { &d.keys.segment };
                let i = self.place_drain_link(file, link.name.clone(), link.position)
                    .ok_or_else(|| format!("Weeper drain {} is missing link template {file}", d.bolt))?;
                links.push(i);
            }
            if d.target.eq_ignore_ascii_case("Connor") && !connor_react_saved {
                self.requests.push(Request::ConnorSpecial { items: d.keys.hurt_anim.clone(), at: "restored Weeper drain".into() });
            }
            self.weeper_drains.push(Drain {
                bolt: d.bolt.clone(), shooter: d.shooter.clone(), target: d.target.clone(),
                keys: d.keys.clone(), ticks: d.ticks, pulses: d.pulses, links,
            });
        }
        Ok(())
    }

    fn drain_actor_position(&self, name: &str) -> Option<[f32; 3]> {
        if name.eq_ignore_ascii_case("Connor") {
            return self.connor_at.filter(|_| !self.connor_dying);
        }
        let actor = &self.anims[self.anim_index(name)?];
        let character = actor.character.as_deref()?;
        (!actor.gone && !character.unborn && character.state != State::Die).then_some(actor.player.base.position)
    }

    fn drain_animation(&mut self, name: &str, movement: Option<&str>) {
        if name.eq_ignore_ascii_case("Connor") {
            let at = "Weeper drain".into();
            self.requests.push(match movement {
                Some(m) => Request::ConnorSpecial { items: m.into(), at },
                None => Request::ConnorWait { at },
            });
        } else if let Some(i) = self.anim_index(name) {
            let now = self.now();
            let actor = &mut self.anims[i];
            if let Some(c) = actor.character.as_deref_mut() {
                if let Some(m) = movement {
                    c.set_state(&mut actor.player, "Special", &[m.to_owned()], now);
                } else {
                    c.set_state(&mut actor.player, "Wait", &[], now);
                }
            }
        }
    }

    // Each shooter reuses its own links. A later shot can need fewer segments.
    fn place_drain_link(&mut self, file: &str, name: String, position: [f32; 3]) -> Option<usize> {
        if let Some(i) = self.anim_index(&name) {
            let a = &mut self.anims[i];
            a.player.base.position = position;
            a.player.start("stop");
            a.gone = false;
            return Some(i);
        }
        let file = file.to_ascii_lowercase();
        let TemplateKind::Anim(model) = &self.templates.get(&file)?.kind else { return None };
        let mut player = (**model).clone();
        player.name = name.clone();
        player.base.position = position;
        let placement = player.base;
        let i = self.anims.len();
        self.anims.push(AnimObject { player, script: None, character: None, gone: false, translucency: None });
        self.weeper_links.insert(i);
        self.requests.push(Request::Spawn { file, name, placement, at: "Weeper drain".into() });
        Some(i)
    }

    pub(crate) fn begin_weeper_drain(&mut self, bolt: String, shooter: String, target: String, keys: DrainKeys) {
        let (Some(mut from), Some(mut to)) = (self.drain_actor_position(&shooter), self.drain_actor_position(&target)) else { return };
        let special = if target.eq_ignore_ascii_case("Connor") {
            self.connor_state == Some(state_number::SPECIAL)
        } else {
            self.anim_index(&target).and_then(|i| self.anims[i].character.as_ref()).is_some_and(|c| c.state == State::Special)
        };
        if special || self.weeper_drains.iter().any(|d| d.shooter == shooter || d.target == target) {
            self.fight_note(&shooter, format!("{bolt} touches {target} in Special or already draining: discarded"));
            return;
        }
        from[2] += keys.shoot_height;
        to[2] += keys.hit_height;
        let delta: [f32; 3] = std::array::from_fn(|k| to[k] - from[k]);
        let length = delta[0].hypot(delta[1]).hypot(delta[2]);
        let mut links = Vec::new();
        // Approximately 250 units; the original rounding is unresolved.
        for n in 0..(length / 250.0).ceil() as usize {
            let point = std::array::from_fn(|k| from[k] + delta[k] * (n as f32 * 250.0 / length));
            links.extend(self.place_drain_link(&keys.segment, format!("WeepLink.{shooter}.{n}"), point));
        }
        links.extend(self.place_drain_link(&keys.end, format!("WeepEnd.{shooter}"), to));
        self.drain_animation(&shooter, Some(&keys.fire_anim));
        self.drain_animation(&target, Some(&keys.hurt_anim));
        self.fight_note(&shooter, format!("{bolt} starts drain on {target}: {} links, {} / {}, {} s", links.len(), keys.fire_anim, keys.hurt_anim, keys.duration));
        self.weeper_drains.push(Drain { bolt, shooter, target, keys, ticks: 0, pulses: 0, links });
    }

    fn drain_one_hp(&mut self, target: &str) -> Option<i64> {
        if target.eq_ignore_ascii_case("Connor") {
            let stats = self.connor_stats.as_mut()?;
            if self.god_mode || self.connor_roping {
                return Some(stats.stats.0[0]);
            }
            stats.stats.0[0] -= 1;
            let hp = stats.stats.0[0];
            self.connor_stats_events += 1;
            if hp < 0 {
                self.request_connor_death(HitKind::Projectile, "Weeper drain".into());
            }
            Some(hp)
        } else {
            let i = self.anim_index(target)?;
            let f = self.anims[i].character.as_mut()?.fight.as_mut()?;
            f.stats.0[0] -= 1;
            let hp = f.stats.0[0];
            if hp < 0 {
                self.character_dies(i, HitKind::Projectile, false);
            }
            Some(hp)
        }
    }

    pub(crate) fn tick_weeper_drains(&mut self) {
        for mut d in std::mem::take(&mut self.weeper_drains) {
            d.ticks += 1;
            let elapsed = d.ticks as f64 / crate::UPDATES_PER_SECOND;
            let mut live = self.drain_actor_position(&d.target).is_some();
            let shooter_live = self.drain_actor_position(&d.shooter).is_some();
            let duration = f64::from(d.keys.duration);
            let due = (elapsed.min(duration) * 4.0).floor() as u32;
            while shooter_live && live && d.pulses < due {
                let Some(hp) = self.drain_one_hp(&d.target) else {
                    live = false;
                    break;
                };
                d.pulses += 1;
                self.fight_note(&d.shooter, format!("{} drain pulse {} on {}: health {hp}", d.bolt, d.pulses, d.target));
                live = hp >= 0;
            }
            if shooter_live && live && elapsed < duration {
                self.weeper_drains.push(d);
                continue;
            }
            for i in d.links {
                self.anims[i].gone = true;
            }
            if live {
                self.drain_animation(&d.target, None);
            }
            if shooter_live {
                self.drain_animation(&d.shooter, Some(&d.keys.done_anim));
                let now = self.now();
                if let Some(f) = self.anim_index(&d.shooter).and_then(|i| self.anims[i].character.as_mut()?.fight.as_mut()) {
                    f.attack_time = now + f64::from(f.settings.attack_delay);
                }
            }
            self.fight_note(&d.shooter, format!("{} ends drain after {} pulses; {}", d.bolt, d.pulses, d.keys.done_anim));
        }
    }
}
