//! Ordinary deep-liquid attacks, from `behaviour/terrain-hazards.md`.
//! `ontop` sinking and the untraced special status are deliberately outside
//! this path. The sound checkpoint is Rust continuation, not evidence of
//! the original executable's save format or clock restoration.

use kq8_data::{envinfo::{FAST_POISON, LIQUID, POISON, SWAMP}, world_script::Vars};
use crate::fight::HitKind;

pub const FIRE: u32 = 0x200;
pub const ICE: u32 = 0x400;
pub const ONTOP: u32 = 0x800;
pub const DAMAGE_DEPTH: f32 = 250.0;
pub const ALL_IMMUNE: u16 = u16::MAX;
pub const CONNOR_IMMUNITY: u16 = ALL_IMMUNE & !((POISON | FAST_POISON | SWAMP | FIRE | ICE) as u16);
const SOUND_DELAY: f64 = 2.5;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub poison: f64,
    pub fast_poison: f64,
    pub swamp: f64,
}

impl Settings {
    pub fn from_vars(vars: &Vars) -> Self {
        let get = |name| vars.get(name).map(crate::script::number_value).unwrap_or(0.0);
        Self { poison: get("KQWorld::PoisonTime"), fast_poison: get("KQWorld::FastPoisonTime"), swamp: get("KQWorld::SwampTime") }
    }

    /// Rule 1: these are amount-source flags; fire/ice only select kind.
    pub fn source(self, flags: u32) -> Option<(&'static str, f64)> {
        if flags & POISON != 0 { Some(("PoisonTime", self.poison)) }
        else if flags & FAST_POISON != 0 { Some(("FastPoisonTime", self.fast_poison)) }
        else if flags & SWAMP != 0 { Some(("SwampTime", self.swamp)) }
        else { None }
    }
}

/// Rules 2–5: shared world phase, never elapsed exposure or accumulated damage.
pub fn amount(time: f64, connor_level: Option<i64>, now: f64) -> i64 {
    let Some(level) = connor_level else { return 0 };
    if time == 0.0 { return 1000; }
    // Negative/nonfinite settings and nonpositive levels are outside the
    // studied data. Do not invent damage for those unsupported inputs.
    if !time.is_finite() || time < 0.0 || level <= 0 || !now.is_finite() || now < 0.0 { return 0; }
    let q = (0.66 * level as f64 / time) as f32;
    if q >= 1.0 { return q as i64; }
    let stride = ((1.0 / q).trunc().min(8.0) as u32).max(1);
    let slot = (now.fract() * 15.0 + 0.5) as u32;
    i64::from(slot % stride == 0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub base: i64,
    pub kind: HitKind,
    pub source: &'static str,
}

/// Rules 6–9, 12: check only the resulting position, including the exact
/// 250-unit boundary; the animation's 600-unit threshold is independent.
pub fn ordinary(surface: Option<(f32, u32)>, feet: f32, immunity: u16,
    settings: Settings, connor_level: Option<i64>, now: f64) -> Option<Hit> {
    let (surface, flags) = surface?;
    if flags & ONTOP != 0 || flags & LIQUID == 0 || surface < feet + DAMAGE_DEPTH
        || !surface.is_finite() || !feet.is_finite() || flags as u16 & !immunity == 0 { return None; }
    let (source, time) = settings.source(flags)?;
    let base = amount(time, connor_level, now);
    if base == 0 { return None; }
    let kind = if flags & FIRE != 0 { HitKind::Fire } else if flags & ICE != 0 { HitKind::Ice } else { HitKind::Normal };
    Some(Hit { base, kind, source })
}

/// Connor's live immunity and the separate world-hit sound gate (rules 8, 10).
#[derive(Debug, Clone)]
pub struct State {
    pub immunity: u16,
    sound: u8,
    rearm_at: Option<f64>,
}

impl Default for State {
    fn default() -> Self { Self { immunity: CONNOR_IMMUNITY, sound: 1, rearm_at: None } }
}

impl State {
    pub fn add_immune(&mut self, bits: i64) { self.immunity |= bits as u16; }

    pub fn rearm(&mut self, now: f64) {
        if self.rearm_at.is_some_and(|due| now >= due) {
            self.rearm_at = None;
            self.sound = self.sound % 3 + 1;
        }
    }

    pub fn world_hit_sound(&mut self, now: f64) -> Option<&'static str> {
        self.rearm(now);
        if self.rearm_at.is_some() { return None; }
        self.rearm_at = Some(now + SOUND_DELAY);
        Some(match self.sound { 2 => "poison2.aud", 3 => "poison3.aud", _ => "poison1.aud" })
    }

    pub fn checkpoint(&self, now: f64) -> Saved {
        let mut state = self.clone();
        state.rearm(now);
        Saved { immunity: state.immunity, sound: state.sound, rearm_in: state.rearm_at.map(|due| due - now) }
    }
}

