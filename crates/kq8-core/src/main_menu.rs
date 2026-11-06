//! Startup screen, New Game difficulty and Quit (docs/behaviour/main-menu.md).
use crate::gui_draw::{Canvas, Pic};
use kq8_data::font::{bitmap_array, Font};
use kq8_data::gui::{self, Control, Kind};
use kq8_data::msg::MessageFile;
use kq8_data::{Bitmap, GameDir, Palette};
use std::collections::BTreeMap;
use std::path::Path;

// IDs are from Mainmenu.gui. Geometry, justification and translated labels
// are read from the installation; the label case numbers are rule 2.
const ITEMS: [(u32, u8); 7] = [
    (9, 13),
    (10, 10),
    (11, 24),
    (14, 19),
    (500, 22),
    (12, 20),
    (502, 2),
];
const ACTIVE: [usize; 4] = [0, 3, 4, 5];
const ACTIVE_DURING_GAME: [usize; 6] = [0, 2, 3, 4, 5, 6];
const ACTIVE_WITH_SAVES: [usize; 5] = [0, 1, 3, 4, 5];
const ACTIVE_DURING_GAME_WITH_SAVES: [usize; 7] = [0, 1, 2, 3, 4, 5, 6];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Release,
    Escape,
    Click(i32, i32),
    Drag(i32, i32),
    MouseUp,
    Tab,
    Delete,
    Backspace,
    Character(char),
    /// Windows key number captured in the Keyboard options screen.
    Physical(u8),
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Choice {
    game: bool,
    row: usize,
    saves: bool,
    dialog: Option<Dialog>,
    button: usize,
    pressed: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Dialog {
    Difficulty,
    Quit,
    Restart,
    Corrupt(u16),
}

impl Dialog {
    fn buttons(self) -> &'static [(i32, u8)] {
        match self {
            Self::Difficulty => &[(158, 121), (274, 122), (392, 123)],
            Self::Quit | Self::Restart | Self::Corrupt(_) => &[(188, 29), (361, 14)],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Stay,
    Start(i64),
    Quit,
    Options,
    Replay,
    Resume,
    Save,
    Load,
    DeleteCorrupt(u16),
}

impl Choice {
    /// Rule 16: while Enter is held on a popup, the pointer sits sixteen
    /// layout pixels below the selected button's centre.
    pub fn key_pointer(&self) -> Option<[i32; 2]> {
        if !self.pressed {
            return None;
        }
        let (left, _) = self.dialog?.buttons()[self.button];
        Some([left + 44, 284])
    }
    pub fn in_game() -> Self {
        Self {
            game: true,
            row: 6,
            ..Self::default()
        }
    }

    pub fn has_game(&self)->bool {self.game}

    fn available(&self) -> &'static [usize] {
        match (
            self.game,
            self.saves && !matches!(self.dialog, Some(Dialog::Corrupt(_))),
        ) {
            (true, true) => &ACTIVE_DURING_GAME_WITH_SAVES,
            (true, false) => &ACTIVE_DURING_GAME,
            (false, true) => &ACTIVE_WITH_SAVES,
            (false, false) => &ACTIVE,
        }
    }
    pub fn set_saves(&mut self, available: bool) {
        self.saves = available;
        if !available && self.row == 1 {
            self.row = 0;
        }
    }

    pub fn ask_corrupt(&mut self, slot: u16) {
        self.dialog = Some(Dialog::Corrupt(slot));
        self.button = 1; // No
        self.pressed = false;
    }
    /// Rule 15 and rules 18–21. Highlight movement clicks even at the
    /// edge of a popup; selecting a list item plays the selection sound.
    pub fn sound(&self, key: Key, rows: &[[i32; 4]; 7]) -> Option<&'static str> {
        match key {
            Key::Up | Key::Down => Some("click.aud"),
            Key::Left | Key::Right if self.dialog.is_some() => Some("click.aud"),
            Key::Enter if self.dialog.is_none() => Some("intrface.aud"),
            Key::Click(x, y)
                if self.dialog.is_none()
                    && self.available().iter().any(|&i| {
                        let [l, t, r, b] = rows[i];
                        (l..r).contains(&x) && (t..b).contains(&y)
                    }) =>
            {
                Some("intrface.aud")
            }
            _ => None,
        }
    }

    fn select(&mut self) -> Action {
        let popup = match self.row {
            0 if self.game => Dialog::Restart,
            0 => Dialog::Difficulty,
            5 => Dialog::Quit,
            3 => return Action::Options,
            4 => return Action::Replay,
            6 => return Action::Resume,
            2 => return Action::Save,
            1 => return Action::Load,
            _ => return Action::Stay,
        };
        self.dialog = Some(popup);
        self.button = 1;
        self.pressed = false;
        Action::Stay
    }

    fn answer(&mut self) -> Action {
        self.pressed = false;
        match self.dialog.take() {
            Some(Dialog::Difficulty) => Action::Start(self.button as i64 + 1),
            Some(Dialog::Quit) if self.button == 0 => Action::Quit,
            Some(Dialog::Corrupt(slot)) if self.button == 0 => Action::DeleteCorrupt(slot),
            Some(Dialog::Restart) if self.button == 0 => {
                self.dialog = Some(Dialog::Difficulty);
                self.button = 1;
                Action::Stay
            }
            _ => Action::Stay,
        }
    }

    pub fn input(&mut self, key: Key, rows: &[[i32; 4]; 7]) -> Action {
        if let Key::Up | Key::Down = key {
            let active = self.available();
            let pos = active.iter().position(|n| *n == self.row).unwrap_or(0);
            self.row = active[(pos
                + if key == Key::Down {
                    1
                } else {
                    active.len() - 1
                })
                % active.len()];
        } else if let Some(dialog) = self.dialog {
            match key {
                Key::Enter => self.pressed = true,
                Key::Release if self.pressed => return self.answer(),
                Key::Left => self.button = self.button.saturating_sub(1),
                Key::Right => self.button = (self.button + 1).min(dialog.buttons().len() - 1),
                Key::Escape if dialog == Dialog::Difficulty => {
                    self.dialog = None;
                    self.pressed = false;
                }
                Key::Escape if self.game => return Action::Resume,
                Key::Click(x, y) if (250..286).contains(&y) => {
                    if let Some(i) = dialog
                        .buttons()
                        .iter()
                        .position(|(left, _)| (*left..*left + 88).contains(&x))
                    {
                        self.button = i;
                        return self.answer();
                    }
                }
                _ => {}
            }
        } else {
            match key {
                Key::Enter => return self.select(),
                Key::Escape if self.game => return Action::Resume,
                Key::Click(x, y) => {
                    if let Some(&row) = self.available().iter().find(|&&i| {
                        let [l, t, r, b] = rows[i];
                        (l..r).contains(&x) && (t..b).contains(&y)
                    }) {
                        self.row = row;
                        return self.select();
                    }
                }
                _ => {}
            }
        }
        Action::Stay
    }
}

