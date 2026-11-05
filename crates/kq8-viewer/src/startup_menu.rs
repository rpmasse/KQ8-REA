//! Bevy input and display for the original startup menu.
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow};
use kq8_audio::{Command, Output};
pub use kq8_core::main_menu::Key;
use kq8_core::main_menu::{Action, Art, Choice};
use kq8_core::save_menu::{self, Mode};
use kq8_core::sound::Groups;
use kq8_data::{aud, GameDir};
use std::{collections::BTreeMap, sync::Arc};
use std::{collections::VecDeque, path::Path};

#[derive(Resource, Default)]
pub struct MenuRequest(pub bool, pub Vec<Key>);

#[derive(Resource)]
struct Source {
    game: std::path::PathBuf,
    sound: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, bevy::ecs::schedule::ScheduleLabel)]
struct DisplayMenu;

/// A failed asynchronous restore has already taken its world down. Present
/// the startup menu again so the player can pick another save or start New.
pub fn return_to_title(world: &mut World) -> Result<(), String> {
    let log = world.resource::<crate::Shared>().log.clone();
    let source = world.resource::<Source>();
    let menu = Menu::open(
        &source.game,
        Vec::new(),
        source.sound,
        &mut log.lock().unwrap(),
    )?;
    world.insert_resource(menu);
    world.run_schedule(DisplayMenu);
    Ok(())
}

pub fn register(app: &mut App, game: &Path, sound: bool) {
    app.insert_resource(Source {
        game: game.to_path_buf(),
        sound,
    })
    .init_resource::<MenuRequest>()
    .add_systems(DisplayMenu, show)
    .add_systems(Update, fade_theme.run_if(resource_exists::<FadingTheme>))
    .add_systems(
        Last,
        (
            open_if_requested,
            finish_capture,
            typed_name,
            step.run_if(resource_exists::<Menu>),
        )
            .chain()
            .before(crate::world_build::world_change_step),
    );
}

fn open_if_requested(world: &mut World) {
    if world.contains_resource::<Menu>()
        || !world
            .resource::<crate::world_build::WorldChange>()
            .accepts_menu()
    {
        return;
    }
    let requested = std::mem::take(&mut *world.resource_mut::<MenuRequest>());
    let Some(connor) = world.get_resource::<crate::connor_view::ConnorSim>() else {
        return;
    };
    let dead = connor.walker.mode == kq8_core::connor::Mode::Dead;
    let escape = world
        .resource::<ButtonInput<KeyCode>>()
        .just_pressed(KeyCode::Escape);
    if !dead && (connor.hands_off || (!escape && !requested.0)) {
        return;
    }
    let log = world.resource::<crate::Shared>().log.clone();
    let source = world.resource::<Source>();
    let menu = Menu::open(
        &source.game,
        requested.1,
        source.sound,
        &mut log.lock().unwrap(),
    );
    match menu {
        Err(e) => {
            log.lock()
                .unwrap()
                .summary(format!("menu: could not open: {e}"));
        }
        Ok(mut menu) => {
            if !dead {
                menu.state = Choice::in_game();
            }
            if let Some(sim) = world.get_resource::<crate::sound_view::SoundSim>() {
                sim.menu_pause(true);
            }
            // Freeze the clock immediately. Capture the gameplay view before
            // an opaque menu node is added to the render target (rule 7).
            if !dead {
                let entity = world
                    .spawn(Screenshot::primary_window())
                    .observe(captured)
                    .id();
                menu.capture = Some((entity, std::time::Instant::now()));
                world.insert_resource(menu);
            } else {
                world.insert_resource(menu);
                world.run_schedule(DisplayMenu);
            }
        }
    }
}

fn captured(
    event: On<ScreenshotCaptured>,
    menu: Option<ResMut<Menu>>,
    gui: Option<Res<crate::gui_view::Gui>>,
) {
    let (Some(mut menu), Some(gui)) = (menu, gui) else {
        return;
    };
    if menu.capture.is_none_or(|(e, _)| e != event.entity) {
        return;
    }
    if let Ok(dynamic) = event.image.clone().try_into_dynamic() {
        let pixels = dynamic.to_rgba8();
        menu.current_thumbnail = save_menu::thumbnail(
            pixels.as_raw(),
            pixels.width() as usize,
            pixels.height() as usize,
            &gui.palette,
        );
    }
    menu.capture = None;
}

