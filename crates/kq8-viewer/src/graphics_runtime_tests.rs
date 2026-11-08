use super::*;

fn graphics(name: &str, high: bool) -> Graphics {
    Graphics {
        mode: name.into(),
        high_resolution: high,
        ..Default::default()
    }
}

fn acknowledged(profile: Profile, presentation: Presentation, name: &str, high: bool) -> Runtime {
    let (mut runtime, _) = Runtime::startup(profile, presentation, &graphics(name, high)).unwrap();
    runtime.confirm(1).unwrap();
    runtime
}

#[test]
fn graphics_runtime_profile_and_original_startup_matrix() {
    assert_eq!(Profile::default(), Profile::Direct3D);
    assert_eq!(Presentation::default(), Presentation::Fullscreen);
    assert_eq!(Profile::parse("d3d").unwrap(), Profile::Direct3D);
    assert!(
        Profile::parse("auto").is_err(),
        "host WGPU is not a legacy-device probe"
    );
    for profile in [Profile::Direct3D, Profile::Glide] {
        let inventory = profile.inventory();
        assert_eq!(
            inventory.choice(Family::Software, false),
            Some(&Device::software(true))
        );
        assert!(inventory.choice(Family::Software, true).is_none());
        assert_eq!(
            inventory.choice(Family::Direct3D, false).is_some(),
            profile == Profile::Direct3D
        );
        assert_eq!(
            inventory.choice(Family::Glide, false).is_some(),
            profile == Profile::Glide
        );
    }
    for (profile, name, high, expected_name, expected_high) in [
        (Profile::Direct3D, "Software", true, "Software", false),
        (Profile::Direct3D, "D3D-display", true, "D3D-display", false),
        (Profile::Direct3D, "Glide", true, "Software", false),
        (Profile::Glide, "Glide", true, "Glide", true),
        (Profile::Glide, "Glide", false, "Glide", false),
    ] {
        let original = graphics(name, high);
        let (runtime, validated) =
            Runtime::startup(profile, Presentation::Fullscreen, &original).unwrap();
        assert_eq!(
            original,
            graphics(name, high),
            "startup has no file/preference side effects"
        );
        assert_eq!(
            (validated.mode.as_str(), validated.high_resolution),
            (expected_name, expected_high)
        );
        assert_eq!(runtime.fullscreen_device().name, expected_name);
        assert!(
            runtime.active().is_none(),
            "requested presentation is not acknowledgement"
        );
        assert_eq!(runtime.pending().unwrap().surface.high, expected_high);
    }
    for (profile, name) in [
        (Profile::Direct3D, "D3D-other"),
        (Profile::Direct3D, "glide"),
        (Profile::Direct3D, "software"),
        (Profile::Glide, "D3D-display"),
    ] {
        assert!(
            Runtime::startup(profile, Presentation::Fullscreen, &graphics(name, false)).is_err()
        );
    }
    let saved = graphics("Glide", true);
    assert!(
        Runtime::startup(Profile::Glide, Presentation::Windowed, &saved)
            .unwrap_err()
            .contains("640x480")
    );
    assert_eq!(saved, graphics("Glide", true));
}

#[test]
fn graphics_runtime_window_device_and_palette_are_independent_of_saved_fullscreen() {
    let runtime = acknowledged(
        Profile::Direct3D,
        Presentation::Windowed,
        "D3D-display",
        false,
    );
    assert_eq!(runtime.fullscreen_device().family, Family::Direct3D);
    let active = runtime.active().unwrap();
    assert_eq!(active.device, Device::software(false));
    assert!(!runtime.can_high());
    assert!(active.output(None).is_err());
    let system = [[17, 42, 91]; 20];
    assert_eq!(
        active.output(Some(system)).unwrap(),
        Output::SoftwareWindow { system }
    );
    for (profile, name, output) in [
        (Profile::Direct3D, "Software", Output::Software),
        (Profile::Direct3D, "D3D-display", Output::Direct3D),
        (Profile::Glide, "Glide", Output::Glide),
    ] {
        let runtime = acknowledged(profile, Presentation::Fullscreen, name, false);
        assert_eq!(runtime.active().unwrap().output(None).unwrap(), output);
    }
    let invalid = Surface {
        device: Device::direct3d("display", false),
        presentation: Presentation::Windowed,
        high: false,
    };
    assert!(invalid.output(Some(system)).is_err());
}

