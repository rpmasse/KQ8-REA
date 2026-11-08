//! `--renderer auto|vulkan|dx12`: which graphics API Bevy draws with
//! (docs/DESIGN.md, "choosing the graphics API"). Our own option, not
//! from the original game.
//!
//! Bevy draws through wgpu, which can talk to Vulkan, DirectX 12, Metal
//! or OpenGL. The Windows game frontend defaults to DX12 with
//! DirectComposition. Explicit API and environment choices still win;
//! model inspection and other platforms retain Bevy's defaults.

use std::ffi::{OsStr, OsString};

use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;

use crate::Shared;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    Auto,
    Vulkan,
    Dx12,
}

impl Renderer {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "vulkan" | "vk" => Ok(Self::Vulkan),
            "dx12" | "d3d12" | "directx" => Ok(Self::Dx12),
            // Bevy is built without its OpenGL part (feature `gles`), and
            // on Windows OpenGL would also need the ANGLE library.
            "gl" | "opengl" | "gles" => Err("--renderer gl is not available: this viewer is built without OpenGL; use auto, vulkan or dx12".into()),
            v => Err(format!("--renderer needs auto, vulkan or dx12, not '{v}'")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Vulkan => "vulkan",
            Self::Dx12 => "dx12",
        }
    }

    /// Decide defaults without reading or changing process-global state.
    /// Presence, rather than successful parsing, preserves even an empty or
    /// invalid environment override for Bevy to interpret as before.
    pub fn startup_policy(
        self,
        windows: bool,
        game_frontend: bool,
        backend_env: Option<&OsStr>,
        presentation_env: Option<&OsStr>,
    ) -> StartupPolicy {
        let windows_game = windows && game_frontend;
        let backends = match self {
            Self::Auto if windows_game && backend_env.is_none() => Some(Backends::DX12),
            Self::Auto => None,
            Self::Vulkan => Some(Backends::VULKAN),
            Self::Dx12 => Some(Backends::DX12),
        };
        StartupPolicy {
            backends,
            default_visual: windows_game && presentation_env.is_none(),
            backend_env: backend_env.map(OsStr::to_os_string),
            presentation_env: presentation_env.map(OsStr::to_os_string),
        }
    }
}

/// Process startup policy, separate from the original game's logical device.
/// DX12 composition presents inside the physical mode owned by Winit; it
/// does not claim DXGI swapchain-exclusive ownership or legacy raster parity.
#[derive(Debug)]
pub struct StartupPolicy {
    backends: Option<Backends>,
    default_visual: bool,
    backend_env: Option<OsString>,
    presentation_env: Option<OsString>,
}

impl StartupPolicy {
    /// Call once immediately after argument parsing, before asset/audio/Bevy
    /// workers. Locked Bevy 0.20 exposes its DX12 presentation choice only
    /// through this environment variable, not a WgpuSettings field.
    pub fn apply_presentation_default_before_workers(&self) {
        #[cfg(windows)]
        if self.default_visual && std::env::var_os("WGPU_DX12_PRESENTATION_SYSTEM").is_none() {
            // Windows-only, process-local startup initialization. No worker
            // has started and an inherited value is never overwritten. The
            // selected value must remain available until Bevy initializes.
            std::env::set_var("WGPU_DX12_PRESENTATION_SYSTEM", "Visual");
        }
    }

    pub fn summary(&self) -> String {
        let backend = match self.backends {
            Some(backends) => format!("allowed {backends:?}"),
            None if self.backend_env.is_some() => "inherited WGPU_BACKEND (Bevy parsing)".into(),
            None => "Bevy default".into(),
        };
        let presentation = if self.default_visual {
            "Visual Windows game default".into()
        } else if let Some(value) = &self.presentation_env {
            format!("inherited {value:?}")
        } else {
            "Bevy default".into()
        };
        format!(
            "renderer policy: {backend}; inherited WGPU_BACKEND={:?}; DX12 presentation {presentation}",
            self.backend_env,
        )
    }

    pub fn plugin(&self) -> RenderPlugin {
        let Some(backends) = self.backends else {
            return RenderPlugin::default();
        };
        RenderPlugin {
            render_creation: RenderCreation::from(WgpuSettings {
                backends: Some(backends),
                ..default()
            }),
            ..default()
        }
    }
}