fn finish_capture(world: &mut World) {
    let Some(menu) = world.get_resource::<Menu>() else {
        return;
    };
    if menu.shown.is_some() {
        return;
    }
    if let Some((_, started)) = menu.capture {
        // A minimised window or lost GPU surface can skip readback. Use the
        // original no-thumbnail picture and keep the save menu accessible.
        if started.elapsed().as_secs_f32() < 2.0 {
            return;
        }
    }
    world.resource_mut::<Menu>().capture = None;
    let available = world.resource::<Menu>().current_thumbnail.is_some();
    world
        .resource::<crate::Shared>()
        .log
        .lock()
        .unwrap()
        .summary(format!(
            "menu: captured 160x120 gameplay thumbnail={available}"
        ));
    world.run_schedule(DisplayMenu);
}

pub fn ordinary_launch(args: &[String]) -> bool {
    let mut iter = args.iter().map(String::as_str);
    while let Some(flag) = iter.next() {
        match flag {
            "--game" | "--renderer" | "--sound" | "--light" | "--haze" | "--dynamic"
            | "--corners" | "--palette" => {
                iter.next();
            }
            "--smooth" => {}
            _ => return false,
        }
    }
    true
}

pub fn parse_keys(input: &str) -> Result<Vec<Key>, String> {
    let mut out = Vec::new();
    for token in input.split(',') {
        if let Some(text) = token.strip_prefix("type=") {
            out.extend(text.chars().map(Key::Character));
            continue;
        }
        out.push(match token {
            "up" => Key::Up,
            "down" => Key::Down,
            "left" => Key::Left,
            "right" => Key::Right,
            "enter" => Key::Enter,
            "release" => Key::Release,
            "escape" => Key::Escape,
            "delete" => Key::Delete,
            "backspace" => Key::Backspace,
            "tab" => Key::Tab,
            _ => return Err(format!("unsupported --menu-input key {token}")),
        });
    }
    Ok(out)
}

fn image(art: &Art, choice: &Choice) -> Image {
    canvas_image(art.render(choice))
}

fn canvas_image(canvas: kq8_core::gui_draw::Canvas) -> Image {
    let mut image = crate::object_light::texture_image(
        canvas.width,
        canvas.height,
        canvas.rgba,
        bevy::image::ImageAddressMode::ClampToEdge,
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}

#[derive(Resource)]
pub struct Menu {
    art: Art,
    state: Choice,
    slot_page: Option<(save_menu::Screen, save_menu::Art)>,
    typed: Vec<char>,
    capture: Option<(Entity, std::time::Instant)>,
    current_thumbnail: Option<Vec<u8>>,
    keys: VecDeque<Key>,
    shown: Option<std::time::Instant>,
    image: Handle<Image>,
    audio: MenuAudio,
    old_cursor: Option<CursorIcon>,
    return_pointer: Option<Vec2>,
    paused_at_update: Option<u64>,
}

impl Menu {
    pub fn open(
        path: &Path,
        keys: Vec<Key>,
        sound: bool,
        log: &mut crate::log::Log,
    ) -> Result<Self, String> {
        Ok(Self {
            art: Art::open(path)?,
            state: Choice::default(),
            slot_page: None,
            typed: Vec::new(),
            capture: None,
            current_thumbnail: None,
            keys: keys.into(),
            shown: None,
            image: default(),
            audio: MenuAudio::open(path, sound, log)?,
            old_cursor: None,
            return_pointer: None,
            paused_at_update: None,
        })
    }
}

struct MenuAudio {
    output: Option<Output>,
    sounds: BTreeMap<&'static str, Arc<aud::Sound>>,
    groups: Groups,
    next_id: u64,
}

const THEME_ID: u64 = 0;

impl MenuAudio {
    fn open(path: &Path, sound: bool, log: &mut crate::log::Log) -> Result<Self, String> {
        let game = GameDir::open(path).map_err(|e| e.to_string())?;
        let mut groups = Groups::default();
        if let Some(bytes) = game.find("Options.cs").and_then(|p| std::fs::read(p).ok()) {
            groups.apply_script(&bytes);
        }
        let mut sounds = BTreeMap::new();
        for name in ["theme.aud", "click.aud", "intrface.aud"] {
            let Some(path) = aud::find(&game, "", name) else {
                log.summary(format!("menu: missing sound {name}"));
                continue;
            };
            match std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| aud::parse(&b))
            {
                Ok(sample) => {
                    sounds.insert(name, Arc::new(sample));
                }
                Err(e) => log.summary(format!("menu: cannot decode {name}: {e}")),
            }
        }
        let output = if sound {
            Output::open()
                .map_err(|e| log.summary(format!("menu: no audio output: {e}")))
                .ok()
        } else {
            None
        };
        Ok(Self {
            output,
            sounds,
            groups,
            next_id: 1,
        })
    }

    fn play(&mut self, name: &'static str) {
        let theme = name == "theme.aud";
        let gain = self.groups.loudness(if theme { 4 } else { 1 });
        if let (Some(out), Some(sound)) = (&self.output, self.sounds.get(name)) {
            out.send([Command::Start {
                id: if theme { THEME_ID } else { self.next_id },
                sound: sound.clone(),
                looping: theme,
                gains: [gain; 2],
            }]);
        }
        self.next_id += 1;
    }
}

