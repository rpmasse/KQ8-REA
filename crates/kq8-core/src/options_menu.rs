//! Options.gui and Options.cs. Volume rules: docs/behaviour/sound.md 50–52.
//! The GOG manual describes these options on pages 12–14.
use crate::{
    gui_draw::Canvas,
    main_menu::{Art as MenuArt, Key},
    menu_layout::Layout,
    sound::Groups,
};
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settings {
    // Music (groups 4/5), effects (0/1/2) and speech (3), in menu order.
    pub volume: [u8; 3],
    pub subtitles: bool,
    pub help: bool,
    /// The original reserves notStartingHelp 2 for a locked-off preference.
    pub help_locked: bool,
    pub graphics: Graphics,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Graphics {
    /// The original engine's mode name, retained verbatim from Options.cs.
    pub mode: String,
    pub high_resolution: bool,
    pub brightness: u8,
    /// Bits 1, 2, 4 and 8 control Connor, monsters, other characters and objects.
    pub shadow_mask: u32,
    pub dynamic_lighting: bool,
}

impl Default for Graphics {
    fn default() -> Self {
        Self { mode: "D3D-display".into(), high_resolution: false, brightness: 50, shadow_mask: 1, dynamic_lighting: false }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            volume: [100; 3],
            subtitles: true,
            help: true,
            help_locked: false,
            graphics: Graphics::default(),
        }
    }
}

fn position(n: u8) -> u8 {
    if n == 0 {
        0
    } else {
        n.clamp(10, 100)
    }
}

impl Settings {
    pub fn from_script(bytes: &[u8]) -> Self {
        let mut groups = Groups::default();
        groups.apply_script(bytes);
        let mut settings = Self {
            // Script commands accept 0..1. The ten-percent floor applies
            // when a player moves the slider (sound.md rules 51–52).
            volume: [4, 0, 3].map(|g| (groups.slider[g].clamp(0.0, 1.0) * 100.).round() as u8),
            ..Self::default()
        };
        let text = String::from_utf8_lossy(bytes);
        for line in text.lines() {
            let words: Vec<_> = line
                .split("//")
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect();
            let command = words.first().map(|w| w.to_ascii_lowercase()).unwrap_or_default();
            let value = words.get(1).copied();
            match (command.as_str(), value) {
                ("set", Some(key)) if key.eq_ignore_ascii_case("Options::Captions") => {
                    settings.subtitles = words.get(2).is_some_and(|v| v.eq_ignore_ascii_case("true") || *v == "1");
                }
                ("notstartinghelp", Some(v)) => {
                    settings.help = matches!(v, "0" | "false");
                    settings.help_locked = v == "2";
                }
                ("assigngmodename", Some(v)) if v.len() <= 64 && !v.chars().any(char::is_control) => settings.graphics.mode = v.into(),
                ("set800mode", Some(v)) if matches!(v, "0" | "1") => settings.graphics.high_resolution = v == "1",
                ("setbrightness", Some(v)) => {
                    if let Ok(n) = v.parse::<u8>() { settings.graphics.brightness = n.min(100); }
                }
                ("setshadowmask", Some(v)) => {
                    if let Ok(n) = v.parse::<u32>() { settings.graphics.shadow_mask = n; }
                }
                ("usedynamiclighting", Some(v)) if matches!(v, "0" | "1") => settings.graphics.dynamic_lighting = v == "1",
                _ => {}
            }
        }
        settings
    }

    pub fn write_script(&self) -> String {
        let [music, effects, speech] = self.volume.map(|v| f32::from(v) / 100.);
        format!("KQSound::setSpeechVol {speech:.2}\nKQSound::setSfxVol {effects:.2}\nKQSound::setMusicVol {music:.2}\nset Options::Captions {}\nnotStartingHelp {}\nassignGModeName {}\nset800mode {}\nsetBrightness {}\nsetShadowMask {}\nuseDynamicLighting {}\n",
            self.subtitles, if self.help_locked { 2 } else { u8::from(!self.help) },
            self.graphics.mode, u8::from(self.graphics.high_resolution), self.graphics.brightness,
            self.graphics.shadow_mask, u8::from(self.graphics.dynamic_lighting))
    }