/// Our own save/world-travel continuation; original persistence is not tested.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Saved {
    immunity: u16,
    sound: u8,
    rearm_in: Option<f64>,
}

impl Default for Saved {
    fn default() -> Self { State::default().checkpoint(0.0) }
}

impl Saved {
    pub fn valid(&self) -> bool {
        (1..=3).contains(&self.sound) && self.rearm_in.is_none_or(|left| left.is_finite() && (0.0..=SOUND_DELAY).contains(&left))
    }

    pub fn restore(&self, now: f64) -> State {
        State { immunity: self.immunity, sound: self.sound, rearm_at: self.rearm_in.map(|left| now + left) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{active::{Active, Request}, stats::{ConnorStats, Stats}};
    use kq8_data::envinfo::WATER;

    const SETTINGS: Settings = Settings { poison: 5.0, fast_poison: 0.0, swamp: 5.0 };

    fn active(health: i64, protection: i64) -> Active {
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], Vec::<(String, Vec<u8>)>::new());
        a.set_connor_stats(ConnorStats { stats: Stats([health, 1, 0, protection, 0]), level: 12 });
        a.world.connor_at = Some([10.0, 20.0, 0.0]);
        a.scripts.vars.set("KQWorld::PoisonTime", "5".into());
        a.scripts.vars.set("KQWorld::FastPoisonTime", "0".into());
        a.scripts.vars.set("KQWorld::SwampTime", "5".into());
        a
    }

    #[test]
    fn terrain_hazards_grouped_sources_clock_depth_and_immunity() {
        for flags in [POISON, FAST_POISON, SWAMP, POISON | FAST_POISON | SWAMP, FAST_POISON | SWAMP] {
            let hit = ordinary(Some((250.0, flags)), 0.0, CONNOR_IMMUNITY, SETTINGS, Some(12), 0.5).unwrap();
            assert_eq!(hit.base, if flags & POISON != 0 || flags == SWAMP { 1 } else { 1000 });
            assert_eq!(hit.kind, HitKind::Normal);
        }
        for time in [0.0, -0.0] {
            for now in [0.0, 0.97, 1.0, 1000.5] {
                assert_eq!(amount(time, Some(1), now), 1000);
                assert_eq!(amount(time, None, now), 0);
            }
        }
        assert_eq!(amount(5.0, Some(12), 0.5), 1);
        assert_eq!(amount(5.0, Some(23), 0.5), 3);
        for slot in 0..15 {
            assert_eq!(amount(5.0, Some(1), slot as f64 / 15.0), i64::from([0, 7, 14].contains(&slot)), "slot {slot}");
        }
        assert_eq!(amount(5.0, Some(1), 0.97), 0, "rounded slot 15 is not clamped to 14");
        assert_eq!(amount(5.0, Some(1), 1.0), 1, "phase restarts each second");
        assert_eq!(amount(5.0, Some(1), 1.0 + 7.0 / 15.0), 1);
        for slot in 0..15 { assert_eq!(amount(1000.0, Some(1), slot as f64 / 15.0), i64::from(slot % 8 == 0)); }
        for depth in [249.0, 250.0, 599.0, 600.0] {
            let surface = Some((depth, POISON));
            assert_eq!(ordinary(surface, 0.0, CONNOR_IMMUNITY, SETTINGS, Some(12), 0.0).is_some(), depth >= 250.0);
            assert_eq!(crate::water::submerged(surface, 0.0), depth >= 600.0);
        }
        for flags in [WATER, FIRE, ICE, FIRE | ICE, POISON | ONTOP] {
            assert!(ordinary(Some((1000.0, flags)), 0.0, 0, SETTINGS, Some(12), 0.0).is_none());
        }
        assert!(ordinary(None, 0.0, 0, SETTINGS, Some(12), 0.0).is_none());
        assert!(ordinary(Some((1000.0, POISON)), 0.0, ALL_IMMUNE, SETTINGS, Some(12), 0.0).is_none());
        // The complete material is tested: immunity to the selected poison
        // source still allows the material's nonimmune fire flag to hurt.
        let poison_immune = CONNOR_IMMUNITY | POISON as u16;
        assert!(ordinary(Some((1000.0, POISON)), 0.0, poison_immune, SETTINGS, Some(12), 0.0).is_none());
        for (flags, kind) in [(POISON | FIRE, HitKind::Fire), (POISON | ICE, HitKind::Ice), (POISON | FIRE | ICE, HitKind::Fire)] {
            assert_eq!(ordinary(Some((1000.0, flags)), 0.0, poison_immune, SETTINGS, Some(12), 0.0).unwrap().kind, kind);
        }
        assert_eq!(Settings::from_vars(&Vars::default()).fast_poison, 0.0);
    }

