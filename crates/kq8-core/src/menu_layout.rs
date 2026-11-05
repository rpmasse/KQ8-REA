//! Menus use the GUI file for rectangles and tagged pictures, fonts and text.
use crate::gui_draw::{text_width, Canvas, Pic};
use crate::main_menu::Art as MenuArt;
use kq8_data::font::{bitmap_array, Font};
use kq8_data::gui::{self, Kind, Tags, Text};
use kq8_data::{Bitmap, GameDir};
use std::{collections::BTreeMap, path::Path};

pub(crate) struct Control {
    pub rect: [i32; 4],
    pic: Option<gui::Picture>,
    pub text: Option<Text>,
    label: Option<u8>,
}

pub struct Layout {
    pub(crate) controls: BTreeMap<String, Control>,
    pictures: BTreeMap<i32, Vec<Bitmap>>,
    pub(crate) fonts: BTreeMap<i32, Font>,
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
                    entry.insert(Font::parse(&read(&format!(
                        "GAME/8Gui/{}",
                        tagged(t.font)?
                    ))?)?);
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
                    rect: c.rect,
                    pic,
                    text,
                    label,
                },
            );
        }
        Ok(Self {
            controls,
            pictures,
            fonts,
        })
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
        let c = &self.controls[key];
        let [l, _, r, _] = rect;
        if let Some(pic) = self.controls[picture].pic {
            let frames = &self.pictures[&pic.bitmap];
            if let Some(bitmap) = Pic::of(&frames[frame % frames.len()]) {
                canvas.picture(rect, bitmap, pic.modifier & 0x8000 != 0, 1.0, &menu.palette);
            }
        }
        if let Some(text) = c.text.filter(|t| t.font != 0) {
            let font = &self.fonts[&text.font];
            let mut value = caption
                .or_else(|| c.label.map(|id| menu.label(id)))
                .unwrap_or("");
            while !value.is_empty() && text_width(font, value) > r - l {
                value = &value[..value.char_indices().next_back().unwrap().0];
            }
            canvas.text(rect, font, text.justification, value, 1.0, &menu.palette);
        }
    }
}