#[test]
fn graphics_runtime_live_restart_failure_and_stale_acknowledgement_matrix() {
    let mut runtime = acknowledged(
        Profile::Direct3D,
        Presentation::Windowed,
        "D3D-display",
        false,
    );
    let original = runtime.active().unwrap().clone();
    assert!(
        runtime
            .request_device(Device::software(true))
            .unwrap()
            .is_none(),
        "same family is a no-op"
    );
    assert!(runtime.request_size(true).is_err());
    assert!(runtime
        .request_device(Device::direct3d("other", true))
        .is_err());
    let request = runtime
        .request_device(Device::direct3d("display", true))
        .unwrap()
        .unwrap();
    assert_eq!(runtime.active(), Some(&original));
    assert_eq!(request.surface.presentation, Presentation::Fullscreen);
    assert!(
        runtime.request_size(false).is_err(),
        "one request owns presentation at a time"
    );
    assert!(runtime.confirm(request.serial - 1).is_err());
    assert!(!runtime.reject(request.serial - 1, "stale failure".into()));
    assert!(runtime.reject(request.serial, "host rejected exact mode".into()));
    assert_eq!(runtime.active(), Some(&original));
    assert_eq!(runtime.failure(), Some("host rejected exact mode"));
    assert!(runtime.confirm(request.serial).is_err());
    let retry = runtime
        .request_device(Device::direct3d("display", true))
        .unwrap()
        .unwrap();
    assert!(retry.serial > request.serial);
    assert!(runtime.failure().is_none());
    runtime.confirm(retry.serial).unwrap();
    assert_eq!(runtime.active().unwrap().device.family, Family::Direct3D);
    assert!(
        runtime.request_device(Device::software(true)).is_err(),
        "restart cannot be bypassed by live routing"
    );
    let current = graphics("D3D-display", false);
    let restart = runtime.restart_preferences(&current).unwrap();
    assert_eq!(restart.mode, "Software");
    assert_eq!(current.mode, "D3D-display");
    assert_eq!(
        runtime.active().unwrap().device.family,
        Family::Direct3D,
        "the store/exit owner acknowledges restart"
    );
}

#[test]
fn graphics_runtime_glide_size_and_retained_unsupported_target_matrix() {
    let mut runtime = acknowledged(Profile::Glide, Presentation::Fullscreen, "Software", false);
    assert!(runtime
        .restart_preferences(&graphics("Software", false))
        .is_err());
    let glide = runtime
        .inventory()
        .choice(Family::Glide, false)
        .unwrap()
        .clone();
    let mut forged = glide.clone();
    forged.first_glide_board.as_mut().unwrap().framebuffer_mb = 8;
    assert!(runtime.request_device(forged).is_err());
    let request = runtime.request_device(glide).unwrap().unwrap();
    runtime.confirm(request.serial).unwrap();
    assert!(runtime.can_high());
    let resize = runtime.request_size(true).unwrap().unwrap();
    assert_eq!(runtime.active().unwrap().pixels(), [640, 480]);
    runtime.confirm(resize.serial).unwrap();
    assert_eq!(runtime.active().unwrap().pixels(), [800, 600]);
    assert!(runtime.request_size(true).unwrap().is_none());
    assert!(
        runtime.request_device(Device::software(true)).is_err(),
        "retained disabled target cannot silently lower resolution"
    );
    assert_eq!(runtime.active().unwrap().device.family, Family::Glide);
    assert_eq!(runtime.active().unwrap().pixels(), [800, 600]);
    let resize = runtime.request_size(false).unwrap().unwrap();
    runtime.confirm(resize.serial).unwrap();
    let device = runtime
        .request_device(Device::software(true))
        .unwrap()
        .unwrap();
    runtime.confirm(device.serial).unwrap();
    assert_eq!(runtime.fullscreen_device(), &Device::software(true));
    assert!(!runtime.can_high());
}

