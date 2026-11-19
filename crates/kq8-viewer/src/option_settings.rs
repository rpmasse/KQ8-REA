//! Player options shared by the menu and each newly constructed world.
//! The installation supplies defaults. Changes go into the recreation's
//! own Options.cs so the original game can still use its copy.
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use kq8_core::options_menu::Settings;
use kq8_core::mouse_menu::MouseSettings;
use kq8_core::modern_display::{Anisotropic, AntiAliasing, Choice as DisplayChoice, DrawFps};
use kq8_data::GameDir;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Resource)]
pub struct Preferences {
    pub settings: Settings,
    pub display: Option<DisplayChoice>,
    pub mouse: MouseSettings,
    pub draw_fps: DrawFps,
    pub anti_aliasing: AntiAliasing,
    pub anisotropic: Anisotropic,
    saved: Settings,
    saved_display: Option<DisplayChoice>,
    saved_mouse: MouseSettings,
    saved_draw_fps: DrawFps,
    saved_anti_aliasing: AntiAliasing,
    saved_anisotropic: Anisotropic,
    path: PathBuf,
    display_path: PathBuf,
    mouse_path: PathBuf,
    draw_fps_path: PathBuf,
    anti_aliasing_path: PathBuf,
    anisotropic_path: PathBuf,
    other_lines: Vec<Vec<u8>>,
}

impl Preferences {
    #[cfg(test)]
    pub(crate) fn isolated_path(&self) -> &Path { &self.path }

    #[cfg(test)]
    pub(crate) fn isolated_for_test(game: &Path, settings: Settings) -> Self {
        let mut preferences = Self::load(game);
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        preferences.path = PathBuf::from("kq8-artifacts/scratch/preferences").join(nonce.to_string()).join("Options.cs");
        preferences.display_path = preferences.path.with_file_name("display.json");
        preferences.mouse_path = preferences.path.with_file_name("mouse.json");
        preferences.draw_fps_path = preferences.path.with_file_name("draw-fps.json");
        preferences.anti_aliasing_path = preferences.path.with_file_name("anti-aliasing.json");
        preferences.anisotropic_path = preferences.path.with_file_name("anisotropic.json");
        preferences.settings = settings.clone();
        preferences.saved = settings;
        preferences.display = None;
        preferences.saved_display = None;
        preferences.mouse = MouseSettings::default();
        preferences.saved_mouse = preferences.mouse;
        preferences.draw_fps = DrawFps::default();
        preferences.saved_draw_fps = preferences.draw_fps;
        preferences.anti_aliasing = AntiAliasing::default();
        preferences.saved_anti_aliasing = preferences.anti_aliasing;
        preferences.anisotropic = Anisotropic::default();
        preferences.saved_anisotropic = preferences.anisotropic;
        preferences
    }