/// What was asked for, kept for the log line once the card is set up.
#[derive(Resource)]
pub struct RendererChoice(pub Renderer);

/// Writes once to `viewer.log` which API, card and driver are in use. The
/// card is set up while the app starts, so this waits until its details
/// appear.
pub fn report_renderer(
    info: Option<Res<RenderAdapterInfo>>,
    choice: Res<RendererChoice>,
    shared: Res<Shared>,
    mut done: Local<bool>,
) {
    let Some(info) = info.filter(|_| !*done) else {
        return;
    };
    *done = true;
    shared.log.lock().unwrap().summary(format!(
        "renderer: asked {} got {:?} on \"{}\" (driver {} {})",
        choice.0.name(),
        info.backend,
        info.name,
        info.driver,
        info.driver_info
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_names() {
        assert_eq!(Renderer::parse("auto"), Ok(Renderer::Auto));
        assert_eq!(Renderer::parse("Vulkan"), Ok(Renderer::Vulkan));
        assert_eq!(Renderer::parse("vk"), Ok(Renderer::Vulkan));
        assert_eq!(Renderer::parse("DX12"), Ok(Renderer::Dx12));
        assert_eq!(Renderer::parse("d3d12"), Ok(Renderer::Dx12));
        assert!(Renderer::parse("gl").is_err());
        assert!(Renderer::parse("metal").is_err());
    }

    #[test]
    fn windows_game_renderer_policy_precedence_matrix() {
        let policy = |renderer: Renderer,
                      windows,
                      game,
                      backend: Option<&str>,
                      presentation: Option<&str>| {
            renderer.startup_policy(
                windows,
                game,
                backend.map(OsStr::new),
                presentation.map(OsStr::new),
            )
        };

        let game = policy(Renderer::Auto, true, true, None, None);
        assert_eq!(game.backends, Some(Backends::DX12));
        assert!(game.default_visual);
        assert!(game.summary().contains("Visual Windows game default"));
        match game.plugin().render_creation {
            RenderCreation::Automatic(settings) => {
                assert_eq!(settings.backends, Some(Backends::DX12))
            }
            RenderCreation::Manual(_) => {
                panic!("frontend defaults must retain automatic renderer setup")
            }
        }

        // Do not replace environment parsing, including its existing behavior
        // for empty/invalid input, with an inferred fallback backend.
        for backend in ["vulkan", "dx12", "vk, dx12", "", "invalid-backend"] {
            let inherited = policy(Renderer::Auto, true, true, Some(backend), None);
            assert_eq!(inherited.backends, None, "{backend:?}");
            assert_eq!(inherited.backend_env.as_deref(), Some(OsStr::new(backend)));
            assert!(inherited.default_visual);
        }

        // CLI API selection still outranks WGPU_BACKEND. Presentation remains
        // an independent DX12 option, inert when Vulkan is explicitly chosen.
        for (renderer, backend, expected) in [
            (Renderer::Vulkan, "dx12", Backends::VULKAN),
            (Renderer::Dx12, "vulkan", Backends::DX12),
        ] {
            let explicit = policy(renderer, true, true, Some(backend), None);
            assert_eq!(explicit.backends, Some(expected));
            assert!(explicit.default_visual);
        }
        for presentation in [
            "Hwnd",
            "Visual",
            "dxgifromvisual",
            "",
            "invalid-presentation",
        ] {
            let inherited = policy(Renderer::Auto, true, true, None, Some(presentation));
            assert_eq!(inherited.backends, Some(Backends::DX12));
            assert!(!inherited.default_visual, "{presentation:?}");
            assert_eq!(
                inherited.presentation_env.as_deref(),
                Some(OsStr::new(presentation))
            );
        }

        // Model inspection and other platforms keep their previous defaults
        // and their explicit selections; they never receive the env default.
        for (windows, game) in [(true, false), (false, true), (false, false)] {
            let excluded = policy(Renderer::Auto, windows, game, None, None);
            assert_eq!(excluded.backends, None);
            assert!(!excluded.default_visual);
            assert_eq!(
                policy(Renderer::Vulkan, windows, game, None, None).backends,
                Some(Backends::VULKAN)
            );
            assert_eq!(
                policy(Renderer::Dx12, windows, game, None, None).backends,
                Some(Backends::DX12)
            );
        }
    }
}
