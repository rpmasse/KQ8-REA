//! Drawing one panel of the game-play screen into a picture (M4 step
//! 4c-2): its pictures and numbers, placed by its layout block.
//!
//! From `docs/behaviour/interface.md`:
//!
//! - Children's rectangles are relative to their dialog (rule 5).
//! - A text with a rectangle whose right is left of its left is anchored
//!   at (left, top) and aligned by its justification (rule 5); otherwise
//!   it is aligned inside its rectangle. Justification: horizontal 1 left,
//!   2 right, 4 centred; vertical 0x8 centred, 0x10 from the top (rule 6).
//! - A bar is its picture stretched to the shown width, the rest
//!   `4black.pbm` stretched over the remaining width (rule 20, Q108 b).
//! - A picture whose layout modifier has 0x8000 is stretched to its box
//!   (nearest pixel); without it, drawn once at its own size from the
//!   box's top-left; never tiled (rule 20a, Q108 b).
//!
//! Our reading of the data, not in the note: a dialog's children are
//! drawn **last first**, so the first one listed is on top (in
//! `Gplayscr.gui` every panel lists its numbers before its pictures).
//! A picture is see-through where its index is 0 when its layout
//! attribute's drawing kind is 1 (rule 20a); pictures not from a layout
//! control keep their own flags' bit 1 (the community notes
//! `kq8pbmp.txt`: "transparent").
//! `graphics-modes.md` rules 11–15: authored controls are normalized
//! against their own parent, converted with +0.1 then truncation, and
//! composed as integer rectangles. Fonts and unstretched pictures keep
//! their own pixel size. Callers draw resolved rectangles at scale 1.

use kq8_data::font::Font;
use kq8_data::palette::Rgb;

/// A picture: size, palette indices, whether index 0 is see-through.
#[derive(Clone, Copy)]
pub struct Pic<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
    pub see_through: bool,
}

impl<'a> Pic<'a> {
    /// From a bitmap (8-bit only).
    pub fn of(b: &'a kq8_data::Bitmap) -> Option<Self> {
        (b.bit_count == 8 && b.width > 0 && b.height > 0).then_some(Pic { width: b.width, height: b.height, pixels: &b.pixels, see_through: b.flags & 1 != 0 })
    }
}