    pub fn load(game: &Path) -> Self {
        let path = PathBuf::from("kq8-artifacts/Options.cs");
        let display_path = PathBuf::from("kq8-artifacts/display.json");
        let mouse_path = PathBuf::from("kq8-artifacts/mouse.json");
        let draw_fps_path = PathBuf::from("kq8-artifacts/draw-fps.json");
        let anti_aliasing_path = PathBuf::from("kq8-artifacts/anti-aliasing.json");
        let anisotropic_path = PathBuf::from("kq8-artifacts/anisotropic.json");
        let display = fs::read(&display_path).ok()
            .and_then(|bytes| serde_json::from_slice::<Option<DisplayChoice>>(&bytes).ok())
            .flatten().filter(|choice|choice.valid());
        let mouse = read_mouse(&mouse_path);
        let draw_fps = read_draw_fps(&draw_fps_path);
        let anti_aliasing = read_anti_aliasing(&anti_aliasing_path);
        let anisotropic = read_anisotropic(&anisotropic_path);
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
            display,
            mouse,
            draw_fps,
            anti_aliasing,
            anisotropic,
            saved: settings,
            saved_display: display,
            saved_mouse: mouse,
            saved_draw_fps: draw_fps,
            saved_anti_aliasing: anti_aliasing,
            saved_anisotropic: anisotropic,
            path,
            display_path,
            mouse_path,
            draw_fps_path,
            anti_aliasing_path,
            anisotropic_path,
            other_lines: retained_lines(local.as_deref(), original.as_deref()),
        }
    }

    pub fn store(&mut self) -> Result<(), String> {
        if self.settings != self.saved {
            let mut contents = self.settings.write_script().into_bytes();
            for line in &self.other_lines {
                contents.extend_from_slice(line);
                contents.push(b'\n');
            }
            write_preferences(&self.path, &contents)?;
            self.saved = self.settings.clone();
        }
        if self.display != self.saved_display {
            let contents = serde_json::to_vec_pretty(&self.display)
                .map_err(|error| format!("display preference: {error}"))?;
            write_preferences(&self.display_path, &contents)?;
            self.saved_display = self.display;
        }
        if self.mouse != self.saved_mouse {
            let contents = serde_json::to_vec_pretty(&self.mouse)
                .map_err(|error|format!("mouse preference: {error}"))?;
            write_preferences(&self.mouse_path,&contents)?;
            self.saved_mouse = self.mouse;
        }
        if self.draw_fps != self.saved_draw_fps {
            let contents = serde_json::to_vec_pretty(&self.draw_fps)
                .map_err(|error|format!("draw FPS preference: {error}"))?;
            write_preferences(&self.draw_fps_path,&contents)?;
            self.saved_draw_fps = self.draw_fps;
        }
        if self.anti_aliasing != self.saved_anti_aliasing {
            let contents = serde_json::to_vec_pretty(&self.anti_aliasing)
                .map_err(|error| format!("anti-aliasing preference: {error}"))?;
            write_preferences(&self.anti_aliasing_path, &contents)?;
            self.saved_anti_aliasing = self.anti_aliasing;
        }
        if self.anisotropic != self.saved_anisotropic {
            let contents = serde_json::to_vec_pretty(&self.anisotropic)
                .map_err(|error| format!("anisotropic preference: {error}"))?;
            write_preferences(&self.anisotropic_path, &contents)?;
            self.saved_anisotropic = self.anisotropic;
        }
        Ok(())
    }
}

fn read_mouse(path: &Path) -> MouseSettings {
    fs::read(path).ok()
        .and_then(|bytes|serde_json::from_slice::<MouseSettings>(&bytes).ok())
        .filter(|settings|settings.valid()).unwrap_or_default()
}

fn read_draw_fps(path: &Path) -> DrawFps {
    fs::read(path).ok()
        .and_then(|bytes|serde_json::from_slice::<DrawFps>(&bytes).ok())
        .filter(|setting|setting.valid()).unwrap_or_default()
}

