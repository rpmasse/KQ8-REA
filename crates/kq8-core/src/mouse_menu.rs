//! Viewer-only mouse camera preference, drawn with the installed Options art.
use crate::{gui_draw::Canvas, main_menu::{Art, Key}, menu_layout::Layout};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const MIN_PERCENT: u16 = 25;
pub const MAX_PERCENT: u16 = 200;
pub const STEP_PERCENT: u16 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MouseSettings {
    pub camera_sensitivity_percent: u16,
}

impl Default for MouseSettings {
    fn default() -> Self { Self { camera_sensitivity_percent: 100 } }
}

impl MouseSettings {
    pub fn valid(self) -> bool {
        (MIN_PERCENT..=MAX_PERCENT).contains(&self.camera_sensitivity_percent)
            && self.camera_sensitivity_percent % STEP_PERCENT == 0
    }

    pub fn camera_gain(self) -> f32 { f32::from(self.camera_sensitivity_percent) / 100.0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Row { Sensitivity, Back }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action { Stay, Back }

pub struct Screen {
    layout: Layout,
    background: Canvas,
    row: Row,
    dragging: bool,
}

impl Screen {
    pub fn open(game: &Path, art: &Art) -> Result<Self, String> {
        let mut layout = Layout::read(game, "Options.gui", "IDCTL_OPT_")?;
        layout.resize(art.size(),false);
        for key in ["BACKGROUND","OPTIONS","MUSICVOLUME_LABEL","MUSICVOLUME_VALUE",
            "BACKTOMAIN","CURSOR1","CURSOR2"] {
            if !layout.controls.contains_key(key) { return Err(format!("Options.gui lacks {key}")); }
        }
        let background = Self::backdrop(&layout,art);
        Ok(Self { layout, background, row: Row::Sensitivity, dragging: false })
    }

    fn backdrop(layout: &Layout, art: &Art) -> Canvas {
        let mut out = Canvas::new(layout.size[0],layout.size[1]);
        for key in ["BACKGROUND","BACKTOMAIN"] {
            layout.draw(&mut out,art,key,key,layout.controls[key].rect,0,None);
        }
        for (key,caption) in [("OPTIONS","Mouse"),
            ("MUSICVOLUME_LABEL","Camera Sensitivity")] {
            layout.draw(&mut out,art,key,key,layout.controls[key].rect,0,Some(caption));
        }
        out
    }

    pub fn resize(&mut self, art: &Art) {
        self.layout.resize(art.size(),false);
        self.background = Self::backdrop(&self.layout,art);
    }

    pub fn palette_changed(&mut self, art: &Art) {
        self.background = Self::backdrop(&self.layout,art);
    }

    fn hit(rect: [i32;4], x: i32, y: i32) -> bool {
        (rect[0]..rect[2]).contains(&x) && (rect[1]..rect[3]).contains(&y)
    }

    fn set_x(&self, x: i32, settings: &mut MouseSettings) {
        let [l,_,r,_] = self.layout.controls["MUSICVOLUME_VALUE"].rect;
        let travel = (r-l-10).max(1);
        let at = (x-l-5).clamp(0,travel);
        let percent = i32::from(MIN_PERCENT)
            + (i32::from(MAX_PERCENT-MIN_PERCENT)*at+travel/2)/travel;
        settings.camera_sensitivity_percent = ((percent+2)/5*5)
            .clamp(i32::from(MIN_PERCENT),i32::from(MAX_PERCENT)) as u16;
    }

    pub fn input(&mut self, key: Key, settings: &mut MouseSettings) -> Action {
        match key {
            Key::Escape => return Action::Back,
            Key::Up | Key::Down | Key::Tab => self.row = match self.row {
                Row::Sensitivity => Row::Back, Row::Back => Row::Sensitivity,
            },
            Key::Click(x,y) => {
                let label = self.layout.controls["MUSICVOLUME_LABEL"].rect;
                let value = self.layout.controls["MUSICVOLUME_VALUE"].rect;
                let back = self.layout.controls["BACKTOMAIN"].rect;
                if Self::hit(back,x,y) { self.row = Row::Back; return Action::Back; }
                if Self::hit(label,x,y) || Self::hit(value,x,y) {
                    self.row = Row::Sensitivity;
                    self.dragging = Self::hit(value,x,y);
                    if self.dragging { self.set_x(x,settings); }
                }
            }
            Key::Drag(x,_) if self.dragging => self.set_x(x,settings),
            Key::MouseUp => self.dragging = false,
            Key::Left if self.row == Row::Sensitivity => {
                settings.camera_sensitivity_percent = settings.camera_sensitivity_percent
                    .saturating_sub(STEP_PERCENT).max(MIN_PERCENT);
            }
            Key::Right if self.row == Row::Sensitivity => {
                settings.camera_sensitivity_percent = settings.camera_sensitivity_percent
                    .saturating_add(STEP_PERCENT).min(MAX_PERCENT);
            }
            Key::Enter if self.row == Row::Back => return Action::Back,
            _ => {}
        }
        Action::Stay
    }

    pub fn render(&self, settings: MouseSettings, art: &Art, frame: usize) -> Canvas {
        let mut out = self.background.clone();
        let [l,t,r,b] = self.layout.controls["MUSICVOLUME_VALUE"].rect;
        let key = "MUSICVOLUME_VALUE";
        self.layout.draw(&mut out,art,key,key,[l,t,r,b],0,None);
        let travel = r-l-10;
        let position = i32::from(settings.camera_sensitivity_percent.saturating_sub(MIN_PERCENT))
            *travel/i32::from(MAX_PERCENT-MIN_PERCENT);
        self.layout.draw(&mut out,art,key,key,[l+position,t,l+position+10,b],1,None);
        self.layout.draw(&mut out,art,"MUSICVOLUME_LABEL","MUSICVOLUME_LABEL",
            [r+8,t,self.layout.size[0] as i32-10,b],0,
            Some(&format!("{}%",settings.camera_sensitivity_percent)));
        let selected = self.layout.controls[match self.row {
            Row::Sensitivity => "MUSICVOLUME_LABEL", Row::Back => "BACKTOMAIN",
        }].rect;
        for key in ["CURSOR1","CURSOR2"] {
            let [cl,ct,cr,cb] = self.layout.controls[key].rect;
            let y = (selected[1]+selected[3]-(cb-ct))/2;
            self.layout.draw(&mut out,art,key,key,[cl,y,cr,y+cb-ct],frame,None);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn camera_sensitivity_slider_uses_both_menu_sizes_and_keyboard_mouse_input() {
        let game = std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR");
        let game = Path::new(&game);
        let mut art = Art::open(game).unwrap();
        for high in [false,true] {
            art.resize(high);
            let mut page = Screen::open(game,&art).unwrap();
            let mut settings = MouseSettings::default();
            let canvas = page.render(settings,&art,0);
            assert_eq!([canvas.width,canvas.height],art.size());
            let [l,t,r,b] = page.layout.controls["MUSICVOLUME_VALUE"].rect;
            assert!(r+8 < art.size()[0] as i32-10,"percentage must fit beside slider");
            let percent_rect = [r+8,t,art.size()[0] as i32-10,b];
            let mut visible = 0;
            for y in t..b { for x in percent_rect[0]..percent_rect[2] {
                if canvas.at(x as u32,y as u32) != page.background.at(x as u32,y as u32) {
                    visible += 1;
                }
            }}
            assert!(visible > 0,"current percentage must be visible");
            assert_eq!(page.input(Key::Right,&mut settings),Action::Stay);
            assert_eq!(settings.camera_sensitivity_percent,105);
            page.input(Key::Left,&mut settings);
            assert_eq!(settings,MouseSettings::default());
            page.input(Key::Click(l+1,(t+b)/2),&mut settings);
            assert_eq!(settings.camera_sensitivity_percent,MIN_PERCENT);
            page.input(Key::Drag(r-1,(t+b)/2),&mut settings);
            assert_eq!(settings.camera_sensitivity_percent,MAX_PERCENT);
            page.input(Key::MouseUp,&mut settings);
            page.input(Key::Drag(l+1,(t+b)/2),&mut settings);
            assert_eq!(settings.camera_sensitivity_percent,MAX_PERCENT);
            assert_ne!(page.render(settings,&art,0).rgba,canvas.rgba);
            assert_eq!(page.input(Key::Escape,&mut settings),Action::Back);
        }
    }

    #[test]
    fn default_gain_preserves_camera_input_and_limits_are_valid() {
        assert_eq!(MouseSettings::default().camera_gain(),1.0);
        assert_eq!(MouseSettings { camera_sensitivity_percent: MIN_PERCENT }.camera_gain(),0.25);
        assert_eq!(MouseSettings { camera_sensitivity_percent: MAX_PERCENT }.camera_gain(),2.0);
        assert!(!MouseSettings { camera_sensitivity_percent: 0 }.valid());
    }
}
