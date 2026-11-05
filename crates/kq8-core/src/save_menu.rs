//! Savegame.gui and Loadgame.gui control the placement, pictures and fonts.
//! Menu labels and prompts come from 500.MSG (119, 125, 126, 154 and 166).
use crate::gui_draw::{text_width, Canvas};
use crate::main_menu::{Art as MenuArt, Key};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Save,
    Load,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    None,
    Back,
    Save(u16, String),
    Load(u16),
    Delete(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prompt {
    Delete,
    Overwrite,
    Name,
    Full,
    Invalid,
}

#[derive(Debug)]
pub struct Screen {
    pub mode: Mode,
    slots: Vec<(u16, String)>,
    selected: usize,
    first: usize,
    name: String,
    button: usize,
    prompt: Option<Prompt>,
    yes: bool,
    pressed: bool,
    dragging: bool,
    thumbnail: Option<Vec<u8>>,
}

const ROWS: usize = 10;

impl Screen {
    pub fn dragging(&self) -> bool {
        self.dragging
    }
    pub fn new(mode: Mode, slots: Vec<(u16, String)>) -> Self {
        let selected = if mode == Mode::Save { slots.len() } else { 0 };
        let mut screen = Self {
            mode,
            slots,
            selected: 0,
            first: 0,
            name: String::new(),
            button: 0,
            prompt: None,
            yes: false,
            pressed: false,
            dragging: false,
            thumbnail: None,
        };
        screen.select(selected);
        screen
    }

    fn row_count(&self) -> usize {
        self.slots.len() + usize::from(self.mode == Mode::Save)
    }

    pub fn selected_slot(&self) -> Option<u16> {
        self.slots.get(self.selected).map(|s| s.0)
    }

    /// The saved image uses its world's colours. Both slot screens display
    /// it in Menus.ppl (interface.md rule 1h).
    pub fn set_preview(&mut self, pixels: Option<&[u8]>, menu: &MenuArt) {
        self.thumbnail = pixels.and_then(|p| thumbnail(p, 160, 120, &menu.palette));
    }

    fn select(&mut self, index: usize) {
        self.selected = index.min(self.row_count().saturating_sub(1));
        self.first = self
            .first
            .min(self.row_count().saturating_sub(ROWS))
            .clamp(self.selected.saturating_sub(ROWS - 1), self.selected);
        self.name = self
            .slots
            .get(self.selected)
            .map(|s| s.1.clone())
            .unwrap_or_default();
    }

    pub fn after_delete(&mut self, slots: Vec<(u16, String)>) {
        self.slots = slots;
        self.button = 0;
        self.select(self.selected);
    }

    pub fn failed(&mut self) {
        self.ask(Prompt::Invalid);
    }

    fn ask(&mut self, prompt: Prompt) {
        self.prompt = Some(prompt);
        self.yes = false;
        self.pressed = false;
    }

    fn activate(&mut self) -> Command {
        let slot = self.slots.get(self.selected).map(|s| s.0);
        match (self.button, self.mode, slot) {
            (2, _, _) => return Command::Back,
            (1, _, Some(_)) => self.ask(Prompt::Delete),
            (0, Mode::Load, Some(id)) => return Command::Load(id),
            (0, Mode::Save, _) if self.name.trim().is_empty() => self.ask(Prompt::Name),
            (0, Mode::Save, Some(_)) => self.ask(Prompt::Overwrite),
            (0, Mode::Save, None) => {
                // Slot IDs are sorted. Reuse holes left by deleted saves.
                let mut free = (1..=u16::MAX).zip(self.slots.iter().map(|s| s.0));
                let next = free
                    .find(|(wanted, used)| wanted != used)
                    .map(|(id, _)| id)
                    .or_else(|| u16::try_from(self.slots.len() + 1).ok());
                if let Some(id) = next {
                    return Command::Save(id, self.name.trim().to_owned());
                }
                self.ask(Prompt::Full);
            }
            _ => {}
        }
        Command::None
    }

    fn answer(&mut self) -> Command {
        self.pressed = false;
        let prompt = self.prompt.take();
        if self.yes {
            if let Some((id, _)) = self.slots.get(self.selected) {
                match prompt {
                    Some(Prompt::Delete) => return Command::Delete(*id),
                    Some(Prompt::Overwrite) => {
                        return Command::Save(*id, self.name.trim().to_owned())
                    }
                    _ => {}
                }
            }
        }
        Command::None
    }

    /// Hardware and scripted input share this path and the layout's hit boxes.
    pub fn input(&mut self, key: Key, art: &Art) -> Command {
        let rect = |name: &str| art.layout.controls[name].rect;
        let hit = |x, y, [l, t, r, b]: [i32; 4]| (l..r).contains(&x) && (t..b).contains(&y);
        if let Some(prompt) = self.prompt {
            let confirm = matches!(prompt, Prompt::Delete | Prompt::Overwrite);
            match key {
                Key::Escape => {
                    self.prompt = None;
                    self.pressed = false;
                }
                Key::Left if confirm => self.yes = true,
                Key::Right => self.yes = false,
                Key::Tab if confirm => self.yes = !self.yes,
                Key::Enter => self.pressed = true,
                Key::Release if self.pressed => return self.answer(),
                Key::Click(x, y) => {
                    for (name, yes) in [("POPUP_BTN_CANCEL", false), ("POPUP_BTN_YES", true)] {
                        if (!yes || confirm) && hit(x, y, rect(name)) {
                            self.yes = yes;
                            return self.answer();
                        }
                    }
                }
                _ => {}
            }
            return Command::None;
        }

        match key {
            Key::Escape => return Command::Back,
            Key::Enter => self.pressed = true,
            Key::Release if self.pressed => {
                self.pressed = false;
                return self.activate();
            }
            Key::Delete if self.selected < self.slots.len() => self.ask(Prompt::Delete),
            Key::Backspace if self.mode == Mode::Save => {
                self.name.pop();
            }
            Key::Character(c) if self.mode == Mode::Save && !c.is_control() && c as u32 <= 255 => {
                let mut next = self.name.clone();
                next.push(c);
                let name_field = &art.layout.controls["NAME"];
                if next.chars().count() <= 128
                    && text_width(&art.layout.fonts[&name_field.text.unwrap().font], &next)
                        < name_field.rect[2] - name_field.rect[0]
                {
                    self.name = next;
                }
            }
            Key::Up => self.select(self.selected.saturating_sub(1)),
            Key::Down => self.select(self.selected + 1),
            Key::Left => self.button = (self.button + 2) % 3,
            Key::Tab | Key::Right => self.button = (self.button + 1) % 3,
            Key::Click(x, y) => {
                for (i, name) in ["OK", "DELETE", "CANCEL"].iter().enumerate() {
                    if hit(x, y, rect(name)) {
                        self.button = i;
                        return self.activate();
                    }
                }
                if hit(x, y, rect("UP")) {
                    self.select(self.selected.saturating_sub(1));
                } else if hit(x, y, rect("DOWN")) {
                    self.select(self.selected + 1);
                } else if hit(x, y, art.track()) {
                    self.dragging = true;
                    self.scroll_to(y, art);
                } else {
                    for i in 0..ROWS {
                        if hit(x, y, rect(&format!("LIST{i}"))) && self.first + i < self.row_count()
                        {
                            self.select(self.first + i);
                            self.button = 0;
                            break;
                        }
                    }
                }
            }
            Key::Drag(_, y) if self.dragging => self.scroll_to(y, art),
            Key::MouseUp => self.dragging = false,
            _ => {}
        }
        Command::None
    }

    fn scroll_to(&mut self, y: i32, art: &Art) {
        let max = self.row_count().saturating_sub(ROWS);
        let [_, top, _, bottom] = art.track();
        let tab = art.layout.controls["TAB"].rect;
        let height = tab[3] - tab[1];
        let travel = (bottom - top - height).max(1) as usize;
        let offset = (y - top - height / 2).clamp(0, travel as i32) as usize;
        self.first = (offset * max + travel / 2) / travel;
        self.select(self.selected.clamp(self.first, self.first + ROWS - 1));
    }
}

pub struct Art {
    layout: crate::menu_layout::Layout,
}

impl Art {
    pub fn open(path: &Path, mode: Mode) -> Result<Self, String> {
        let (file, prefix) = match mode {
            Mode::Save => ("Savegame.gui", "IDCTL_SAVE_"),
            Mode::Load => ("Loadgame.gui", "IDCTL_LOAD_"),
        };
        let art = Self {
            layout: crate::menu_layout::Layout::read(path, file, prefix)?,
        };
        // All dynamic control references must exist before accepting the screen.
        for key in [
            "BACKGROUND",
            "TITLE",
            "NAME",
            "OK",
            "DELETE",
            "CANCEL",
            "THUMBNAIL",
            "HILITE",
            "UP",
            "DOWN",
            "TAB",
            "POPUP_BKGND",
            "POPUP_TEXT1",
            "POPUP_TEXT2",
            "POPUP_BTN_YES",
            "POPUP_BTN_CANCEL",
        ] {
            if !art.layout.controls.contains_key(key) {
                return Err(format!("{file} lacks {key}"));
            }
        }
        for i in 0..ROWS {
            if !art.layout.controls.contains_key(&format!("LIST{i}")) {
                return Err(format!("{file} lacks row {i}"));
            }
        }
        if art.layout.controls["NAME"]
            .text
            .is_none_or(|t| !art.layout.fonts.contains_key(&t.font))
        {
            return Err(format!("{file} lacks name font"));
        }
        Ok(art)
    }

    fn track(&self) -> [i32; 4] {
        let [l, t, r, _] = self.layout.controls["TAB"].rect;
        [l, t, r, self.layout.controls["DOWN"].rect[1]]
    }

    pub fn render(&self, state: &Screen, menu: &MenuArt) -> Canvas {
        let mut canvas = Canvas::new(640, 480);
        let draw = |canvas: &mut Canvas,
                    key: &str,
                    picture: &str,
                    dy: i32,
                    frame: usize,
                    caption: Option<&str>| {
            let [l, t, r, b] = self.layout.controls[key].rect;
            self.layout.draw(
                canvas,
                menu,
                key,
                picture,
                [l, t + dy, r, b + dy],
                frame,
                caption,
            );
        };
        let normal = |canvas: &mut Canvas, key: &str| draw(canvas, key, key, 0, 0, None);
        for key in ["BACKGROUND", "TITLE", "THUMBNAIL", "UP", "DOWN"] {
            normal(&mut canvas, key);
        }
        if let Some(rgba) = state
            .thumbnail
            .as_deref()
            .filter(|p| p.len() == 160 * 120 * 4)
        {
            let [x, y, right, bottom] = self.layout.controls["THUMBNAIL"].rect;
            if x >= 0
                && y >= 0
                && right <= 640
                && bottom <= 480
                && right - x == 160
                && bottom - y == 120
            {
                for (i, row) in rgba.chunks_exact(160 * 4).enumerate() {
                    let start = ((y as usize + i) * 640 + x as usize) * 4;
                    canvas.rgba[start..start + row.len()].copy_from_slice(row);
                }
            }
        }
        if self.layout.controls.contains_key("BAR") {
            normal(&mut canvas, "BAR");
        }
        let max = state.row_count().saturating_sub(ROWS);
        let tab = self.layout.controls["TAB"].rect;
        let travel = self.track()[3] - tab[3];
        let tab_offset = if max == 0 {
            0
        } else {
            (state.first as i32 * travel) / max as i32
        };
        draw(&mut canvas, "TAB", "TAB", tab_offset, 0, None);
        for i in 0..ROWS {
            let index = state.first + i;
            if index >= state.row_count() {
                break;
            }
            let key = format!("LIST{i}");
            if index == state.selected {
                let offset =
                    self.layout.controls[&key].rect[1] - self.layout.controls["HILITE"].rect[1];
                draw(&mut canvas, "HILITE", "HILITE", offset, 0, None);
            }
            let label = state.slots.get(index).map_or("", |s| s.1.as_str());
            draw(&mut canvas, &key, &key, 0, 0, Some(label));
        }
        draw(&mut canvas, "NAME", "NAME", 0, 0, Some(&state.name));
        for (i, key) in ["OK", "DELETE", "CANCEL"].iter().enumerate() {
            let selected = state.button == i;
            draw(
                &mut canvas,
                key,
                if selected { "OK" } else { "CANCEL" },
                0,
                usize::from(selected && state.pressed && state.prompt.is_none()),
                None,
            );
        }
        if let Some(prompt) = state.prompt {
            normal(&mut canvas, "POPUP_BKGND");
            let id = match prompt {
                Prompt::Delete => 119,
                Prompt::Overwrite => 125,
                Prompt::Name => 154,
                Prompt::Full => 166,
                Prompt::Invalid => 126,
            };
            let question = menu.label(id);
            let confirm = matches!(prompt, Prompt::Delete | Prompt::Overwrite);
            let name = state.slots.get(state.selected).map_or("", |s| s.1.as_str());
            let lines: Vec<_> = question.lines().collect();
            let text2 = if confirm {
                format!("{} {}", lines.get(1).unwrap_or(&""), name)
            } else {
                String::new()
            };
            let mut text_keys = ["POPUP_TEXT1", "POPUP_TEXT2"];
            text_keys.sort_by_key(|k| self.layout.controls[*k].rect[1]);
            let first = lines.first().copied().unwrap_or("");
            // The name prompt uses a single message line longer than the popup.
            // Fit the line to its two supplied text controls at a word boundary.
            let c = &self.layout.controls[text_keys[0]];
            let font = &self.layout.fonts[&c.text.unwrap().font];
            let split = first
                .match_indices(' ')
                .filter(|(i, _)| text_width(font, &first[..*i]) <= c.rect[2] - c.rect[0])
                .next_back();
            let (top, bottom) =
                if text_width(font, first) > c.rect[2] - c.rect[0] && lines.len() == 1 {
                    split.map_or((first, ""), |(i, _)| (&first[..i], &first[i + 1..]))
                } else {
                    (first, text2.as_str())
                };
            draw(&mut canvas, text_keys[0], text_keys[0], 0, 0, Some(top));
            draw(&mut canvas, text_keys[1], text_keys[1], 0, 0, Some(bottom));
            for (k, yes) in [("POPUP_BTN_YES", true), ("POPUP_BTN_CANCEL", false)] {
                if yes && !confirm {
                    continue;
                }
                let active = yes == state.yes;
                draw(
                    &mut canvas,
                    k,
                    if active {
                        "POPUP_BTN_CANCEL"
                    } else {
                        "POPUP_BTN_YES"
                    },
                    0,
                    usize::from(active && state.pressed),
                    (!confirm).then(|| menu.label(148)),
                );
            }
        }
        canvas
    }
}

/// Rule 22: sample the gameplay screen at 160 by 120, then choose the world
/// palette colour with the smallest sum of channel differences. Slots store
/// the displayed RGBA values so previews survive a change of world palette.
pub fn thumbnail(
    rgba: &[u8],
    width: usize,
    height: usize,
    palette: &[[u8; 3]; 256],
) -> Option<Vec<u8>> {
    if width == 0 || height == 0 || width.checked_mul(height)?.checked_mul(4)? != rgba.len() {
        return None;
    }
    let mut colors = std::collections::HashMap::new();
    let mut out = Vec::with_capacity(160 * 120 * 4);
    for y in 0..120 {
        for x in 0..160 {
            let i = ((y * height / 120) * width + x * width / 160) * 4;
            let pixel = [rgba[i], rgba[i + 1], rgba[i + 2]];
            let matched = colors.entry(pixel).or_insert_with(|| {
                *palette
                    .iter()
                    .min_by_key(|c| {
                        (0..3)
                            .map(|ch| c[ch].abs_diff(pixel[ch]) as u16)
                            .sum::<u16>()
                    })
                    .unwrap()
            });
            out.extend_from_slice(matched);
            out.push(255);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_samples_nearest_pixel_and_minimum_palette_distance() {
        let mut pal = [[0; 3]; 256];
        pal[1] = [248, 32, 40];
        pal[2] = [8, 240, 20];
        let mut source = vec![255; 320 * 240 * 4];
        for y in (0..240).step_by(2) {
            for x in (0..320).step_by(2) {
                let i = (320 * y + x) * 4;
                source[i..i + 3].copy_from_slice(if x < 160 {
                    &[239, 31, 42]
                } else {
                    &[17, 243, 26]
                });
            }
        }
        let pixels = thumbnail(&source, 320, 240, &pal).unwrap();
        for row in pixels.chunks_exact(160 * 4) {
            assert_eq!(&row[..4], &[248, 32, 40, 255]);
            assert_eq!(&row[80 * 4..81 * 4], &[8, 240, 20, 255]);
        }
        assert!(thumbnail(&source, 0, 240, &pal).is_none());
    }

    #[test]
    #[ignore = "requires original assets in KQ8_GAME_DIR"]
    fn installed_save_and_load_screens_draw_and_edit_slots() {
        let game = std::env::var_os("KQ8_GAME_DIR").unwrap();
        let path = std::path::Path::new(&game);
        let menu = MenuArt::open(path).unwrap();
        let entries: Vec<_> = (1..=13)
            .filter(|&i| i != 3)
            .map(|i| (i, format!("Slot {i}")))
            .collect();
        let art = Art::open(path, Mode::Save).unwrap();
        let mut page = Screen::new(Mode::Save, entries.clone());
        assert_eq!(page.selected_slot(), None);
        let blank = art.render(&page, &menu);
        for k in [
            Key::Click(445, 210),
            Key::Escape,
            Key::Character('D'),
            Key::Character('x'),
            Key::Backspace,
            Key::Character('v'),
        ] {
            assert_eq!(page.input(k, &art), Command::None);
        }
        let typed = art.render(&page, &menu);
        assert!(blank.rgba != typed.rgba);
        assert_eq!(
            page.input(Key::Click(445, 210), &art),
            Command::Save(3, "Dv".into())
        );
        page.input(Key::Up, &art);
        assert_eq!(page.selected_slot(), Some(13));
        page.input(Key::Click(445, 210), &art);
        assert_eq!(page.prompt, Some(Prompt::Overwrite));
        assert_eq!(page.input(Key::Click(400, 200), &art), Command::None);
        page.input(Key::Click(445, 210), &art);
        assert_eq!(
            page.input(Key::Click(220, 200), &art),
            Command::Save(13, "Slot 13".into())
        );

        let art = Art::open(path, Mode::Load).unwrap();
        let mut page = Screen::new(Mode::Load, entries);
        assert_eq!(page.selected_slot(), Some(1));
        page.input(Key::Click(70, 407), &art);
        assert_eq!(page.first, 2);
        page.input(Key::Drag(70, 205), &art);
        assert_eq!(page.first, 0);
        page.input(Key::MouseUp, &art);
        page.input(Key::Click(110, 210), &art);
        assert_eq!(page.selected_slot(), Some(2));
        let pixels = vec![77; 160 * 120 * 4];
        page.set_preview(Some(&pixels), &menu);
        let shown = page.thumbnail.as_ref().unwrap()[..4].to_vec();
        assert_eq!(shown[3], 255);
        let thumb = art.render(&page, &menu);
        assert_eq!(thumb.at(428, 316).as_slice(), shown);
        assert_eq!(thumb.at(587, 435).as_slice(), shown);
        page.input(Key::Click(450, 250), &art);
        let prompt = art.render(&page, &menu);
        assert!(prompt.rgba != thumb.rgba);
        assert_eq!(page.input(Key::Click(212, 213), &art), Command::Delete(2));
        assert_eq!(page.input(Key::Click(445, 210), &art), Command::Load(2));
    }
}