/// One thing to draw, its rectangle in layout pixels relative to the
/// panel.
pub enum Piece<'a> {
    /// A picture in its rectangle: stretched to it (nearest pixel), or
    /// drawn once at its own size from the top-left (rule 20a).
    Picture { rect: [i32; 4], pic: Pic<'a>, stretch: bool },
    /// A meter's bar (rule 20): `pic` stretched over the first `shown`
    /// screen pixels of the width, `black` over the rest.
    Bar { rect: [i32; 4], pic: Pic<'a>, black: Pic<'a>, shown: u32 },
    /// A picture stretched to its rectangle, only its lowest `rows`
    /// screen rows (a potion's timer icon, the part left: `potions.md`
    /// rule 35).
    Lower { rect: [i32; 4], pic: Pic<'a>, rows: u32 },
    /// A text in a font with a justification.
    Text { rect: [i32; 4], font: &'a Font, justification: u32, text: String },
    /// A rectangle of one colour (4c-8: the black behind the meters'
    /// bars, the user's check).
    Fill { rect: [i32; 4], rgb: [u8; 3] },
    /// The magic map (M6 6d-3, `magic-map.md` rules 6, 18): a 460 × 460
    /// picture drawn at `view` (left, top, right, bottom in layout pixels
    /// from the centre of `rect`), nearest pixel, cut to `rect`; the arrow
    /// on top at map pixel `arrow.1`, using the same integer position as
    /// follow and the selected bitmap's own dimensions, cut too.
    Map { rect: [i32; 4], pixels: &'a [u8], view: [f64; 4], arrow: Option<(Pic<'a>, [i32; 2])>, content_scale: f32 },
}

/// A legacy root-layout conversion. Resolved pixel rectangles pass scale
/// 1; normalization and rounding then belong only to `resolve_layout`.
pub fn scaled(v: i32, scale: f32) -> i32 {
    if scale == 1.0 { v } else { (v as f32 * scale + 0.1) as i32 }
}

/// Normalize immutable authored edges against their containing dimensions,
/// then convert using that parent's new integer size (Q207, rule 12).
pub fn resolve_rect(rect: [i32; 4], original_parent: [i32; 2], parent: [i32; 2]) -> [i32; 4] {
    std::array::from_fn(|i| {
        let axis = i % 2;
        (rect[i] as f32 / original_parent[axis].max(1) as f32 * parent[axis] as f32 + 0.1) as i32
    })
}

/// Each rectangle remains relative to its parent. Do not resize the
/// returned tree again: always start from the authored tree.
pub fn resolve_layout(authored: &kq8_data::gui::Control, screen: [u32; 2]) -> kq8_data::gui::Control {
    fn convert(source: &kq8_data::gui::Control, original: [i32; 2], parent: [i32; 2]) -> kq8_data::gui::Control {
        let mut result = source.clone();
        result.rect = resolve_rect(source.rect, original, parent);
        let old_size = [source.rect[2] - source.rect[0], source.rect[3] - source.rect[1]];
        let size = [result.rect[2] - result.rect[0], result.rect[3] - result.rect[1]];
        result.dialogs = source.dialogs.iter().map(|c| convert(c, old_size, size)).collect();
        result.controls = source.controls.iter().map(|c| convert(c, old_size, size)).collect();
        result
    }
    convert(authored, [640, 480], screen.map(|v| v as i32))
}

/// Compose converted integer origins; no subsequent floating-point scale.
pub fn absolute_rects(root: &kq8_data::gui::Control) -> std::collections::BTreeMap<u32, [i32; 4]> {
    fn walk(c: &kq8_data::gui::Control, origin: [i32; 2], out: &mut std::collections::BTreeMap<u32, [i32; 4]>) {
        let r = std::array::from_fn(|i| c.rect[i] + origin[i % 2]);
        out.insert(c.id, r);
        for child in c.dialogs.iter().chain(&c.controls) {
            walk(child, [r[0], r[1]], out);
        }
    }
    let mut result = std::collections::BTreeMap::new();
    walk(root, [0, 0], &mut result);
    result
}

/// Shared GUI text overwrites the PFT scale fields, not its glyph data.
pub fn gui_font(mut font: Font) -> Font {
    font.h_scale = 1.0;
    font.v_scale = 1.0;
    font
}

/// Select an original resolution-specific bitmap without renaming files.
pub fn resolution_picture(game: &kq8_data::GameDir, names: &[&str], screen: [u32; 2]) -> Result<kq8_data::Bitmap, String> {
    for name in names {
        let Some(path) = game.find(&format!("GAME/8Gui/{name}")) else { continue };
        let bytes = std::fs::read(path).map_err(|e| format!("{name}: {e}"))?;
        let bitmap = kq8_data::Bitmap::parse(&bytes)?;
        if [bitmap.width, bitmap.height] == screen && bitmap.bit_count == 8 { return Ok(bitmap); }
    }
    Err(format!("no original {}x{} bitmap among {names:?}", screen[0], screen[1]))
}

fn code(c: char) -> u8 {
    u8::try_from(c as u32).unwrap_or(b'?')
}

/// A text's width in a font (glyph width × horizontal scale + spacing,
/// as the caption's, `interface.md` rule 47).
pub fn text_width(font: &Font, text: &str) -> i32 {
    text.chars().map(|c| font.glyph(code(c)).map_or(0.0, |g| g.width as f32 * font.h_scale) + font.char_spacing as f32).sum::<f32>().round() as i32
}

/// Where a text's top-left goes (screen pixels, relative to the panel):
/// rule 5 (anchor) and rule 6 (justification).
pub fn text_origin(rect: [i32; 4], justification: u32, width: i32, height: i32, scale: f32) -> (i32, i32) {
    let [l, t, r, b] = rect.map(|v| scaled(v, scale));
    let anchor = rect[2] < rect[0];
    let h = justification & 0x7;
    let v = (justification >> 3) & 0x7;
    let x = match (anchor, h) {
        (true, 2) => l - width,
        (true, 4) => l - width / 2,
        (true, _) => l,
        (false, 2) => r - width,
        (false, 4) => l + (r - l - width) / 2,
        (false, _) => l,
    };
    let y = match (anchor, v) {
        (true, 1) => t - height / 2,
        (true, _) => t,
        (false, 1) => t + (b - t - height) / 2,
        (false, _) => t,
    };
    (x, y)
}

/// A panel's picture: `size` its layout size; RGBA, see-through where
/// nothing is drawn.
#[derive(Clone)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    /// The panel's top-left on the screen; pieces are drawn at their
    /// screen places less this.
    offset: [i32; 2],
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Canvas { width, height, rgba: vec![0; (width * height * 4) as usize], offset: [0, 0] }
    }

    fn put(&mut self, x: i32, y: i32, index: u8, see_through: bool, palette: &[Rgb; 256]) {
        let (x, y) = (x - self.offset[0], y - self.offset[1]);
        if (see_through && index == 0) || x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let at = ((y as u32 * self.width + x as u32) * 4) as usize;
        let [r, g, b] = palette[index as usize];
        self.rgba[at..at + 4].copy_from_slice(&[r, g, b, 255]);
    }

    /// RGBA at (x, y), for tests and logs.
    pub fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let at = ((y * self.width + x) * 4) as usize;
        self.rgba[at..at + 4].try_into().unwrap()
    }

    /// Draws `pic` in the layout rectangle `rect` (rule 20a): stretched
    /// to it, or once at its own pixel size from its top-left, cut
    /// to the rectangle; never tiled.
    pub fn picture(&mut self, rect: [i32; 4], pic: Pic, stretch: bool, scale: f32, palette: &[Rgb; 256]) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        if stretch {
            self.stretched([l, t, r, b], t, pic, palette);
            return;
        }
        let (w, h) = (pic.width as i32, pic.height as i32);
        let (right, bottom) = (l + w, t + h);
        for y in t..bottom.min(b) {
            for x in l..right.min(r) {
                let sx = (((x - l) as f32 / w as f32 * pic.width as f32) as u32).min(pic.width - 1);
                let sy = (((y - t) as f32 / h as f32 * pic.height as f32) as u32).min(pic.height - 1);
                self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], pic.see_through, palette);
            }
        }
    }

    /// `pic` stretched over the screen rectangle `[l, t, r, b]` (nearest
    /// pixel), drawing only the rows from `from` down.
    fn stretched(&mut self, [l, t, r, b]: [i32; 4], from: i32, pic: Pic, palette: &[Rgb; 256]) {
        let (w, h) = ((r - l).max(1) as f32, (b - t).max(1) as f32);
        for y in from.max(t)..b {
            for x in l..r {
                let sx = (((x - l) as f32 / w * pic.width as f32) as u32).min(pic.width - 1);
                let sy = (((y - t) as f32 / h * pic.height as f32) as u32).min(pic.height - 1);
                self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], pic.see_through, palette);
            }
        }
    }

    /// A meter's bar (rule 20): `pic` stretched over the first `shown`
    /// screen pixels of the width, `black` stretched over the rest.
    pub fn bar(&mut self, rect: [i32; 4], pic: Pic, black: Pic, shown: u32, scale: f32, palette: &[Rgb; 256]) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let cut = (l + shown as i32).min(r);
        if cut > l {
            self.stretched([l, t, cut, b], t, pic, palette);
        }
        if r > cut {
            self.stretched([cut, t, r, b], t, black, palette);
        }
    }

    /// Draws `pic` stretched to the layout rectangle `rect`, only its
    /// lowest `rows` screen rows (the rows above are left as they are).
    pub fn picture_lower(&mut self, rect: [i32; 4], pic: Pic, rows: u32, scale: f32, palette: &[Rgb; 256]) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        self.stretched([l, t, r, b], b - rows as i32, pic, palette);
    }

    /// Fills the layout rectangle `rect` with one colour.
    pub fn fill(&mut self, rect: [i32; 4], rgb: [u8; 3], scale: f32) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let [ox, oy] = self.offset;
        for y in (t - oy).max(0)..(b - oy).min(self.height as i32) {
            for x in (l - ox).max(0)..(r - ox).min(self.width as i32) {
                let at = ((y as u32 * self.width + x as u32) * 4) as usize;
                self.rgba[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
    }

    /// Draws the magic map (see [`Piece::Map`]); `rect` in layout pixels
    /// where the panel stands.
    pub fn map(&mut self, rect: [i32; 4], pixels: &[u8], view: [f64; 4], arrow: Option<(Pic, [i32; 2])>, scale: f32, content_scale: f32, palette: &[Rgb; 256]) {
        let size = kq8_data::magic_map::MAP_SIZE;
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let s = (scale * content_scale) as f64;
        let view = crate::magic_map::View { left: view[0] * s, top: view[1] * s, right: view[2] * s, bottom: view[3] * s };
        let [left, top, right, bottom] = view.picture_rect([l,t,r,b]);
        let (w, h) = (right-left, bottom-top);
        if pixels.len() == size * size && w > 0 && h > 0 {
            for y in t..b {
                let my = ((f64::from(y - top) + 0.5) / f64::from(h) * size as f64).floor();
                if my < 0.0 || my >= size as f64 {
                    continue;
                }
                for x in l..r {
                    let mx = ((f64::from(x - left) + 0.5) / f64::from(w) * size as f64).floor();
                    if mx < 0.0 || mx >= size as f64 {
                        continue;
                    }
                    self.put(x, y, pixels[my as usize * size + mx as usize], false, palette);
                }
            }
        }
        if let Some((pic, pixel)) = arrow {
            let [aw, ah] = [pic.width,pic.height].map(|v| (f64::from(v) * s) as u32);
            let [ax,ay] = view.arrow_position((pixel[0],pixel[1]), [l,t,r,b], [aw,ah]);
            let (aw,ah) = (aw as i32,ah as i32);
            for y in ay.max(t)..(ay + ah).min(b) {
                for x in ax.max(l)..(ax + aw).min(r) {
                    let sx = (((x - ax) as f64 / s) as u32).min(pic.width - 1);
                    let sy = (((y - ay) as f64 / s) as u32).min(pic.height - 1);
                    self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], true, palette);
                }
            }
        }
    }

    /// Draws `text` in `font`, placed by `rect` and `justification`
    /// (rules 5–6); letters at the font's own size, index 0 see-through.
    pub fn text(&mut self, rect: [i32; 4], font: &Font, justification: u32, text: &str, scale: f32, palette: &[Rgb; 256]) {
        let (mut x, y) = text_origin(rect, justification, text_width(font, text), font.char_height, scale);
        for c in text.chars() {
            let Some(g) = font.glyph(code(c)) else { continue };
            if let Some(px) = font.glyph_pixels(g) {
                for gy in 0..g.height as i32 {
                    for gx in 0..g.width as i32 {
                        self.put(x + gx, y + gy, px[(gy * g.width as i32 + gx) as usize], true, palette);
                    }
                }
            }
            x += (g.width as f32 * font.h_scale + font.char_spacing as f32).round() as i32;
        }
    }
}

