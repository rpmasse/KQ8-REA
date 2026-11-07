//! Original Graphics.gui layout; device-profile selection remains separate.
use crate::{
    gui_draw::Canvas,
    main_menu::{Art, Key},
    menu_layout::Layout,
    options_menu::Graphics,
};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Row {
    Low,
    High,
    Lights,
    Connor,
    Monster,
    Character,
    Object,
    Brightness,
    Back,
}

const ROWS: [Row; 9] = [
    Row::Low,
    Row::High,
    Row::Lights,
    Row::Connor,
    Row::Monster,
    Row::Character,
    Row::Object,
    Row::Brightness,
    Row::Back,
];

impl Row {
    fn key(self) -> &'static str {
        match self {
            Self::Low => "640X480",
            Self::High => "800X600",
            Self::Lights => "DYNAMIC_LIGHTS",
            Self::Connor => "CONNOR_SHADOWS",
            Self::Monster => "MONSTER_SHADOWS",
            Self::Character => "CHARACTER_SHADOWS",
            Self::Object => "OBJECT_SHADOWS",
            Self::Brightness => "BRIGHTNESS",
            Self::Back => "BACKTOMAIN",
        }
    }
    fn value(self) -> Option<&'static str> {
        Some(match self {
            Self::Lights => "LIGHTS_BOOL",
            Self::Connor => "CONNOR_BOOL",
            Self::Monster => "MONSTER_BOOL",
            Self::Character => "CHARACTER_BOOL",
            Self::Object => "OBJECT_BOOL",
            Self::Brightness => "BRIGHTNESS_SLIDER",
            _ => return None,
        })
    }
    fn shadow_bit(self) -> Option<u32> {
        Some(match self {
            Self::Connor => 1,
            Self::Monster => 2,
            Self::Character => 4,
            Self::Object => 8,
            _ => return None,
        })
    }
}

pub struct Screen {
    layout: Layout,
    background: Canvas,
    row: Row,
    pub dragging: bool,
    high_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Stay,
    Back,
}

impl Screen {
    pub fn open(game: &Path, art: &Art) -> Result<Self, String> {
        let mut layout = Layout::read(game, "Graphics.gui", "IDCTL_GRA_")?;
        layout.resize(art.size(), false);
        for key in [
            "BACKGROUND",
            "TITLE",
            "MODE",
            "DIRECTDRAW",
            "DIRECT3D",
            "3DFX",
            "RESOLUTION",
            "CURSOR1",
            "CURSOR2",
        ]
        .into_iter()
        .chain(ROWS.iter().map(|r| r.key()))
        .chain(ROWS.iter().filter_map(|r| r.value()))
        {
            if !layout.controls.contains_key(key) {
                return Err(format!("Graphics.gui lacks {key}"));
            }
        }
        let background = Self::backdrop(&layout,art);
        Ok(Self { layout, background, row: Row::Low, dragging: false, high_available: true })
    }

    pub fn resize(&mut self, art: &Art) {
        self.layout.resize(art.size(), false);
        self.background = Self::backdrop(&self.layout, art);
    }

    /// Explicit capability input. This does not identify a host GPU as Glide.
    pub fn set_high_resolution_available(&mut self, available: bool) {
        self.high_available = available;
        if !available && self.row == Row::High { self.row = Row::Low; }
    }

    fn available(&self, row: Row) -> bool { row != Row::High || self.high_available }

    fn backdrop(layout: &Layout, art: &Art) -> Canvas {
        let mut background = Canvas::new(layout.size[0], layout.size[1]);
        for key in [
            "BACKGROUND",
            "TITLE",
            "MODE",
            "DIRECTDRAW",
            "DIRECT3D",
            "3DFX",
            "RESOLUTION",
        ]
        .into_iter()
        .chain(ROWS.iter().filter(|r| !matches!(r, Row::Low | Row::High)).map(|r| r.key()))
        {
            let caption = match key {
                "DIRECTDRAW" => Some("DirectDraw"),
                "DIRECT3D" => Some("Direct3D"),
                "3DFX" => Some("3Dfx"),
                _ => None,
            };
            layout.draw(
                &mut background,
                art,
                key,
                key,
                layout.controls[key].rect,
                0,
                caption,
            );
        }
        background
    }

