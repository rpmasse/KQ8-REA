//! The thirteen configurable controls in movekeys.kq and Movekey.gui.
use crate::{
    gui_draw::Canvas,
    main_menu::{Art, Key},
    menu_layout::Layout,
};
use kq8_data::kq_text;
use std::path::Path;

const ROWS: [&str; 13] = [
    "FORWARD",
    "BACK",
    "RIGHT",
    "LEFT",
    "RUN",
    "JUMP",
    "DRAW",
    "CTRCAMERA",
    "MAP",
    "MODIFY",
    "TGLCURS",
    "BTMBAR",
    "INVBAR",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bindings(pub [u8; 13]);

impl Default for Bindings {
    fn default() -> Self {
        Self([38, 40, 39, 37, 35, 45, 46, 36, 9, 16, 17, 18, 32])
    }
}

// Special keys use the localized names in 500.MSG. Printable keys have
// their own key name, as distinct from a numbered message in KeyName<n>.
fn special(v: u8) -> Option<(u8, &'static str)> {
    Some(match v {
        8 => (107, "backspace"),
        9 => (93, "tab"),
        13 => (108, "enter"),
        16 => (89, "shift"),
        17 => (90, "control"),
        18 => (106, "alt"),
        32 => (91, "space"),
        33 => (87, "prior"),
        34 => (88, "next"),
        35 => (80, "end"),
        36 => (92, "home"),
        37 => (83, "left"),
        38 => (84, "up"),
        39 => (81, "right"),
        40 => (85, "down"),
        45 => (82, "insert"),
        46 => (86, "delete"),
        112..=123 => (
            94 + v - 112,
            [
                "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12",
            ][(v - 112) as usize],
        ),
        _ => return None,
    })
}

/// KeyName uses a localized 500.MSG label for named keys, and the
/// printable key itself for a letter or digit.
pub fn localized_key(code: u8, mut label: impl FnMut(u8) -> Option<String>) -> String {
    if let Some((case, _)) = special(code) {
        return label(case).unwrap_or_default();
    }
    if code.is_ascii_graphic() {
        char::from(code).to_string()
    } else {
        String::new()
    }
}

pub fn keyword(v: u8) -> Option<String> {
    if v.is_ascii_uppercase() || v.is_ascii_digit() {
        Some(char::from(v).to_ascii_lowercase().to_string())
    } else {
        special(v).map(|(_, s)| s.to_owned())
    }
}

pub fn virtual_key(word: &str) -> Option<u8> {
    (0..=123).find(|&v| keyword(v).is_some_and(|s| s.eq_ignore_ascii_case(word)))
}

impl Bindings {
    pub fn read(bytes: &[u8]) -> Option<Self> {
        let sections = kq_text::parse(bytes);
        let section = sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case("MovekeysScreen"))?;
        let mut keys = [0; 13];
        for i in 0..keys.len() {
            let n = section.get(&format!("KeyChar{i}"))?.parse().ok()?;
            if keyword(n).is_none() || keys[..i].contains(&n) {
                return None;
            }
            keys[i] = n;
        }
        Some(Self(keys))
    }

    pub fn file(&self, template: &str) -> String {
        let mut lines = Vec::new();
        for line in template.lines() {
            let mut new = line.to_owned();
            if let Some((key, _)) = line.split_once('=') {
                let key = key.trim().to_ascii_lowercase();
                for (prefix, kind) in [("keychar", 0), ("keyname", 1), ("keyword", 2)] {
                    let Some(row) = key
                        .strip_prefix(prefix)
                        .and_then(|s| s.parse::<usize>().ok())
                    else {
                        continue;
                    };
                    if let Some(&v) = self.0.get(row) {
                        let value = match kind {
                            0 => v.to_string(),
                            1 => special(v)
                                .map(|(id, _)| format!("#{id}"))
                                .unwrap_or_else(|| char::from(v).to_string()),
                            _ => keyword(v).unwrap(),
                        };
                        new = format!("{}={value}", line.split_once('=').unwrap().0);
                    }
                }
            }
            lines.push(new);
        }
        lines.push(String::new());
        lines.join("\r\n")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Capture,
    Rejected,
    Bound(usize, u8),
    Back,
}

pub struct Screen {
    layout: Layout,
    background: Canvas,
    row: usize,
    pub capture: bool,
    rejected: bool,
}

impl Screen {
    pub fn open(path: &Path, art: &Art) -> Result<Self, String> {
        let layout = Layout::read(path, "movekey.gui", "IDCTL_MOVEKEYS_")?;
        let mut background = Canvas::new(640, 480);
        for name in ["BACKGROUND", "TITLE", "TO_MAIN", "CURS1", "CURS2"]
            .into_iter()
            .map(str::to_owned)
            .chain(
                ROWS.iter()
                    .flat_map(|r| [r.to_string(), format!("{r}_VAL")]),
            )
        {
            let control = layout
                .controls
                .get(&name)
                .ok_or_else(|| format!("movekey.gui lacks {name}"))?;
            if !name.starts_with("CURS") && !name.ends_with("_VAL") {
                layout.draw(&mut background, art, &name, &name, control.rect, 0, None);
            }
        }
        Ok(Self {
            layout,
            background,
            row: 0,
            capture: false,
            rejected: false,
        })
    }

    pub fn input(&mut self, key: Key, bindings: &mut Bindings) -> Action {
        if self.capture {
            if key == Key::Escape {
                self.capture = false;
                self.rejected = false;
                return Action::None;
            }
            let Key::Physical(v) = key else {
                return Action::None;
            };
            // These keys already operate the game outside the configurable rows
            // (Camera.sae, Connor.sae and interface.md, rules 14, 17 and 40).
            if keyword(v).is_none()
                || matches!(v, 33 | 34 | 49..=56 | 112 | 113 | 122)
                || bindings
                    .0
                    .iter()
                    .enumerate()
                    .any(|(i, k)| i != self.row && *k == v)
            {
                self.rejected = true;
                return Action::Rejected;
            }
            bindings.0[self.row] = v;
            self.capture = false;
            self.rejected = false;
            return Action::Bound(self.row, v);
        }
        match key {
            Key::Escape => return Action::Back,
            Key::Up => self.row = (self.row + ROWS.len()) % (ROWS.len() + 1),
            Key::Down | Key::Tab => self.row = (self.row + 1) % (ROWS.len() + 1),
            Key::Click(x, y) => {
                let hit = |key: &str| {
                    let [l, t, r, b] = self.layout.controls[key].rect;
                    (l..r).contains(&x) && (t..b).contains(&y)
                };
                if hit("TO_MAIN") {
                    return Action::Back;
                }
                let Some(i) = ROWS
                    .iter()
                    .position(|key| hit(key) || hit(&format!("{key}_VAL")))
                else {
                    return Action::None;
                };
                self.row = i;
                self.capture = true;
                return Action::Capture;
            }
            Key::Enter if self.row == ROWS.len() => return Action::Back,
            Key::Enter => {
                self.capture = true;
                return Action::Capture;
            }
            _ => {}
        }
        Action::None
    }

    pub fn render(&self, bindings: &Bindings, art: &Art, frame: usize) -> Canvas {
        let mut out = self.background.clone();
        for (i, row) in ROWS.iter().enumerate() {
            let key = format!("{row}_VAL");
            let code = bindings.0[i];
            let text = if i == self.row && self.capture {
                art.label(if self.rejected { 67 } else { 112 }).to_owned()
            } else {
                localized_key(code, |label| Some(art.label(label).to_owned()))
            };
            self.layout.draw(
                &mut out,
                art,
                &key,
                &key,
                self.layout.controls[&key].rect,
                0,
                Some(&text),
            );
        }
        let row = ROWS.get(self.row).copied().unwrap_or("TO_MAIN");
        let [_, t, _, b] = self.layout.controls[row].rect;
        for name in ["CURS1", "CURS2"] {
            let [l, top, r, bottom] = self.layout.controls[name].rect;
            let h = bottom - top;
            let y = (t + b - h) / 2;
            self.layout
                .draw(&mut out, art, name, name, [l, y, r, y + h], frame, None);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_and_printable_keys_have_distinct_numbers() {
        for code in 0..=123 {
            if let Some(word) = keyword(code) {
                assert_eq!(virtual_key(&word), Some(code));
            }
        }
        assert_eq!(virtual_key("F1"), Some(112));
        assert_eq!(virtual_key("p"), Some(80));
    }

    #[test]
    #[ignore = "requires the installed original Keyboard menu and message catalogue"]
    fn all_original_rows_capture_and_localize_keys_and_reject_duplicates() {
        let path = std::env::var_os("KQ8_GAME_DIR").unwrap();
        let dir = Path::new(&path);
        let file = kq8_data::GameDir::open(dir)
            .unwrap()
            .find("movekeys.kq")
            .unwrap();
        let template = std::fs::read_to_string(file).unwrap();
        let mut bindings = Bindings::read(template.as_bytes()).unwrap();
        let art = Art::open(dir).unwrap();
        let mut page = Screen::open(dir, &art).unwrap();
        let original = page.render(&bindings, &art, 0);
        assert_ne!(original.rgba, page.render(&bindings, &art, 1).rgba);
        assert_eq!(original.rgba, page.render(&bindings, &art, 16).rgba);
        assert!(!art.label(67).is_empty() && !art.label(112).is_empty());
        for i in 0..ROWS.len() {
            let [l, t, _, _] = page.layout.controls[ROWS[i]].rect;
            assert_eq!(
                page.input(Key::Click(l + 1, t + 1), &mut bindings),
                Action::Capture
            );
            let prompted = page.render(&bindings, &art, 0);
            let other = bindings.0[(i + 1) % ROWS.len()];
            assert_eq!(
                page.input(Key::Physical(other), &mut bindings),
                Action::Rejected
            );
            assert_ne!(prompted.rgba, page.render(&bindings, &art, 0).rgba);
            let code = b'A' + i as u8;
            assert_eq!(
                page.input(Key::Physical(code), &mut bindings),
                Action::Bound(i, code)
            );
        }
        let written = bindings.file(&template);
        assert_eq!(Bindings::read(written.as_bytes()), Some(bindings));
        assert!(written.contains("KeyName0=A") && written.contains("KeyWord0=a"));
        assert_eq!(page.input(Key::Down, &mut bindings), Action::None);
        assert_eq!(page.input(Key::Enter, &mut bindings), Action::Back);
    }
}