    #[test]
    fn terrain_hazards_grouped_immunity_command() {
        let poison_immune = CONNOR_IMMUNITY | POISON as u16;
        let mut a = active(39, 11);
        assert_eq!(a.test_command("KQConner::addImmunity 65600", "matrix"), Some("0".into()));
        assert_eq!(a.world.terrain_hazards.immunity, poison_immune);
        for command in ["KQConner::addImmunity", "KQConner::addImmunity 128 256"] {
            assert_eq!(a.test_command(command, "matrix"), Some("0".into()));
            assert_eq!(a.world.terrain_hazards.immunity, poison_immune);
        }
        assert!(a.world.log.iter().any(|line| line.contains("usage: KQConner::addImmune")), "the original diagnostic typo is retained");
        assert_eq!(a.test_command("KQConner::addImmune 256", "matrix"), None, "the usage-message typo is not a command alias");
        assert_eq!(a.world.terrain_hazards.immunity, poison_immune);
        a.test_command("KQConner::addImmunity -1", "matrix");
        assert_eq!(a.world.terrain_hazards.immunity, ALL_IMMUNE);
    }

    #[test]
    #[ignore = "requires read-only KQ8_GAME_DIR original assets; no graphics"]
    fn terrain_hazards_grouped_immunity_original_wisp_callback() {
        use kq8_data::{GameDir, VolArchive};
        let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let vol = VolArchive::open(&game.volume_path("Swamp", "kq").unwrap()).unwrap();
        let source = vol.unpack(vol.find("WHISP.CS").unwrap()).unwrap().data;
        let mut a = Active::new(Vars::default(), vec![], vec![],
            ["Connor".into(), "Wisp1".into(), "flower1".into()], [("whisp.cs".into(), source)]);
        a.scripts.vars.set("ConnorLoc", "0 0 0 0 0 0".into());
        a.scripts.vars.set("FlowerLoc", "0 0 0 0 0 0".into());
        assert!(a.run_script("whisp.cs", &["Wisp1".into(), "Cue".into(), "10".into()]));
        let mask = a.world.terrain_hazards.immunity;
        assert_eq!(mask, CONNOR_IMMUNITY | SWAMP as u16, "original Cue 10 grants bit 256, preserving the other bits");
        assert!(!a.take_log().iter().any(|line| line.contains("ERROR unknown command")));
        assert!(ordinary(Some((1000.0, SWAMP)), 0.0, CONNOR_IMMUNITY, SETTINGS, Some(12), 0.0).is_some());
        assert!(ordinary(Some((1000.0, SWAMP)), 0.0, mask, SETTINGS, Some(12), 0.0).is_none());
        assert!(ordinary(Some((1000.0, POISON)), 0.0, mask, SETTINGS, Some(12), 0.0).is_some());
    }

