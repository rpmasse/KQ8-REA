//! Original Graphics behavior over an explicitly declared recreation backend.
//!
//! graphics-modes.md supplies the original device/menu/startup rules. These
//! profiles are our supported emulation inputs, not legacy hardware probes:
//! Direct3D is the development default, while the original compiled mode-name
//! default remains Software. The alternate Glide profile declares a Voodoo
//! board with 4MB framebuffer capability; it never reads host GPU memory.
//!
//! Normal launch is fullscreen. The explicit windowed diagnostic uses the
//! original Software window device, independently of the saved fullscreen
//! name. A saved 800 Glide request in windowed mode is rejected, not rewritten.
//! Host exclusive mode is selected from exact advertised 640/800 modes and
//! checked through Winit's actual window, fullscreen state and monitor size.
//! Bevy's borderless fallback must never count as exclusive acceptance.
use bevy::{
    prelude::*,
    window::{
        Monitor, MonitorSelection, OnMonitor, PrimaryMonitor, VideoMode, VideoModeSelection,
        WindowMode,
    },
};
use kq8_core::{
    graphics_devices::{Device, Family, GlideBoard, Inventory},
    options_menu::Graphics,
};
use kq8_data::palette::{Output, Rgb};

#[path = "graphics_runtime_driver.rs"]
mod driver;
#[path = "graphics_present.rs"]
mod present;
pub(crate) use driver::{
    accept_host, apply_palette, device_action, palette_values, pending, register, runtime_output,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Profile {
    #[default]
    Direct3D,
    Glide,
}

impl Profile {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "direct3d" | "d3d" => Ok(Self::Direct3D),
            "glide" => Ok(Self::Glide),
            _ => Err(format!(
                "--display-profile expects direct3d or glide, not {value:?}"
            )),
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Direct3D => "direct3d",
            Self::Glide => "glide",
        }
    }

    fn hardware(self) -> Device {
        match self {
            Self::Direct3D => Device::direct3d("display", true),
            Self::Glide => Device::glide(
                true,
                Some(GlideBoard {
                    board_type: 0,
                    framebuffer_mb: 4,
                }),
            ),
        }
    }

    pub(crate) fn inventory(self) -> Inventory {
        // The original provider omitted D3D enumeration when Glide existed.
        Inventory::new(vec![Device::software(true), self.hardware()])
    }

    fn device(self, name: &str) -> Option<Device> {
        if name == "Software" {
            Some(Device::software(true))
        } else {
            let device = self.hardware();
            (name == device.name).then_some(device)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Presentation {
    Windowed,
    #[default]
    Fullscreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Surface {
    pub(crate) device: Device,
    pub(crate) presentation: Presentation,
    pub(crate) high: bool,
}

impl Surface {
    pub(crate) fn pixels(&self) -> [u32; 2] {
        kq8_core::caption::screen(self.high)
    }

    /// Colors for the actual logical device; a saved fullscreen preference
    /// cannot select hardware colors on the Software window device.
    pub(crate) fn output(&self, system: Option<[Rgb; 20]>) -> Result<Output, String> {
        match (self.presentation, self.device.family) {
            (Presentation::Windowed, Family::Software) => system
                .map(|system| Output::SoftwareWindow { system })
                .ok_or_else(|| "Software window requires Windows system palette colors".into()),
            (Presentation::Windowed, _) => {
                Err("hardware renderer is not an original window device".into())
            }
            (Presentation::Fullscreen, Family::Software) => Ok(Output::Software),
            (Presentation::Fullscreen, Family::Direct3D) => Ok(Output::Direct3D),
            (Presentation::Fullscreen, Family::Glide) => Ok(Output::Glide),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Request {
    pub(crate) serial: u64,
    pub(crate) surface: Surface,
}

/// The selected fullscreen device is separate from the active surface.
/// Requests are not acknowledgement; only confirm() changes active state.
#[derive(Resource, Debug)]
pub(crate) struct Runtime {
    profile: Profile,
    fullscreen: Device,
    active: Option<Surface>,
    pending: Option<Request>,
    revision: u64,
    failure: Option<String>,
}

impl Runtime {
    /// Returns validated startup preferences without writing any file. The
    /// caller applies them to its in-memory Preferences before making art.
    pub(crate) fn startup(
        profile: Profile,
        presentation: Presentation,
        graphics: &Graphics,
    ) -> Result<(Self, Graphics), String> {
        let inventory = profile.inventory();
        let checked = inventory.startup_request(&graphics.mode, graphics.high_resolution);
        if let Some(gap) = checked.unresolved {
            return Err(format!(
                "saved graphics mode {:?} is unresolved in the {} compatibility profile: {gap:?}",
                graphics.mode,
                profile.name()
            ));
        }
        let fullscreen = profile
            .device(&checked.name)
            .ok_or_else(|| format!("unsupported graphics mode {:?}", checked.name))?;
        if presentation == Presentation::Windowed && checked.high_resolution {
            return Err("the original Software window device does not support 800x600; request 640x480 or launch fullscreen; saved preferences were retained".into());
        }
        let mut validated = graphics.clone();
        validated.mode = checked.name;
        validated.high_resolution = checked.high_resolution;
        let surface = Surface {
            device: if presentation == Presentation::Windowed {
                Device::software(false)
            } else {
                fullscreen.clone()
            },
            presentation,
            high: checked.high_resolution,
        };
        Ok((
            Self {
                profile,
                fullscreen,
                active: None,
                pending: Some(Request { serial: 1, surface }),
                revision: 1,
                failure: None,
            },
            validated,
        ))
    }

    pub(crate) fn profile(&self) -> Profile {
        self.profile
    }
    pub(crate) fn inventory(&self) -> Inventory {
        self.profile.inventory()
    }
    pub(crate) fn fullscreen_device(&self) -> &Device {
        &self.fullscreen
    }
    pub(crate) fn active(&self) -> Option<&Surface> {
        self.active.as_ref()
    }
    pub(crate) fn pending(&self) -> Option<&Request> {
        self.pending.as_ref()
    }
    pub(crate) fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub(crate) fn can_high(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|surface| surface.device.supports_800())
    }

    /// The core menu already resolved same-family selection and the distinct
    /// Software restart confirmation. This additionally validates the target
    /// against this actual backend profile, including a retained disabled row.
    pub(crate) fn request_device(&mut self, target: Device) -> Result<Option<Request>, String> {
        if self.pending.is_some() {
            return Err("a display request is already awaiting acknowledgement".into());
        }
        let active = self
            .active
            .as_ref()
            .ok_or("no display has been acknowledged")?;
        let supported = self.profile.device(&target.name).ok_or_else(|| {
            format!(
                "device {:?} is unavailable in the {} compatibility profile",
                target.name,
                self.profile.name()
            )
        })?;
        if target != supported {
            return Err(format!(
                "device {:?} does not match the declared backend capability",
                target.name
            ));
        }
        if active.device.family == target.family {
            return Ok(None);
        }
        if active.device.family == Family::Direct3D && target.family == Family::Software {
            return Err("Direct3D to Software requires the accepted restart confirmation".into());
        }
        let high = active.high;
        if high && !target.supports_800() {
            return Err(format!(
                "device {:?} does not support 800x600; current device and preferences retained",
                target.name
            ));
        }
        let surface = Surface {
            device: target,
            presentation: Presentation::Fullscreen,
            high,
        };
        Ok(Some(self.stage(surface)))
    }

    pub(crate) fn request_size(&mut self, high: bool) -> Result<Option<Request>, String> {
        if self.pending.is_some() {
            return Err("a display request is already awaiting acknowledgement".into());
        }
        let active = self
            .active
            .as_ref()
            .ok_or("no display has been acknowledged")?;
        if active.high == high {
            return Ok(None);
        }
        if high && !active.device.supports_800() {
            return Err("800x600 is unavailable for the active original device".into());
        }
        let mut surface = active.clone();
        surface.high = high;
        Ok(Some(self.stage(surface)))
    }

    fn stage(&mut self, surface: Surface) -> Request {
        self.revision += 1;
        self.failure = None;
        let request = Request {
            serial: self.revision,
            surface,
        };
        self.pending = Some(request.clone());
        request
    }

    /// Called after actual host dimensions/mode, camera and palette all match.
    /// Stale replies cannot acknowledge a newer or failed request.
    pub(crate) fn confirm(&mut self, serial: u64) -> Result<Surface, String> {
        let request = self
            .pending
            .as_ref()
            .filter(|request| request.serial == serial)
            .ok_or("stale display acknowledgement")?;
        let surface = request.surface.clone();
        if surface.presentation == Presentation::Fullscreen {
            self.fullscreen = surface.device.clone();
        }
        self.active = Some(surface.clone());
        self.pending = None;
        self.failure = None;
        Ok(surface)
    }

    pub(crate) fn reject(&mut self, serial: u64, why: String) -> bool {
        if !self
            .pending
            .as_ref()
            .is_some_and(|request| request.serial == serial)
        {
            return false;
        }
        self.pending = None;
        self.failure = Some(why);
        true
    }

    /// Produce the exact local preference write for the accepted restart
    /// decision. Runtime active state stays unchanged even if the write fails.
    pub(crate) fn restart_preferences(&self, current: &Graphics) -> Result<Graphics, String> {
        if self.pending.is_some() {
            return Err("a display change is pending".into());
        }
        let active = self
            .active
            .as_ref()
            .ok_or("no display has been acknowledged")?;
        if active.device.family != Family::Direct3D {
            return Err("Software restart is only valid from active Direct3D".into());
        }
        let mut changed = current.clone();
        changed.mode = "Software".into();
        Ok(changed)
    }
}

/// Exact size first. Remaining selection is a host policy: keep the desktop
/// refresh rate when possible and prefer its highest bit depth. This does not
/// assert an original 8-bit host framebuffer on a modern WGPU swapchain.
pub(crate) fn exact_mode(
    modes: &[VideoMode],
    pixels: [u32; 2],
    desktop_rate: Option<u32>,
) -> Option<VideoMode> {
    let rate = desktop_rate.unwrap_or(60_000);
    modes
        .iter()
        .copied()
        .filter(|mode| mode.physical_size.to_array() == pixels)
        .min_by_key(|mode| {
            (
                mode.refresh_rate_millihertz.abs_diff(rate),
                std::cmp::Reverse(mode.bit_depth),
                mode.refresh_rate_millihertz,
            )
        })
}

/// The monitor inventory may not be populated on the window's first frame.
/// None means wait for it; an unavailable exact mode is a clear error.
pub(crate) fn requested_window_mode(
    world: &mut World,
    window: Entity,
    surface: &Surface,
) -> Result<Option<WindowMode>, String> {
    if surface.presentation == Presentation::Windowed {
        return Ok(Some(WindowMode::Windowed));
    }
    let current = world.get::<OnMonitor>(window).map(|m| m.0);
    let primary = world
        .query_filtered::<Entity, With<PrimaryMonitor>>()
        .iter(world)
        .next();
    let Some(entity) = current.or(primary) else {
        return Ok(None);
    };
    let Some(monitor) = world.get::<Monitor>(entity) else {
        return Ok(None);
    };
    let pixels = surface.pixels();
    let mode = exact_mode(&monitor.video_modes,pixels,monitor.refresh_rate_millihertz)
        .ok_or_else(||format!("monitor {:?} has no exclusive fullscreen {}x{} mode; no borderless fallback was accepted",monitor.name,pixels[0],pixels[1]))?;
    Ok(Some(WindowMode::Fullscreen(
        MonitorSelection::Entity(entity),
        VideoModeSelection::Specific(mode),
    )))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostPresentation {
    Windowed,
    Exclusive,
    Borderless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostState {
    pub(crate) presentation: HostPresentation,
    pub(crate) window_pixels: [u32; 2],
    pub(crate) monitor_pixels: Option<[u32; 2]>,
    pub(crate) mode: Option<VideoMode>,
}

impl HostState {
    pub(crate) fn accepts(self, surface: &Surface) -> bool {
        if self.window_pixels != surface.pixels() {
            return false;
        }
        match surface.presentation {
            Presentation::Windowed => self.presentation == HostPresentation::Windowed,
            Presentation::Fullscreen => {
                self.presentation == HostPresentation::Exclusive
                    && self.monitor_pixels == Some(surface.pixels())
                    && self
                        .mode
                        .is_some_and(|mode| mode.physical_size.to_array() == surface.pixels())
            }
        }
    }
}

/// Main-thread observation, using Winit rather than the requested Bevy mode.
/// Requires direct winit 0.30.13 (already locked through Bevy) at integration.
pub(crate) fn observe_host(window: Entity) -> Option<HostState> {
    bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        let window = windows.get_window(window)?;
        let size = window.inner_size();
        let monitor_pixels = window.current_monitor().map(|monitor| {
            let size = monitor.size();
            [size.width, size.height]
        });
        let (presentation, mode) = match window.fullscreen() {
            None => (HostPresentation::Windowed, None),
            Some(winit::window::Fullscreen::Borderless(_)) => (HostPresentation::Borderless, None),
            Some(winit::window::Fullscreen::Exclusive(mode)) => {
                let size = mode.size();
                (
                    HostPresentation::Exclusive,
                    Some(VideoMode {
                        physical_size: UVec2::new(size.width, size.height),
                        bit_depth: mode.bit_depth(),
                        refresh_rate_millihertz: mode.refresh_rate_millihertz(),
                    }),
                )
            }
        };
        Some(HostState {
            presentation,
            window_pixels: [size.width, size.height],
            monitor_pixels,
            mode,
        })
    })
}

#[cfg(test)]
#[path = "graphics_runtime_tests.rs"]
mod tests;