    pub fn apply_groups(&self, groups: &mut Groups) {
        let [music, effects, speech] = self.volume.map(|v| f32::from(v) / 100.);
        groups.slider[..3].fill(effects);
        groups.slider[3] = speech;
        groups.slider[4..6].fill(music);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Row {
    Graphics,
    Keys,
    Music,
    Effects,
    Speech,
    Subtitles,
    Help,
    Credits,
    Back,
}

const ROWS: [Row; 9] = [
    Row::Graphics,
    Row::Keys,
    Row::Music,
    Row::Effects,
    Row::Speech,
    Row::Subtitles,
    Row::Help,
    Row::Credits,
    Row::Back,
];
// Subscreens need their own runtime implementation; keep their original labels visible.
const ACTIVE: [Row; 8] = [
    Row::Keys,
    Row::Music,
    Row::Effects,
    Row::Speech,
    Row::Subtitles,
    Row::Help,
    Row::Credits,
    Row::Back,
];

impl Row {
    fn label(self) -> &'static str {
        match self {
            Self::Graphics => "MODE_LABEL",
            Self::Keys => "REMAP_MOVEKEYS",
            Self::Music => "MUSICVOLUME_LABEL",
            Self::Effects => "SFXVOLUME_LABEL",
            Self::Speech => "SPEECHVOLUME_LABEL",
            Self::Subtitles => "SUBTITLES_LABEL",
            Self::Help => "STARTHELP_LABEL",
            Self::Credits => "CREDITS",
            Self::Back => "BACKTOMAIN",
        }
    }
    fn value(self) -> Option<&'static str> {
        Some(match self {
            Self::Music => "MUSICVOLUME_VALUE",
            Self::Effects => "SFXVOLUME_VALUE",
            Self::Speech => "SPEECHVOLUME_VALUE",
            Self::Subtitles => "SUBTITLES_VALUE",
            Self::Help => "STARTHELP_VALUE",
            _ => return None,
        })
    }
    fn slider(self) -> Option<usize> {
        match self {
            Self::Music => Some(0),
            Self::Effects => Some(1),
            Self::Speech => Some(2),
            _ => None,
        }
    }
}

pub struct Screen {
    layout: Layout,
    background: Canvas,
    row: Row,
    pub dragging: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    Stay,
    Keyboard,
    Credits,
    Back,
}

impl Screen {
    pub fn open(game: &Path, menu: &MenuArt) -> Result<Self, String> {
        let layout = Layout::read(game, "Options.gui", "IDCTL_OPT_")?;
        for name in ["BACKGROUND", "OPTIONS", "CURSOR1", "CURSOR2"]
            .into_iter()
            .chain(ROWS.iter().map(|r| r.label()))
            .chain(ROWS.iter().filter_map(|r| r.value()))
        {
            if !layout.controls.contains_key(name) {
                return Err(format!("Options.gui lacks {name}"));
            }
        }
        Ok(Self {
            background: Self::backdrop(&layout, menu),
            layout,
            row: Row::Keys,
            dragging: false,
        })
    }

    fn hit(&self, key: &str, x: i32, y: i32) -> bool {
        let [l, t, r, b] = self.layout.controls[key].rect;
        (l..r).contains(&x) && (t..b).contains(&y)
    }
    fn set_x(&self, x: i32, settings: &mut Settings) {
        if let (Some(i), Some(k)) = (self.row.slider(), self.row.value()) {
            let [l, _, r, _] = self.layout.controls[k].rect;
            let travel = (r - l - 10).max(1);
            settings.volume[i] =
                position(((100 * (x - l - 5).clamp(0, travel) + travel / 2) / travel) as u8);
        }
    }