#[derive(Resource)]
pub struct FadingTheme {
    output: Output,
    started: std::time::Instant,
    gain: f32,
}

pub fn fade_theme(theme: Res<FadingTheme>, mut commands: Commands) {
    let remaining = (1.0 - theme.started.elapsed().as_secs_f32()).max(0.0);
    if remaining == 0.0 {
        theme.output.send([Command::Stop { id: THEME_ID }]);
        commands.remove_resource::<FadingTheme>();
    } else {
        theme.output.send([Command::Gains {
            id: THEME_ID,
            gains: [theme.gain * remaining; 2],
        }]);
    }
}

#[derive(Component)]
struct MenuRoot;

pub fn show(
    mut commands: Commands,
    mut menu: ResMut<Menu>,
    mut images: ResMut<Assets<Image>>,
    shared: Res<crate::Shared>,
    camera: Query<(), With<Camera3d>>,
    window: Query<(Entity, Option<&CursorIcon>), With<PrimaryWindow>>,
    sim: Option<Res<crate::door_view::DoorSim>>,
) {
    match crate::world_build::scan_slots() {
        Ok((slots, corrupt)) => {
            menu.state.set_saves(!slots.is_empty());
            if let Some(id) = corrupt {
                menu.state.ask_corrupt(id);
            }
        }
        Err(e) => shared
            .log
            .lock()
            .unwrap()
            .summary(format!("menu: could not scan saves: {e}")),
    }
    menu.image = images.add(image(&menu.art, &menu.state));
    menu.shown = Some(std::time::Instant::now());
    menu.paused_at_update = sim.map(|s| s.active.world.updates);
    menu.audio.play("theme.aud");
    if let Ok((window, cursor)) = window.single() {
        menu.old_cursor = cursor.cloned();
        let handle = images.add(canvas_image(menu.art.cursor()));
        commands
            .entity(window)
            .insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                handle,
                hotspot: (0, 0),
                ..default()
            })));
    }
    if camera.is_empty() {
        commands.spawn((Camera2d, MenuRoot));
    }
    commands
        .spawn((
            MenuRoot,
            GlobalZIndex(i32::MAX),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|p| {
            p.spawn((
                ImageNode {
                    image: menu.image.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: Val::Px(640.),
                    height: Val::Px(480.),
                    ..default()
                },
                MenuPicture,
            ));
        });
    shared.log.lock().unwrap().summary(format!(
        "menu: shown {:?}, theme.aud, world clock stopped at {:?}",
        menu.state, menu.paused_at_update
    ));
}

#[derive(Component)]
pub struct MenuPicture;

