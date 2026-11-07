//! Menus use the GUI file for rectangles and tagged pictures, fonts and text.
use crate::gui_draw::{text_width, Canvas, Pic};
use crate::main_menu::Art as MenuArt;
use kq8_data::font::{bitmap_array, Font};
use kq8_data::gui::{self, Kind, Tags, Text};
use kq8_data::{Bitmap, GameDir};
use std::{collections::BTreeMap, path::Path};

pub(crate) struct Control {
    id: u32,
    pub rect: [i32; 4],
    pic: Option<gui::Picture>,
    pub text: Option<Text>,
    label: Option<u8>,
}

pub struct Layout {
    authored: gui::Control,
    pub(crate) size: [u32; 2],
    pub(crate) controls: BTreeMap<String, Control>,
    pictures: BTreeMap<i32, Vec<Bitmap>>,
    pub(crate) fonts: BTreeMap<i32, Font>,
    backgrounds: Option<(i32, [Bitmap; 2])>,
}

impl Layout {
    pub(crate) fn read(path: &Path, file: &str, prefix: &str) -> Result<Self, String> {
        let game = GameDir::open(path).map_err(|e| e.to_string())?;
        let read = |name: &str| {
            let path = game.find(name).ok_or_else(|| format!("missing {name}"))?;
            std::fs::read(path).map_err(|e| format!("{name}: {e}"))
        };
        let layout = gui::parse(&read(&format!("GAME/Resource/{file}"))?)?;
        let (tags, _) = Tags::parse(&String::from_utf8_lossy(&read(
            "GAME/Resource/kqGuiTag.TTAG",
        )?));
        let tagged = |id| {
            tags.string(id)
                .ok_or_else(|| format!("{file}: missing resource tag {id}"))
        };
        let mut controls = BTreeMap::new();
        let mut pictures = BTreeMap::new();
        let mut fonts = BTreeMap::new();
        let mut backgrounds = None;
        for (depth, c) in layout.walk() {
            let name = if depth == 0 {
                "BACKGROUND"
            } else {
                tags.name(c.id as i32)
                    .and_then(|s| match (prefix, s) {
                        // This one name is misspelled in the shipped TTAG.
                        ("IDCTL_MOVEKEYS_", "IDCTL_MOVRKEYS_CURS2") => Some("CURS2"),
                        _ => s.strip_prefix(prefix),
                    })
                    .ok_or_else(|| format!("{file}: missing control tag {}", c.id))?
            };
            let (pic, text) = match c.kind {
                Kind::Dialog { picture, .. } => (picture, None),
                Kind::Bitmap(pic) => (Some(pic), None),
                Kind::Text { text, pictures } => (pictures, Some(text)),
                _ => return Err(format!("{file}: unexpected control type for {name}")),
            };
            if let Some(pic) = pic {
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    pictures.entry(pic.bitmap)
                {
                    let source = tagged(pic.bitmap)?;
                    if source.eq_ignore_ascii_case("saveload.pbm") {
                        backgrounds = Some((pic.bitmap, [
                            crate::gui_draw::resolution_picture(&game, &["saveload.pbm", "save4x6.pbm", "save6x8.pbm"], [640, 480])?,
                            crate::gui_draw::resolution_picture(&game, &["saveload.pbm", "save4x6.pbm", "save6x8.pbm"], [800, 600])?,
                        ]));
                    }
                    let bytes = read(&format!("GAME/8Gui/{source}"))?;
                    let frames = if source.to_ascii_lowercase().ends_with(".pba") {
                        bitmap_array(&bytes)?
                    } else {
                        vec![Bitmap::parse(&bytes)?]
                    };
                    if frames.is_empty() || frames.iter().any(|b| b.bit_count != 8) {
                        return Err(format!("invalid menu image {source}"));
                    }
                    entry.insert(frames);
                }
            }
            if let Some(t) = text.filter(|t| t.font != 0) {
                if let std::collections::btree_map::Entry::Vacant(entry) = fonts.entry(t.font) {
                    entry.insert(crate::gui_draw::gui_font(Font::parse(&read(&format!(
                        "GAME/8Gui/{}",
                        tagged(t.font)?
                    ))?)?));
                }
            }
            let label = text
                .and_then(|t| tags.string(t.text))
                .and_then(|s| s.strip_prefix('#'))
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok());
            controls.insert(
                name.to_owned(),
                Control {
                    id: c.id,
                    rect: c.rect,
                    pic,
                    text,
                    label,
                },
            );
        }
        let mut result = Self {
            authored: layout,
            size: [640, 480],
            controls,
            pictures,
            fonts,
            backgrounds,
        };
        result.resize([640, 480], false);
        Ok(result)
    }

    pub(crate) fn resize(&mut self, size: [u32; 2], save_load: bool) {
        let converted = crate::gui_draw::absolute_rects(&crate::gui_draw::resolve_layout(&self.authored, size));
        let original = crate::gui_draw::absolute_rects(&crate::gui_draw::resolve_layout(&self.authored, [640, 480]));
        let offset = [(size[0] as i32 - 640) / 2, (size[1] as i32 - 480) / 2];
        for (name, c) in &mut self.controls {
            // Q207: title/popup controls use normal resizing. Only these
            // explicitly named Save/Load families retain their 640 size.
            let centered = save_load && (matches!(name.as_str(), "NAME" | "OK" | "DELETE" | "CANCEL" | "UP" | "TAB" | "DOWN" | "HILITE" | "THUMBNAIL" | "BAR" | "RED")
                || name.strip_prefix("LIST").and_then(|s| s.parse::<usize>().ok()).is_some_and(|n| n < 10));
            c.rect = if centered { std::array::from_fn(|i| original[&c.id][i] + offset[i % 2]) } else { converted[&c.id] };
        }
        if let Some((tag, frames)) = &self.backgrounds {
            self.pictures.insert(*tag, vec![frames[usize::from(size[0] == 800)].clone()]);
        }
        self.size = size;
    }

    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        menu: &MenuArt,
        key: &str,
        picture: &str,
        rect: [i32; 4],
        frame: usize,
        caption: Option<&str>,
    ) {
        let caption = caption.or_else(|| self.controls[key].label.map(|id| menu.label(id)));
        self.draw_palette(canvas, &menu.palette, key, picture, rect, frame, caption);
    }

    /// Credits has its own original active palette and message catalogue;
    /// retain the same GUI picture/font drawing without borrowing Main's.
    pub(crate) fn draw_palette(
        &self,
        canvas: &mut Canvas,
        palette: &[kq8_data::palette::Rgb; 256],
        key: &str,
        picture: &str,
        rect: [i32; 4],
        frame: usize,
        caption: Option<&str>,
    ) {
        let c = &self.controls[key];
        let [l, _, r, _] = rect;
        if let Some(pic) = self.controls[picture].pic {
            let frames = &self.pictures[&pic.bitmap];
            if let Some(bitmap) = Pic::of(&frames[frame % frames.len()]) {
                canvas.picture(rect, bitmap, pic.modifier & 0x8000 != 0, 1.0, palette);
            }
        }
        if let Some(text) = c.text.filter(|t| t.font != 0) {
            let font = &self.fonts[&text.font];
            let mut value = caption.unwrap_or("");
            while !value.is_empty() && text_width(font, value) > r - l {
                value = &value[..value.char_indices().next_back().unwrap().0];
            }
            canvas.text(rect, font, text.justification, value, 1.0, palette);
        }
    }
}

