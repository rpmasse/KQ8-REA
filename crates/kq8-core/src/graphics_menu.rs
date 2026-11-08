//! Original Graphics.gui layout and explicitly supplied device policy.
use crate::{
    graphics_devices::{Device, Family, Inventory, Selection},
    gui_draw::Canvas,
    main_menu::{Art, Key},
    menu_layout::Layout,
    options_menu::Graphics,
};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Row {
    DirectDraw,
    Direct3D,
    Glide,
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

const ROWS: [Row; 12] = [
    Row::DirectDraw,
    Row::Direct3D,
    Row::Glide,
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
            Self::DirectDraw => "DIRECTDRAW",
            Self::Direct3D => "DIRECT3D",
            Self::Glide => "3DFX",
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
    fn family(self) -> Option<Family> {
        match self {
            Self::DirectDraw => Some(Family::Software),
            Self::Direct3D => Some(Family::Direct3D),
            Self::Glide => Some(Family::Glide),
            _ => None,
        }
    }
    fn caption<'a>(self, layout: &Layout, art: &'a Art) -> &'a str {
        // Final IDSTR_DIRECTDRAW / IDSTR_DIRECT3D / IDSTR_3DFX tags in
        // kqGuiTag.TTAG. Graphics.gui's device text is a placeholder.
        match self {
            Self::DirectDraw => art.label(7),
            Self::Direct3D => art.label(8),
            Self::Glide => art.label(1),
            _ => layout.caption(self.key(), art).unwrap_or(""),
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
    devices: Option<(Inventory, Device)>,
    selection: Option<DeviceAction>,
    restart: Option<Restart>,
}

/// Only accepted UI decisions reach the presentation owner. Opening the
/// confirmation is internal; it must never be mistaken for permission to exit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceAction {
    Live(Device),
    RestartSoftware,
}

struct Restart {
    answer: usize,
    key_pressed: bool,
    mouse_pressed: Option<usize>,
    pointer: Option<[i32; 2]>,
}

const ANSWERS: [[i32; 4]; 2] = [[188, 213, 276, 238], [361, 213, 449, 238]];

fn answer_at([x,y]: [i32; 2]) -> Option<usize> {
    ANSWERS.iter().position(|&[l,t,r,b]| (l..=r).contains(&x) && (t..=b).contains(&y))
}