fn typed_name(
    mut chars: MessageReader<bevy::input::keyboard::KeyboardInput>,
    menu: Option<ResMut<Menu>>,
) {
    let Some(mut menu) = menu else {
        chars.clear();
        return;
    };
    let editing = menu
        .slot_page
        .as_ref()
        .is_some_and(|(s, _)| s.mode == Mode::Save);
    for event in chars.read() {
        if editing && event.state.is_pressed() {
            if let Some(text) = &event.text {
                menu.typed.extend(text.chars().filter(|c| !c.is_control()));
            }
        }
    }
}

fn redraw(world: &mut World) {
    let menu = world.resource::<Menu>();
    let image = match &menu.slot_page {
        Some((state, art)) => canvas_image(art.render(state, &menu.art)),
        None => image(&menu.art, &menu.state),
    };
    let handle = menu.image.clone();
    if let Some(mut target) = world.resource_mut::<Assets<Image>>().get_mut(&handle) {
        *target = image;
    }
}

fn open_slots(world: &mut World, mode: Mode) -> Result<(), String> {
    let slots = crate::world_build::list_slots()?;
    let path = &world.resource::<Source>().game;
    let art = save_menu::Art::open(path, mode)?;
    let mut state = save_menu::Screen::new(mode, slots);
    let preview = match (mode, state.selected_slot()) {
        (Mode::Save, _) => world.resource::<Menu>().current_thumbnail.clone(),
        (Mode::Load, Some(slot)) => crate::world_build::thumbnail(slot).unwrap_or_else(|e| {
            world
                .resource::<crate::Shared>()
                .log
                .lock()
                .unwrap()
                .summary(format!("menu: thumbnail {slot}: {e}"));
            None
        }),
        _ => None,
    };
    state.set_preview(preview.as_deref(), &world.resource::<Menu>().art);
    world.resource_mut::<Menu>().slot_page = Some((state, art));
    redraw(world);
    Ok(())
}

fn slot_key(world: &mut World, key: Key) -> Action {
    use save_menu::Command;
    let (mode, old_slot, command) = {
        let mut menu = world.resource_mut::<Menu>();
        let (state, art) = menu.slot_page.as_mut().unwrap();
        let mode = state.mode;
        let old_slot = state.selected_slot();
        let command = state.input(key, art);
        let cue = match key {
            Key::Up | Key::Down | Key::Tab | Key::Left | Key::Right => Some("click.aud"),
            Key::Enter | Key::Click(_, _) => Some("intrface.aud"),
            _ => None,
        };
        if let Some(name) = cue {
            menu.audio.play(name);
        }
        (mode, old_slot, command)
    };
    // Re-scan after each mutation. Subscreens leave the clock and theme alone.
    let log = world.resource::<crate::Shared>().log.clone();
    if command != Command::None {
        log.lock()
            .unwrap()
            .summary(format!("menu: {mode:?} screen -> {command:?}"));
    }
    let restore = matches!(command, Command::Load(_));
    let result = match command {
        Command::None => Ok(()),
        Command::Save(id, label) => crate::world_build::save_slot_with_thumbnail(
            world,
            id,
            &label,
            world.resource::<Menu>().current_thumbnail.clone(),
        )
        .map(|()| {
            let mut menu = world.resource_mut::<Menu>();
            menu.state.set_saves(true);
            menu.slot_page = None;
        }),
        Command::Load(id) => crate::world_build::load_slot(world, id),
        Command::Back => {
            world.resource_mut::<Menu>().slot_page = None;
            Ok(())
        }
        Command::Delete(id) => crate::world_build::delete_slot(id)
            .and_then(|()| crate::world_build::list_slots())
            .map(|slots| {
                let mut menu = world.resource_mut::<Menu>();
                menu.state.set_saves(!slots.is_empty());
                if slots.is_empty() && mode == Mode::Load {
                    menu.slot_page = None;
                } else if let Some((page, _)) = &mut menu.slot_page {
                    page.after_delete(slots);
                }
            }),
    };
    match result {
        Ok(()) if restore => return Action::Resume,
        Err(e) => {
            log.lock()
                .unwrap()
                .summary(format!("menu: {mode:?} failed: {e}"));
            if let Some((page, _)) = &mut world.resource_mut::<Menu>().slot_page {
                page.failed();
            }
        }
        _ => {}
    }
    let menu = &mut *world.resource_mut::<Menu>();
    if let Some((state, _)) = &mut menu.slot_page {
        if mode == Mode::Load && state.selected_slot() != old_slot {
            let preview =
                state
                    .selected_slot()
                    .and_then(|slot| match crate::world_build::thumbnail(slot) {
                        Ok(image) => image,
                        Err(e) => {
                            log.lock()
                                .unwrap()
                                .summary(format!("menu: thumbnail {slot}: {e}"));
                            None
                        }
                    });
            state.set_preview(preview.as_deref(), &menu.art);
        }
    }
    redraw(world);
    Action::Stay
}

