//! Connor's Choice state (`characters.md` "Connor's Choice state" rules
//! 1–13, Q155): the player picks one of two or three directions by
//! pressing forward or back while facing one of them. Daventry's roof jump
//! (`rooftrap.cs`), Barren's basilisk, Ice World's orcs.
//!
//! This module has the list, the points and the pick; the viewer runs the
//! state on Connor (`connor_view.rs`).

/// The probe's distance ahead of or behind Connor (rule 7).
pub const PROBE: f32 = 200.0;

/// `KQMonster::setState Connor Choice <list>`'s list (rule 2): a script
/// name, then triples `<number>,<x>,<y>`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChoiceList {
    pub script: String,
    /// The triples as written: the number (`DoChoice`'s `$3`) and the point.
    pub points: Vec<(String, [f32; 2])>,
}

impl ChoiceList {
    pub fn valid(&self) -> bool {
        let label = |s: &str| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control);
        label(&self.script) && !self.script.contains(['/', '\\', ':']) && !self.script.contains("..")
            && (1..=128).contains(&self.points.len())
            && self.points.iter().all(|(n, p)| label(n) && p.iter().all(|v| v.is_finite()))
    }

    /// Reads the list (rule 2). `Err`: why not.
    pub fn parse(list: &str) -> Result<Self, String> {
        let mut words = list.split(',').map(str::trim);
        let script = words.next().filter(|s| !s.is_empty()).ok_or("no script name")?.to_string();
        let rest: Vec<&str> = words.collect();
        if rest.is_empty() || !rest.len().is_multiple_of(3) {
            return Err(format!("{} words after the script, not triples", rest.len()));
        }
        let mut points = Vec::new();
        for t in rest.chunks(3) {
            let x = t[1].parse::<f32>().map_err(|_| format!("x '{}' is not a number", t[1]))?;
            let y = t[2].parse::<f32>().map_err(|_| format!("y '{}' is not a number", t[2]))?;
            points.push((t[0].to_string(), [x, y]));
        }
        Ok(ChoiceList { script, points })
    }
}

/// The state while it lasts.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Choice {
    pub list: ChoiceList,
    /// The points on the circle (rule 6), measured at the first update
    /// after `StartChoice` ran (rule 5); `None` until then.
    pub circle: Option<Vec<(String, [f32; 2])>>,
    /// `DoChoice` was sent (rule 10): any request is taken now and ends
    /// the state (rule 12).
    pub made: bool,
}

impl Choice {
    pub fn valid(&self) -> bool {
        self.list.valid() && (!self.made || self.circle.is_some())
            && self.circle.as_ref().is_none_or(|circle| circle.len() == self.list.points.len()
                && circle.iter().zip(&self.list.points).all(|((n, p), (original, _))| n == original && p.iter().all(|v| v.is_finite())))
    }

    pub fn new(list: ChoiceList) -> Self {
        Choice { list, circle: None, made: false }
    }
}

/// Rule 6: the level distance from Connor to the first point is the
/// radius; every other point is moved along its own direction from Connor
/// until it is that far too. A point on Connor stays where it is.
pub fn on_circle(at: [f32; 2], points: &[(String, [f32; 2])]) -> Vec<(String, [f32; 2])> {
    let dist = |p: [f32; 2]| ((p[0] - at[0]).powi(2) + (p[1] - at[1]).powi(2)).sqrt();
    let Some(r) = points.first().map(|p| dist(p.1)) else { return Vec::new() };
    points
        .iter()
        .map(|(n, p)| {
            let d = dist(*p);
            let q = if d > 0.0 { [at[0] + (p[0] - at[0]) * r / d, at[1] + (p[1] - at[1]) * r / d] } else { *p };
            (n.clone(), q)
        })
        .collect()
}

