//! One headless transition matrix. Native resize/driver acceptance is not
//! simulated as evidence: only the acknowledgement messages are controlled.
use super::*;
use crate::{
    display_mode::{self, Capabilities, Size},
    gui_view, talk_view, world_build, Shared, Stats,
};
use bevy::{
    camera::{CameraProjection, PerspectiveProjection, RenderTargetInfo},
    window::WindowResized,
};
use std::sync::Mutex;

fn camera_target(world: &mut World, camera: Entity, pixels: [u32; 2], dpi: f32) {
    world
        .get_mut::<Camera>(camera)
        .unwrap()
        .computed
        .target_info = Some(RenderTargetInfo {
        physical_size: UVec2::from_array(pixels),
        scale_factor: dpi,
    });
}

fn window_reply(world: &mut World, window: Entity, pixels: [u32; 2]) {
    let mut w = world.get_mut::<Window>(window).unwrap();
    w.resolution.set_physical_resolution(pixels[0], pixels[1]);
    let (width, height) = (w.width(), w.height());
    world.write_message(WindowResized {
        window,
        width,
        height,
    });
    world.run_schedule(First);
}

fn row(path: &Path, name: &str, size: [u32; 2]) -> [i32; 4] {
    let game = GameDir::open(path).unwrap();
    let tags = std::fs::read(game.find("GAME/Resource/kqGuiTag.TTAG").unwrap()).unwrap();
    let (tags, _) = kq8_data::gui::Tags::parse(&String::from_utf8_lossy(&tags));
    let layout = kq8_data::gui::parse(
        &std::fs::read(game.find("GAME/Resource/Graphics.gui").unwrap()).unwrap(),
    )
    .unwrap();
    kq8_core::gui_draw::absolute_rects(&kq8_core::gui_draw::resolve_layout(&layout, size))
        [&(tags.id_of(name).unwrap() as u32)]
}

fn caption_image(world: &mut World) -> (Handle<Image>, u32, u32) {
    let node = world
        .query_filtered::<&ImageNode, With<talk_view::CaptionPicture>>()
        .single(world)
        .unwrap();
    let image = world.resource::<Assets<Image>>().get(&node.image).unwrap();
    (node.image.clone(), image.width(), image.height())
}

fn hud_pixels(world: &mut World) -> Vec<(u64, u32, u32, u64)> {
    let mut result: Vec<_> = world
        .query_filtered::<(Entity, &ImageNode), With<gui_view::GuiPicture>>()
        .iter(world)
        .map(|(entity, node)| {
            let image = world.resource::<Assets<Image>>().get(&node.image).unwrap();
            (
                entity.to_bits(),
                image.width(),
                image.height(),
                crate::live_palette::fingerprint(image.data.as_ref().unwrap()),
            )
        })
        .collect();
    result.sort_by_key(|r| r.0);
    result
}