/// The screen pixels a panel at layout point `origin`, `size` layout
/// pixels big, covers: its edges rounded where they stand (4c-9).
pub fn screen_size(origin: [i32; 2], size: [i32; 2], scale: f32) -> [u32; 2] {
    [0, 1].map(|k| (scaled(origin[k] + size[k], scale) - scaled(origin[k], scale)).max(1) as u32)
}

/// Draws a panel standing at layout point `origin`, `size` layout pixels
/// big, from its pieces (rectangles relative to the panel), **last
/// first** (the first piece ends on top).
pub fn draw(origin: [i32; 2], size: [i32; 2], scale: f32, pieces: &[Piece], palette: &[Rgb; 256]) -> Canvas {
    let [w, h] = screen_size(origin, size, scale);
    let mut c = Canvas::new(w, h);
    c.offset = origin.map(|v| scaled(v, scale));
    let [ox, oy] = origin;
    let at = |r: &[i32; 4]| [r[0] + ox, r[1] + oy, r[2] + ox, r[3] + oy];
    for piece in pieces.iter().rev() {
        match piece {
            Piece::Picture { rect, pic, stretch } => c.picture(at(rect), *pic, *stretch, scale, palette),
            Piece::Bar { rect, pic, black, shown } => c.bar(at(rect), *pic, *black, *shown, scale, palette),
            Piece::Lower { rect, pic, rows } => c.picture_lower(at(rect), *pic, *rows, scale, palette),
            Piece::Text { rect, font, justification, text } => c.text(at(rect), font, *justification, text, scale, palette),
            Piece::Fill { rect, rgb } => c.fill(at(rect), *rgb, scale),
            Piece::Map { rect, pixels, view, arrow, content_scale } => c.map(at(rect), pixels, *view, *arrow, scale, *content_scale, palette),
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_interface_geometry_raster_matrix() {
        use kq8_data::gui::{Control, Kind};
        let node = |id, rect| Control { tag: "UIDL".into(), version: 1, id, flags: 0,
            rect, kind: Kind::Dialog { caption: None, picture: None }, dialogs: vec![], controls: vec![] };
        let mut source = node(1, [0, 0, 640, 480]);
        let mut parent = node(2, [1, 1, 15, 15]);
        parent.controls.push(node(3, [3, 3, 7, 7]));
        source.dialogs.push(parent);
        let original = source.clone();
        let mut font = crate::caption::tests::font();
        font.h_scale = 2.0;
        font.v_scale = 3.0;
        let font = gui_font(font);
        assert_eq!((font.h_scale, font.v_scale, text_width(&font, "ab")), (1.0, 1.0, 20));
        let mut palette = [[0; 3]; 256];
        palette[7] = [20, 30, 40];
        let pixels = [7; 4];
        let pic = Pic { width: 2, height: 2, pixels: &pixels, see_through: false };
        let mut first = None;
        for high in [false, true, false] {
            let size = crate::caption::screen(high);
            let resolved = resolve_layout(&source, size);
            let rectangles = absolute_rects(&resolved);
            assert_eq!(rectangles[&1], [0, 0, size[0] as i32, size[1] as i32]);
            assert_eq!(rectangles[&2], if high { [1, 1, 18, 18] } else { [1, 1, 15, 15] });
            assert_eq!(rectangles[&3], if high { [4, 4, 9, 9] } else { [4, 4, 8, 8] }, "parent and local edges are converted separately");
            assert_eq!(resolve_rect([102, -2, 106, 2], [640, 480], size.map(|v| v as i32)),
                if high { [127, -2, 132, 2] } else { [102, -1, 106, 2] });
            let mut canvas = Canvas::new(size[0], size[1]);
            let box_at = |r| resolve_rect(r, [640, 480], size.map(|v| v as i32));
            let once = box_at([20, 20, 30, 30]);
            canvas.picture(once, pic, false, 1.0, &palette);
            assert_eq!(canvas.at((once[0] + 1) as u32, (once[1] + 1) as u32), [20, 30, 40, 255]);
            assert_eq!(canvas.at((once[0] + 2) as u32, once[1] as u32), [0; 4], "native picture width remains two pixels");
            let stretch = box_at([40, 20, 50, 30]);
            canvas.picture(stretch, pic, true, 1.0, &palette);
            assert_eq!(canvas.at((stretch[2] - 1) as u32, (stretch[3] - 1) as u32), [20, 30, 40, 255]);
            let label = box_at([80, 20, 120, 40]);
            canvas.text(label, &font, 0x11, "ab", 1.0, &palette);
            assert_eq!(canvas.at((label[0] + 19) as u32, (label[1] + 1) as u32), [20, 30, 40, 255]);
            assert_eq!(canvas.at((label[0] + 20) as u32, label[1] as u32), [0; 4]);
            let caption = crate::caption::Box { width: size[0] / 2, row_height: 15, text_y: 2, scale: crate::caption::scale(size) };
            let rows = crate::caption::wrap(&font, &"a".repeat(35), caption.width as f32);
            assert_eq!(rows.len(), if high { 1 } else { 2 });
            assert_eq!(caption.height(rows.len()), if high { 19 } else { 34 });
            if !high {
                if let Some(pixels) = &first { assert_eq!(&canvas.rgba, pixels, "640 return has no accumulated rounding"); }
                else { first = Some(canvas.rgba); }
            }
        }
        assert_eq!(source, original, "authored tree stays immutable");
    }

    #[test]
    fn numbers_anchored_at_a_point_or_aligned_in_their_box() {
        // IDCTL_HEALTHBAR_1TEXT (29, 2)-(28, 1), 0x12: right edge at x 29,
        // top at y 2 (rule 5).
        assert_eq!(text_origin([29, 2, 28, 1], 0x12, 10, 15, 1.0), (19, 2));
        // IDCTL_INVBAR_ARMORTEXT (26, 5)-(56, 19), 0x12: right edge at 56.
        assert_eq!(text_origin([26, 5, 56, 19], 0x12, 10, 15, 1.0), (46, 5));
        // IDCTL_METERS_LEVEL (100, 0)-(156, 14), 0xC: centred across, top.
        assert_eq!(text_origin([100, 0, 156, 14], 0xC, 10, 14, 1.0), (123, 0));
        // 0x9: left, centred down.
        assert_eq!(text_origin([0, 0, 50, 20], 0x9, 10, 10, 1.0), (0, 5));
        // Scaled by 1.25: the anchor moves, the text keeps its size.
        assert_eq!(text_origin([56, 5, 26, 4], 0x12, 10, 15, 1.25), (60, 6));
    }

    #[test]
    fn a_lower_part_keeps_its_own_rows() {
        let pal = [[0u8; 3]; 256].map(|_| [0, 0, 0]);
        let mut pal = pal;
        pal[1] = [10, 0, 0];
        pal[2] = [20, 0, 0];
        let empty = [1u8; 4];
        let full = [2u8; 4];
        // 2 x 2: the lower row from `full`, the upper from `empty`.
        let c = draw([0, 0], [2, 2], 1.0, &[
            Piece::Lower { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &full, see_through: true }, rows: 1 },
            Piece::Picture { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &empty, see_through: true }, stretch: true },
        ], &pal);
        assert_eq!((c.at(0, 0)[0], c.at(1, 1)[0]), (10, 20));
    }

    #[test]
    fn a_stretched_picture_is_not_repeated() {
        let mut pal = [[0u8; 3]; 256];
        pal[1] = [255, 0, 0];
        pal[2] = [0, 255, 0];
        // 1 x 2: red on top, green below; stretched over 1 x 4.
        let pixels = [1u8, 2];
        let c = draw([0, 0], [1, 4], 1.0, &[Piece::Picture { rect: [0, 0, 1, 4], pic: Pic { width: 1, height: 2, pixels: &pixels, see_through: false }, stretch: true }], &pal);
        assert_eq!([c.at(0, 0)[0], c.at(0, 1)[0], c.at(0, 2)[1], c.at(0, 3)[1]], [255, 255, 255, 255], "top half red, bottom half green");
    }

    #[test]
    fn pictures_are_stretched_or_drawn_once_bars_end_in_black_and_the_first_piece_is_on_top() {
        let mut pal = [[0u8; 3]; 256];
        pal[1] = [255, 0, 0];
        pal[2] = [0, 255, 0];
        pal[3] = [0, 0, 9];
        let red = [1u8; 4];
        let black = [3u8; 4];
        let back = [2u8, 2, 0, 2];
        let pieces = [
            // interface.md rule 20 (Q108 b): a bar 10 wide, its 2x2 picture
            // stretched over the 4 pixels shown, `4black` over the other 6.
            Piece::Bar { rect: [0, 0, 10, 2], pic: Pic { width: 2, height: 2, pixels: &red, see_through: false }, black: Pic { width: 2, height: 2, pixels: &black, see_through: false }, shown: 4 },
            // Under it: a background stretched to its box (rule 20a).
            Piece::Picture { rect: [0, 0, 10, 4], pic: Pic { width: 2, height: 2, pixels: &back, see_through: false }, stretch: true },
        ];
        let c = draw([0, 0], [10, 4], 1.0, &pieces, &pal);
        assert_eq!((c.width, c.height), (10, 4));
        assert_eq!(c.at(3, 1), [255, 0, 0, 255], "the bar on top");
        assert_eq!(c.at(4, 0), [0, 0, 9, 255], "past 4 pixels: 4black");
        // Stretched 2x2 over 10x4: its lower-left index 0 covers x 0-4,
        // y 2-3 (not repeated), drawn: the picture is not see-through.
        assert_eq!((c.at(0, 3), c.at(4, 3), c.at(5, 3)), ([0, 0, 0, 255], [0, 0, 0, 255], [0, 255, 0, 255]));
        // See-through index 0 (attribute kind 1).
        let st = draw([0, 0], [2, 2], 1.0, &[Piece::Picture { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &back, see_through: true }, stretch: true }], &pal);
        assert_eq!(st.at(0, 1), [0, 0, 0, 0]);
        // Scaled by 2: a 2x2 picture covers 4x4.
        let big = draw([0, 0], [2, 2], 2.0, &[Piece::Picture { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &back, see_through: true }, stretch: true }], &pal);
        assert_eq!((big.width, big.at(1, 1), big.at(1, 2)), (4, [0, 255, 0, 255], [0, 0, 0, 0]));
        // Without 0x8000: once at its own size from the top-left, the rest
        // left as it was (not tiled).
        let once = draw([0, 0], [6, 2], 1.0, &[Piece::Picture { rect: [0, 0, 6, 2], pic: Pic { width: 2, height: 2, pixels: &red, see_through: false }, stretch: false }], &pal);
        assert_eq!((once.at(1, 1), once.at(2, 0)), ([255, 0, 0, 255], [0, 0, 0, 0]));
        // An empty bar is all black; a full one all picture.
        let empty = draw([0, 0], [10, 2], 1.0, &[Piece::Bar { rect: [0, 0, 10, 2], pic: Pic { width: 2, height: 2, pixels: &red, see_through: false }, black: Pic { width: 2, height: 2, pixels: &black, see_through: false }, shown: 0 }], &pal);
        assert_eq!((empty.at(0, 0), empty.at(9, 1)), ([0, 0, 9, 255], [0, 0, 9, 255]));
        // Q207: 582.5 + 0.1 truncates to 582, leaving 18 rows.
        assert_eq!(screen_size([192, 466], [256, 14], 1.25), [320, 18]);
        let full = draw([192, 466], [2, 14], 1.25, &[Piece::Fill { rect: [0, 0, 2, 14], rgb: [9, 9, 9] }], &pal);
        assert_eq!((full.height, full.at(0, 0), full.at(1, 17)), (18, [9, 9, 9, 255], [9, 9, 9, 255]));
    }

    #[test]
    fn the_map_at_its_view_with_the_arrow_cut_to_the_area() {
        let size = kq8_data::magic_map::MAP_SIZE;
        let mut pal = [[0u8; 3]; 256];
        pal[1] = [10, 0, 0];
        pal[2] = [20, 0, 0];
        pal[3] = [30, 0, 0];
        let mut pixels = vec![1u8; size * size];
        // The top-left 10 x 10 map pixels (one layout pixel at this view).
        for y in 0..10 {
            pixels[y * size..y * size + 10].fill(2);
        }
        // Area 10 x 10 at (0, 0); the view puts the picture's top-left at
        // the area's centre, 460 map pixels over 46 layout pixels.
        let arrow = [3u8; 4];
        let pic = Pic { width: 2, height: 2, pixels: &arrow, see_through: true };
        let c = draw([0, 0], [10, 10], 1.0, &[Piece::Map { rect: [0, 0, 10, 10], pixels: &pixels, view: [0.0, 0.0, 46.0, 46.0], arrow: Some((pic, [40, 40])), content_scale: 1.0 }], &pal);
        assert_eq!(c.at(4, 4), [0, 0, 0, 0], "left of and above the picture: nothing");
        assert_eq!(c.at(5, 5), [20, 0, 0, 255], "the picture's first pixel at the centre");
        assert_eq!(c.at(6, 6), [10, 0, 0, 255]);
        assert_eq!((c.at(8, 8), c.at(9, 9)), ([30, 0, 0, 255], [30, 0, 0, 255]), "the arrow at centre + 3");
        // Cut at the area's edge: the arrow's second column would be x 10.
        let c = draw([0, 0], [11, 10], 1.0, &[Piece::Map { rect: [0, 0, 10, 10], pixels: &pixels, view: [0.0, 0.0, 46.0, 46.0], arrow: Some((pic, [50, 10])), content_scale: 1.0 }], &pal);
        assert_eq!((c.at(9, 5), c.at(10, 5)), ([30, 0, 0, 255], [0, 0, 0, 0]));
        // Odd area and frame dimensions plus fractional negative offset:
        // origin = 0 + 11/2 + trunc(-2.9) = 3; extent = trunc(46.9)=46;
        // arrow = origin + 20*46/460 - 3/2 = 4, not rounded float 3.
        let odd_arrow = [3;9];
        let odd_pic = Pic { width:3,height:3,pixels:&odd_arrow,see_through:true };
        let c = draw([0,0],[11,11],1.0,&[Piece::Map { rect:[0,0,11,11],pixels:&pixels,
            view:[-2.9,-2.9,44.0,44.0],arrow:Some((odd_pic,[20,20])),content_scale:1.0 }],&pal);
        assert_eq!(c.at(3,3),[20,0,0,255],"fractional picture origin truncates separately");
        assert_eq!(c.at(4,4),[30,0,0,255]);
        assert_eq!(c.at(6,6),[30,0,0,255]);
        assert_eq!(c.at(7,7),[10,0,0,255]);
    }

    #[test]
    fn a_text_is_drawn_at_its_anchor() {
        let f = crate::caption::tests::font();
        let mut pal = [[0u8; 3]; 256];
        pal[7] = [200, 150, 50];
        let c = draw([0, 0], [40, 4], 1.0, &[Piece::Text { rect: [30, 1, 29, 0], font: &f, justification: 0x12, text: "ab".into() }], &pal);
        // "ab" is 20 wide: x 10..30, y 1..3.
        assert_eq!(c.at(10, 1), [200, 150, 50, 255]);
        assert_eq!(c.at(29, 2), [200, 150, 50, 255]);
        assert_eq!(c.at(9, 1), [0, 0, 0, 0]);
        assert_eq!(c.at(30, 1), [0, 0, 0, 0]);
    }
}
