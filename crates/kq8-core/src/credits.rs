//! Original credits cards, real-time presentation and late completion score.
//! Rules: docs/behaviour/credits-completion.md.
use crate::{gui_draw::Canvas, main_menu::Key, menu_layout::Layout, stats::ConnorStats};
use kq8_data::{
    msg::MessageFile,
    palette::{Output, Rgb},
    GameDir, Palette,
};
use std::{path::Path, time::Duration};

const CARD_COUNT: usize = 25;
const BASIC_TIME: Duration = Duration::from_secs(2);
const EXIT_FADE: Duration = Duration::from_secs(1);

/// Preserve script call order without taking an early score snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Presentation {
    EndGame,
    Movie(String),
    Credits,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{active::Active, stats::Stats};
    use kq8_data::{world_script::Vars, VolArchive};

    fn game_path() -> std::path::PathBuf {
        std::env::var_os("KQ8_GAME_DIR")
            .expect("set KQ8_GAME_DIR")
            .into()
    }
    fn active_score(level: i64, experience: i64) -> ScoreState {
        ScoreState {
            active: true,
            connor: Some(ConnorStats {
                level,
                stats: Stats([0, 0, 0, 0, experience]),
            }),
        }
    }
    fn no_sample() -> ScoreState {
        panic!("score must be sampled only while preparing card 25")
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR; controlled-clock credits behavior matrix"]
    fn credits_grouped_clock_input_score() {
        let path = game_path();
        let start = Duration::from_secs(100);
        let mut page = Screen::open(&path, [640, 480], start).unwrap();
        let first_deadline = page.deadline;
        assert_eq!(
            first_deadline,
            start + page.cards[0].as_ref().unwrap().duration(false)
        );
        for key in [
            Key::Enter,
            Key::Release,
            Key::Character(' '),
            Key::Up,
            Key::Down,
            Key::Left,
            Key::Right,
            Key::Click(320, 240),
        ] {
            assert_eq!(
                page.update(
                    first_deadline - Duration::from_millis(1),
                    Some(key),
                    no_sample
                )
                .unwrap(),
                Update::Stay
            );
            assert_eq!(page.index, 0);
        }
        assert_eq!(
            page.update(first_deadline, Some(Key::Enter), no_sample)
                .unwrap(),
            Update::Changed
        );
        assert_eq!(page.index, 1);
        let late = page.deadline + Duration::from_secs(100);
        assert_eq!(page.update(late, None, no_sample).unwrap(), Update::Changed);
        assert_eq!(page.index, 2, "a stall advances one card only");
        assert_eq!(
            page.deadline,
            late + page.cards[2].as_ref().unwrap().duration(false)
        );
        assert_eq!(
            page.update(late, Some(Key::Escape), no_sample).unwrap(),
            Update::BeginExit
        );
        assert_eq!(page.exit_gain(late), Some(1.0));
        assert_eq!(page.exit_gain(late + Duration::from_millis(500)), Some(0.5));
        assert_eq!(
            page.update(
                late + Duration::from_millis(999),
                Some(Key::Enter),
                no_sample
            )
            .unwrap(),
            Update::Stay
        );
        assert_eq!(
            page.update(late + EXIT_FADE, None, no_sample).unwrap(),
            Update::Main
        );

        for (level, experience, expected) in [
            (1, 7, 7),
            (2, 3, 13),
            (15, 7, 163_837),
            (16, 7, 313_837),
            (17, 0, 463_830),
        ] {
            assert_eq!(active_score(level, experience).value().unwrap(), expected);
        }
        let mut changed_level = active_score(2, 3);
        changed_level.connor.as_mut().unwrap().level = 3;
        assert_eq!(
            changed_level.value().unwrap(),
            33,
            "level-only changes affect the score"
        );
        changed_level.active = false;
        assert_eq!(
            changed_level.value().unwrap(),
            0,
            "inactive leftover Connor is ignored"
        );
        assert!(active_score(0, 0).value().is_err());
        assert!(active_score(i64::MAX, 0).value().is_err());
        assert!(active_score(16, i64::MAX).value().is_err());
        assert!(ScoreState {
            active: true,
            connor: None
        }
        .value()
        .is_err());
        assert_eq!(active_score(16, -7).value().unwrap(), 313_823);
        assert!(
            active_score(100, 0).value().unwrap() > 6_631_364,
            "no score cap"
        );

        let mut page = Screen::open(&path, [640, 480], start).unwrap();
        assert!(page.score().is_none());
        page.index = 23;
        page.deadline = start;
        let mut samples = 0;
        assert_eq!(
            page.update(start, None, || {
                samples += 1;
                active_score(16, 7)
            })
            .unwrap(),
            Update::Changed
        );
        assert_eq!(samples, 1);
        assert_eq!(page.score(), Some(313_837));
        assert!(page.team_photo());
        assert_eq!(page.deadline, start + Duration::from_secs(19));
        assert_eq!(
            page.update(
                page.deadline - Duration::from_millis(1),
                Some(Key::Click(320, 240)),
                no_sample
            )
            .unwrap(),
            Update::Stay
        );
        assert_eq!(
            page.update(page.deadline, None, no_sample).unwrap(),
            Update::BeginExit
        );
        assert_eq!(
            page.update(page.deadline + EXIT_FADE, None, no_sample)
                .unwrap(),
            Update::Main
        );

        let mut missing = Screen::open(&path, [640, 480], start).unwrap();
        missing.cards[1] = None;
        assert_eq!(
            missing.update(missing.deadline, None, no_sample).unwrap(),
            Update::Changed
        );
        let retry = missing.deadline;
        assert_eq!(
            missing.update(retry, None, no_sample).unwrap(),
            Update::Stay
        );
        assert_eq!(missing.index, 1);
        assert_eq!(missing.deadline, retry + BASIC_TIME);
        assert!(missing
            .render()
            .rgba
            .chunks_exact(4)
            .all(|p| p[..3] == missing.palette[0] && p[3] == 255));
        for (raw, last) in [("", false), ("%s", false), ("%d %d", true), ("score", true)] {
            assert!(Card::parse(raw, last).is_err());
        }
        assert!(Card::parse(&vec!["name"; 14].join("\n"), false).is_err());
        assert!(Card::parse(&vec!["*name"; 17].join("\n"), false).is_err());
        println!("credits clock/input/score: fixed deadlines, one late advance, no skip, fade/exit, late sample, level boundary, missing-card retry and malformed-data rejection");
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR; original credits data/layout and final request matrix"]
    fn credits_grouped_original_layout_and_request() {
        let path = game_path();
        let game = GameDir::open(&path).unwrap();
        let messages = MessageFile::parse(
            400,
            &std::fs::read(game.find("GAME/English/400.MSG").unwrap()).unwrap(),
        )
        .unwrap();
        let order: Vec<_> = messages
            .lines
            .iter()
            .filter(|m| m.tuple.noun == 0 && m.tuple.verb == 0 && m.tuple.seq == 1)
            .map(|m| m.tuple.case)
            .collect();
        assert_eq!(order.len(), 25);
        assert!(
            order.windows(2).any(|pair| pair[0] > pair[1]),
            "physical order is not card order"
        );
        let mut final_pictures = Vec::new();
        for size in [[640, 480], [800, 600]] {
            let mut page = Screen::open(&path, size, Duration::ZERO).unwrap();
            assert_eq!(page.count(), 25);
            let cards: Vec<_> = page.cards.iter().map(|c| c.as_ref().unwrap()).collect();
            assert_eq!(cards.iter().map(|c| c.lines.len()).sum::<usize>(), 132);
            assert_eq!(
                cards
                    .iter()
                    .enumerate()
                    .map(|(i, c)| c.duration(i == 24))
                    .sum::<Duration>(),
                Duration::from_secs(128)
            );
            for &(count, gap, top) in if size[0] == 640 {
                &[(2, 36, 204), (12, 35, 30), (16, 26, 32), (10, 36, 60)]
            } else {
                &[(2, 36, 264), (12, 36, 84), (16, 34, 28), (10, 36, 120)]
            } {
                assert!(cards.iter().any(|c| c.lines.len() == count));
                assert_eq!(page.line_rect(0, count), [0, top, size[0] as i32, top + 30]);
                assert_eq!(page.line_rect(1, count)[1] - top, gap);
            }
            for key in ["1122", "1136"] {
                let text = page.layout.controls[key].text.unwrap();
                assert_eq!(text.justification, 0x0c);
                let font = &page.layout.fonts[&text.font];
                assert_eq!([font.h_scale, font.v_scale], [1.0, 1.0]);
            }
            assert_eq!(
                page.layout.controls["1162"].rect,
                if size[0] == 640 {
                    [120, 10, 520, 310]
                } else {
                    [150, 10, 650, 385]
                }
            );
            // The current GOG install omits the optional saveload palette;
            // credits.KQ selects Menus as the retained fallback.
            let palette_path = game.find("GAME/8Gui/saveload.ppl")
                .unwrap_or_else(|| game.find("GAME/8Gui/Menus.ppl").unwrap());
            let original_palette = Palette::parse(&std::fs::read(palette_path).unwrap()).unwrap();
            assert_eq!(
                page.palette,
                kq8_data::palette::as_shown(original_palette.colors)
            );
            let first = page.render().rgba;
            let mut signatures = std::collections::BTreeSet::new();
            for i in 0..25 {
                assert_eq!(page.index, i);
                assert_eq!(page.team_photo(), i == 24);
                let card = page.cards[i].as_ref().unwrap();
                let original = messages
                    .lines
                    .iter()
                    .find(|m| {
                        usize::from(m.tuple.case) == i + 1
                            && m.tuple.noun == 0
                            && m.tuple.verb == 0
                            && m.tuple.seq == 1
                    })
                    .unwrap();
                assert_eq!(
                    card.lines,
                    original.raw.lines().map(str::to_owned).collect::<Vec<_>>()
                );
                for line in &card.lines {
                    let (key, text) = line
                        .strip_prefix('*')
                        .map_or(("1122", line.as_str()), |s| ("1136", s));
                    let font = &page.layout.fonts[&page.layout.controls[key].text.unwrap().font];
                    assert!(
                        crate::gui_draw::text_width(font, &text.replace("%d", "313837"))
                            <= size[0] as i32
                    );
                }
                let pixels = page.render().rgba;
                use std::hash::Hasher;
                let mut digest = std::hash::DefaultHasher::new();
                digest.write(&pixels);
                assert!(
                    signatures.insert(digest.finish()),
                    "card {i} must replace prior rows"
                );
                if i < 24 {
                    assert_eq!(
                        page.update(page.deadline, None, || active_score(16, 7))
                            .unwrap(),
                        Update::Changed
                    );
                } else {
                    assert_eq!(card.lines[..8], vec![" ".to_owned(); 8]);
                    assert!(card.lines[9].starts_with('*'));
                    assert!(card.lines[9].contains("6631364"));
                    assert_eq!(page.score(), Some(313_837));
                    assert_eq!(
                        page.line_rect(8, 10)[1],
                        if size[0] == 640 { 348 } else { 408 }
                    );
                    assert_eq!(
                        page.line_rect(9, 10)[1],
                        if size[0] == 640 { 384 } else { 444 }
                    );
                    final_pictures.push(pixels);
                }
            }
            let reopened = Screen::open(&path, size, Duration::from_secs(300)).unwrap();
            assert!(!reopened.team_photo());
            assert_eq!(reopened.index, 0);
            assert_eq!(
                reopened.render().rgba,
                first,
                "reentry clears previous rows/photo"
            );
            let mut title = Screen::open(&path, size, Duration::ZERO).unwrap();
            title.index = 23;
            title.deadline = Duration::ZERO;
            title
                .update(Duration::ZERO, None, ScoreState::default)
                .unwrap();
            assert_eq!(title.score(), Some(0));
            let before_score = page.line_rect(9, 10)[1] as usize * size[0] as usize * 4;
            assert_eq!(
                &title.render().rgba[..before_score],
                &final_pictures.last().unwrap()[..before_score]
            );
            assert_ne!(title.render().rgba, *final_pictures.last().unwrap());
        }
        let vol = VolArchive::open(&game.volume_path("temple4", "kq").unwrap()).unwrap();
        let source = vol.unpack(vol.find("vortrap.cs").unwrap()).unwrap().data;
        let mut active = Active::new(
            Vars::default(),
            vec![],
            vec![],
            ["Connor".into()],
            [("vortrap.cs".into(), source)],
        );
        active.world.connor_stats = active_score(16, 123).connor;
        let args = ["vortrap", "Cue", "300", "0", "c_imdone.kqs"].map(str::to_owned);
        assert!(active.scripts.call("vortrap.cs", &args, &mut active.world));
        assert_eq!(
            active.world.presentations.pop_front(),
            Some(Presentation::EndGame)
        );
        assert!(
            matches!(active.world.presentations.pop_front(), Some(Presentation::Movie(n)) if n.eq_ignore_ascii_case("w32ndg_1.dll"))
        );
        assert_eq!(
            active.world.presentations.pop_front(),
            Some(Presentation::Credits)
        );
        assert!(active.world.presentations.is_empty());
        let before = active.world.connor_stats;
        for command in ["doCredits 711", "doCredits extra value"] {
            assert_eq!(
                active.test_command(command, "credits matrix"),
                Some("1".into())
            );
            assert!(active.world.presentations.is_empty());
        }
        assert_eq!(
            active.test_command("doCredits", "credits matrix"),
            Some("0".into())
        );
        assert_eq!(
            active.world.presentations.pop_front(),
            Some(Presentation::Credits)
        );
        assert_eq!(
            active.world.connor_stats, before,
            "doCredits neither samples nor awards a score"
        );
        println!("credits original matrix: 25 cards/132 lines/128s; 640/800 layout, palette, fonts, photograph, score/reentry; ending retains request order without a score snapshot");
    }

    #[test]
    #[ignore = "requires the original Snowexit portal script"]
    fn accepting_the_original_snowexit_portal_plays_the_middle_movie() {
        let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
        let vol = VolArchive::open(&game.volume_path("Snowexit", "kq").unwrap()).unwrap();
        let script = vol.unpack(vol.find("PORTAL4.CS").unwrap()).unwrap().data;
        let mut active = Active::new(
            Vars::default(),
            vec![],
            vec![],
            ["Connor".into()],
            [("portal4.cs".into(), script)],
        );
        for event in ["Decline", "Accept"] {
            let args = ["portal4", event].map(str::to_owned);
            assert!(active.scripts.call("portal4.cs", &args, &mut active.world));
            let expected = (event == "Accept").then(|| Presentation::Movie("w32mdl_1.dll".into()));
            assert_eq!(active.world.presentations.pop_front(), expected);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Update {
    Stay,
    Changed,
    BeginExit,
    Main,
}

/// The same active-game flag as Main's Back to Game, sampled on card 25.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScoreState {
    pub active: bool,
    pub connor: Option<ConnorStats>,
}

impl ScoreState {
    pub fn value(self) -> Result<i64, String> {
        if !self.active {
            return Ok(0);
        }
        let connor = self
            .connor
            .ok_or("active credits session has no Connor stats")?;
        let level = connor.level;
        let completed = match level {
            1..=15 => Some(10 * ((1_i64 << (level - 1)) - 1)),
            16.. => (level - 15)
                .checked_mul(150_000)
                .and_then(|v| v.checked_add(163_830)),
            _ => None,
        };
        completed
            .and_then(|v| v.checked_add(connor.stats.0[4]))
            .ok_or_else(|| "invalid or overflowing credits level/experience".into())
    }
}

#[derive(Debug)]
struct Card {
    lines: Vec<String>,
}

impl Card {
    fn parse(raw: &str, final_card: bool) -> Result<Self, String> {
        let lines: Vec<_> = raw.lines().map(str::to_owned).collect();
        let small = lines.iter().filter(|s| s.starts_with('*')).count();
        if lines.is_empty() || small > 16 || lines.len() - small > 13 {
            return Err("empty credits card or original text-row capacity exceeded".into());
        }
        let formats = raw.matches('%').count();
        if formats != usize::from(final_card) || (final_card && raw.matches("%d").count() != 1) {
            return Err("invalid credits score format".into());
        }
        Ok(Self { lines })
    }
    fn duration(&self, final_card: bool) -> Duration {
        BASIC_TIME
            + Duration::from_millis(500 * self.lines.len() as u64)
            + if final_card {
                Duration::from_secs(12)
            } else {
                Duration::ZERO
            }
    }
}

pub struct Screen {
    layout: Layout,
    cards: Vec<Option<Card>>,
    pub index: usize,
    deadline: Duration,
    exit_deadline: Option<Duration>,
    score: Option<i64>,
    source_palette: [Rgb; 256],
    palette: [Rgb; 256],
}

impl Screen {
    pub fn open(path: &Path, size: [u32; 2], now: Duration) -> Result<Self, String> {
        if !matches!(size, [640, 480] | [800, 600]) {
            return Err("unsupported original credits resolution".into());
        }
        let mut layout = Layout::read(path, "credits.gui", "IDCTL_")?;
        for key in ["BACKGROUND", "1122", "1136", "1162"] {
            if !layout.controls.contains_key(key) {
                return Err(format!("credits.gui lacks {key}"));
            }
        }
        layout.resize(size, false);
        // The photo's resized dimensions are retained, but its top is 10
        // output pixels rather than the generic resized y coordinate.
        let photo = &mut layout.controls.get_mut("1162").unwrap().rect;
        let height = photo[3] - photo[1];
        photo[1] = 10;
        photo[3] = 10 + height;
        let game = GameDir::open(path).map_err(|e| e.to_string())?;
        let read = |name: &str| -> Result<Vec<u8>, String> {
            let file = game.find(name).ok_or_else(|| format!("missing {name}"))?;
            std::fs::read(file).map_err(|e| format!("{name}: {e}"))
        };
        let messages = MessageFile::parse(400, &read("GAME/English/400.MSG")?)?;
        let mut cards: Vec<Option<Card>> = (0..CARD_COUNT).map(|_| None).collect();
        for message in messages
            .lines
            .into_iter()
            .filter(|m| m.tuple.noun == 0 && m.tuple.verb == 0 && m.tuple.seq == 1)
        {
            let case = usize::from(message.tuple.case);
            if !(1..=CARD_COUNT).contains(&case) || cards[case - 1].is_some() {
                return Err("invalid or duplicate 400.MSG credits case".into());
            }
            cards[case - 1] = Some(Card::parse(&message.raw, case == CARD_COUNT)?);
        }
        if cards.iter().all(Option::is_none) {
            return Err("empty 400.MSG credits catalogue".into());
        }
        let initial = Palette::parse(&read("GAME/8Gui/Menus.ppl")?)?;
        let selected = read("GAME/8Gui/saveload.ppl")
            .ok()
            .and_then(|b| Palette::parse(&b).ok())
            .unwrap_or(initial);
        let source_palette = kq8_data::palette::as_shown(selected.colors);
        let first_duration = cards[0].as_ref().map_or(BASIC_TIME, |c| c.duration(false));
        Ok(Self {
            layout,
            cards,
            index: 0,
            deadline: now.saturating_add(first_duration),
            exit_deadline: None,
            score: None,
            source_palette,
            palette: source_palette,
        })
    }

    pub fn set_palette(&mut self, brightness: u8, output: Output) {
        self.palette = kq8_data::palette::for_output(self.source_palette, brightness, output);
    }
    pub fn count(&self) -> usize {
        CARD_COUNT
    }
    pub fn team_photo(&self) -> bool {
        self.index == CARD_COUNT - 1 && self.cards[self.index].is_some()
    }
    pub fn score(&self) -> Option<i64> {
        self.score
    }
    pub fn exit_gain(&self, now: Duration) -> Option<f32> {
        self.exit_deadline
            .map(|end| end.saturating_sub(now).as_secs_f32().clamp(0.0, 1.0))
    }
    fn begin_exit(&mut self, now: Duration) -> Update {
        self.exit_deadline = Some(now.saturating_add(EXIT_FADE));
        Update::BeginExit
    }

    /// Absolute real time prevents pauses/frame-rate changes from changing
    /// the cadence. A delayed update selects only one card, starting now.
    pub fn update(
        &mut self,
        now: Duration,
        key: Option<Key>,
        score: impl FnOnce() -> ScoreState,
    ) -> Result<Update, String> {
        if let Some(end) = self.exit_deadline {
            return Ok(if now >= end {
                Update::Main
            } else {
                Update::Stay
            });
        }
        if key == Some(Key::Escape) {
            return Ok(self.begin_exit(now));
        }
        if now < self.deadline {
            return Ok(Update::Stay);
        }
        if self.cards[self.index].is_none() {
            self.deadline = now.saturating_add(BASIC_TIME);
            return Ok(Update::Stay);
        }
        if self.index + 1 == CARD_COUNT {
            return Ok(self.begin_exit(now));
        }
        let next = self.index + 1;
        if next + 1 == CARD_COUNT && self.cards[next].is_some() {
            self.score = Some(score().value()?);
        }
        self.index = next;
        let duration = self.cards[next]
            .as_ref()
            .map_or(BASIC_TIME, |c| c.duration(next + 1 == CARD_COUNT));
        self.deadline = now.saturating_add(duration);
        Ok(Update::Changed)
    }

    fn line_rect(&self, row: usize, count: usize) -> [i32; 4] {
        let [width, height] = self.layout.size.map(|n| n as i32);
        let count = count as i32;
        let gap = 36.min((height - 50) / count);
        let top = height / 2 - count * gap / 2 + row as i32 * gap;
        [0, top, width, top + 30]
    }

    pub fn render(&self) -> Canvas {
        let [width, height] = self.layout.size;
        let mut out = Canvas::new(width, height);
        out.fill([0, 0, width as i32, height as i32], self.palette[0], 1.0);
        let Some(card) = &self.cards[self.index] else {
            return out;
        };
        if self.team_photo() {
            self.layout.draw_palette(
                &mut out,
                &self.palette,
                "1162",
                "1162",
                self.layout.controls["1162"].rect,
                0,
                None,
            );
        }
        for (i, line) in card.lines.iter().enumerate() {
            let (key, text) = line
                .strip_prefix('*')
                .map_or(("1122", line.as_str()), |text| ("1136", text));
            let formatted;
            let text = if text.contains("%d") {
                formatted = text.replacen("%d", &self.score.unwrap_or(0).to_string(), 1);
                &formatted
            } else {
                text
            };
            self.layout.draw_palette(
                &mut out,
                &self.palette,
                key,
                key,
                self.line_rect(i, card.lines.len()),
                0,
                Some(text),
            );
        }
        out
    }
}