#[cfg(test)]
mod graphics_probe {
    use super::*;

    #[test]
    #[ignore = "requires KQ8_GAME_DIR; headless original-data menu matrix"]
    fn original_interface_menu_matrix() {
        use crate::{graphics_menu, keyboard_menu, main_menu, options_menu, save_menu};
        let game = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let path = Path::new(&game);
        let mut art = MenuArt::open(path).unwrap();
        let mut first = None;
        for high in [false, true, false] {
            art.resize(high);
            let size = art.size();
            let mut options = options_menu::Screen::open(path, &art).unwrap();
            let mut graphics = graphics_menu::Screen::open(path, &art).unwrap();
            let mut keyboard = keyboard_menu::Screen::open(path, &art).unwrap();
            let mut settings = options_menu::Settings::default();
            let mut bindings = keyboard_menu::Bindings::default();
            let center = |r: [i32; 4]| main_menu::Key::Click((r[0] + r[2]) / 2, (r[1] + r[3]) / 2);
            let layout = |file, prefix| {
                let mut layout = Layout::read(path, file, prefix).unwrap();
                layout.resize(size, false);
                layout
            };
            let options_layout = layout("Options.gui", "IDCTL_OPT_");
            assert_eq!(options.input(center(options_layout.controls["MODE_LABEL"].rect), &mut settings), options_menu::Destination::Graphics);
            let graphics_layout = layout("Graphics.gui", "IDCTL_GRA_");
            let [l, _, r, b] = graphics_layout.controls["BACKTOMAIN"].rect;
            assert_eq!(graphics.input(main_menu::Key::Click((l+r)/2, b+1), &mut settings.graphics), graphics_menu::Action::Stay);
            assert_eq!(graphics.input(main_menu::Key::Click((l+r)/2, b), &mut settings.graphics), graphics_menu::Action::Back, "Graphics row bottom is inclusive");
            let keys_layout = layout("movekey.gui", "IDCTL_MOVEKEYS_");
            assert_eq!(keyboard.input(center(keys_layout.controls["TO_MAIN"].rect), &mut bindings), keyboard_menu::Action::Back);
            let mut canvases = vec![art.render(&main_menu::Choice::default()), options.render(&settings, &art, 0),
                graphics.render(&settings.graphics, &art, 0), keyboard.render(&bindings, &art, 0)];
            for (mode, file, prefix) in [(save_menu::Mode::Save, "Savegame.gui", "IDCTL_SAVE_"), (save_menu::Mode::Load, "Loadgame.gui", "IDCTL_LOAD_")] {
                let mut slots_art = save_menu::Art::open(path, mode).unwrap();
                slots_art.resize(size);
                let slots = (1..=15).map(|id| (id, format!("Slot {id}"))).collect();
                let mut state = save_menu::Screen::new(mode, slots);
                let mut layout = Layout::read(path, file, prefix).unwrap();
                let low: BTreeMap<_, _> = layout.controls.iter().map(|(k,c)| (k.clone(), c.rect)).collect();
                layout.resize(size, true);
                for name in ["NAME", "OK", "DELETE", "CANCEL", "THUMBNAIL", "UP", "TAB", "DOWN", "HILITE", "LIST0", "LIST9"] {
                    let offset = if high { [80, 60] } else { [0, 0] };
                    assert_eq!(layout.controls[name].rect, std::array::from_fn(|i| low[name][i] + offset[i % 2]), "{mode:?} {name}");
                }
                let ordinary = crate::gui_draw::absolute_rects(&crate::gui_draw::resolve_layout(&layout.authored, size));
                for name in ["TITLE", "POPUP_BKGND", "POPUP_TEXT1", "POPUP_BTN_YES"] {
                    let c = &layout.controls[name];
                    assert_eq!(c.rect, ordinary[&c.id], "{name} is not in the centered exception");
                }
                if mode == save_menu::Mode::Load {
                    assert_eq!(state.input(center(layout.controls["LIST6"].rect), &slots_art), save_menu::Command::None);
                    assert_eq!(state.selected_slot(), Some(7));
                    state.input(center(layout.controls["DOWN"].rect), &slots_art);
                    assert_eq!(state.selected_slot(), Some(8));
                }
                let [x, y, right, bottom] = layout.controls["THUMBNAIL"].rect;
                assert_eq!([right-x, bottom-y], [160, 120]);
                let sample = [art.palette[7][0], art.palette[7][1], art.palette[7][2], 255];
                let preview = sample.repeat(160 * 120);
                state.set_preview(Some(&preview), &art);
                let canvas = slots_art.render(&state, &art);
                assert_eq!(canvas.at((x+80) as u32, (y+60) as u32), sample, "thumbnail pixels retain their size and centered placement");
                canvases.push(canvas);
                layout.resize([640, 480], true);
                for (name, rect) in low { assert_eq!(layout.controls[&name].rect, rect, "{name} returns without rounding drift"); }
            }
            for canvas in &canvases { assert_eq!([canvas.width, canvas.height], size); }
            if !high {
                let pixels: Vec<_> = canvases.into_iter().map(|c| c.rgba).collect();
                if let Some(before) = &first { assert_eq!(&pixels, before, "all six menu families return to their original640 pixels"); }
                else { first = Some(pixels); }
            }
        }
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn installed_graphics_controls_have_named_layout() {
        let game = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let layout = Layout::read(Path::new(&game), "Graphics.gui", "IDCTL_GRA_").unwrap();
        let art = MenuArt::open(Path::new(&game)).unwrap();
        for name in ["MODE", "DIRECTDRAW", "DIRECT3D", "3DFX", "RESOLUTION", "640X480", "800X600",
            "DYNAMIC_LIGHTS", "LIGHTS_BOOL", "CONNOR_BOOL", "MONSTER_BOOL", "CHARACTER_BOOL", "OBJECT_BOOL",
            "BRIGHTNESS", "BRIGHTNESS_SLIDER", "BACKTOMAIN", "CURSOR1", "CURSOR2"] {
            assert!(layout.controls.contains_key(name), "missing Graphics.gui control {name}");
        }
        assert_eq!(layout.controls["BRIGHTNESS_SLIDER"].rect, [330, 383, 530, 393]);
        assert_eq!(layout.controls["TITLE"].label.map(|id| art.label(id)), Some("Graphic Settings"));
    }
}
