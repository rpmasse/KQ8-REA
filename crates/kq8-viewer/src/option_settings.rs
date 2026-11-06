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
    other_lines: Vec<Vec<u8>>,
}

impl Preferences {
    pub fn load(game: &Path) -> Self {
        let path = PathBuf::from("kq8-artifacts/Options.cs");
        let original = GameDir::open(game)
            .ok()
            .and_then(|g| g.find("Options.cs"))
            .and_then(|p| fs::read(p).ok());
        let local = fs::read(&path).ok();
        // The local file may predate the Graphics screen. Its commands
        // override the installed defaults, while omitted commands inherit.
        let combined = merge_scripts(original.as_deref(), local.as_deref());
        let settings = Settings::from_script(&combined);
        Self {
            settings: settings.clone(),
            saved: settings,
            path,
            other_lines: retained_lines(local.as_deref(), original.as_deref()),
        }
    }

    pub fn store(&mut self) -> Result<(), String> {
        if self.settings == self.saved {
            return Ok(());
        }
        let mut contents = self.settings.write_script().into_bytes();
        for line in &self.other_lines {
            contents.extend_from_slice(line);
            contents.push(b'\n');
        }
        write_preferences(&self.path, &contents)?;
        self.saved = self.settings.clone();
        Ok(())
    }
}

fn merge_scripts(original: Option<&[u8]>, local: Option<&[u8]>) -> Vec<u8> {
    let mut combined = original.unwrap_or_default().to_vec();
    combined.push(b'\n');
    if let Some(local) = local { combined.extend_from_slice(local); }
    combined
}

/// The player's local changes override installed defaults without writing to
/// the GOG folder. World loading and interface loading use this same text.
pub fn effective_script(game: &GameDir) -> Vec<u8> {
    let original = game.find("Options.cs").and_then(|p| fs::read(p).ok());
    let local = fs::read("kq8-artifacts/Options.cs").ok();
    merge_scripts(original.as_deref(), local.as_deref())
}

pub fn brightness(game: &GameDir) -> u8 {
    Settings::from_script(&effective_script(game)).graphics.brightness
}

/// Fresh-screen palette adjustment; the display-gamma stage is still separate.
pub fn screen_palette(game: &GameDir, colors: [[u8; 3]; 256]) -> [[u8; 3]; 256] {
    kq8_data::palette::with_brightness(colors, brightness(game))
}

fn retained_lines(local: Option<&[u8]>, original: Option<&[u8]>) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut keys = std::collections::BTreeSet::new();
    for (source_index, source) in [local, original].into_iter().enumerate() {
        let Some(source) = source else { continue };
        for raw in source.split(|&c| c == b'\n') {
            let line = raw.strip_suffix(b"\r").unwrap_or(raw);
            let words: Vec<_> = line
                .split(|c| c.is_ascii_whitespace())
                .filter(|word| !word.is_empty())
                .collect();
            let Some(command) = words.first() else { continue };
            let name = String::from_utf8_lossy(command).to_ascii_lowercase();
            if matches!(name.as_str(), "kqsound::setspeechvol" | "kqsound::setsfxvol" | "kqsound::setmusicvol" | "notstartinghelp" | "assigngmodename" | "set800mode" | "setbrightness" | "setshadowmask" | "usedynamiclighting")
                || (name == "set" && words.get(1).is_some_and(|value| value.eq_ignore_ascii_case(b"Options::Captions")))
            {
                continue;
            }
            let key = if name == "set" {
                format!("set {}", words.get(1).map(|word| String::from_utf8_lossy(word).to_ascii_lowercase()).unwrap_or_default())
            } else {
                name
            };
            if source_index == 0 || keys.insert(key.clone()) {
                lines.push(line.to_vec());
            }
            keys.insert(key);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::retained_lines;
    use kq8_core::options_menu::Settings;

    #[test]
    fn older_local_options_retain_gog_graphics_defaults() {
        let local = b"KQSound::setMusicVol 0.40\nnotStartingHelp 1\nset800mode 1\n";
        let original = b"KQSound::setMusicVol 0.87\r\nsetShadowMask 1\r\nuseDynamicLighting 0\r\nset800mode 0\r\nsetBrightness 49\r\noriginalExtra 6\r\n";
        let mut combined = original.to_vec();
        combined.extend_from_slice(local);
        let settings = Settings::from_script(&combined);
        assert!(settings.graphics.high_resolution);
        assert_eq!(settings.graphics.shadow_mask, 1);
        assert_eq!(settings.graphics.brightness, 49);
        assert_eq!(settings.volume[0], 40);
        let lines = retained_lines(Some(local), Some(original));
        assert_eq!(lines, [b"originalExtra 6".to_vec()]);
    }
}

pub(crate) fn write_preferences(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    let write = (|| -> std::io::Result<()> {
        let mut file = fs::File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if let Err(e) = write {
        let _ = fs::remove_file(&temp);
        return Err(format!("{}: {e}", path.display()));
    }
    Ok(())
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
    let current = (change.generation, preferences.settings.clone());
    if last.as_ref() == Some(&current) {
        return;
    }
    *last = Some(current);
    let settings = &preferences.settings;
    sim.active
        .scripts
        .vars
        .set("Options::Captions", settings.subtitles.to_string());
    let captions = sim.active.scripts.vars.get("Options::Captions");
    let mixer = match sound {
        Some(mut sound) => format!("{:?}", sound.apply_options(settings)),
        None => "unavailable".to_owned(),
    };
    shared.log.lock().unwrap().summary(format!(
        "options: {} world mixer sliders {mixer}, active captions {captions:?}",
        change.current
    ));
}
