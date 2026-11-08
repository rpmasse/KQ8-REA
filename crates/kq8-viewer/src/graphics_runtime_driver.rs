//! Native presentation ownership for the explicit original-device profiles.
//! Host acknowledgement/recovery are recreation safeguards; they are not an
//! assertion about the original driver's failure handling or pixel fidelity.
//! Winit's Windows backend can panic on ChangeDisplaySettingsExW failure;
//! restoring an observed mismatch is not a promise to recover from that panic.
use super::*;
use crate::{live_palette::Values, option_settings::Preferences, Shared};
use bevy::{app::AppExit, window::PrimaryWindow};
use kq8_core::graphics_menu::DeviceAction;
use std::time::{Duration, Instant};

// An unavailable or unresponsive host must not leave input blocked forever.
// This is a backend failure deadline, not an original keep-this-mode dialog.
const HOST_DEADLINE: Duration = Duration::from_secs(5);

#[derive(Debug)]
struct HostRequest {
    serial: u64,
    frame: present::Ticket,
    window: Entity,
    previous_mode: WindowMode,
    previous_pixels: [u32; 2],
    previous_surface: Option<Surface>,
    issued: Instant,
}

#[derive(Debug)]
struct Restoration {
    window: Entity,
    surface: Surface,
    frame: present::Ticket,
    issued: Instant,
}

#[derive(Resource, Default, Debug)]
struct Driver {
    waiting_since: Option<(u64, Instant)>,
    issued: Option<HostRequest>,
    restoring: Option<Restoration>,
    last_observation: Option<String>,
}

pub(crate) fn register(app: &mut App, runtime: Runtime) {
    present::register(app);
    app.insert_resource(runtime)
        .init_resource::<Driver>()
        // Exclusive systems run on the main thread, where WINIT_WINDOWS lives.
        .add_systems(First, drive);
}

pub(crate) fn pending(world: &World) -> bool {
    world
        .get_resource::<Runtime>()
        .is_some_and(|runtime| runtime.pending().is_some())
        || world
            .get_resource::<Driver>()
            .is_some_and(|driver| driver.restoring.is_some())
}

pub(crate) fn surface_output(surface: &Surface) -> Result<Output, String> {
    let system = if surface.presentation == Presentation::Windowed {
        Some(crate::system_palette::read()?.0)
    } else {
        None
    };
    surface.output(system)
}

/// A pending startup has no active surface yet; later pending requests keep
/// using the preceding active palette until the new host mode is accepted.
pub(crate) fn runtime_output(runtime: &Runtime) -> Result<Output, String> {
    let surface = runtime
        .active()
        .or_else(|| runtime.pending().map(|request| &request.surface))
        .ok_or("no active or requested graphics surface")?;
    surface_output(surface)
}

pub(crate) fn palette_values(world: &World) -> Result<Values, String> {
    let graphics = &world.resource::<Preferences>().settings.graphics;
    let output = match world.get_resource::<Runtime>() {
        Some(runtime) => runtime_output(runtime)?,
        None => Output::from_mode(&graphics.mode)
            .ok_or_else(|| format!("unrecognized saved palette device {:?}", graphics.mode))?,
    };
    Ok(Values {
        brightness: graphics.brightness,
        output,
    })
}

pub(crate) fn apply_palette(world: &mut World, why: &str) -> Result<(), String> {
    let values = palette_values(world)?;
    crate::live_palette::try_apply_to_surface(world, values, why)
}

pub(crate) fn device_action(world: &mut World, action: DeviceAction) -> Result<bool, String> {
    match action {
        DeviceAction::Live(device) => {
            let request = world
                .get_resource_mut::<Runtime>()
                .ok_or("graphics runtime unavailable")?
                .request_device(device)?;
            if let Some(request) = &request {
                note(world, format!("requested device {:?}, {:?}, {:?}; awaiting host and palette acknowledgement",
                    request.surface.device.name, request.surface.presentation, request.surface.pixels()));
            }
            Ok(request.is_some())
        }
        DeviceAction::RestartSoftware => {
            let previous = world.resource::<Preferences>().settings.graphics.clone();
            let changed = world
                .get_resource::<Runtime>()
                .ok_or("graphics runtime unavailable")?
                .restart_preferences(&previous)?;
            world.resource_mut::<Preferences>().settings.graphics = changed;
            if let Err(error) = world.resource_mut::<Preferences>().store() {
                world.resource_mut::<Preferences>().settings.graphics = previous;
                return Err(format!("Software restart was not accepted because local preferences could not be saved: {error}"));
            }
            note(world, "saved Software for the accepted DirectDraw restart; exiting without replacing the active device");
            world.write_message(AppExit::Success);
            Ok(true)
        }
        DeviceAction::MissingTarget(family) => Err(format!(
            "the retained {family:?} row has no resolved device target; current device retained"
        )),
    }
}

