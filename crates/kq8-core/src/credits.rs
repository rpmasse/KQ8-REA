//! Credits.gui and the numbered cards in the installation's 400.MSG.
use crate::{gui_draw::Canvas, main_menu::Art, menu_layout::Layout};
use kq8_data::{msg::MessageFile, GameDir};
use std::{path::Path, time::Duration};

// The assets supply cards and font slots, but contain no timed track.
// This dwell time and layout of mixed-size lines await an original trace.
const CARD_TIME: Duration = Duration::from_secs(5);

/// Preserve script call order across a finished world's presentations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presentation {
    Credits { score: Option<i64> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        active::Active,
        stats::{ConnorStats, Stats},
    };
    use kq8_data::{world_script::Vars, VolArchive};

    #[test]
    #[ignore = "requires the 400.MSG catalogue, original fonts and team portrait"]
    fn original_roster_and_team_card_render_in_case_order() {
        let path = std::env::var_os("KQ8_GAME_DIR").unwrap();
        let path = Path::new(&path);
        let menu = Art::open(path).unwrap();
        let mut page = Screen::open(path, Some(711)).unwrap();
        let mut previous = Vec::new();
        let mut pictures = std::collections::BTreeSet::new();
        assert_eq!(page.count(), 25);
        for i in 0..25 {
            assert_eq!(page.index, i);
            assert_eq!(page.team_photo(), i == 24);
            for line in &page.cards[i] {
                let (key, text) = line
                    .strip_prefix('*')
                    .map_or(("1122", line.as_str()), |s| ("1136", s));
                let control = &page.layout.controls[key];
                let font = &page.layout.fonts[&control.text.unwrap().font];
                let text = text.replace("%d", "6631364");
                assert!(
                    crate::gui_draw::text_width(font, &text) <= control.rect[2] - control.rect[0],
                    "400.MSG case {} would cut off a credit",
                    i + 1
                );
            }
            let pixels = page.render(&menu).rgba;
            use std::hash::Hasher;
            let mut digest = std::hash::DefaultHasher::new();
            digest.write(&pixels);
            assert!(
                pictures.insert(digest.finish()),
                "card {} repeated a previous image",
                i + 1
            );
            previous = pixels;
            if i < 24 {
                assert!(!page.tick(CARD_TIME));
            }
        }
        let mut from_title = Screen::open(path, None).unwrap();
        assert!(!from_title.tick(CARD_TIME * 24));
        let title_pic = from_title.render(&menu);
        // Same original team image and title, distinct score row.
        let before_score = 320 * 640 * 4;
        assert_eq!(&previous[..before_score], &title_pic.rgba[..before_score]);
        assert_ne!(&previous[before_score..], &title_pic.rgba[before_score..]);
        let photo = (12..288)
            .step_by(4)
            .flat_map(|y| (121..519).step_by(4).map(move |x| (y * 640 + x) * 4));
        assert!(
            photo
                .filter(|&i| previous[i..i + 3].iter().any(|v| *v > 16))
                .count()
                > 100
        );
        assert!(page.tick(CARD_TIME));
    }

    #[test]
    #[ignore = "requires the original final Temple script in kq.vol"]
    fn original_final_animation_cue_queues_the_score_before_teardown() {
        let path = std::env::var_os("KQ8_GAME_DIR").unwrap();
        let game = GameDir::open(Path::new(&path)).unwrap();
        let vol = VolArchive::open(&game.volume_path("temple4", "kq").unwrap()).unwrap();
        let source = vol.unpack(vol.find("vortrap.cs").unwrap()).unwrap().data;
        let lines = String::from_utf8_lossy(&source).to_ascii_lowercase();
        let end = lines.find("killgame").unwrap();
        assert!(lines[end..].find("playavi").unwrap() < lines[end..].find("docredits").unwrap());
        let mut a = Active::new(
            Vars::default(),
            vec![],
            vec![],
            ["Connor".into()],
            [("vortrap.cs".into(), source)],
        );
        a.world.connor_stats = Some(ConnorStats {
            stats: Stats([0, 0, 0, 0, 123]),
            level: 16,
        });
        let args = ["vortrap", "Cue", "300", "0", "c_imdone.kqs"].map(str::to_owned);
        assert!(a.scripts.call("vortrap.cs", &args, &mut a.world));
        assert_eq!(
            a.world.presentations.pop_front(),
            Some(Presentation::Credits {
                score: Some(313_953)
            })
        );
        assert!(a.world.presentations.is_empty());
    }
}