fn read_anti_aliasing(path: &Path) -> AntiAliasing {
    fs::read(path).ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn read_anisotropic(path: &Path) -> Anisotropic {
    fs::read(path).ok()
        .and_then(|bytes| serde_json::from_slice::<Anisotropic>(&bytes).ok())
        .filter(|choice| choice.valid())
        .unwrap_or_default()
}

pub fn draw_fps_settings(draw_fps: DrawFps) -> WinitSettings {
    let mut settings = WinitSettings::game();
    let fps = if draw_fps.valid() { draw_fps.fps } else { DrawFps::default().fps };
    settings.focused_mode = UpdateMode::Reactive {
        wait: std::time::Duration::from_secs_f64(1.0 / f64::from(fps)),
        react_to_device_events: false,
        react_to_user_events: false,
        react_to_window_events: false,
    };
    settings
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

pub fn palette_output(game: &GameDir) -> kq8_data::palette::Output {
    let mode = Settings::from_script(&effective_script(game)).graphics.mode;
    kq8_data::palette::Output::from_mode(&mode).unwrap_or(kq8_data::palette::Output::Direct3D)
}

/// Fresh-screen palette adjustment. The original brightness control does
/// not select a display-gamma ramp (brightness.md Q186).
pub fn screen_palette(game: &GameDir, colors: [[u8; 3]; 256]) -> [[u8; 3]; 256] {
    kq8_data::palette::for_output(colors, brightness(game), palette_output(game))
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
    use super::{read_anisotropic,read_anti_aliasing,read_draw_fps,read_mouse,retained_lines,Preferences};
    use kq8_core::mouse_menu::MouseSettings;
    use kq8_core::options_menu::Settings;
    use kq8_core::modern_display::{Anisotropic,AntiAliasing,Choice,DrawFps,Ratio,ViewMode};
    use std::path::Path;

    #[test]
    fn modern_display_preference_is_separate_from_original_options_script() {
        let game = Path::new(".");
        let mut preferences = Preferences::isolated_for_test(game,Settings::default());
        let choice = Choice { mode: ViewMode::Windowed, ratio: Ratio::TwentyOneNine,
            pixels: [3440,1440] };
        preferences.display = Some(choice);
        preferences.store().unwrap();
        assert!(!preferences.path.exists(),"display alone must not write Options.cs");
        let bytes = std::fs::read(&preferences.display_path).unwrap();
        assert_eq!(serde_json::from_slice::<Option<Choice>>(&bytes).unwrap(),Some(choice));
        std::fs::remove_file(&preferences.display_path).unwrap();
    }

    #[test]
    fn mouse_camera_preference_saves_separately_and_loads_with_validation() {
        let game = Path::new(".");
        let mut preferences = Preferences::isolated_for_test(game,Settings::default());
        preferences.mouse = MouseSettings { camera_sensitivity_percent: 135 };
        preferences.store().unwrap();
        assert!(!preferences.path.exists(),"mouse sensitivity must not write Options.cs");
        let bytes = std::fs::read(&preferences.mouse_path).unwrap();
        assert_eq!(serde_json::from_slice::<MouseSettings>(&bytes).unwrap(),preferences.mouse);
        assert_eq!(read_mouse(&preferences.mouse_path),preferences.mouse);
        std::fs::write(&preferences.mouse_path,b"{\"camera_sensitivity_percent\":203}").unwrap();
        assert_eq!(read_mouse(&preferences.mouse_path),MouseSettings::default());
        assert!(!MouseSettings { camera_sensitivity_percent: 0 }.valid());
        assert!(!MouseSettings { camera_sensitivity_percent: 203 }.valid());
        std::fs::remove_file(&preferences.mouse_path).unwrap();
    }

    #[test]
    fn draw_fps_preference_saves_separately_and_rejects_unknown_rates() {
        let mut preferences = Preferences::isolated_for_test(Path::new("."),Settings::default());
        preferences.draw_fps = DrawFps { fps: 120 };
        preferences.store().unwrap();
        assert!(!preferences.path.exists());
        assert_eq!(read_draw_fps(&preferences.draw_fps_path),DrawFps { fps: 120 });
        std::fs::write(&preferences.draw_fps_path,b"{\"fps\":144}").unwrap();
        assert_eq!(read_draw_fps(&preferences.draw_fps_path),DrawFps::default());
        std::fs::remove_file(&preferences.draw_fps_path).unwrap();
    }

    #[test]
    fn anti_aliasing_preference_saves_separately_and_rejects_unknown_values() {
        let mut preferences = Preferences::isolated_for_test(Path::new("."),Settings::default());
        preferences.anti_aliasing = AntiAliasing::Smaa;
        preferences.store().unwrap();
        assert!(!preferences.path.exists());
        assert_eq!(read_anti_aliasing(&preferences.anti_aliasing_path),AntiAliasing::Smaa);
        std::fs::write(&preferences.anti_aliasing_path,b"\"Unknown\"").unwrap();
        assert_eq!(read_anti_aliasing(&preferences.anti_aliasing_path),AntiAliasing::Off);
        std::fs::remove_file(&preferences.anti_aliasing_path).unwrap();
    }

    #[test]
    fn anisotropic_preference_saves_separately_and_rejects_unknown_levels() {
        let mut preferences = Preferences::isolated_for_test(Path::new("."),Settings::default());
        preferences.anisotropic = Anisotropic { level: 1 };
        preferences.store().unwrap();
        assert_eq!(read_anisotropic(&preferences.anisotropic_path),Anisotropic { level: 1 });
        preferences.anisotropic = Anisotropic { level: 16 };
        preferences.store().unwrap();
        assert!(!preferences.path.exists());
        assert_eq!(read_anisotropic(&preferences.anisotropic_path),Anisotropic { level: 16 });
        std::fs::write(&preferences.anisotropic_path,b"{\"level\":32}").unwrap();
        assert_eq!(read_anisotropic(&preferences.anisotropic_path),Anisotropic::default());
        std::fs::remove_file(&preferences.anisotropic_path).unwrap();
    }

    #[test]
    fn draw_fps_pacing_ignores_input_events() {
        use bevy::winit::UpdateMode;
        for fps in DrawFps::CHOICES {
            let settings = super::draw_fps_settings(DrawFps { fps });
            match settings.focused_mode {
                UpdateMode::Reactive { wait, react_to_device_events: false,
                    react_to_user_events: false, react_to_window_events: false } => {
                    assert!((wait.as_secs_f64() - 1.0 / f64::from(fps)).abs() < 0.000_001);
                }
                other => panic!("draw rate {fps} has unexpected pacing: {other:?}"),
            }
        }
    }

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