/// The original question uses character-count thirds, preferring the later
/// space at equal distance, then halves the remaining text (Q207 addendum).
fn question_lines(text: &str) -> [String; 3] {
    fn split(text: &str, divisor: usize) -> (&str, &str) {
        let target = text.chars().count() / divisor;
        let space = text.char_indices().enumerate().filter(|(_,(_,c))| *c == ' ')
            .min_by_key(|(index,_)| (index.abs_diff(target), std::cmp::Reverse(*index)));
        space.map_or((text,""), |(_, (byte,_))| (&text[..byte], &text[byte+1..]))
    }
    let (first, rest) = split(text, 3);
    let (second, third) = split(rest, 2);
    [first.into(), second.into(), third.into()]
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
            "POP_BKGND", "TEXT1", "TEXT2", "TEXT3", "BTN1", "BTN2",
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
        Ok(Self { layout, background, row: Row::Low, dragging: false, high_available: true,
            devices: None, selection: None, restart: None })
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

    /// The presentation owner supplies actual logical device facts. Existing
    /// callers without a profile retain their independently supplied 640/800
    /// capability; no legacy inventory is inferred from the host GPU.
    pub fn set_device_inventory(&mut self, inventory: Inventory, active: Device) {
        self.set_high_resolution_available(active.supports_800());
        self.devices = Some((inventory, active));
        self.selection = None;
        self.restart = None;
        self.row = Row::Low;
    }

    /// Acknowledge a successful live presentation change. Choosing a row
    /// alone never changes the active-device marker or saved mode name.
    pub fn set_active_device(&mut self, active: Device) {
        if self.devices.is_none() { return; }
        self.set_high_resolution_available(active.supports_800());
        self.devices.as_mut().unwrap().1 = active;
        self.selection = None;
    }

    pub fn take_device_selection(&mut self) -> Option<DeviceAction> { self.selection.take() }

    pub fn modal_open(&self) -> bool { self.restart.is_some() }

    /// The frontend owns saving/restoring the actual system cursor. These
    /// are original GUI coordinates, not a claim about desktop adaptation.
    pub fn key_pointer(&self) -> Option<[i32; 2]> {
        let restart = self.restart.as_ref().filter(|r| r.key_pressed)?;
        let [l,t,r,b] = ANSWERS[restart.answer];
        Some([(l+r)/2, (t+b)/2+16])
    }

    /// Feed normal GUI pointer coordinates before MouseUp. Pointer movement
    /// cannot change the keyboard answer or its temporary held-Enter pointer.
    pub fn pointer_moved(&mut self, point: [i32; 2]) {
        if let Some(restart) = &mut self.restart {
            if !restart.key_pressed { restart.pointer = Some(point); }
        }
    }

    /// Call before input(), as for the Main menu's sound contract.
    pub fn sound(&self, key: Key) -> Option<&'static str> {
        if let Some(restart) = &self.restart {
            return match key {
                Key::Enter if !restart.key_pressed => Some("button1.aud"),
                Key::Left if restart.answer == 1 && !restart.key_pressed => Some("button1.aud"),
                Key::Right if restart.answer == 0 && !restart.key_pressed => Some("button1.aud"),
                _ => None,
            };
        }
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right | Key::Tab => Some("click.aud"),
            Key::Click(_,_) | Key::Enter => Some("intrface.aud"),
            _ => None,
        }
    }

    fn answer_restart(&mut self, answer: usize) {
        self.restart = None;
        self.dragging = false;
        if answer == 0 { self.selection = Some(DeviceAction::RestartSoftware); }
    }

    fn restart_input(&mut self, key: Key) -> Action {
        let restart = self.restart.as_mut().unwrap();
        match key {
            Key::Escape => {
                self.restart = None;
                self.dragging = false;
                return Action::Back;
            }
            Key::Release if restart.key_pressed => {
                let answer = restart.answer;
                self.answer_restart(answer);
            }
            _ if restart.key_pressed => {},
            Key::Left => restart.answer = 0,
            Key::Right => restart.answer = 1,
            Key::Enter => restart.key_pressed = true,
            Key::Click(x,y) => {
                restart.pointer = Some([x,y]);
                restart.mouse_pressed = answer_at([x,y]);
                self.dragging = restart.mouse_pressed.is_some();
            }
            Key::Drag(x,y) => restart.pointer = Some([x,y]),
            Key::MouseUp => {
                let hit = restart.pointer.and_then(answer_at);
                let pressed = restart.mouse_pressed.take();
                self.dragging = false;
                if let Some(answer) = pressed.filter(|answer| Some(*answer) == hit) {
                    self.answer_restart(answer);
                }
            }
            _ => {},
        }
        Action::Stay
    }

    fn available(&self, row: Row, high_resolution: bool) -> bool {
        if let Some(family) = row.family() {
            return self.devices.as_ref().is_some_and(|(inventory, _)| inventory.choice(family, high_resolution).is_some());
        }
        row != Row::High || self.high_available
    }

    fn backdrop(layout: &Layout, art: &Art) -> Canvas {
        let mut background = Canvas::new(layout.size[0], layout.size[1]);
        for key in [
            "BACKGROUND",
            "TITLE",
            "MODE",
            "RESOLUTION",
        ]
        .into_iter()
        .chain(ROWS.iter().filter(|r| r.family().is_none() && !matches!(r, Row::Low | Row::High)).map(|r| r.key()))
        {
            layout.draw(
                &mut background,
                art,
                key,
                key,
                layout.controls[key].rect,
                0,
                None,
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
        if self.modal_open() { return None; }
        if (width,height) != (self.background.width,self.background.height) || rgba.len() != width as usize*height as usize*4 { return None; }
        let mut dynamic: Vec<[i32;4]> = ROWS.iter().filter_map(|r|r.value()).map(|key|self.layout.controls[key].rect).collect();
        dynamic.extend([Row::DirectDraw, Row::Direct3D, Row::Glide, Row::Low, Row::High].map(|row| self.layout.controls[row.key()].rect));
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

    fn choose(&mut self, settings: &mut Graphics) -> Action {
        if let Some(family) = self.row.family() {
            if let Some((inventory, active)) = &self.devices {
                match inventory.select(active, family, settings.high_resolution) {
                    Some(Selection::Live(device)) => self.selection = Some(DeviceAction::Live(device)),
                    Some(Selection::ConfirmSoftwareRestart) => {
                        self.selection = None;
                        self.dragging = false;
                        self.restart = Some(Restart { answer: 1, key_pressed: false, mouse_pressed: None, pointer: None });
                    }
                    None => {},
                }
            }
            return Action::Stay;
        }
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
        if self.restart.is_some() { return self.restart_input(key); }
        match key {
            Key::Escape => return Action::Back,
            Key::Up | Key::Down | Key::Tab => {
                let i = ROWS.iter().position(|r| *r == self.row).unwrap();
                let by = if key == Key::Up { ROWS.len() - 1 } else { 1 };
                self.row = (1..=ROWS.len()).map(|step| ROWS[(i + by * step) % ROWS.len()])
                    .find(|&row| self.available(row, settings.high_resolution)).unwrap();
            }
            Key::Click(x, y) => {
                for row in ROWS {
                    if self.available(row, settings.high_resolution) && (self.hit(row.key(), x, y) || row.value().is_some_and(|v| self.hit(v, x, y))) {
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
        for row in [Row::DirectDraw, Row::Direct3D, Row::Glide, Row::Low, Row::High] {
            let key = row.key();
            let selected = match row {
                Row::Low => !settings.high_resolution,
                Row::High => settings.high_resolution,
                _ => self.devices.as_ref().is_some_and(|(_, active)| row.family() == Some(active.family)),
            };
            let label = row.caption(&self.layout, art);
            let marked = if selected { format!("> {label} <") } else { label.to_owned() };
            let rect = self.layout.controls[key].rect;
            // Until a caller supplies an inventory, preserve the existing
            // backdrop labels instead of asserting historical availability.
            if self.available(row, settings.high_resolution) || (row.family().is_some() && self.devices.is_none()) {
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
        if let Some(restart) = &self.restart {
            self.layout.draw(&mut out,art,"POP_BKGND","POP_BKGND",[140,135,500,255],0,None);
            for (i, line) in question_lines(art.label(169)).iter().enumerate() {
                let key = ["TEXT1","TEXT2","TEXT3"][i];
                let top = 152 + i as i32 * 18;
                self.layout.draw(&mut out,art,key,key,[170,top,470,top+20],0,Some(line));
            }
            for (answer, rect) in ANSWERS.into_iter().enumerate() {
                let key = if answer == restart.answer { "BTN2" } else { "BTN1" };
                let pressed = (restart.key_pressed && answer == restart.answer)
                    || (restart.mouse_pressed == Some(answer) && restart.pointer.and_then(answer_at) == Some(answer));
                self.layout.draw(&mut out,art,key,key,rect,usize::from(pressed),Some(""));
                if let Some(text) = self.layout.controls[key].text {
                    let font = &self.layout.fonts[&text.font];
                    let mut caption = art.label(if answer == 0 {29} else {35});
                    while !caption.is_empty() && crate::gui_draw::text_width(font,caption) > rect[2]-rect[0] {
                        caption = &caption[..caption.char_indices().next_back().unwrap().0];
                    }
                    let caption_rect = rect.map(|edge| edge + i32::from(pressed));
                    out.text(caption_rect,font,text.justification,caption,1.0,&art.palette);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires KQ8_GAME_DIR; headless original-data device menu matrix"]
    fn graphics_device_menu_grouped_original_data_selection_and_availability() {
        use crate::graphics_devices::GlideBoard;
        let path = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let game = Path::new(&path);
        let mut art = Art::open(game).unwrap();
        let software = Device::software(true);
        let d3d = Device::direct3d("display", true);
        let glide = Device::glide(true, Some(GlideBoard { board_type: 0, framebuffer_mb: 4 }));
        let center = |screen: &Screen, row: Row| {
            let [l,t,r,b] = screen.layout.controls[row.key()].rect;
            Key::Click((l+r)/2, (t+b)/2)
        };
        let assert_row = |screen: &Screen, settings: &Graphics, art: &Art, row: Row, text: &str, enabled: bool| {
            let actual = screen.render(settings, art, 0);
            let key = row.key();
            let [l,t,r,b] = screen.layout.controls[key].rect;
            let mut glyphs = Canvas::new(actual.width, actual.height);
            screen.layout.draw(&mut glyphs, art, key, key, [l,t,r,b], 0, Some(text));
            let mut visible = 0;
            let mut skipped = 0;
            for y in t..b { for x in l..r {
                let pixel = glyphs.at(x as u32, y as u32);
                if pixel[3] == 0 { continue; }
                if enabled || (x+y)%2 == 0 {
                    assert_eq!(actual.at(x as u32,y as u32), pixel, "{key}: selected/localized glyph at {x},{y}");
                    visible += 1;
                } else {
                    assert_eq!(actual.at(x as u32,y as u32), screen.background.at(x as u32,y as u32),
                        "{key}: disabled glyph retains the original checker gaps");
                    skipped += 1;
                }
            }}
            assert!(visible > 0);
            if !enabled { assert!(skipped > 0); }
        };
        for high_art in [false, true] {
            art.resize(high_art);
            let mut screen = Screen::open(game, &art).unwrap();
            for (row, label) in [(Row::DirectDraw,"DirectDraw"), (Row::Direct3D,"Direct3D"), (Row::Glide,"3Dfx")] {
                assert_eq!(row.caption(&screen.layout, &art), label, "final original text tag, not GUI placeholder");
            }
            let mut settings = Graphics { mode: "Software".into(), brightness: 37, shadow_mask: 13,
                dynamic_lighting: false, high_resolution: false };
            screen.set_device_inventory(Inventory::new(vec![software.clone(), d3d.clone()]), software.clone());
            let initial = screen.render(&settings, &art, 0);
            let palette_sample = screen.palette_sample(&initial.rgba, initial.width, initial.height).unwrap();
            assert_row(&screen, &settings, &art, Row::DirectDraw, "> DirectDraw <", true);
            assert_row(&screen, &settings, &art, Row::Direct3D, "Direct3D", true);
            assert_row(&screen, &settings, &art, Row::Glide, "3Dfx", false);
            let before = settings.clone();
            assert_eq!(screen.input(center(&screen, Row::Direct3D), &mut settings), Action::Stay);
            assert_eq!(screen.take_device_selection(), Some(DeviceAction::Live(d3d.clone())));
            assert_eq!(screen.take_device_selection(), None, "request is delivered once");
            assert_eq!(settings, before, "selection alone cannot commit preferences");
            assert_row(&screen, &settings, &art, Row::DirectDraw, "> DirectDraw <", true);
            screen.set_active_device(d3d.clone());
            settings.mode = d3d.name.clone(); // presentation-owner acknowledgement
            assert_row(&screen, &settings, &art, Row::Direct3D, "> Direct3D <", true);
            screen.input(center(&screen, Row::Direct3D), &mut settings);
            assert_eq!(screen.take_device_selection(), None, "same active device is a no-op");
            screen.set_active_device(Device::direct3d("another-driver",true));
            assert_row(&screen, &settings, &art, Row::Direct3D, "> Direct3D <", true);
            screen.input(center(&screen, Row::Direct3D), &mut settings);
            assert_eq!(screen.take_device_selection(), None, "same family with another retained driver is also a no-op");
            screen.set_active_device(d3d.clone());
            if !high_art {
                // Only the established 640 modal path: DirectDraw is disabled
                // at800, so forced800 popup geometry is not inferred here.
                let warning = art.label(169);
                let lines = question_lines(warning);
                // Tag169 resolves the catalogue warning, not the short TTAG
                // placeholder. Keep only its measured boundaries in source.
                assert_eq!(warning.chars().count(),106);
                assert_eq!(lines.each_ref().map(|line| line.chars().count()),[37,33,34]);
                assert_eq!(lines.join(" "),warning,"splitting preserves the resolved original message");
                assert_eq!(question_lines("abc d efghijk"), ["abc d", "efghijk", ""].map(str::to_owned), "later space wins a tie");
                let before = settings.clone();
                screen.input(center(&screen, Row::DirectDraw), &mut settings);
                assert!(screen.modal_open());
                assert_eq!(screen.take_device_selection(), None, "opening is not acceptance");
                assert_eq!(screen.key_pointer(), None, "opening does not move the pointer");
                assert_eq!(screen.restart.as_ref().unwrap().answer, 1, "Cancel is the default, not No");
                assert_eq!(screen.sound(Key::Right), None);
                assert_eq!(screen.sound(Key::Left), Some("button1.aud"));
                screen.input(Key::Left, &mut settings);
                assert_eq!(screen.sound(Key::Left), None, "arrow ends do not wrap or make a sound");
                for key in [Key::Up,Key::Down,Key::Tab] { screen.input(key, &mut settings); }
                assert_eq!(screen.restart.as_ref().unwrap().answer, 0);
                assert_eq!(screen.key_pointer(), None, "answer navigation does not move the pointer");
                assert_eq!(screen.sound(Key::Right), Some("button1.aud"));
                screen.input(Key::Right, &mut settings);
                screen.input(center(&screen,Row::Brightness), &mut settings);
                screen.input(Key::MouseUp, &mut settings);
                screen.input(center(&screen,Row::Back), &mut settings);
                screen.input(Key::MouseUp, &mut settings);
                assert!(screen.modal_open(), "underlying rows and Back are locked");
                assert_eq!(settings, before);
                let normal = screen.render(&settings,&art,0);
                assert_eq!(screen.sound(Key::Enter),Some("button1.aud"));
                screen.input(Key::Enter, &mut settings);
                assert_eq!(screen.key_pointer(), Some([405,241]));
                screen.pointer_moved([0,0]);
                screen.input(Key::Drag(0,0), &mut settings);
                assert_eq!(screen.key_pointer(), Some([405,241]), "held Enter owns the pointer");
                let pressed = screen.render(&settings,&art,0);
                assert_ne!(pressed.rgba,normal.rgba,"held button uses depressed artwork");
                let game_dir = kq8_data::GameDir::open(game).unwrap();
                let button_bytes = std::fs::read(game_dir.find("GAME/8Gui/g_yhibtn.pba").unwrap()).unwrap();
                let buttons = kq8_data::font::bitmap_array(&button_bytes).unwrap();
                assert_eq!(buttons.len(),2);
                assert_eq!((buttons[1].width,buttons[1].height),(88,36));
                let mut expected = screen.background.clone();
                screen.layout.draw(&mut expected,&art,"POP_BKGND","POP_BKGND",[140,135,500,255],0,None);
                expected.picture([361,213,449,238],crate::gui_draw::Pic::of(&buttons[1]).unwrap(),true,1.0,&art.palette);
                let text = screen.layout.controls["BTN2"].text.unwrap();
                expected.text([362,214,450,239],&screen.layout.fonts[&text.font],text.justification,art.label(35),1.0,&art.palette);
                for y in 213..238 { for x in 361..449 {
                    assert_eq!(pressed.at(x,y),expected.at(x,y),"pressed bitmap stretched88x25, caption shifted1px at{x},{y}");
                }}
                assert_eq!(screen.take_device_selection(),None,"press alone cannot invoke Cancel");
                screen.input(Key::Release, &mut settings);
                assert!(!screen.modal_open());
                assert_eq!(screen.key_pointer(),None,"frontend can restore saved pointer on this edge");
                assert_eq!(screen.take_device_selection(),None);
                assert_eq!(settings,before);

                screen.input(center(&screen,Row::DirectDraw), &mut settings);
                screen.input(Key::Left, &mut settings);
                screen.input(Key::Enter, &mut settings);
                assert_eq!(screen.key_pointer(),Some([232,241]));
                assert_eq!(screen.take_device_selection(),None);
                screen.input(Key::Release, &mut settings);
                assert_eq!(screen.take_device_selection(),Some(DeviceAction::RestartSoftware));
                assert_eq!(settings,before,"the presentation owner saves Software before exit");

                screen.input(center(&screen,Row::DirectDraw), &mut settings);
                screen.pointer_moved([232,225]);
                assert_eq!(screen.restart.as_ref().unwrap().answer,1,"hover does not change keyboard selection");
                screen.input(Key::Click(188,213), &mut settings);
                assert!(screen.modal_open());
                assert_eq!(screen.take_device_selection(),None,"mouse press alone does not invoke Yes");
                screen.input(Key::MouseUp, &mut settings);
                assert_eq!(screen.take_device_selection(),Some(DeviceAction::RestartSoftware),"inclusive top/left boundary and actual pointer answer");

                screen.input(center(&screen,Row::DirectDraw), &mut settings);
                screen.input(Key::Left, &mut settings);
                screen.input(Key::Click(449,238), &mut settings);
                screen.input(Key::MouseUp, &mut settings);
                assert!(!screen.modal_open());
                assert_eq!(screen.take_device_selection(),None,"right-side Cancel remains Cancel after selecting Yes; bottom/right boundaries inclusive");
                assert_eq!(settings,before);
                screen.input(center(&screen,Row::DirectDraw), &mut settings);
                assert_eq!(screen.input(Key::Escape,&mut settings),Action::Back,"Escape goes to Options");
                assert!(!screen.modal_open());
                assert_eq!(screen.take_device_selection(),None);
            }
            screen.input(center(&screen, Row::High), &mut settings);
            assert!(!settings.high_resolution, "Direct3D has no original800 capability");
            screen.input(center(&screen, Row::Glide), &mut settings);
            assert_eq!(screen.take_device_selection(), None, "disabled row cannot request a device");

            screen.set_device_inventory(Inventory::new(vec![software.clone(), d3d.clone(), glide.clone()]), glide.clone());
            settings.mode = "Glide".into();
            screen.input(center(&screen, Row::High), &mut settings);
            assert!(settings.high_resolution);
            assert_row(&screen, &settings, &art, Row::DirectDraw, "DirectDraw", false);
            assert_row(&screen, &settings, &art, Row::Direct3D, "Direct3D", false);
            assert_row(&screen, &settings, &art, Row::Glide, "> 3Dfx <", true);
            screen.row = Row::Low;
            screen.input(Key::Up, &mut settings);
            assert_eq!(screen.row, Row::Glide);
            screen.input(Key::Up, &mut settings);
            assert_eq!(screen.row, Row::Back, "navigation skips both disabled device rows");
            screen.input(center(&screen, Row::DirectDraw), &mut settings);
            assert_eq!(screen.take_device_selection(), None);
            screen.input(center(&screen, Row::Low), &mut settings);
            screen.input(center(&screen, Row::DirectDraw), &mut settings);
            assert_eq!(screen.take_device_selection(), Some(DeviceAction::Live(software.clone())), "Glide to Software is live");
            assert_eq!((settings.brightness,settings.shadow_mask,settings.dynamic_lighting), (37,13,false));
            let changed = screen.render(&settings, &art, 7);
            assert_eq!(screen.palette_sample(&changed.rgba,changed.width,changed.height).unwrap(), palette_sample,
                "dynamic device text is excluded from retained brightness evidence");

            screen.set_device_inventory(Inventory::new(vec![software.clone(),d3d.clone(),Device::glide(false,None)]),software.clone());
            settings.mode = "Software".into();
            assert_row(&screen, &settings, &art, Row::Direct3D, "Direct3D", false);
            screen.input(Key::Up, &mut settings);
            assert_eq!(screen.row, Row::DirectDraw, "unavailable Glide still suppresses Direct3D");
        }
        println!("device GUI matrix: original captions, markers/checker, supplied availability, navigation, request/acknowledgement, restart decision and unchanged settings at640/800");
    }

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
