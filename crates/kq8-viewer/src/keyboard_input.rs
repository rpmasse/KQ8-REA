//! Translate the player's configured physical keys before gameplay reads
//! its action maps. Menus keep the physical keyboard for navigation and
//! binding capture. An old binding becomes available for another action.
use bevy::prelude::*;
use kq8_core::keyboard_menu::{self, Bindings};
use kq8_data::GameDir;
use std::{fs, path::Path};

const LETTERS: [KeyCode; 26] = {
    use KeyCode::*;
    [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO,
        KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    ]
};
const NUMBERS: [KeyCode; 10] = {
    use KeyCode::*;
    [
        Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    ]
};
const FUNCTIONS: [KeyCode; 12] = {
    use KeyCode::*;
    [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12]
};
const SPECIAL: [(u8, KeyCode); 18] = {
    use KeyCode::*;
    [
        (8, Backspace),
        (9, Tab),
        (13, Enter),
        (16, ShiftLeft),
        (17, ControlLeft),
        (18, AltLeft),
        (27, Escape),
        (32, Space),
        (33, PageUp),
        (34, PageDown),
        (35, End),
        (36, Home),
        (37, ArrowLeft),
        (38, ArrowUp),
        (39, ArrowRight),
        (40, ArrowDown),
        (45, Insert),
        (46, Delete),
    ]
};

pub fn physical(code: u8) -> Option<KeyCode> {
    match code {
        b'A'..=b'Z' => Some(LETTERS[(code - b'A') as usize]),
        b'0'..=b'9' => Some(NUMBERS[(code - b'0') as usize]),
        112..=123 => Some(FUNCTIONS[(code - 112) as usize]),
        _ => SPECIAL.iter().find(|(vk, _)| *vk == code).map(|(_, k)| *k),
    }
}

pub fn virtual_key(key: KeyCode) -> Option<u8> {
    let key = match key {
        KeyCode::ShiftRight => KeyCode::ShiftLeft,
        KeyCode::AltRight => KeyCode::AltLeft,
        KeyCode::ControlRight => KeyCode::ControlLeft,
        KeyCode::NumpadEnter => KeyCode::Enter,
        other => other,
    };
    for (base, keys) in [
        (b'A', LETTERS.as_slice()),
        (b'0', NUMBERS.as_slice()),
        (112, FUNCTIONS.as_slice()),
    ] {
        if let Some(i) = keys.iter().position(|k| *k == key) {
            return Some(base + i as u8);
        }
    }
    SPECIAL.iter().find(|(_, k)| *k == key).map(|(v, _)| *v)
}

#[derive(Resource)]
pub struct Keyboard {
    installed: Bindings,
    saved: Bindings,
    pub active: Bindings,
    template: String,
}

impl Keyboard {
    fn load(game: &Path) -> Self {
        let original = GameDir::open(game)
            .ok()
            .and_then(|game| game.find("movekeys.kq"))
            .and_then(|p| fs::read(p).ok())
            .unwrap_or_default();
        let installed = Bindings::read(&original).unwrap_or_default();
        let active = fs::read("kq8-artifacts/movekeys.kq")
            .ok()
            .and_then(|b| Bindings::read(&b))
            .unwrap_or(installed);
        Self {
            installed,
            active,
            saved: active,
            template: String::from_utf8_lossy(&original).into_owned(),
        }
    }

    pub fn store(&mut self) -> Result<(), String> {
        if self.active == self.saved {
            return Ok(());
        }
        if Bindings::read(self.template.as_bytes()).is_none() {
            return Err("invalid or missing original movekeys.kq".into());
        }
        crate::option_settings::write_preferences(
            Path::new("kq8-artifacts/movekeys.kq"),
            self.active.file(&self.template).as_bytes(),
        )?;
        self.saved = self.active;
        Ok(())
    }

    fn logical(&self, key: KeyCode) -> Option<KeyCode> {
        let Some(code) = virtual_key(key) else {
            return Some(key);
        };
        if let Some(row) = self.active.0.iter().position(|v| *v == code) {
            return physical(self.installed.0[row]);
        }
        (!self.installed.0.contains(&code)).then_some(key)
    }
}