pub fn step(world: &mut World) {
    let Some(win) = world
        .query_filtered::<&Window, With<bevy::window::PrimaryWindow>>()
        .iter(world)
        .next()
    else {
        return;
    };
    let size = Vec2::new(win.width(), win.height());
    let scale = (size.x / 640.).min(size.y / 480.);
    if scale <= 0. {
        return;
    }
    let pos = win
        .cursor_position()
        .map(|p| (p - (size - Vec2::new(640., 480.) * scale) * 0.5) / scale);
    for mut node in world
        .query_filtered::<&mut Node, With<MenuPicture>>()
        .iter_mut(world)
    {
        if node.width != Val::Px(scale * 640.) || node.height != Val::Px(scale * 480.) {
            node.width = Val::Px(scale * 640.);
            node.height = Val::Px(scale * 480.);
        }
    }
    if world
        .resource::<Menu>()
        .shown
        .is_none_or(|at| at.elapsed().as_secs_f32() < 0.2)
    {
        return;
    }
    let mut input = Vec::new();
    let keys = world.resource::<ButtonInput<KeyCode>>();
    for (code, key) in [
        (KeyCode::ArrowUp, Key::Up),
        (KeyCode::ArrowDown, Key::Down),
        (KeyCode::ArrowLeft, Key::Left),
        (KeyCode::ArrowRight, Key::Right),
        (KeyCode::Escape, Key::Escape),
        (KeyCode::Enter, Key::Enter),
        (KeyCode::Backspace, Key::Backspace),
        (KeyCode::Delete, Key::Delete),
        (KeyCode::Tab, Key::Tab),
    ] {
        if keys.just_pressed(code) {
            input.push((key, true));
        }
    }
    if keys.just_released(KeyCode::Enter) {
        input.push((Key::Release, true));
    }
    let mouse = world.resource::<ButtonInput<MouseButton>>();
    if let Some(p) = pos {
        if mouse.just_pressed(MouseButton::Left) {
            input.push((Key::Click(p.x.floor() as i32, p.y.floor() as i32), true));
        } else if mouse.pressed(MouseButton::Left)
            && world
                .resource::<Menu>()
                .slot_page
                .as_ref()
                .is_some_and(|(s, _)| s.dragging())
        {
            input.push((Key::Drag(p.x.floor() as i32, p.y.floor() as i32), true));
        }
    }
    if mouse.just_released(MouseButton::Left) && world.resource::<Menu>().slot_page.is_some() {
        input.push((Key::MouseUp, true));
    }
    input.extend(
        std::mem::take(&mut world.resource_mut::<Menu>().typed)
            .into_iter()
            .map(|c| (Key::Character(c), true)),
    );
    if let Some(key) = world.resource_mut::<Menu>().keys.pop_front() {
        input.push((key, false));
    }
    if input.is_empty() {
        return;
    }
    for (key, live) in input {
        let (action, image, cue, pointer) = if world.resource::<Menu>().slot_page.is_some() {
            (slot_key(world, key), None, None, None)
        } else {
            let mut menu = world.resource_mut::<Menu>();
            let rows = menu.art.rows;
            let old = menu.state.clone();
            let cue = old.sound(key, &rows);
            if let Some(name) = cue {
                menu.audio.play(name);
            }
            let action = menu.state.input(key, &rows);
            let pointer =
                if live && old.key_pointer().is_none() && menu.state.key_pointer().is_some() {
                    let origin = (size - Vec2::new(640., 480.) * scale) * 0.5;
                    menu.return_pointer = pos.map(|p| p * scale + origin);
                    menu.state
                        .key_pointer()
                        .map(|[x, y]| origin + Vec2::new(x as f32, y as f32) * scale)
                } else if old.key_pointer().is_some() && menu.state.key_pointer().is_none() {
                    menu.return_pointer.take()
                } else {
                    None
                };
            let image =
                (menu.state != old).then(|| (menu.image.clone(), image(&menu.art, &menu.state)));
            (action, image, cue, pointer)
        };
        if let Some(pointer) = pointer {
            if let Ok(mut window) = world
                .query_filtered::<&mut Window, With<PrimaryWindow>>()
                .single_mut(world)
            {
                window.set_cursor_position(Some(pointer));
            }
        }
        if let Some((handle, image)) = image {
            if let Some(mut target) = world.resource_mut::<Assets<Image>>().get_mut(&handle) {
                *target = image;
            }
        }
        world
            .resource::<crate::Shared>()
            .log
            .lock()
            .unwrap()
            .summary(format!(
                "menu: {key:?} -> {:?}, {action:?}, cue={cue:?}",
                world.resource::<Menu>().state
            ));
        match action {
            Action::Quit => {
                world.write_message(AppExit::Success);
                return;
            }
            Action::Start(_) | Action::Resume => {
                let roots: Vec<_> = world
                    .query_filtered::<Entity, With<MenuRoot>>()
                    .iter(world)
                    .collect();
                for e in roots {
                    world.despawn(e);
                }
                if let Some(menu) = world.remove_resource::<Menu>() {
                    if let Some(before) = menu.paused_at_update {
                        let after = world
                            .get_resource::<crate::door_view::DoorSim>()
                            .map(|s| s.active.world.updates);
                        world
                            .resource::<crate::Shared>()
                            .log
                            .lock()
                            .unwrap()
                            .summary(format!(
                                "menu: leaving at update {after:?}, entered at {before}"
                            ));
                    }
                    if let Some(window) = world
                        .query_filtered::<Entity, With<PrimaryWindow>>()
                        .iter(world)
                        .next()
                    {
                        world
                            .entity_mut(window)
                            .insert(menu.old_cursor.unwrap_or_default());
                    }
                    if let Some(output) = menu.audio.output {
                        world.insert_resource(FadingTheme {
                            output,
                            gain: menu.audio.groups.loudness(4),
                            started: std::time::Instant::now(),
                        });
                    }
                }
                if let Action::Start(level) = action {
                    world.resource_mut::<crate::door_view::HeldItem>().1 = Some(level);
                    crate::world_build::first_world(world);
                } else if let Some(sim) = world.get_resource::<crate::sound_view::SoundSim>() {
                    sim.menu_pause(false);
                }
                return;
            }
            // The options screen and intro movie have separate M4/M6 deliveries.
            Action::Save | Action::Load => {
                let mode = if action == Action::Save {
                    Mode::Save
                } else {
                    Mode::Load
                };
                if let Err(e) = open_slots(world, mode) {
                    world
                        .resource::<crate::Shared>()
                        .log
                        .lock()
                        .unwrap()
                        .summary(format!("menu: could not open {mode:?} screen: {e}"));
                }
            }
            Action::DeleteCorrupt(id) => {
                match crate::world_build::delete_slot(id)
                    .and_then(|_| crate::world_build::list_slots())
                {
                    Ok(slots) => world
                        .resource_mut::<Menu>()
                        .state
                        .set_saves(!slots.is_empty()),
                    Err(e) => world
                        .resource::<crate::Shared>()
                        .log
                        .lock()
                        .unwrap()
                        .summary(format!("menu: could not delete unreadable save {id}: {e}")),
                }
                redraw(world);
            }
            Action::Options | Action::Replay | Action::Stay => {}
        }
    }
}