/// Rule 7: the point nearest to a probe [`PROBE`] ahead of Connor
/// (`forward`) or behind him; on a tie the earlier one. `heading`: his
/// heading (0 = facing −Y). Returns its place in the list.
pub fn pick(at: [f32; 2], heading: f32, forward: bool, circle: &[(String, [f32; 2])]) -> Option<usize> {
    let s = if forward { PROBE } else { -PROBE };
    let probe = [at[0] + s * heading.sin(), at[1] - s * heading.cos()];
    let d2 = |p: [f32; 2]| (p[0] - probe[0]).powi(2) + (p[1] - probe[1]).powi(2);
    let mut best: Option<(usize, f32)> = None;
    for (i, (_, p)) in circle.iter().enumerate() {
        if best.is_none_or(|(_, b)| d2(*p) < b) {
            best = Some((i, d2(*p)));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOF: &str = "rooftrap.cs,0,220000,183200,1,230000,183200";

    #[test]
    fn a_checkpoint_keeps_measured_points_and_rejects_malformed_cache_state() {
        let mut c = Choice::new(ChoiceList::parse(ROOF).unwrap());
        c.circle = Some(on_circle([228239.,183179.], &c.list.points));
        c.made = true;
        let saved: Choice = serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        assert_eq!(saved, c);
        assert!(saved.valid());
        for bad in 0..5 {
            let mut invalid = saved.clone();
            match bad {
                0 => invalid.circle = None,
                1 => invalid.circle.as_mut().unwrap().swap(0, 1),
                2 => invalid.circle.as_mut().unwrap().pop().map(|_| ()).unwrap(),
                3 => invalid.list.points[0].1[0] = f32::NAN,
                _ => invalid.list.script = "../outside.cs".into(),
            }
            assert!(!invalid.valid());
        }
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR pointing to the GOG installation"]
    fn original_roof_basilisk_and_orc_choice_lists_keep_their_authored_labels() {
        let game = kq8_data::GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let mut counts = Vec::new();
        for (region, files) in [("Daventry", vec!["rooftrap.cs"]), ("Barren", vec!["bastrap.cs", "bastrap1.cs"]), ("IceWorld", vec!["killorc.cs"])] {
            let volume = kq8_data::VolArchive::open(&game.volume_path(region, "kq").unwrap()).unwrap();
            for file in files {
                let bytes = volume.unpack(volume.find(file).unwrap()).unwrap().data;
                let text = String::from_utf8_lossy(&bytes);
                for line in text.lines().map(str::trim).filter(|line| line.starts_with("KQMonster::setState Connor Choice ")) {
                    let list = ChoiceList::parse(line.split_whitespace().nth(3).unwrap()).unwrap();
                    assert!(list.valid(), "{region}/{file}");
                    let at = list.points[0].1;
                    let mut state = Choice::new(list);
                    state.circle = Some(on_circle([at[0] + 100.,at[1] + 100.], &state.list.points));
                    assert!(state.valid());
                    let saved: Choice = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
                    assert_eq!(saved, state);
                    counts.push(state.list.points.iter().map(|(n,_)| n.clone()).collect::<Vec<_>>());
                }
            }
        }
        assert_eq!(counts, vec![vec!["0","1"],vec!["0","1"],vec!["0","1"],vec!["0","1","2"],vec!["0","2"],vec!["1","2"]]);
    }

    #[test]
    fn the_list_is_a_script_then_triples() {
        let l = ChoiceList::parse(ROOF).unwrap();
        assert_eq!(l.script, "rooftrap.cs");
        assert_eq!(l.points, vec![("0".to_string(), [220000.0, 183200.0]), ("1".to_string(), [230000.0, 183200.0])]);
        assert!(ChoiceList::parse("rooftrap.cs,0,220000").is_err());
        assert!(ChoiceList::parse("rooftrap.cs").is_err());
    }

    #[test]
    fn daventry_s_roof_points_and_picks() {
        // Rule 6: after StartChoice he stands at (228239, 183179): R = 8,239,
        // point 1 becomes about (236478, 183277).
        let at = [228239.0, 183179.0];
        let c = on_circle(at, &ChoiceList::parse(ROOF).unwrap().points);
        let r = ((c[0].1[0] - at[0]).powi(2) + (c[0].1[1] - at[1]).powi(2)).sqrt();
        assert!((r - 8239.0).abs() < 1.0, "{r}");
        assert!((c[1].1[0] - 236478.0).abs() < 2.0 && (c[1].1[1] - 183277.0).abs() < 2.0, "{:?}", c[1]);
        // Rule 7: facing -1.57, forward picks 0 (west), back picks 1.
        assert_eq!(pick(at, -1.57, true, &c), Some(0));
        assert_eq!(pick(at, -1.57, false, &c), Some(1));
        // Turned round: the other way.
        assert_eq!(pick(at, 1.57, true, &c), Some(1));
    }
}