#[derive(Default, Resource, Deref)]
pub struct GameKeys {
    #[deref]
    buttons: ButtonInput<KeyCode>,
    original: Bindings,
}

impl GameKeys {
    pub fn modify_held(&self) -> bool {
        physical(self.original.0[9]).is_some_and(|k| self.pressed(k))
    }

    pub fn change_cursor_held(&self) -> bool {
        physical(self.original.0[10]).is_some_and(|k| self.pressed(k))
    }
}

pub fn register(app: &mut App, game: &Path) {
    app.insert_resource(Keyboard::load(game))
        .init_resource::<GameKeys>()
        .add_systems(
            PreUpdate,
            (scripted_key, translate)
                .chain()
                .after(bevy::input::InputSystems),
        );
}

fn translate(raw: Res<ButtonInput<KeyCode>>, map: Res<Keyboard>, mut keys: ResMut<GameKeys>) {
    // Retain the make/break semantics when both left and right modifiers
    // map to the same control, or a complete tap happens within one frame.
    let pressed: Vec<_> = raw.get_pressed().filter_map(|k| map.logical(*k)).collect();
    let old: Vec<_> = keys.get_pressed().copied().collect();
    keys.original = map.installed;
    keys.buttons.clear();
    for k in old {
        if !pressed.contains(&k) {
            keys.buttons.release(k);
        }
    }
    for &k in &pressed {
        keys.buttons.press(k);
    }
    for &k in raw.get_just_pressed().filter(|k| raw.just_released(**k)) {
        let Some(key) = map.logical(k) else { continue };
        if raw
            .get_pressed()
            .any(|other| *other != k && map.logical(*other) == Some(key))
        {
            continue;
        }
        if raw.pressed(k) {
            keys.buttons.release(key);
            keys.buttons.press(key);
        } else {
            keys.buttons.press(key);
            keys.buttons.release(key);
        }
    }
}

// The acceptance test uses a single physical keypress. It travels
// through the same mapping and gameplay input systems as a real key.
fn scripted_key(
    queue: Option<ResMut<crate::gui_view::KeyAt>>,
    sim: Option<Res<crate::door_view::DoorSim>>,
    menu: Option<Res<crate::startup_menu::Menu>>,
    mut raw: ResMut<ButtonInput<KeyCode>>,
    mut release: Local<Vec<KeyCode>>,
) {
    for k in release.drain(..) {
        raw.release(k);
    }
    let (Some(mut queue), Some(sim), None) = (queue, sim, menu) else {
        return;
    };
    queue.0.retain(|(name, tick)| {
        let Some(word) = name.strip_prefix("physical:") else {
            return true;
        };
        if *tick > sim.active.world.updates {
            return true;
        }
        if let Some(key) = keyboard_menu::virtual_key(word).and_then(physical) {
            raw.press(key);
            release.push(key);
        }
        false
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_thirteen_actions_receive_only_the_assigned_press_and_release() {
        let installed = Bindings::default();
        let mut configured = installed;
        for (i, v) in configured.0.iter_mut().enumerate() {
            *v = b'A' + i as u8;
        }
        let mut app = App::new();
        app.insert_resource(Keyboard {
            installed,
            saved: configured,
            active: configured,
            template: String::new(),
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<GameKeys>()
        .add_systems(Update, translate);
        for (source, target) in configured.0.into_iter().zip(installed.0) {
            let (source, target) = (physical(source).unwrap(), physical(target).unwrap());
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(target);
            app.world_mut().run_schedule(Update);
            assert!(
                !app.world().resource::<GameKeys>().pressed(target),
                "old binding {target:?}"
            );
            let mut raw = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            raw.release(target);
            raw.clear();
            raw.press(source);
            app.world_mut().run_schedule(Update);
            let keys = app.world().resource::<GameKeys>();
            assert!(keys.just_pressed(target) && !keys.pressed(source));
            let mut raw = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            raw.clear();
            raw.release(source);
            app.world_mut().run_schedule(Update);
            assert!(app.world().resource::<GameKeys>().just_released(target));
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }
    }
}