pub struct Art {
    pub(crate) palette: [kq8_data::palette::Rgb; 256],
    source_palette: [kq8_data::palette::Rgb; 256],
    layout: Control,
    pub rows: [[i32; 4]; 7],
    pictures: BTreeMap<&'static str, Vec<Bitmap>>,
    heading: Font,
    item: Font,
    small: Font,
    titles: [String; 2],
    texts: BTreeMap<u8, String>,
}

impl Art {
    pub fn open(path: &Path) -> Result<Self, String> {
        let game = GameDir::open(path).map_err(|e| e.to_string())?;
        let read = |name: &str| {
            let path = game.find(name).ok_or_else(|| format!("missing {name}"))?;
            std::fs::read(path).map_err(|e| format!("{name}: {e}"))
        };
        let gui = |name: &str| read(&format!("GAME/8Gui/{name}"));
        let layout = gui::parse(&read("GAME/Resource/Mainmenu.gui")?)?;
        let mut rows = [[0; 4]; 7];
        for (i, (id, _)) in ITEMS.iter().enumerate() {
            rows[i] = layout
                .find(*id)
                .ok_or_else(|| format!("menu control {id} missing"))?
                .rect;
        }
        for id in [1114, 1115, 1118, 146] {
            if layout.find(id).is_none() {
                return Err(format!("menu control {id} missing"));
            }
        }
        let mut pictures = BTreeMap::new();
        for name in [
            "main18.pbm",
            "45-line.pbm",
            "mskcur.pba",
            "gm_win2.pbm",
            "g_ybtn.pba",
            "g_yhibtn.pba",
            "cursgui.pba",
        ] {
            let bytes = gui(name)?;
            let frames = if name.ends_with(".pbm") {
                vec![Bitmap::parse(&bytes)?]
            } else {
                bitmap_array(&bytes)?
            };
            if frames.is_empty() || frames.iter().any(|b| b.bit_count != 8) {
                return Err(format!("invalid menu image {name}"));
            }
            pictures.insert(name, frames);
        }
        let titles = read("starttxt.txt")?;
        let title_text = String::from_utf8_lossy(&titles);
        let mut lines = title_text.lines();
        let titles = [
            lines.next().unwrap_or("").trim().into(),
            lines.next().unwrap_or("").trim().into(),
        ];
        let msgs = MessageFile::parse(500, &read("GAME/English/500.msg")?)?;
        let mut texts = BTreeMap::new();
        for line in msgs.lines {
            if line.tuple.noun == 0 && line.tuple.verb == 0 {
                texts.entry(line.tuple.case).or_insert(line.raw);
            }
        }
        let source_palette = kq8_data::palette::as_shown(Palette::parse(&gui("menus.ppl")?)?.colors);
        Ok(Self {
            rows,
            layout,
            pictures,
            titles,
            texts,
            palette: source_palette,
            source_palette,
            heading: Font::parse(&gui("45.pft")?)?,
            item: Font::parse(&gui("27.pft")?)?,
            small: Font::parse(&gui("consoleg.pft")?)?,
        })
    }