pub struct Screen {
    layout: Layout,
    cards: Vec<Vec<String>>,
    pub index: usize,
    elapsed: Duration,
    score: Option<i64>,
}

impl Screen {
    pub fn open(path: &Path, score: Option<i64>) -> Result<Self, String> {
        let layout = Layout::read(path, "credits.gui", "IDCTL_")?;
        for k in ["BACKGROUND", "1122", "1123", "1135", "1136", "1162"] {
            if !layout.controls.contains_key(k) {
                return Err(format!("credits.gui lacks {k}"));
            }
        }
        let game = GameDir::open(path).map_err(|e| e.to_string())?;
        let name = game
            .find("GAME/English/400.MSG")
            .ok_or("missing credits catalogue 400.MSG")?;
        let mut messages =
            MessageFile::parse(400, &std::fs::read(name).map_err(|e| e.to_string())?)?.lines;
        messages.retain(|l| l.tuple.noun == 0 && l.tuple.verb == 0 && l.tuple.seq == 1);
        messages.sort_by_key(|l| l.tuple.case);
        let mut cards = Vec::new();
        for l in messages {
            if usize::from(l.tuple.case) != cards.len() + 1 {
                return Err("non-consecutive 400.MSG credits cards".into());
            }
            cards.push(l.raw.lines().map(str::to_owned).collect());
        }
        if cards.is_empty() {
            return Err("empty 400.MSG credits catalogue".into());
        }
        Ok(Self {
            layout,
            cards,
            index: 0,
            elapsed: Duration::ZERO,
            score,
        })
    }

    pub fn count(&self) -> usize {
        self.cards.len()
    }
    pub fn team_photo(&self) -> bool {
        self.cards[self.index].iter().any(|l| l.contains("%d"))
    }
    pub fn score(&self) -> Option<i64> {
        self.score
    }
    pub fn advance(&mut self) -> bool {
        self.elapsed = Duration::ZERO;
        if self.index + 1 == self.cards.len() {
            return true;
        }
        self.index += 1;
        false
    }
    pub fn tick(&mut self, dt: Duration) -> bool {
        self.elapsed = self.elapsed.saturating_add(dt);
        while self.elapsed >= CARD_TIME {
            self.elapsed -= CARD_TIME;
            if self.index + 1 == self.cards.len() {
                return true;
            }
            self.index += 1;
        }
        false
    }
    pub fn render(&self, art: &Art) -> Canvas {
        let mut out = Canvas::new(640, 480);
        let rect = |k: &str| self.layout.controls[k].rect;
        for k in ["BACKGROUND", "1162"] {
            if k == "BACKGROUND" || self.team_photo() {
                self.layout.draw(&mut out, art, k, k, rect(k), 0, None);
            }
        }
        // The final card deliberately contains eight blank large lines;
        // these place the team row beneath the original 400x300 photograph.
        let gap = rect("1123")[1] - rect("1122")[3];
        let mut y = rect("1122")[1];
        for line in &self.cards[self.index] {
            let (key, text) = match line.strip_prefix('*') {
                Some(t) => ("1136", t),
                None => ("1122", line.as_str()),
            };
            let [l, t, r, b] = rect(key);
            let h = b - t;
            let score_line;
            let text = if text.contains("%d") {
                let Some(score) = self.score else { continue };
                score_line = text.replacen("%d", &score.to_string(), 1);
                &score_line
            } else {
                text
            };
            self.layout
                .draw(&mut out, art, key, key, [l, y, r, y + h], 0, Some(text));
            y += h + gap;
        }
        out
    }
}