fn mode(size: [u32; 2], rate: u32, bits: u16) -> VideoMode {
    VideoMode {
        physical_size: UVec2::from_array(size),
        refresh_rate_millihertz: rate,
        bit_depth: bits,
    }
}

#[test]
fn graphics_runtime_exact_host_mode_and_actual_acceptance_matrix() {
    let modes = [
        mode([1920, 1080], 60_000, 32),
        mode([800, 600], 60_000, 32),
        mode([640, 480], 75_000, 32),
        mode([640, 480], 60_000, 16),
        mode([640, 480], 60_000, 32),
    ];
    assert_eq!(exact_mode(&modes, [640, 480], Some(59_940)), Some(modes[4]));
    assert_eq!(exact_mode(&modes, [640, 480], Some(75_000)), Some(modes[2]));
    assert_eq!(exact_mode(&modes, [800, 600], None), Some(modes[1]));
    assert_eq!(
        exact_mode(&modes[..2], [640, 480], None),
        None,
        "nearest size is not original-size acceptance"
    );
    let fullscreen = Surface {
        device: Device::software(true),
        presentation: Presentation::Fullscreen,
        high: false,
    };
    let accepted = HostState {
        presentation: HostPresentation::Exclusive,
        window_pixels: [640, 480],
        monitor_pixels: Some([640, 480]),
        mode: Some(modes[4]),
    };
    assert!(accepted.accepts(&fullscreen));
    for rejected in [
        HostState {
            presentation: HostPresentation::Borderless,
            ..accepted
        },
        HostState {
            presentation: HostPresentation::Windowed,
            ..accepted
        },
        HostState {
            window_pixels: [1920, 1080],
            ..accepted
        },
        HostState {
            monitor_pixels: Some([1920, 1080]),
            ..accepted
        },
        HostState {
            mode: None,
            ..accepted
        },
        HostState {
            mode: Some(modes[0]),
            ..accepted
        },
    ] {
        assert!(!rejected.accepts(&fullscreen), "{rejected:?}");
    }
    let window = Surface {
        device: Device::software(false),
        presentation: Presentation::Windowed,
        high: false,
    };
    assert!(HostState {
        presentation: HostPresentation::Windowed,
        window_pixels: [640, 480],
        monitor_pixels: Some([1920, 1080]),
        mode: None
    }
    .accepts(&window));
    assert!(!accepted.accepts(&window));

    let mut world = World::new();
    let entity = world.spawn_empty().id();
    assert!(requested_window_mode(&mut world, entity, &fullscreen)
        .unwrap()
        .is_none());
    let monitor = world
        .spawn((
            Monitor {
                name: Some("fixture display".into()),
                physical_width: 1920,
                physical_height: 1080,
                physical_position: IVec2::ZERO,
                refresh_rate_millihertz: Some(60_000),
                scale_factor: 1.0,
                video_modes: modes.to_vec(),
            },
            PrimaryMonitor,
        ))
        .id();
    assert_eq!(
        requested_window_mode(&mut world, entity, &fullscreen).unwrap(),
        Some(WindowMode::Fullscreen(
            MonitorSelection::Entity(monitor),
            VideoModeSelection::Specific(modes[4])
        ))
    );
    world.get_mut::<Monitor>(monitor).unwrap().video_modes = vec![modes[0]];
    assert!(requested_window_mode(&mut world, entity, &fullscreen)
        .unwrap_err()
        .contains("no exclusive fullscreen 640x480"));
    assert_eq!(
        requested_window_mode(&mut world, entity, &window).unwrap(),
        Some(WindowMode::Windowed)
    );
}