    pub fn input(&mut self, key: Key, settings: &mut Settings) -> Destination {
        match key {
            Key::Escape => return Destination::Back,
            Key::Up | Key::Down | Key::Tab => {
                let i = ACTIVE.iter().position(|r| *r == self.row).unwrap_or(0);
                self.row =
                    ACTIVE[(i + if key == Key::Up { ACTIVE.len() - 1 } else { 1 }) % ACTIVE.len()];
            }
            Key::Click(x, y) => {
                for row in ACTIVE {
                    if self.hit(row.label(), x, y) || row.value().is_some_and(|k| self.hit(k, x, y))
                    {
                        self.row = row;
                        if row.slider().is_some() {
                            self.dragging = self.hit(row.value().unwrap(), x, y);
                            if self.dragging {
                                self.set_x(x, settings);
                            }
                        } else {
                            return self.toggle(settings);
                        }
                        break;
                    }
                }
            }
            Key::Drag(x, _) if self.dragging => self.set_x(x, settings),
            Key::MouseUp => self.dragging = false,
            Key::Left | Key::Right => {
                if let Some(i) = self.row.slider() {
                    let v = settings.volume[i];
                    settings.volume[i] = if key == Key::Left && v <= 10 {
                        0
                    } else if key == Key::Left {
                        v - 1
                    } else {
                        position(v.saturating_add(1))
                    };
                } else if self.row != Row::Back {
                    self.toggle(settings);
                }
            }
            Key::Enter => return self.toggle(settings),
            _ => {}
        }
        Destination::Stay
    }

    fn toggle(&self, s: &mut Settings) -> Destination {
        match self.row {
            Row::Subtitles => s.subtitles = !s.subtitles,
            Row::Help => { if !s.help_locked { s.help = !s.help; } },
            Row::Keys => return Destination::Keyboard,
            Row::Credits => return Destination::Credits,
            Row::Back => return Destination::Back,
            _ => {}
        }
        Destination::Stay
    }

    fn backdrop(layout: &Layout, menu: &MenuArt) -> Canvas {
        let mut out = Canvas::new(640, 480);
        let rect = |key: &str| layout.controls[key].rect;
        let draw = |c: &mut Canvas, key: &str, r, frame, caption| {
            layout.draw(c, menu, key, key, r, frame, caption)
        };
        for key in ["BACKGROUND", "OPTIONS"] {
            draw(&mut out, key, rect(key), 0, None);
        }
        for row in ROWS {
            let key = row.label();
            if ACTIVE.contains(&row) {
                draw(&mut out, key, rect(key), 0, None);
            } else {
                // Disabled items use the original menu's alternating pixels.
                let mut label = Canvas::new(640, 480);
                draw(&mut label, key, rect(key), 0, None);
                let [l, t, r, b] = rect(key);
                for y in t.clamp(0, 480)..b.clamp(0, 480) {
                    for x in l.clamp(0, 640)..r.clamp(0, 640) {
                        let p = ((y * 640 + x) * 4) as usize;
                        if (x + y) % 2 == 0 && label.rgba[p + 3] != 0 {
                            out.rgba[p..p + 4].copy_from_slice(&label.rgba[p..p + 4]);
                        }
                    }
                }
            }
        }
        out
    }