fn cameras_match(world: &mut World, pixels: [u32; 2]) -> bool {
    world.query::<&Camera>().iter(world).all(|camera| {
        camera
            .physical_target_size()
            .is_some_and(|size| size.to_array() == pixels)
    })
}

/// This shared acceptance boundary is also exercised by headless fixtures.
/// Their supplied HostState is model evidence, never a native-mode claim.
/// The production driver additionally requires the issued render epoch to
/// have submitted an acquired surface texture before calling this boundary.
pub(crate) fn accept_host(world: &mut World, serial: u64, host: HostState) -> Result<bool, String> {
    let request = world
        .resource::<Runtime>()
        .pending()
        .filter(|request| request.serial == serial)
        .cloned()
        .ok_or("stale native graphics acknowledgement")?;
    if !host.accepts(&request.surface) {
        return Ok(false);
    }
    let pixels = request.surface.pixels();
    let window_matches = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world)
        .is_ok_and(|window| [window.physical_width(), window.physical_height()] == pixels);
    if !window_matches || !cameras_match(world, pixels) {
        return Ok(false);
    }
    let output = surface_output(&request.surface)?;
    let brightness = world.resource::<Preferences>().settings.graphics.brightness;
    crate::live_palette::try_apply_to_surface(
        world,
        Values { brightness, output },
        "accepted graphics device/presentation",
    )?;
    let surface = world.resource_mut::<Runtime>().confirm(serial)?;
    let mode = world.resource::<Runtime>().fullscreen_device().name.clone();
    {
        let mut preferences = world.resource_mut::<Preferences>();
        preferences.settings.graphics.mode = mode;
        preferences.settings.graphics.high_resolution = surface.high;
    }
    crate::display_mode::accept_profile(world, surface.high, surface.device.supports_800());
    crate::startup_menu::refresh_graphics_device(world);
    note(world,format!("confirmed {} profile, active {:?} {:?}, {:?} pixels, actual host {host:?}; saved fullscreen {:?}; world retained",
        world.resource::<Runtime>().profile().name(),surface.device.name,surface.presentation,pixels,
        world.resource::<Runtime>().fullscreen_device().name));
    Ok(true)
}

fn note(world: &World, text: impl AsRef<str>) {
    if let Some(shared) = world.get_resource::<Shared>() {
        shared
            .log
            .lock()
            .unwrap()
            .summary(format!("graphics runtime: {}", text.as_ref()));
    }
}

/// Log only changed transition states, never one line per rendered frame.
/// A fullscreen token can survive native loss of the requested dimensions.
fn observe_transition(world: &mut World, window: Entity, phase: &str, since: Instant) {
    let host = observe_host(window);
    let native = bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        windows.get_window(window).map(|window| (window.has_focus(), window.is_minimized()))
    });
    let bevy = world.get::<Window>(window).map(|window| (
        [window.physical_width(), window.physical_height()], window.mode, window.focused,
    ));
    let cameras: Vec<_> = world.query::<(Entity, &Camera)>().iter(world)
        .map(|(entity, camera)| (entity, camera.physical_target_size()))
        .collect();
    let submitted = present::observed(world);
    let observation = format!("{phase}: host {host:?}; native (focused, minimized) {native:?}; Bevy (pixels, mode, focused) {bevy:?}; cameras {cameras:?}; submitted frame {submitted:?}");
    if world.resource::<Driver>().last_observation.as_ref() != Some(&observation) {
        note(world, format!("transition +{} ms {observation}", since.elapsed().as_millis()));
        world.resource_mut::<Driver>().last_observation = Some(observation);
    }
}

fn fail(world: &mut World, serial: u64, error: String) {
    let initial = world.resource::<Runtime>().active().is_none();
    if !world
        .resource_mut::<Runtime>()
        .reject(serial, error.clone())
    {
        return;
    }
    note(
        world,
        format!("request {serial} failed: {error}; preferences retained"),
    );
    let issued = world.resource_mut::<Driver>().issued.take();
    world.resource_mut::<Driver>().waiting_since = None;
    if let Some(issued) = &issued {
        present::complete(world, issued.frame);
    }
    if initial {
        note(
            world,
            "initial presentation could not be created; exiting with failure",
        );
        world.write_message(AppExit::error());
        return;
    }
    if let Some(issued) = issued {
        if let (Some(surface), Some(mut window)) = (
            issued.previous_surface,
            world.get_mut::<Window>(issued.window),
        ) {
            window.mode = issued.previous_mode;
            window
                .resolution
                .set_physical_resolution(issued.previous_pixels[0], issued.previous_pixels[1]);
            let frame = present::issue(world, issued.window, surface.pixels());
            world.resource_mut::<Driver>().restoring = Some(Restoration {
                window: issued.window,
                surface,
                frame,
                issued: Instant::now(),
            });
            note(world,"requested restoration of the preceding host presentation; input waits for acknowledgement");
        } else {
            note(
                world,
                "preceding window is unavailable for restoration; exiting with failure",
            );
            world.write_message(AppExit::error());
        }
    }
}