fn ray_samples(world: &World, camera: Entity, window: Entity) -> Vec<([f32; 3], [f32; 3])> {
    let c = world.get::<Camera>(camera).unwrap();
    let t = world.get::<GlobalTransform>(camera).unwrap();
    let w = world.get::<Window>(window).unwrap();
    [[0.5, 0.5], [0.2, 0.7], [0.9, 0.1]]
        .into_iter()
        .map(|p| {
            let ray = c
                .viewport_to_world(t, Vec2::new(w.width() * p[0], w.height() * p[1]))
                .unwrap();
            (ray.origin.to_array(), ray.direction.to_array())
        })
        .collect()
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; grouped headless original-size transition matrix"]
fn display_transition_grouped_window_menu_hud_caption_pointer_and_state() {
    let path: std::path::PathBuf = std::env::var_os("KQ8_GAME_DIR")
        .expect("set KQ8_GAME_DIR")
        .into();
    let game = GameDir::open(&path).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = crate::log::Log::create("display-transition").unwrap();
    let mut gui = gui_view::load_gui(&game, &region, &mut log).unwrap();
    let mut caption = talk_view::load_caption(&game, &region, &mut log).unwrap();
    gui.resize([640, 480]);
    caption.resize([640, 480]);
    gui.panels
        .set(kq8_core::panels::Group::Map, true, "transition fixture");
    gui.panels.update(75.0);
    let panels_before = serde_json::to_value(&gui.panels).unwrap();
    let mut preferences = Preferences::load(&path);
    preferences.settings.graphics.high_resolution = false;
    preferences.settings.subtitles = true;
    let mut menu = Menu::open(&path, vec![], false, &mut log).unwrap();
    menu.art.resize(false);
    menu.options = Some(options_menu::Screen::open(&path, &menu.art).unwrap());
    menu.graphics = Some(graphics_menu::Screen::open(&path, &menu.art).unwrap());
    let mut keys = keyboard_menu::Bindings::default();
    let mut keyboard = keyboard_menu::Screen::open(&path, &menu.art).unwrap();
    keyboard.input(Key::Down, &mut keys);
    keyboard.input(Key::Down, &mut keys);
    assert_eq!(
        keyboard.input(Key::Enter, &mut keys),
        keyboard_menu::Action::Capture
    );
    let keyboard_before = keyboard.render(&keys, &menu.art, 0);
    menu.keyboard = Some(keyboard);
    menu.state = Choice::in_game();
    let choice_before = menu.state.clone();
    let clocks_before = (menu.main_clock, menu.option_clock, menu.child_clock);

    let mut app = App::new();
    app.add_message::<WindowResized>();
    app.add_systems(DisplayMenu, show);
    app.insert_resource(Source {
        game: path.clone(),
        sound: false,
    });
    app.insert_resource(preferences);
    app.insert_resource(Shared {
        log: Arc::new(Mutex::new(log)),
        stats: Arc::new(Mutex::new(Stats::default())),
    });
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Time<Real>>();
    app.init_resource::<world_build::WorldChange>();
    app.init_resource::<gui_view::ScreenFrame>();
    app.init_resource::<gui_view::CaptionPlace>();
    app.init_resource::<gui_view::GuiCover>();
    app.init_resource::<gui_view::MapDrag>();
    app.init_resource::<gui_view::BarActions>();
    app.init_resource::<crate::keyboard_input::GameKeys>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<MenuRequest>();
    app.insert_resource(talk_view::Speech { caption: Some(("Connor".into(), "The journey continues through the old kingdom, over the river and beyond the mountains.".into())), ..default() });
    app.insert_resource(gui);
    app.insert_resource(caption);
    app.insert_resource(menu);
    let mut active = kq8_core::active::Active::new(Default::default(), vec![], vec![], [], []);
    active.world.connor_stats = Some(kq8_core::stats::ConnorStats {
        level: 16,
        stats: kq8_core::stats::Stats([100, 20, 30, 40, 123]),
    });
    let connor_before = active.world.connor_stats;
    app.insert_resource(crate::door_view::DoorSim::for_test(
        active,
        crate::log::Log::create("display-transition-active").unwrap(),
    ));
    display_mode::register(&mut app, Capabilities { high: true });
    let world = app.world_mut();
    let window = world
        .spawn((
            Window {
                resolution: [640, 480].into(),
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let projection = PerspectiveProjection {
        fov: 50.0_f32.to_radians(),
        aspect_ratio: 4.0 / 3.0,
        ..default()
    };
    let clip = projection.get_clip_from_view();
    let transform = Transform::from_xyz(4.0, 5.0, 6.0);
    let camera = world
        .spawn((
            Camera3d::default(),
            Projection::Perspective(projection),
            transform,
            GlobalTransform::from(transform),
        ))
        .id();
    world
        .get_mut::<Camera>(camera)
        .unwrap()
        .computed
        .clip_from_view = clip;
    camera_target(world, camera, [640, 480], 1.0);
    let retained = world.spawn(world_build::WorldEntity).id();
    {
        let mut state = world.resource_mut::<world_build::WorldChange>();
        state.current = "Daventry".into();
        state.generation = 3;
        state.game_started = true;
        state.gameplay_available = true;
    }
    world.run_schedule(DisplayMenu);
    redraw(world);
    display_mode::sync_world_presentation(world);
    let mut setup = Schedule::default();
    setup.add_systems((gui_view::setup_gui, talk_view::setup_caption_picture));
    setup.run(world);
    let mut draw = Schedule::default();
    draw.add_systems((gui_view::gui_draw, talk_view::caption_picture_draw).chain());
    draw.run(world);
    let initial_caption = caption_image(world);
    assert_eq!(initial_caption.1, 320);
    let initial_hud = hud_pixels(world);
    let initial_rays = ray_samples(world, camera, window);
    let [left, top, right, bottom] = row(&path, "IDCTL_GRA_800X600", [640, 480]);
    graphics_key(world, Key::Click((left + right) / 2, (top + bottom) / 2));
    assert!(display_mode::pending(world));
    assert!(
        !world
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution
    );
    assert_eq!(world.resource::<Menu>().art.size(), [640, 480]);
    assert!(
        !display_mode::request(world, Size::High).unwrap(),
        "duplicate pending request is a no-op"
    );
    window_reply(world, window, [800, 600]);
    assert!(
        display_mode::pending(world),
        "old camera target prevents committing new pointer dimensions"
    );
    camera_target(world, camera, [800, 600], 1.0);
    world.run_schedule(First);
    assert!(!display_mode::pending(world));
    assert!(
        world
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution
    );
    assert_eq!(world.resource::<Menu>().art.size(), [800, 600]);
    assert_eq!(world.resource::<gui_view::Gui>().screen_size(), [800, 600]);
    assert_eq!(
        world.resource::<talk_view::CaptionStyle>().screen_size(),
        [800, 600]
    );
    draw.run(world);
    let high_caption = caption_image(world);
    assert_eq!(
        high_caption.1, 400,
        "unchanged text rewraps even though physical/canvas ratio remains 1"
    );
    assert_ne!(initial_caption.0, high_caption.0);
    assert_ne!(initial_hud, hud_pixels(world));
    assert_eq!(world.resource::<Menu>().state, choice_before);
    let menu = world.resource::<Menu>();
    assert_eq!(
        (menu.main_clock, menu.option_clock, menu.child_clock),
        clocks_before
    );
    let retained_image = menu.image.clone();
    assert!(
        !display_mode::request(world, Size::High).unwrap(),
        "selected mode does not resize again"
    );
    assert_eq!(world.resource::<Menu>().image, retained_image);
    // High remained the active row through resize: Down selects Lights.
    let mut graphics = world.resource::<Preferences>().settings.graphics.clone();
    let lights_before = graphics.dynamic_lighting;
    {
        let mut menu = world.resource_mut::<Menu>();
        let screen = menu.graphics.as_mut().unwrap();
        screen.input(Key::Down, &mut graphics);
        screen.input(Key::Enter, &mut graphics);
    }
    assert_eq!(graphics.dynamic_lighting, !lights_before);

    // Both pixel sizes and a 150% host scale preserve normalized GUI
    // coordinates and the current projection's picking rays.
    for (pixels, dpi) in [([800, 600], 1.0), ([800, 600], 1.5)] {
        world
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set_scale_factor(dpi);
        camera_target(world, camera, pixels, dpi);
        let w = world.get::<Window>(window).unwrap();
        let (origin, scale) = display_mode::canvas_frame(Vec2::new(w.width(), w.height()), pixels);
        let pointer = Vec2::new(w.width() * 0.25, w.height() * 0.75);
        assert!(((pointer - origin) / scale - Vec2::new(200.0, 450.0)).length() < 0.001);
        for (before, after) in initial_rays.iter().zip(ray_samples(world, camera, window)) {
            for (a, b) in before
                .0
                .into_iter()
                .chain(before.1)
                .zip(after.0.into_iter().chain(after.1))
            {
                assert!((a - b).abs() < 0.00001);
            }
        }
    }
    world
        .get_mut::<Window>(window)
        .unwrap()
        .resolution
        .set_scale_factor(1.0);
    camera_target(world, camera, [800, 600], 1.0);
    let [left, _, right, bottom] = row(&path, "IDCTL_GRA_640X480", [800, 600]);
    graphics_key(world, Key::Click((left + right) / 2, bottom));
    assert!(
        display_mode::pending(world),
        "resized row includes its bottom boundary"
    );
    camera_target(world, camera, [640, 480], 1.0);
    window_reply(world, window, [640, 480]);
    draw.run(world);
    assert_eq!(caption_image(world).1, 320);
    assert_eq!(
        hud_pixels(world),
        initial_hud,
        "round trip regenerates the same HUD pixels from authored data"
    );
    assert_eq!(
        serde_json::to_value(&world.resource::<gui_view::Gui>().panels).unwrap(),
        panels_before
    );
    {
        let menu = &mut *world.resource_mut::<Menu>();
        let keyboard = menu.keyboard.as_mut().unwrap();
        assert!(
            keyboard.capture,
            "resize retains the existing key-capture state"
        );
        assert_eq!(
            keyboard.render(&keys, &menu.art, 0).rgba,
            keyboard_before.rgba
        );
        assert_eq!(
            keyboard.input(Key::Physical(b'Z'), &mut keys),
            keyboard_menu::Action::Bound(2, b'Z')
        );
    }
    assert!(world.get_entity(retained).is_ok());
    assert_eq!(world.resource::<world_build::WorldChange>().generation, 3);
    assert_eq!(
        world
            .resource::<crate::door_view::DoorSim>()
            .active
            .world
            .connor_stats,
        connor_before
    );
    assert_eq!(
        world
            .resource::<crate::door_view::DoorSim>()
            .active
            .world
            .updates,
        0
    );
    assert_eq!(
        world.get::<GlobalTransform>(camera).unwrap().to_matrix(),
        GlobalTransform::from(transform).to_matrix()
    );
    assert_eq!(
        world.get::<Camera>(camera).unwrap().computed.clip_from_view,
        clip
    );

    assert!(display_mode::request(world, Size::High).unwrap());
    window_reply(world, window, [700, 500]);
    assert!(!display_mode::pending(world));
    assert!(
        !world
            .resource::<Preferences>()
            .settings
            .graphics
            .high_resolution
    );
    assert_eq!(world.get::<Window>(window).unwrap().physical_width(), 640);
    assert_eq!(world.resource::<Menu>().art.size(), [640, 480]);

    world
        .resource_mut::<display_mode::Display>()
        .capabilities
        .high = false;
    world
        .resource_mut::<Menu>()
        .graphics
        .as_mut()
        .unwrap()
        .set_high_resolution_available(false);
    assert!(display_mode::request(world, Size::High).is_err());
    let mut graphics = world.resource::<Preferences>().settings.graphics.clone();
    let menu = world.resource::<Menu>();
    let disabled = menu
        .graphics
        .as_ref()
        .unwrap()
        .render(&graphics, &menu.art, 0);
    let menu = &mut *world.resource_mut::<Menu>();
    menu.graphics
        .as_mut()
        .unwrap()
        .set_high_resolution_available(true);
    let enabled = menu
        .graphics
        .as_ref()
        .unwrap()
        .render(&graphics, &menu.art, 0);
    menu.graphics
        .as_mut()
        .unwrap()
        .set_high_resolution_available(false);
    let differences: Vec<_> = disabled
        .rgba
        .chunks_exact(4)
        .zip(enabled.rgba.chunks_exact(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| (i % 640, i / 640))
        .collect();
    assert!(!differences.is_empty());
    assert!(
        differences.iter().all(|(x, y)| (x + y) % 2 == 1),
        "disabled glyphs use the original checker pattern"
    );
    let screen = menu.graphics.as_mut().unwrap();
    let [l, t, r, b] = row(&path, "IDCTL_GRA_800X600", [640, 480]);
    screen.input(Key::Click((l + r) / 2, (t + b) / 2), &mut graphics);
    assert!(
        !graphics.high_resolution,
        "disabled high row rejects the click"
    );
    // Low is selected after the round trip. Down skips disabled High.
    let before = graphics.dynamic_lighting;
    screen.input(Key::Down, &mut graphics);
    screen.input(Key::Enter, &mut graphics);
    assert_eq!(graphics.dynamic_lighting, !before);
    assert!(!graphics.high_resolution);
    println!("DISPLAY TRANSITION PASS: delayed window/camera acknowledgement,640/800 round trip,duplicate/disabled/rejected requests,original menu/HUD/caption refresh,equal-ratio caption invalidation,DPI/picking consistency and retained state; no native resize claim");
}