    pub fn render(&self, s: &Settings, menu: &MenuArt, frame: usize) -> Canvas {
        let mut out = self.background.clone();
        let rect = |key: &str| self.layout.controls[key].rect;
        let draw = |c: &mut Canvas, key: &str, r, frame, caption| {
            self.layout.draw(c, menu, key, key, r, frame, caption)
        };
        for row in ROWS {
            if let Some(k) = row.value() {
                let [l, t, r, b] = rect(k);
                if let Some(i) = row.slider() {
                    draw(&mut out, k, [l, t, r, b], 0, None);
                    let x = l + i32::from(s.volume[i]) * (r - l - 10) / 100;
                    draw(&mut out, k, [x, t, x + 10, b], 1, None);
                } else {
                    let yes = if row == Row::Subtitles {
                        s.subtitles
                    } else {
                        s.help
                    };
                    draw(
                        &mut out,
                        k,
                        [l, t, r, b],
                        0,
                        Some(menu.label(if yes { 29 } else { 14 })),
                    );
                }
            }
        }
        let selected = rect(self.row.label());
        for k in ["CURSOR1", "CURSOR2"] {
            let [l, t, r, b] = rect(k);
            let y = (selected[1] + selected[3] - (b - t)) / 2;
            draw(&mut out, k, [l, y, r, y + (b - t)], frame, None);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_off_starting_help_survives_preference_saves() {
        let s=Settings::from_script(b"notStartingHelp 2");
        assert!(s.help_locked && !s.help);
        assert!(s.write_script().contains("notStartingHelp 2"));
    }

    #[test]
    fn option_script_roundtrip_and_mixer_group_mapping() {
        let prefs=Settings::from_script(b"KQSound::setMusicVol 0\r\nKQSound::setSpeechVol 0.93\nKQSound::setSfxVol 0.05\nset Options::Captions false\nnotStartingHelp 1\n");
        assert_eq!(prefs.volume, [0, 5, 93]);
        assert!(!prefs.subtitles && !prefs.help);
        assert_eq!(
            Settings::from_script(prefs.write_script().as_bytes()),
            prefs
        );
        let mut groups = Groups::default();
        groups.duck[0] = 0.2;
        prefs.apply_groups(&mut groups);
        assert_eq!(groups.slider, [0.05, 0.05, 0.05, 0.93, 0.0, 0.0]);
        assert_eq!(groups.duck[0], 0.2);
    }

    #[test]
    fn graphics_commands_roundtrip_without_dropping_shadow_bits() {
        let s = Settings::from_script(b"assignGModeName D3D-display\nset800mode 1\nsetBrightness 49\nsetShadowMask 17\nuseDynamicLighting 1\n");
        assert_eq!(s.graphics.mode, "D3D-display");
        assert!(s.graphics.high_resolution && s.graphics.dynamic_lighting);
        assert_eq!(s.graphics.brightness, 49);
        assert_eq!(s.graphics.shadow_mask, 17);
        assert_eq!(Settings::from_script(s.write_script().as_bytes()), s);
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn installed_options_graphics_values_are_readable() {
        let dir = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let bytes = std::fs::read(Path::new(&dir).join("Options.cs")).unwrap();
        let s = Settings::from_script(&bytes);
        assert_eq!(s.graphics.mode, "D3D-display");
        assert!(!s.graphics.high_resolution && !s.graphics.dynamic_lighting);
        assert_eq!((s.graphics.brightness, s.graphics.shadow_mask), (49, 1));
    }

    #[test]
    #[ignore = "requires original controls and fonts in KQ8_GAME_DIR"]
    fn installed_options_click_drag_key_and_animated_masks() {
        let dir = std::env::var_os("KQ8_GAME_DIR").unwrap();
        let game = Path::new(&dir);
        let menu = MenuArt::open(game).unwrap();
        let mut page = Screen::open(game, &menu).unwrap();
        let mut settings = Settings::default();
        let still = page.render(&settings, &menu, 0);
        let spinning = page.render(&settings, &menu, 1);
        assert!(
            still.rgba != spinning.rgba,
            "Mskicng.pba has animated frames"
        );
        assert_eq!(still.rgba, page.render(&settings, &menu, 16).rgba);
        for (key, expected) in [
            (Key::Click(330, 198), 0),
            (Key::Right, 10),
            (Key::Left, 0),
            (Key::Drag(525, 198), 100),
        ] {
            assert_eq!(page.input(key, &mut settings), Destination::Stay);
            assert_eq!(settings.volume[0], expected);
        }
        page.input(Key::MouseUp, &mut settings);
        page.input(Key::Drag(333, 198), &mut settings);
        assert_eq!(settings.volume[0], 100);
        for k in [
            Key::Down,
            Key::Click(344, 228),
            Key::MouseUp,
            Key::Down,
            Key::Click(330, 258),
            Key::MouseUp,
        ] {
            page.input(k, &mut settings);
        }
        assert_eq!(settings.volume, [100, 10, 0]);
        page.input(Key::Click(481, 285), &mut settings);
        page.input(Key::Down, &mut settings);
        page.input(Key::Enter, &mut settings);
        assert!(!settings.subtitles && !settings.help);
        assert_eq!(
            page.input(Key::Click(315, 451), &mut settings),
            Destination::Back
        );
    }
}