fn drive(world: &mut World) {
    let restoration = world
        .resource::<Driver>()
        .restoring
        .as_ref()
        .map(|restore| (restore.window, restore.surface.clone(), restore.frame, restore.issued));
    if let Some((window, surface, frame, issued)) = restoration {
        observe_transition(world, window, "restoring", issued);
        let host = observe_host(window);
        if present::acknowledged(world, frame)
            && host.is_some_and(|host| host.accepts(&surface))
            && world.get::<Window>(window).is_some_and(|window| {
                [window.physical_width(), window.physical_height()] == surface.pixels()
            })
            && cameras_match(world, surface.pixels())
        {
            present::complete(world, frame);
            world.resource_mut::<Driver>().restoring = None;
            note(world,format!("preceding host presentation restored after frame submission epoch {} {:?}; active device, palette and preferences retained", frame.epoch, frame.pixels));
        } else if issued.elapsed() >= HOST_DEADLINE {
            present::complete(world, frame);
            world.resource_mut::<Driver>().restoring = None;
            note(
                world,
                format!("host restoration was not acknowledged ({host:?}); exiting with failure"),
            );
            world.write_message(AppExit::error());
        }
        return;
    }
    let Some(request) = world.resource::<Runtime>().pending().cloned() else {
        return;
    };
    let issued = world
        .resource::<Driver>()
        .issued
        .as_ref()
        .map(|issued| (issued.serial, issued.window, issued.frame, issued.issued));
    if let Some((serial, window, frame, since)) = issued {
        if serial != request.serial {
            fail(
                world,
                request.serial,
                "native owner has a stale request".into(),
            );
            return;
        }
        observe_transition(world, window, "pending", since);
        let host = observe_host(window);
        if let Some(host) = host.filter(|_| present::acknowledged(world, frame)) {
            match accept_host(world, serial, host) {
                Ok(true) => {
                    note(world, format!("accepted frame submission epoch {} window {:?} {:?} pixels; native host, Bevy window, camera and palette also acknowledged", frame.epoch, frame.window, frame.pixels));
                    present::complete(world, frame);
                    world.resource_mut::<Driver>().issued = None;
                    world.resource_mut::<Driver>().waiting_since = None;
                    return;
                }
                Err(error) => {
                    fail(world, serial, error);
                    return;
                }
                Ok(false) => {}
            }
        }
        if since.elapsed() >= HOST_DEADLINE {
            fail(
                world,
                serial,
                format!(
                    "exact {:?} {:?} presentation was not acknowledged; actual host {host:?}",
                    request.surface.presentation,
                    request.surface.pixels()
                ),
            );
        }
        return;
    }
    let since = {
        let mut driver = world.resource_mut::<Driver>();
        let waiting = driver
            .waiting_since
            .get_or_insert((request.serial, Instant::now()));
        if waiting.0 != request.serial {
            *waiting = (request.serial, Instant::now());
        }
        waiting.1
    };
    let entity = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world)
        .ok();
    let Some(entity) = entity else {
        if since.elapsed() >= HOST_DEADLINE {
            fail(
                world,
                request.serial,
                "primary host window unavailable".into(),
            );
        }
        return;
    };
    let mode = match requested_window_mode(world, entity, &request.surface) {
        Ok(Some(mode)) => mode,
        Ok(None) => {
            if since.elapsed() >= HOST_DEADLINE {
                fail(
                    world,
                    request.serial,
                    "host monitor inventory unavailable".into(),
                );
            }
            return;
        }
        Err(error) => {
            fail(world, request.serial, error);
            return;
        }
    };
    // Verify palette access before changing host mode. The final application
    // is repeated after acknowledgement and can still reject a failed read.
    if let Err(error) = surface_output(&request.surface) {
        fail(world, request.serial, error);
        return;
    }
    let previous_surface = world.resource::<Runtime>().active().cloned();
    let mut window = world.get_mut::<Window>(entity).unwrap();
    let previous_mode = window.mode;
    let previous_pixels = [window.physical_width(), window.physical_height()];
    window.mode = mode;
    let [width, height] = request.surface.pixels();
    window.resolution.set_physical_resolution(width, height);
    let frame = present::issue(world, entity, [width, height]);
    world.resource_mut::<Driver>().issued = Some(HostRequest {
        serial: request.serial,
        frame,
        window: entity,
        previous_mode,
        previous_pixels,
        previous_surface,
        issued: Instant::now(),
    });
    note(world,format!("requested exact host {mode:?}, {:?} pixels for {:?}; render epoch {}; awaiting frame submission and actual Winit, camera and palette acceptance",request.surface.pixels(),request.surface.device.name,frame.epoch));
}