    #[test]
    fn terrain_hazards_grouped_attack_protection_death_and_audio() {
        let surface = Some((1000.0, FAST_POISON));
        // The emitter must reach the shared event/armor path, not directly
        // subtract 1000. Zero-loss world hits still sound and request hurt.
        let mut a = active(120, 100);
        a.connor_terrain_damage(surface, 0.0);
        assert_eq!(a.world.connor_stats.unwrap().stats.0[0], 120);
        assert_eq!(a.world.sounds.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["poison1.aud"]);
        assert!(a.world.requests.iter().any(|r| matches!(r, Request::ConnorHurt { forced: false, world_hit: true, .. })));
        for (update, sounds) in [(15, 1), (37, 1), (38, 2), (75, 2), (76, 3), (114, 4)] {
            a.world.updates = update;
            a.world.advance_clock_for_test(update as f32 / 15.);
            a.connor_terrain_damage(surface, 0.0);
            assert_eq!(a.world.sounds.len(), sounds, "update {update}");
        }
        assert_eq!(a.world.sounds.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["poison1.aud", "poison2.aud", "poison3.aud", "poison1.aud"]);
        // A separate gate does not suppress ordinary attacker requests.
        a.world.hit_connor("monster", 1, HitKind::Normal);
        assert!(matches!(a.world.requests.last(), Some(Request::ConnorHurt { world_hit: false, .. })));
        for protection in [11, 50] {
            let mut fatal = active(39, protection);
            fatal.connor_terrain_damage(surface, 0.0);
            assert!(fatal.world.connor_dying);
            assert_eq!(fatal.world.connor_stats.unwrap().stats.0[0], 0);
            assert!(fatal.world.requests.iter().any(|r| matches!(r, Request::ConnorDies { kind: HitKind::Normal, .. })));
            assert_eq!(fatal.world.sounds[0].name, "poison1.aud", "lethal hit still sounds");
            fatal.world.updates = 60;
            fatal.world.advance_clock_for_test(4.);
            fatal.connor_terrain_damage(surface, 0.0);
            assert_eq!(fatal.world.sounds.len(), 1, "already dying ignores subsequent world sound");
        }
        let mut rope = active(39, 11);
        rope.world.connor_roping = true;
        rope.connor_terrain_damage(surface, 0.0);
        assert_eq!(rope.world.connor_stats.unwrap().stats.0[0], 39);
        assert!(rope.world.sounds.is_empty() && rope.world.requests.is_empty());
        let mut immune = active(39, 11);
        immune.world.terrain_hazards.add_immune(FAST_POISON as i64);
        immune.connor_terrain_damage(surface, 0.0);
        assert_eq!(immune.world.connor_stats.unwrap().stats.0[0], 39);
        assert!(immune.world.sounds.is_empty() && immune.world.requests.is_empty());
    }

    #[test]
    fn terrain_hazards_grouped_rust_journey_continuation_and_legacy_default() {
        let mut a = active(120, 100);
        a.world.terrain_hazards.add_immune(POISON as i64);
        a.world.updates = 150;
        a.world.advance_clock_for_test(10.);
        a.connor_terrain_damage(Some((1000.0, FAST_POISON)), 0.0);
        a.world.updates = 165;
        a.world.advance_clock_for_test(11.);
        let journey = crate::journey::Journey::take(&a.world, &a.scripts.vars, "DeadCity");
        assert_eq!(journey.terrain_hazards.rearm_in, Some(1.5));
        let json = serde_json::to_value(&journey).unwrap();
        let decoded: crate::journey::Journey = serde_json::from_value(json.clone()).unwrap();
        assert!(decoded.terrain_hazards.valid());
        let mut b = active(1, 0);
        b.world.updates = 300;
        b.world.advance_clock_for_test(20.);
        decoded.give(&mut b.world, &mut b.scripts.vars);
        assert_eq!(b.world.terrain_hazards.immunity, a.world.terrain_hazards.immunity);
        assert_eq!(b.world.terrain_hazards.world_hit_sound(21.49), None);
        assert_eq!(b.world.terrain_hazards.world_hit_sound(21.5), Some("poison2.aud"));
        let mut legacy = json;
        legacy.as_object_mut().unwrap().remove("terrain_hazards");
        let old: crate::journey::Journey = serde_json::from_value(legacy).unwrap();
        assert_eq!(old.terrain_hazards, Saved::default());
        for invalid in [Saved { sound: 0, ..Saved::default() }, Saved { rearm_in: Some(-1.0), ..Saved::default() }, Saved { rearm_in: Some(2.6), ..Saved::default() }] {
            assert!(!invalid.valid());
        }
    }
}