    /// Rebuild the palette stage from retained source colours.
    pub fn with_brightness(mut self, value: u8) -> Self {
        self.set_palette(value,kq8_data::palette::Output::Direct3D);
        self
    }

    pub fn set_palette(&mut self, value: u8, output: kq8_data::palette::Output) {
        self.palette = kq8_data::palette::for_output(self.source_palette,value,output);
    }

    pub(crate) fn label(&self, id: u8) -> &str {
        self.texts.get(&id).map_or("", |s| s.trim())
    }

    pub fn cursor(&self) -> Canvas {
        let bitmap = &self.pictures["cursgui.pba"][0];
        let mut canvas = Canvas::new(bitmap.width, bitmap.height);
        if let Some(pic) = Pic::of(bitmap) {
            canvas.picture(
                [0, 0, bitmap.width as i32, bitmap.height as i32],
                pic,
                false,
                1.0,
                &self.palette,
            );
        }
        canvas
    }
    /// The original movies fill the 4:3 menu surface. A missing local
    /// decoder has no original menu asset; report it with the menu font.
    pub fn movie_screen(&self, failure: Option<&str>) -> Canvas {
        let mut out = Canvas::new(640, 480);
        out.fill([0, 0, 640, 480], [0, 0, 0], 1.);
        if let Some(name) = failure {
            for (y, line) in [
                (165, format!("Could not play {name}")),
                (213, "Check the movie file and FFmpeg decoder.".to_owned()),
                (263, "Press Enter to continue.".to_owned()),
            ] {
                out.text(
                    [10, y, 630, y + 40],
                    &self.item,
                    0x0c,
                    &line,
                    1.,
                    &self.palette,
                );
            }
        }
        out
    }

