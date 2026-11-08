//! Live original-size presentation, independent of historical device identity.
//! Q207 establishes the resize/GUI rules. The supplied capability is a
//! recreation-backend input, not a claim that WGPU enumerated Glide hardware.
use crate::{option_settings::Preferences, Shared};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Size {
    Low,
    High,
}

impl Size {
    pub(crate) fn from_high(high: bool) -> Self {
        if high {
            Self::High
        } else {
            Self::Low
        }
    }
    pub(crate) fn high(self) -> bool {
        self == Self::High
    }
    pub(crate) fn pixels(self) -> [u32; 2] {
        kq8_core::caption::screen(self.high())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Capabilities {
    pub high: bool,
}

/// Retain the two sizes already accepted by this viewer. Historical
/// renderer enumeration and original-family output parity remain separate.
pub(crate) const VIEWER_CAPABILITIES: Capabilities = Capabilities { high: true };

#[derive(Debug)]
struct Pending {
    window: Entity,
    requested: Size,
    previous_pixels: [u32; 2],
    acknowledged: bool,
}

#[derive(Resource, Debug)]
pub(crate) struct Display {
    pub(crate) applied: Size,
    pub(crate) capabilities: Capabilities,
    pending: Option<Pending>,
    revision: u64,
}

#[derive(Resource, Default)]
struct WindowReplies(Vec<(Entity, [u32; 2])>);

pub(crate) fn register(app: &mut App, capabilities: Capabilities) {
    let applied = Size::from_high(
        app.world()
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution,
    );
    app.insert_resource(Display {
        applied,
        capabilities,
        pending: None,
        revision: 0,
    })
    .init_resource::<WindowReplies>()
    .add_systems(First, (collect_replies, apply_replies).chain());
}

pub(crate) fn pending(world: &World) -> bool {
    crate::graphics_runtime::pending(world) || world
        .get_resource::<Display>()
        .is_some_and(|d| d.pending.is_some())
}

pub(crate) fn high_available(world: &World) -> bool {
    if let Some(runtime) = world.get_resource::<crate::graphics_runtime::Runtime>() {
        return runtime.can_high();
    }
    world
        .get_resource::<Display>()
        .map_or(VIEWER_CAPABILITIES.high, |d| d.capabilities.high)
}

/// A changed WindowResolution is only a request. Bevy/Winit emits its
/// WindowResized result after request_inner_size or the native callback.
pub(crate) fn request(world: &mut World, requested: Size) -> Result<bool, String> {
    if world.contains_resource::<crate::graphics_runtime::Runtime>() {
        if crate::graphics_runtime::pending(world) {
            return Err("a native display request or restoration is already pending".into());
        }
        return world.resource_mut::<crate::graphics_runtime::Runtime>()
            .request_size(requested.high()).map(|request|request.is_some());
    }
    let state = world
        .get_resource::<Display>()
        .ok_or("display owner unavailable")?;
    if requested == Size::High && !state.capabilities.high {
        return Err("800x600 is unavailable for the supplied capability".into());
    }
    if let Some(pending) = &state.pending {
        return if pending.requested == requested {
            Ok(false)
        } else {
            Err("a resolution request is already pending".into())
        };
    }
    if state.applied == requested {
        return Ok(false);
    }
    let (entity, previous_pixels) = {
        let mut windows = world.query_filtered::<(Entity, &mut Window), With<PrimaryWindow>>();
        let (entity, mut window) = windows
            .single_mut(world)
            .map_err(|e| format!("primary window unavailable: {e}"))?;
        let previous = [window.physical_width(), window.physical_height()];
        let [width, height] = requested.pixels();
        window.resolution.set_physical_resolution(width, height);
        (entity, previous)
    };
    world.resource_mut::<Display>().pending = Some(Pending {
        window: entity,
        requested,
        previous_pixels,
        acknowledged: false,
    });
    world
        .resource::<Shared>()
        .log
        .lock()
        .unwrap()
        .summary(format!(
        "display: requested {:?} physical pixels; waiting for window and camera acknowledgement",
        requested.pixels()
    ));
    Ok(true)
}

/// The profile owner has verified actual host mode, cameras and palette.
/// The older WindowResized path remains available to component fixtures.
pub(crate) fn accept_profile(world: &mut World, high: bool, high_available: bool) {
    if let Some(mut display) = world.get_resource_mut::<Display>() {
        display.applied = Size::from_high(high);
        display.capabilities.high = high_available;
        display.pending = None;
        display.revision += 1;
    }
    sync_world_presentation(world);
    crate::startup_menu::resize_presentation(world, Size::from_high(high).pixels());
}

fn collect_replies(
    mut events: MessageReader<WindowResized>,
    windows: Query<&Window>,
    mut replies: ResMut<WindowReplies>,
) {
    // The Window holds the latest actual dimensions if several resize
    // events arrived together. Do not commit an earlier intermediate size.
    for event in events.read() {
        if let Ok(window) = windows.get(event.window) {
            replies.0.push((
                event.window,
                [window.physical_width(), window.physical_height()],
            ));
        }
    }
}

fn apply_replies(world: &mut World) {
    let replies = std::mem::take(&mut world.resource_mut::<WindowReplies>().0);
    let Some(pending) = world.resource::<Display>().pending.as_ref() else {
        return;
    };
    let window_entity = pending.window;
    if let Some((_, actual)) = replies.into_iter().rev().find(|(e, _)| *e == window_entity) {
        let requested = world
            .resource::<Display>()
            .pending
            .as_ref()
            .unwrap()
            .requested;
        if actual != requested.pixels() {
            let pending = world.resource_mut::<Display>().pending.take().unwrap();
            // Defensive one-attempt restoration, not an original driver
            // failure claim or a timed keep-this-mode dialog.
            if let Some(mut window) = world.get_mut::<Window>(window_entity) {
                window.resolution.set_physical_resolution(
                    pending.previous_pixels[0],
                    pending.previous_pixels[1],
                );
            }
            world.resource::<Shared>().log.lock().unwrap().summary(format!(
                "display: requested {:?}, window reported {actual:?}; retained previous interface/preferences and requested {:?} again",
                requested.pixels(), pending.previous_pixels));
            return;
        }
        world
            .resource_mut::<Display>()
            .pending
            .as_mut()
            .unwrap()
            .acknowledged = true;
    }
    let pending = world.resource::<Display>().pending.as_ref().unwrap();
    if !pending.acknowledged {
        return;
    }
    let requested = pending.requested;
    // UI and gameplay pointer input remain suspended until the existing
    // cameras also report the new target. Do not rewrite their FOV/pose.
    if world.query::<&Camera>().iter(world).any(|camera| {
        camera
            .physical_target_size()
            .is_none_or(|size| size.to_array() != requested.pixels())
    }) {
        return;
    }
    {
        let mut display = world.resource_mut::<Display>();
        display.applied = requested;
        display.pending = None;
        display.revision += 1;
    }
    world
        .resource_mut::<Preferences>()
        .settings
        .graphics
        .high_resolution = requested.high();
    sync_world_presentation(world);
    crate::startup_menu::resize_presentation(world, requested.pixels());
    world.resource::<Shared>().log.lock().unwrap().summary(format!(
        "display: confirmed {:?}, interface revision {}; retained renderer, projection and game state",
        requested.pixels(), world.resource::<Display>().revision));
}

/// World loading can have read older persisted options while a live
/// preference is active. The display owner supplies the final dimensions.
pub(crate) fn sync_world_presentation(world: &mut World) {
    let Some(display) = world.get_resource::<Display>() else {
        return;
    };
    let size = display.applied.pixels();
    if let Some(mut gui) = world.get_resource_mut::<crate::gui_view::Gui>() {
        gui.resize(size);
    }
    if let Some(mut caption) = world.get_resource_mut::<crate::talk_view::CaptionStyle>() {
        caption.resize(size);
    }
    let logical = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world)
        .ok()
        .map(|w| [w.width(), w.height()]);
    if let (Some(logical), Some(mut frame)) = (
        logical,
        world.get_resource_mut::<crate::gui_view::ScreenFrame>(),
    ) {
        frame.0 = kq8_core::panels::screen_frame(logical);
    }
    if world.contains_resource::<crate::graphics_runtime::Runtime>() {
        if let Err(error) = crate::graphics_runtime::apply_palette(world, "active device world/display restoration") {
            world.resource::<Shared>().log.lock().unwrap().summary(format!("palette: world restoration failed: {error}"));
        }
    } else if let Some(graphics) = world.get_resource::<Preferences>().map(|p| p.settings.graphics.clone()) {
        if let Some(output) = kq8_data::palette::Output::from_mode(&graphics.mode).filter(|o| o.is_software()) {
            crate::live_palette::apply_to_surface(world,
                crate::live_palette::Values { brightness: graphics.brightness, output },
                "Software world/display restoration");
        }
    }
}

/// Common host mapping for the menu canvas and its pointer. DPI is
/// already represented by Window's logical size; world picking continues
/// through Bevy's corresponding logical camera viewport.
pub(crate) fn canvas_frame(window: Vec2, canvas: [u32; 2]) -> (Vec2, f32) {
    let size = Vec2::new(canvas[0] as f32, canvas[1] as f32);
    let scale = (window.x / size.x).min(window.y / size.y);
    ((window - size * scale) * 0.5, scale)
}