    pub fn palette_changed(&mut self, art: &Art) {
        self.background = Self::backdrop(&self.layout,art);
    }

    /// Actual uploaded pixels outside dynamic values and the two cursor
    /// columns. The authored rectangles make the sample independent of
    /// selection/animation without assuming that a fixed crop is static.
    pub fn palette_sample(&self, rgba: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
        if (width,height) != (self.background.width,self.background.height) || rgba.len() != width as usize*height as usize*4 { return None; }
        let mut dynamic: Vec<[i32;4]> = ROWS.iter().filter_map(|r|r.value()).map(|key|self.layout.controls[key].rect).collect();
        dynamic.extend([Row::Low, Row::High].map(|row| self.layout.controls[row.key()].rect));
        for key in ["CURSOR1","CURSOR2"] {
            let [l,_,r,_] = self.layout.controls[key].rect;
            dynamic.push([l,0,r,height as i32]);
        }
        let mut sample = Vec::new();
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                if dynamic.iter().any(|&[l,t,r,b]| (l..r).contains(&x) && (t..b).contains(&y)) { continue; }
                let offset = (y as usize*width as usize+x as usize)*4;
                sample.extend_from_slice(&rgba[offset..offset+4]);
            }
        }
        Some(sample)
    }

    fn hit(&self, key: &str, x: i32, y: i32) -> bool {
        let [l, t, r, b] = self.layout.controls[key].rect;
        (l..r).contains(&x) && (t..=b).contains(&y)
    }

    fn brightness_at(&self, x: i32, settings: &mut Graphics) {
        let [l, _, r, _] = self.layout.controls["BRIGHTNESS_SLIDER"].rect;
        let travel = (r - l - 10).max(1);
        settings.brightness = ((100 * (x - l - 5).clamp(0, travel) + travel / 2) / travel) as u8;
    }

    fn choose(&self, settings: &mut Graphics) -> Action {
        match self.row {
            Row::Low => settings.high_resolution = false,
            Row::High if self.high_available => settings.high_resolution = true,
            Row::High => {}
            Row::Lights => settings.dynamic_lighting = !settings.dynamic_lighting,
            Row::Brightness => {}
            Row::Back => return Action::Back,
            row => {
                if let Some(bit) = row.shadow_bit() {
                    settings.shadow_mask ^= bit;
                }
            }
        }
        Action::Stay
    }

    pub fn input(&mut self, key: Key, settings: &mut Graphics) -> Action {
        match key {
            Key::Escape => return Action::Back,
            Key::Up | Key::Down | Key::Tab => {
                let i = ROWS.iter().position(|r| *r == self.row).unwrap();
                let by = if key == Key::Up { ROWS.len() - 1 } else { 1 };
                self.row = (1..=ROWS.len()).map(|step| ROWS[(i + by * step) % ROWS.len()])
                    .find(|&row| self.available(row)).unwrap();
            }
            Key::Click(x, y) => {
                for row in ROWS {
                    if self.available(row) && (self.hit(row.key(), x, y) || row.value().is_some_and(|v| self.hit(v, x, y))) {
                        self.row = row;
                        if row == Row::Brightness {
                            self.dragging = self.hit("BRIGHTNESS_SLIDER", x, y);
                            if self.dragging {
                                self.brightness_at(x, settings);
                            }
                        } else {
                            return self.choose(settings);
                        }
                        break;
                    }
                }
            }
            Key::Drag(x, _) if self.dragging => self.brightness_at(x, settings),
            Key::MouseUp => self.dragging = false,
            Key::Left | Key::Right if self.row == Row::Brightness => {
                settings.brightness = if key == Key::Left {
                    settings.brightness.saturating_sub(1)
                } else {
                    settings.brightness.saturating_add(1).min(100)
                };
            }
            Key::Left | Key::Right if self.row == Row::Low || self.row == Row::High => {
                if key == Key::Left || self.high_available { settings.high_resolution = key == Key::Right; }
            }
            Key::Left | Key::Right => return self.choose(settings),
            Key::Enter => return self.choose(settings),
            _ => {}
        }
        Action::Stay
    }

    pub fn render(&self, settings: &Graphics, art: &Art, frame: usize) -> Canvas {
        let mut out = self.background.clone();
        for (row, selected) in [(Row::Low, !settings.high_resolution), (Row::High, settings.high_resolution)] {
            let key = row.key();
            let label = self.layout.caption(key, art).unwrap_or("");
            let marked = if selected { format!("> {label} <") } else { label.to_owned() };
            let rect = self.layout.controls[key].rect;
            if self.available(row) {
                self.layout.draw(&mut out, art, key, key, rect, 0, Some(&marked));
            } else {
                let mut glyphs = Canvas::new(out.width, out.height);
                self.layout.draw(&mut glyphs, art, key, key, rect, 0, Some(&marked));
                for y in rect[1].max(0)..rect[3].min(out.height as i32) {
                    for x in rect[0].max(0)..rect[2].min(out.width as i32) {
                        let p = (y as usize * out.width as usize + x as usize) * 4;
                        if (x+y)%2 == 0 && glyphs.rgba[p+3] != 0 {
                            out.rgba[p..p+4].copy_from_slice(&glyphs.rgba[p..p+4]);
                        }
                    }
                }
            }
        }
        for row in ROWS {
            let Some(key) = row.value() else { continue };
            let rect = self.layout.controls[key].rect;
            if row == Row::Brightness {
                self.layout.draw(&mut out, art, key, key, rect, 0, None);
                let [l, t, r, b] = rect;
                let x = l + i32::from(settings.brightness) * (r - l - 10) / 100;
                self.layout
                    .draw(&mut out, art, key, key, [x, t, x + 10, b], 1, None);
            } else {
                let enabled = if row == Row::Lights {
                    settings.dynamic_lighting
                } else {
                    settings.shadow_mask & row.shadow_bit().unwrap() != 0
                };
                self.layout.draw(
                    &mut out,
                    art,
                    key,
                    key,
                    rect,
                    0,
                    Some(if enabled { "On" } else { "Off" }),
                );
            }
        }
        let selected = self.layout.controls[self.row.key()].rect;
        for key in ["CURSOR1", "CURSOR2"] {
            let [l, t, r, b] = self.layout.controls[key].rect;
            let y = (selected[1] + selected[3] - (b - t)) / 2;
            self.layout
                .draw(&mut out, art, key, key, [l, y, r, y + (b - t)], frame, None);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn original_graphics_controls_change_only_their_own_settings() {
        let game = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let art = Art::open(Path::new(&game)).unwrap();
        let mut screen = Screen::open(Path::new(&game), &art).unwrap();
        let mut graphics = Graphics {
            mode: "D3D-display".into(),
            high_resolution: false,
            brightness: 49,
            shadow_mask: 17,
            dynamic_lighting: false,
        };
        let before = screen.render(&graphics,&art,0);
        let sample = screen.palette_sample(&before.rgba,before.width,before.height).unwrap();
        assert_eq!(
            screen.input(Key::Click(350, 190), &mut graphics),
            Action::Stay
        );
        assert!(graphics.high_resolution);
        screen.input(Key::Click(350, 232), &mut graphics);
        assert!(graphics.dynamic_lighting);
        screen.input(Key::Click(350, 263), &mut graphics);
        assert_eq!(graphics.shadow_mask, 16);
        screen.input(Key::Click(335, 387), &mut graphics);
        screen.input(Key::Drag(525, 387), &mut graphics);
        screen.input(Key::MouseUp, &mut graphics);
        assert_eq!(graphics.brightness, 100);
        assert_eq!(graphics.mode, "D3D-display");
        let changed = screen.render(&graphics,&art,7);
        assert_eq!(screen.palette_sample(&changed.rgba,changed.width,changed.height).unwrap(),sample,
            "palette evidence must exclude authored changing controls and animated selection cursors");
        assert_eq!(
            screen.input(Key::Click(310, 450), &mut graphics),
            Action::Back
        );
    }
}
