//! Scripted preparation and direct health adjustment, from
//! behaviour/scripted-controller-commands.md rules 1–4. No executable
//! material is used here. Numeric object identifiers are not yet modeled.
use crate::active::{Request, WorldState};
use crate::script::number_value;

/// Sampled delta, not a health assignment. The minimum has the original
/// single-precision rounding; the reduction retains decimal double precision.
pub fn health_delta(level: i64, health: i64, reduction: f64, minimum: f32) -> Option<i64> {
    let maximum = level.checked_mul(10)?;
    if maximum <= 0 || !reduction.is_finite() || !minimum.is_finite() { return None; }
    let whole = |n: f64| (n.is_finite() && n >= i64::MIN as f64 && n < i64::MAX as f64).then(|| n.trunc() as i64);
    let mut target = health.checked_sub(whole(reduction * maximum as f64)?)?;
    if target as f64 / (maximum as f64) < minimum as f64 {
        target = whole(minimum as f64 * maximum as f64)?;
    }
    target.checked_sub(health)
}

impl WorldState {
    pub(crate) fn reduce_health(&mut self, words: &[String], at: &str) -> String {
        let [_, reduction, minimum] = words else {
            self.note(format!("{at}: usage: KQConner::reduceHealth <reduction> <minimum>"));
            return "0".into();
        };
        let delta = self.connor_stats.and_then(|c| health_delta(c.level, c.stats.0[0], number_value(reduction), number_value(minimum) as f32));
        if let Some(delta) = delta {
            self.queue.send(self.now(), 0.0, "Connor", "KQStatsChangeRequestEvent",
                &[delta.to_string(), "0".into(), "0".into(), "0".into(), "0".into()], at);
            self.note(format!("{at}: reduceHealth {reduction} {minimum}: queued health delta {delta}; no attack (scripted-controller-commands.md rules 3–4)"));
        } else {
            self.note(format!("{at}: reduceHealth: Connor stats absent or unsupported numeric range; nothing"));
        }
        "0".into()
    }