    pub fn render(&self, state: &Choice) -> Canvas {
        let mut canvas = Canvas::new(640, 480);
        let draw = |canvas: &mut Canvas, file: &str, rect: [i32; 4], frame: usize| {
            let frames = &self.pictures[file];
            if let Some(pic) = Pic::of(&frames[frame.min(frames.len() - 1)]) {
                canvas.picture(rect, pic, true, 1.0, &self.palette);
            }
        };
        draw(&mut canvas, "main18.pbm", [0, 0, 640, 480], 0);
        let control = |id| self.layout.find(id).unwrap();
        let align = |c: &Control| match &c.kind {
            Kind::Text { text, .. } => text.justification,
            _ => 12,
        };
        draw(&mut canvas, "45-line.pbm", control(1118).rect, 0);
        for (i, id) in [1114, 1115].into_iter().enumerate() {
            canvas.text(
                control(id).rect,
                &self.heading,
                align(control(id)),
                &self.titles[i],
                1.0,
                &self.palette,
            );
        }
        for (i, &(id, label)) in ITEMS.iter().enumerate() {
            let rect = self.rows[i];
            if state.available().contains(&i) {
                canvas.text(
                    rect,
                    &self.item,
                    align(control(id)),
                    self.label(label),
                    1.0,
                    &self.palette,
                );
            } else {
                // Dither the glyphs over the existing background, as in rule 3.
                let mut glyphs = Canvas::new(640, 480);
                glyphs.text(
                    rect,
                    &self.item,
                    align(control(id)),
                    self.label(label),
                    1.0,
                    &self.palette,
                );
                for y in rect[1].clamp(0, 480)..rect[3].clamp(0, 480) {
                    for x in rect[0].clamp(0, 640)..rect[2].clamp(0, 640) {
                        let p = ((640 * y + x) * 4) as usize;
                        if (x + y) % 2 == 0 && glyphs.rgba[p + 3] != 0 {
                            canvas.rgba[p..p + 4].copy_from_slice(&glyphs.rgba[p..p + 4]);
                        }
                    }
                }
            }
        }
        let row = self.rows[state.row];
        let cursor = control(146).rect;
        draw(
            &mut canvas,
            "mskcur.pba",
            [cursor[0], row[1], cursor[2], row[3]],
            0,
        );
        if let Some(dialog) = state.dialog {
            draw(&mut canvas, "gm_win2.pbm", [140, 180, 500, 300], 0);
            let text = self.label(match dialog {
                Dialog::Quit => 159,
                Dialog::Difficulty => 120,
                Dialog::Restart => 158,
                Dialog::Corrupt(_) => 157,
            });
            let split = (text.chars().count() > 38)
                .then(|| {
                    text.match_indices(' ')
                        .min_by_key(|(i, _)| (i * 2).abs_diff(text.len()))
                })
                .flatten();
            if let Some((at, _)) = split {
                canvas.text(
                    [170, 200, 470, 220],
                    &self.small,
                    12,
                    &text[..at],
                    1.0,
                    &self.palette,
                );
                canvas.text(
                    [170, 220, 470, 240],
                    &self.small,
                    12,
                    &text[at + 1..],
                    1.0,
                    &self.palette,
                );
            } else {
                canvas.text(
                    [170, 215, 470, 235],
                    &self.small,
                    12,
                    text,
                    1.0,
                    &self.palette,
                );
            }
            for (i, &(x, label)) in dialog.buttons().iter().enumerate() {
                let selected = state.button == i;
                let delta = i32::from(selected && state.pressed);
                draw(
                    &mut canvas,
                    if selected {
                        "g_yhibtn.pba"
                    } else {
                        "g_ybtn.pba"
                    },
                    [x, 250, x + 88, 286],
                    delta as usize,
                );
                canvas.text(
                    [x + delta, 250 + delta, x + 88 + delta, 286 + delta],
                    &self.small,
                    12,
                    self.label(label),
                    1.0,
                    &self.palette,
                );
            }
        }
        canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_slot_blocks_load_until_the_default_no_or_explicit_delete() {
        let rows = [[0; 4]; 7];
        let mut menu = Choice::default();
        menu.set_saves(true);
        menu.ask_corrupt(4);
        assert!(!menu.available().contains(&1));
        for key in [Key::Enter, Key::Release] {
            assert_eq!(menu.input(key, &rows), Action::Stay);
        }
        assert!(menu.available().contains(&1));
        menu.ask_corrupt(4);
        assert_eq!(
            menu.input(Key::Click(210, 265), &rows),
            Action::DeleteCorrupt(4)
        );
        menu.set_saves(false);
        assert!(!menu.available().contains(&1));
    }
    #[test]
    fn in_game_escape_resumes_and_restarting_requires_confirmation_then_difficulty() {
        let rows = [[0; 4]; 7];
        let mut menu = Choice::in_game();
        assert_eq!(menu.input(Key::Enter, &rows), Action::Resume);
        for key in [Key::Down, Key::Enter, Key::Enter, Key::Release] {
            assert_eq!(menu.input(key, &rows), Action::Stay);
        }
        assert_eq!(menu.dialog, None, "No preserves the running game");
        for key in [Key::Enter, Key::Left, Key::Enter, Key::Release] {
            assert_eq!(menu.input(key, &rows), Action::Stay);
        }
        assert_eq!(menu.dialog, Some(Dialog::Difficulty));
        assert_eq!(menu.input(Key::Escape, &rows), Action::Stay);
        assert_eq!(menu.input(Key::Escape, &rows), Action::Resume);
    }
    #[test]
    fn new_defaults_to_normal_and_opener_release_does_not_start() {
        let mut menu = Choice::default();
        let rows = [[0; 4]; 7];
        for key in [Key::Enter, Key::Release] {
            assert_eq!(menu.input(key, &rows), Action::Stay);
        }
        assert_eq!(menu.dialog, Some(Dialog::Difficulty));
        assert_eq!(menu.input(Key::Enter, &rows), Action::Stay);
        assert_eq!(menu.input(Key::Release, &rows), Action::Start(2));
    }

    #[test]
    fn quit_defaults_to_no_and_difficulty_can_cancel_or_pick_by_mouse() {
        let mut menu = Choice::default();
        let rows = [[0; 4]; 7];
        for key in [Key::Up, Key::Enter, Key::Enter, Key::Release] {
            assert_eq!(menu.input(key, &rows), Action::Stay);
        }
        assert_eq!(menu.row, 5);
        assert_eq!(menu.dialog, None);
        for key in [Key::Down, Key::Enter, Key::Escape, Key::Enter] {
            assert_eq!(menu.input(key, &rows), Action::Stay);
        }
        assert_eq!(menu.input(Key::Click(400, 255), &rows), Action::Start(3));
    }

    #[test]
    #[ignore = "requires the original GOG menu font and palette"]
    fn movie_failure_is_visible_in_the_original_menu_font() {
        let game = std::env::var_os("KQ8_GAME_DIR").unwrap();
        let art = Art::open(std::path::Path::new(&game)).unwrap();
        let picture = art.movie_screen(Some("W32opn_1.dll"));
        for y in [165, 213, 263] {
            let band = &picture.rgba[y * 640 * 4..(y + 40) * 640 * 4];
            assert!(band.chunks_exact(4).filter(|p| p[..3].iter().any(|v| *v > 20)).count() > 200);
        }
        assert!(art.movie_screen(None).rgba.chunks_exact(4).all(|p| p == [0, 0, 0, 255]));
    }

    #[test]
    #[ignore = "requires the original assets in KQ8_GAME_DIR"]
    fn installed_menu_draws_labels_and_difficulty_buttons() {
        let game = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let art = Art::open(std::path::Path::new(&game)).unwrap();
        for (i, (_, case)) in ITEMS.into_iter().enumerate() {
            assert!(!art.label(case).is_empty());
            assert_eq!(art.rows[i][0], 64);
        }
        let mut selection = Choice::default();
        let menu = art.render(&selection);
        assert_eq!(
            selection.input(Key::Click(80, 130), &art.rows),
            Action::Stay
        );
        let prompt = art.render(&selection);
        assert!(
            menu.rgba != prompt.rgba,
            "opening the difficulty popup must change the image"
        );
        assert!(
            prompt.rgba == art.render(&selection).rgba,
            "the same menu state must render identically"
        );
        assert_eq!(
            selection.input(Key::Click(300, 265), &art.rows),
            Action::Start(2)
        );
    }
}
