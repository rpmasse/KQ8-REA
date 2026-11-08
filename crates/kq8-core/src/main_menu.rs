//! Startup screen, New Game difficulty and Quit (docs/behaviour/main-menu.md).
use crate::gui_draw::{self, Canvas, Pic};
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
    size: [u32; 2],
    rects: BTreeMap<u32, [i32; 4]>,
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
        for id in [1114, 1115, 1118, 146, 1004, 1006, 1007] {
            if layout.find(id).is_none() {
                return Err(format!("menu control {id} missing"));
            }
        }
        let mut pictures = BTreeMap::new();
        for name in [
            "main18.pbm",
            "main6x8.pbm",
            "45-line.pbm",
            "mskcur.pba",
            "gm_win2.pbm",
            "g_ybtn.pba",
            "g_yhibtn.pba",
            "cursgui.pba",
        ] {
            if matches!(name, "main18.pbm" | "main6x8.pbm") {
                let size = if name == "main18.pbm" { [640, 480] } else { [800, 600] };
                pictures.insert(name, vec![crate::gui_draw::resolution_picture(&game,
                    &["main18.pbm", "main4x6.pbm", "main6x8.pbm"], size)?]);
                continue;
            }
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
            size: [640, 480],
            rects: crate::gui_draw::absolute_rects(&crate::gui_draw::resolve_layout(&layout, [640, 480])),
            rows,
            layout,
            pictures,
            titles,
            texts,
            palette: source_palette,
            source_palette,
            heading: crate::gui_draw::gui_font(Font::parse(&gui("45.pft")?)?),
            item: crate::gui_draw::gui_font(Font::parse(&gui("27.pft")?)?),
            small: crate::gui_draw::gui_font(Font::parse(&gui("consoleg.pft")?)?),
        })
    }

    pub fn size(&self) -> [u32; 2] { self.size }

    pub fn resize(&mut self, high: bool) {
        self.size = crate::caption::screen(high);
        self.rects = crate::gui_draw::absolute_rects(&crate::gui_draw::resolve_layout(&self.layout, self.size));
        for (i, (id, _)) in ITEMS.iter().enumerate() { self.rows[i] = self.rects[id]; }
    }

    fn layout_rect(&self, rect: [i32; 4]) -> [i32; 4] {
        crate::gui_draw::resolve_rect(rect, [640, 480], self.size.map(|n| n as i32))
    }

    /// The existing main-menu dialog controller uses authored coordinates;
    /// resolve its button rectangles before dispatch, just as for its rows.
    pub fn input_key(&self, state: &Choice, key: Key) -> Key {
        let (Some(dialog), Key::Click(x, y)) = (state.dialog, key) else { return key };
        for &(left, _) in dialog.buttons() {
            let [l, t, r, b] = self.layout_rect([left, 250, left + 88, 286]);
            if (l..r).contains(&x) && (t..b).contains(&y) { return Key::Click(left + 1, 251); }
        }
        Key::Click(-1, -1)
    }

    pub fn key_pointer(&self, state: &Choice) -> Option<[i32; 2]> {
        let [x, y] = state.key_pointer()?;
        let r = self.layout_rect([x, y, x, y]);
        Some([r[0], r[1]])
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
        self.cursor_frame(0)
    }

    pub fn cursor_frames(&self) -> usize { self.pictures["cursgui.pba"].len() }

    /// Interface 6a–6c: the menu pointer shares the gameplay pointer's
    /// authored-screen scaling and keeps each 100ms frame's own flags.
    pub fn cursor_frame(&self, tick: usize) -> Canvas {
        let frames = &self.pictures["cursgui.pba"];
        let bitmap = &frames[tick % frames.len()];
        let rect = gui_draw::resolve_rect([0, 0, bitmap.width as i32, bitmap.height as i32],
            [640, 480], self.size.map(|v| v as i32));
        let mut canvas = Canvas::new(rect[2].max(1) as u32, rect[3].max(1) as u32);
        if let Some(pic) = Pic::of(bitmap) {
            canvas.picture(
                rect,
                pic,
                true,
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
        self.render_frame(state, 0)
    }

    /// The menu's selection mask loops through its installed frames.
    pub fn render_frame(&self, state: &Choice, mask_frame: usize) -> Canvas {
        let [width, height] = self.size;
        let mut canvas = Canvas::new(width, height);
        let draw = |canvas: &mut Canvas, file: &str, rect: [i32; 4], frame: usize, control: &Control| {
            let frames = &self.pictures[file];
            if let Some(pic) = Pic::of(&frames[frame.min(frames.len() - 1)]) {
                let stretch = match control.kind {
                    Kind::Dialog { picture, .. } | Kind::Text { pictures: picture, .. } => picture,
                    Kind::Bitmap(picture) => Some(picture),
                    _ => None,
                }.is_none_or(|picture| picture.modifier & 0x8000 != 0);
                canvas.picture(rect, pic, stretch, 1.0, &self.palette);
            }
        };
        draw(&mut canvas, if width == 800 { "main6x8.pbm" } else { "main18.pbm" }, [0, 0, width as i32, height as i32], 0, &self.layout);
        let control = |id| self.layout.find(id).unwrap();
        let align = |c: &Control| match &c.kind {
            Kind::Text { text, .. } => text.justification,
            _ => 12,
        };
        draw(&mut canvas, "45-line.pbm", self.rects[&1118], 0, control(1118));
        for (i, id) in [1114, 1115].into_iter().enumerate() {
            canvas.text(
                self.rects[&id],
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
                let mut glyphs = Canvas::new(width, height);
                glyphs.text(
                    rect,
                    &self.item,
                    align(control(id)),
                    self.label(label),
                    1.0,
                    &self.palette,
                );
                for y in rect[1].clamp(0, height as i32)..rect[3].clamp(0, height as i32) {
                    for x in rect[0].clamp(0, width as i32)..rect[2].clamp(0, width as i32) {
                        let p = ((width as i32 * y + x) * 4) as usize;
                        if (x + y) % 2 == 0 && glyphs.rgba[p + 3] != 0 {
                            canvas.rgba[p..p + 4].copy_from_slice(&glyphs.rgba[p..p + 4]);
                        }
                    }
                }
            }
        }
        let row = self.rows[state.row];
        let cursor = self.rects[&146];
        draw(
            &mut canvas,
            "mskcur.pba",
            [cursor[0], row[1], cursor[2], row[3]],
            mask_frame % self.pictures["mskcur.pba"].len(),
            control(146),
        );
        if let Some(dialog) = state.dialog {
            draw(&mut canvas, "gm_win2.pbm", self.rects[&1004], 0, control(1004));
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
                    self.layout_rect([170, 200, 470, 220]),
                    &self.small,
                    12,
                    &text[..at],
                    1.0,
                    &self.palette,
                );
                canvas.text(
                    self.layout_rect([170, 220, 470, 240]),
                    &self.small,
                    12,
                    &text[at + 1..],
                    1.0,
                    &self.palette,
                );
            } else {
                canvas.text(
                    self.layout_rect([170, 215, 470, 235]),
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
                    self.layout_rect([x, 250, x + 88, 286]),
                    delta as usize,
                    control(if selected { 1007 } else { 1006 }),
                );
                canvas.text(
                    self.layout_rect([x + delta, 250 + delta, x + 88 + delta, 286 + delta]),
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
    #[ignore = "requires KQ8_GAME_DIR; headless original-data modal matrix"]
    fn original_interface_modal_matrix() {
        let game = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let mut art = Art::open(Path::new(&game)).unwrap();
        let modifier = |id| match art.layout.find(id).unwrap().kind {
            Kind::Bitmap(picture) => picture.modifier,
            Kind::Text { pictures: Some(picture), .. } => picture.modifier,
            _ => panic!("modal picture missing: {id}"),
        };
        assert_eq!(modifier(1004) & 0x8000, 0, "native warning background");
        assert_ne!(modifier(1006) & 0x8000, 0, "stretched normal button");
        assert_ne!(modifier(1007) & 0x8000, 0, "stretched highlighted button");
        let background = &art.pictures["gm_win2.pbm"][0];
        assert_eq!([background.width, background.height], [360, 120]);
        for high in [false, true] {
            art.resize(high);
            let [width, _] = art.size();
            let background_rect = art.rects[&1004];
            assert_eq!(background_rect, if high { [175, 225, 625, 375] } else { [140, 180, 500, 300] });
            for dialog in [Dialog::Quit, Dialog::Difficulty] {
                for pressed in [false, true] {
                    let state = Choice { dialog: Some(dialog), button: 1, pressed, ..Choice::default() };
                    let actual = art.render(&state);
                    let mut expected = art.render(&Choice::default());
                    expected.picture(background_rect, Pic::of(&art.pictures["gm_win2.pbm"][0]).unwrap(), false, 1.0, &art.palette);
                    let equal_row = |expected: &Canvas, y: i32, left: i32, right: i32| {
                        let start = ((y * width as i32 + left) * 4) as usize;
                        let end = ((y * width as i32 + right) * 4) as usize;
                        assert_eq!(&actual.rgba[start..end], &expected.rgba[start..end],
                            "high={high} dialog={dialog:?} pressed={pressed} row={y}");
                    };
                    // The strip precedes text/buttons and includes the area beyond
                    // the native picture's right edge at 800. Stretching fails it.
                    for y in background_rect[1]..background_rect[1] + 15 {
                        equal_row(&expected, y, background_rect[0], background_rect[2]);
                    }
                    for (i, &(left, _)) in dialog.buttons().iter().enumerate() {
                        let selected = i == state.button;
                        let frames = &art.pictures[if selected { "g_yhibtn.pba" } else { "g_ybtn.pba" }];
                        let frame = usize::from(selected && pressed);
                        let rect = art.layout_rect([left, 250, left + 88, 286]);
                        assert_eq!(rect[3] - rect[1], if high { 45 } else { 36 });
                        expected.picture(rect, Pic::of(&frames[frame]).unwrap(), true, 1.0, &art.palette);
                        for y in [rect[1], rect[1] + 1, rect[3] - 2, rect[3] - 1] {
                            equal_row(&expected, y, rect[0], rect[2]);
                        }
                    }
                }
            }
            let mut state = Choice::default();
            assert_eq!(state.input(Key::Enter, &art.rows), Action::Stay);
            let [l, t, r, b] = art.layout_rect([274, 250, 362, 286]);
            let outside = art.input_key(&state, Key::Click(r, b - 1));
            assert_eq!(state.input(outside, &art.rows), Action::Stay);
            let inside = art.input_key(&state, Key::Click((l + r) / 2, (t + b) / 2));
            assert_eq!(state.input(inside, &art.rows), Action::Start(2));
        }
        println!("modal matrix: native background, stretched normal/highlight/pressed buttons, two/three-button geometry and clicks at 640/800");
    }

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
        let second = art.render_frame(&selection, 1);
        let mask = art.layout.find(146).unwrap().rect;
        let row = art.rows[selection.row];
        let mut changed = 0;
        for (i, (first, next)) in menu.rgba.chunks_exact(4).zip(second.rgba.chunks_exact(4)).enumerate() {
            if first != next {
                let width = menu.width as usize;
                let (x, y) = ((i % width) as i32, (i / width) as i32);
                assert!(x >= mask[0] && x < mask[2] && y >= row[1] && y < row[3]);
                changed += 1;
            }
        }
        assert!(changed > 0, "the installed mask must change visible pixels");
        let count = art.pictures["mskcur.pba"].len();
        assert_eq!(count, 16);
        assert!(menu.rgba == art.render_frame(&selection, count).rgba);
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