    pub(crate) fn pre_special(&mut self, words: &[String], at: &str) -> String {
        let [_, name] = words else {
            self.note(format!("{at}: usage: KQMonster::preSpecial <monster>"));
            return "0".into();
        };
        if name == "Connor" && self.known.contains("connor") && !self.is_gone("Connor") {
            // The viewer owns his live playback. Keep requests in script
            // order; the core's same-event death eligibility sees Stop now.
            self.connor_wait_retains_stop = true;
            self.requests.push(Request::ConnorPreSpecial { at: at.into() });
        } else if let Some(actor) = self.anims.iter_mut().find(|a| !a.gone && a.player.name == *name && a.character.is_some()) {
            actor.character.as_mut().unwrap().pre_special(&mut actor.player);
        } else {
            self.note(format!("{at}: preSpecial: no monster named '{name}'; numeric object identifiers are not implemented"));
            return "0".into();
        }
        self.note(format!("{at}: preSpecial {name}: current motion interruptible, controller remembers Stop; playback unchanged"));
        "0".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{active::Active, stats::{ConnorStats, Stats}};
    use kq8_data::world_script::Vars;

    fn active(level: i64, health: i64) -> Active {
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".into()], []);
        a.set_connor_stats(ConnorStats { level, stats: Stats([health, 10, 0, 100, 0]) });
        a
    }

    #[test]
    fn scripted_character_grouped_health_arithmetic_and_queue() {
        for (level, hp, reduction, minimum, expected) in [
            (43,430,0.5,0.02,215), (43,100,0.5,0.02,8), (43,3,0.5,0.02,8),
            (1,1,0.5,0.02,0), (2,8,-0.2,0.0,12), (2,8,2.0,-2.0,-32),
            (1,0,0.0,0.7,6), // 0.7 rounded to f32, then truncated, is six.
        ] { assert_eq!(health_delta(level,hp,reduction,minimum).map(|d| hp+d),Some(expected)); }
        for (level, reduction, minimum) in [(0,0.5,0.02), (1,f64::INFINITY,0.02), (1,0.5,f32::NAN)] {
            assert!(health_delta(level,1,reduction,minimum).is_none());
        }
        let mut a = active(43,100);
        a.world.god_mode=true; a.world.connor_roping=true;
        a.world.connor_state=Some(crate::character::state_number::ROPE);
        a.world.terrain_hazards.immunity=u16::MAX;
        for _ in 0..2 { assert_eq!(a.test_command("KQConner::reduceHealth .5 .02", "matrix"),Some("0".into())); }
        assert_eq!(a.world.connor_stats.unwrap().stats.0[0],100,"commands only queue");
        assert!(a.world.requests.is_empty() && a.world.sounds.is_empty());
        a.update();
        assert_eq!(a.world.connor_stats_events,2);
        assert_eq!(a.world.connor_stats.unwrap().stats.0[0],-84,"both commands sampled 100; rope refuses death state, not stats");
        assert!(a.world.requests.iter().all(|r| !matches!(r, Request::ConnorHurt { .. })));
        assert!(a.scripts.vars.get("CurRelHealth").is_none() && a.scripts.vars.get("CurRelDamage").is_none());
        let mut zero = active(1,1);
        zero.test_command("KQConner::reduceHealth .5 .02","matrix"); zero.update();
        assert_eq!(zero.world.connor_stats.unwrap().stats.0[0],0);
        assert!(!zero.world.connor_dying,"zero is not below zero");
        for command in ["KQConner::reduceHealth", "KQConner::reduceHealth .5", "KQConner::reduceHealth .5 .02 extra"] {
            assert_eq!(zero.test_command(command,"matrix"),Some("0".into()));
        }
        let count=zero.world.connor_stats_events; zero.update(); assert_eq!(zero.world.connor_stats_events,count);
        zero.test_command("KQConner::reduceHealth nonsense .2","matrix"); zero.update();
        assert_eq!(zero.world.connor_stats.unwrap().stats.0[0],2,"malformed reduction is zero; minimum may heal");
        let mut dying=active(2,8); dying.world.connor_dying=true;
        dying.test_command("KQConner::reduceHealth .2 0","matrix"); dying.update();
        assert_eq!(dying.world.connor_stats.unwrap().stats.0[0],4,"direct stats still reach an already dying receiver");
        let mut cap=active(2,8); cap.test_command("KQConner::reduceHealth -2 0","matrix"); cap.update();
        assert_eq!(cap.world.connor_stats.unwrap().stats.0[0],20,"ordinary maximum-health cap still applies");
        let mut experience=active(1,10); experience.world.connor_stats.as_mut().unwrap().stats.0[4]=11;
        experience.test_command("KQConner::reduceHealth 0 0","matrix"); experience.update();
        assert_eq!(experience.world.connor_stats.unwrap().level,2,"zero-XP event still processes existing excess XP");
    }

    #[test]
    #[ignore = "requires read-only KQ8_GAME_DIR original assets; no graphics"]
    fn scripted_character_grouped_original_henchman_and_altar_callbacks() {
        use kq8_data::{GameDir, VolArchive};
        let game=GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let source=|world, file| { let vol=VolArchive::open(&game.volume_path(world,"kq").unwrap()).unwrap(); vol.unpack(vol.find(file).unwrap()).unwrap().data };
        let mut a=Active::new(Vars::default(),vec![],vec![],["Connor".into(),"Henchman".into(),"Chest1".into(),"altar".into()],
            [("swhnchmn.cs".into(),source("Swamp","SWHNCHMN.CS")),("altar.cs".into(),source("temple4","ALTAR.CS"))]);
        a.set_connor_stats(ConnorStats { level:43, stats:Stats([430,10,0,100,0]) });
        assert!(a.run_script("swhnchmn.cs",&["Henchman".into(),"Die".into()]));
        let prep=a.world.requests.iter().position(|r| matches!(r,Request::ConnorPreSpecial { .. })).unwrap();
        let moved=a.world.requests.iter().position(|r| matches!(r,Request::Move { who,.. } if who=="Connor")).unwrap();
        let special=a.world.requests.iter().position(|r| matches!(r,Request::ConnorSpecial { items,.. } if items=="kill2")).unwrap();
        assert!(prep<moved && moved<special);
        a.world.requests.clear();
        // Separate callback fixture, avoiding the Henchman's unrelated XP event.
        let mut altar=Active::new(Vars::default(),vec![],vec![],["Connor".into(),"altar".into()],[("altar.cs".into(),source("temple4","ALTAR.CS"))]);
        altar.set_connor_stats(ConnorStats { level:43, stats:Stats([430,10,0,100,0]) });
        altar.scripts.vars.set("Camera4","0 0 0 0 0 0".into());
        assert!(altar.run_script("altar.cs",&["altar".into(),"Cue".into(),"1".into(),"7".into()]));
        assert_eq!(altar.world.connor_stats.unwrap().stats.0[0],430);
        assert!(altar.world.requests.iter().any(|r| matches!(r,Request::ConnorSpecial { items,.. } if items.contains("c1b"))));
        altar.update();
        assert_eq!(altar.world.connor_stats.unwrap().stats.0[0],215);
        for sim in [&mut a,&mut altar] { assert!(!sim.take_log().iter().any(|l| l.contains("ERROR unknown command"))); }
    }
}
