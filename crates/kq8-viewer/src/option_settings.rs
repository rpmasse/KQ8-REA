//! Player options shared by the menu and each newly constructed world.
//! The installation supplies defaults. Changes go into the recreation's
//! own Options.cs so the original game can still use its copy.
use bevy::prelude::*;
use kq8_core::options_menu::Settings;
use kq8_data::GameDir;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Resource)]
pub struct Preferences {
    pub settings: Settings,
    saved: Settings,
    path: PathBuf,
}

impl Preferences {
    pub fn load(game: &Path) -> Self {
        let path = PathBuf::from("kq8-artifacts/Options.cs");
        let original = GameDir::open(game).ok().and_then(|g| g.find("Options.cs"));
        let bytes = fs::read(&path)
            .ok()
            .or_else(|| original.and_then(|p| fs::read(p).ok()));
        let settings = bytes
            .as_deref()
            .map(Settings::from_script)
            .unwrap_or_default();
        Self {
            settings,
            saved: settings,
            path,
        }
    }

    pub fn store(&mut self) -> Result<(), String> {
        if self.settings == self.saved {
            return Ok(());
        }
        let parent = self.path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temp = parent.join(format!("Options-{}.tmp", std::process::id()));
        let write = (|| -> std::io::Result<()> {
            let mut file = fs::File::create(&temp)?;
            file.write_all(self.settings.write_script().as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temp, &self.path)
        })();
        if let Err(e) = write {
            let _ = fs::remove_file(&temp);
            return Err(format!("{}: {e}", self.path.display()));
        }
        self.saved = self.settings;
        Ok(())
    }
}

/// Runs before the first simulation update after a new game, a region load,
/// a restore or a menu change. Speech consults this same script variable.
pub fn apply_to_world(
    preferences: Res<Preferences>,
    change: Res<crate::world_build::WorldChange>,
    sim: Option<ResMut<crate::door_view::DoorSim>>,
    sound: Option<ResMut<crate::sound_view::SoundSim>>,
    mut last: Local<Option<(u64, Settings)>>,
    shared: Res<crate::Shared>,
) {
    let Some(mut sim) = sim else {
        return;
    };
    if change.current.is_empty() {
        return;
    }
    let current = (change.generation, preferences.settings);
    if *last == Some(current) {
        return;
    }
    *last = Some(current);
    let settings = preferences.settings;
    sim.active
        .scripts
        .vars
        .set("Options::Captions", settings.subtitles.to_string());
    let captions = sim.active.scripts.vars.get("Options::Captions");
    let mixer = match sound {
        Some(mut sound) => format!("{:?}", sound.apply_options(&settings)),
        None => "unavailable".to_owned(),
    };
    shared.log.lock().unwrap().summary(format!(
        "options: {} world mixer sliders {mixer}, active captions {captions:?}",
        change.current
    ));
}
