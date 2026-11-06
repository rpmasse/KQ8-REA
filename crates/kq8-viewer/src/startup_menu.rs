//! Bevy input and display for the original startup menu.
use crate::keyboard_input::Keyboard;
use crate::option_settings::Preferences;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow};
use kq8_audio::{Command, Output};
use kq8_core::credits;
use kq8_core::graphics_menu;
use kq8_core::keyboard_menu;
pub use kq8_core::main_menu::Key;
use kq8_core::main_menu::{Action, Art, Choice};
use kq8_core::options_menu;
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
    .insert_resource(Preferences::load(game))
    .add_systems(First, crate::option_settings::apply_to_world)
    .init_resource::<MenuRequest>()
    .init_resource::<PresentationQueue>()
    .add_systems(DisplayMenu, show)
    .add_systems(Update, fade_theme.run_if(resource_exists::<FadingTheme>))
    .add_systems(
        Last,
        (
            open_presentation,
            open_if_requested,
            finish_capture,
            typed_name,
            step.run_if(resource_exists::<Menu>),
        )
            .chain()
            .before(crate::world_build::world_change_step),
    );
}

#[derive(Default, Resource)]
struct PresentationQueue(VecDeque<credits::Presentation>);

pub fn movie_active(world: &World) -> bool {
    world
        .get_resource::<Menu>()
        .is_some_and(|menu| menu.movie.is_some())
}

fn open_presentation(world: &mut World) {
    if let Some(mut sim) = world.get_resource_mut::<crate::door_view::DoorSim>() {
        let requests = std::mem::take(&mut sim.active.world.presentations);
        world.resource_mut::<PresentationQueue>().0.extend(requests);
    }
    if !world.contains_resource::<Menu>() {
        present_next(world);
    }
}

fn present_next(world: &mut World) {
    use credits::Presentation;
    while let Some(request) = world.resource_mut::<PresentationQueue>().0.pop_front() {
        let result = match request {
            Presentation::EndGame => {
                crate::world_build::end_session(world);
                if !world.resource::<PresentationQueue>().0.is_empty() {
                    continue;
                }
                return_to_title(world)
            }
            Presentation::Movie(name) => {
                let after = if world.contains_resource::<crate::door_view::DoorSim>() {
                    MovieReturn::Gameplay
                } else {
                    MovieReturn::Title
                };
                open_movie(world, &name, after)
            }
            Presentation::Credits { score } => open_ending_credits(world, score),
        };
        if let Err(e) = result {
            world
                .resource::<crate::Shared>()
                .log
                .lock()
                .unwrap()
                .summary(format!("presentation: {e}"));
            continue;
        }
        break;
    }
}

/// The movie player can call this after the preceding final AVI has ended.
/// It also works after the completed region has already been taken down.
pub fn open_ending_credits(world: &mut World, score: Option<i64>) -> Result<(), String> {
    let source = world.resource::<Source>();
    let log = world.resource::<crate::Shared>().log.clone();
    let page = credits::Screen::open(&source.game, score)?;
    if world.contains_resource::<Menu>() {
        let has_saves = !crate::world_build::list_slots()?.is_empty();
        let mut menu = world.resource_mut::<Menu>();
        menu.state = Choice::default();
        menu.state.set_saves(has_saves);
        menu.options = None;
        menu.graphics = None;
        menu.keyboard = None;
        menu.slot_page = None;
        if let Some(output) = &menu.audio.output {
            output.send([Command::Stop { id: THEME_ID }]);
        }
        menu.credits = Some(page);
        credit_status(world, "doCredits");
        redraw(world);
        return Ok(());
    }
    let mut menu = Menu::open(
        &source.game,
        Vec::new(),
        source.sound,
        &mut log.lock().unwrap(),
    )?;
    menu.credits = Some(page);
    if let Some(sound) = world.get_resource::<crate::sound_view::SoundSim>() {
        sound.menu_pause(true);
    }
    world.insert_resource(menu);
    world.run_schedule(DisplayMenu);
    credit_status(world, "doCredits");
    Ok(())
}

fn credit_status(world: &World, source: &str) {
    let menu = world.resource::<Menu>();
    let Some(page) = menu.credits.as_ref() else {
        return;
    };
    world
        .resource::<crate::Shared>()
        .log
        .lock()
        .unwrap()
        .summary(format!(
            "credits: {source} 400.MSG card {}/{}, team photo {}, score {:?}",
            page.index + 1,
            page.count(),
            page.team_photo(),
            page.score()
        ));
}

fn end_credits(world: &mut World) {
    let mut menu = world.resource_mut::<Menu>();
    menu.credits = None;
    if menu.options.is_none() {
        menu.audio.play("theme.aud");
    }
    let next = if menu.options.is_some() {
        "Options"
    } else {
        "title"
    };
    world
        .resource::<crate::Shared>()
        .log
        .lock()
        .unwrap()
        .summary(format!("credits: returned to {next}"));
    redraw(world);
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
            if dead {
                if let Some(mut gui) = world.get_resource_mut::<crate::gui_view::Gui>() {
                    if gui.popup.take().is_some() {
                        gui.panels.set(kq8_core::panels::Group::Popup, false, "the death menu opens");
                    }
                }
            } else {
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
        if let Some(word) = token.strip_prefix("bind=") {
            let v = keyboard_menu::virtual_key(word)
                .ok_or_else(|| format!("unsupported physical key {word}"))?;
            out.push(Key::Physical(v));
            continue;
        }
        if let Some(text) = token.strip_prefix("type=") {
            out.extend(text.chars().map(Key::Character));
            continue;
        }
        if let Some((op, coords)) = token.split_once('=') {
            if matches!(op, "click" | "drag") {
                let (x, y) = coords
                    .split_once('x')
                    .and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)))
                    .ok_or_else(|| format!("expected {op}=XxY"))?;
                out.push(if op == "click" {
                    Key::Click(x, y)
                } else {
                    Key::Drag(x, y)
                });
                continue;
            }
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
            "mouse-up" => Key::MouseUp,
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

#[derive(Clone, Copy, Debug)]
enum MovieReturn {
    Menu,
    Gameplay,
    Title,
}

struct MoviePage {
    name: String,
    after: MovieReturn,
    playback: Result<crate::movie_player::Playback, String>,
    cursor_visible: Option<bool>,
}

#[derive(Resource)]
pub struct Menu {
    art: Art,
    state: Choice,
    slot_page: Option<(save_menu::Screen, save_menu::Art)>,
    options: Option<options_menu::Screen>,
    graphics: Option<graphics_menu::Screen>,
    keyboard: Option<keyboard_menu::Screen>,
    movie: Option<MoviePage>,
    credits: Option<credits::Screen>,
    option_clock: std::time::Instant,
    option_frame: usize,
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
        let graphics = Preferences::load(path).settings.graphics;
        let mut art = Art::open(path)?;
        art.set_palette(graphics.brightness,kq8_data::palette::Output::from_mode(&graphics.mode).unwrap_or(kq8_data::palette::Output::Direct3D));
        Ok(Self {
            art,
            state: Choice::default(),
            slot_page: None,
            options: None,
            graphics: None,
            keyboard: None,
            movie: None,
            credits: None,
            option_clock: std::time::Instant::now(),
            option_frame: 0,
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
const MOVIE_ID: u64 = u64::MAX;

impl MenuAudio {
    fn open(path: &Path, sound: bool, log: &mut crate::log::Log) -> Result<Self, String> {
        let game = GameDir::open(path).map_err(|e| e.to_string())?;
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
            groups: Groups::default(),
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
    preferences: Res<Preferences>,
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
    menu.image = images.add(if let Some(page) = &menu.movie {
        canvas_image(
            menu.art
                .movie_screen(page.playback.is_err().then_some(&page.name)),
        )
    } else if let Some(page) = &menu.credits {
        canvas_image(page.render(&menu.art))
    } else {
        image(&menu.art, &menu.state)
    });
    menu.shown = Some(std::time::Instant::now());
    menu.paused_at_update = sim.map(|s| s.active.world.updates);
    preferences.settings.apply_groups(&mut menu.audio.groups);
    // Credits.KQ specifies only selection and movement sounds. Options
    // already has the menu theme. The ending first enters the title menu
    // when its credits close, which is when to start the title theme.
    let cue = if menu.movie.is_some() {
        "movie"
    } else if menu.credits.is_none() {
        menu.audio.play("theme.aud");
        "theme.aud"
    } else {
        "ending credits"
    };
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
        "menu: shown {:?}, {cue}, world clock stopped at {:?}",
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
    let image = if let Some(page) = &menu.movie {
        canvas_image(
            menu.art
                .movie_screen(page.playback.is_err().then_some(&page.name)),
        )
    } else if let Some(page) = &menu.credits {
        canvas_image(page.render(&menu.art))
    } else if let Some(page) = &menu.graphics {
        canvas_image(page.render(&world.resource::<Preferences>().settings.graphics, &menu.art, menu.option_frame))
    } else if let Some(page) = &menu.keyboard {
        canvas_image(page.render(
            &world.resource::<Keyboard>().active,
            &menu.art,
            menu.option_frame,
        ))
    } else if let Some(page) = &menu.options {
        canvas_image(page.render(
            &world.resource::<Preferences>().settings,
            &menu.art,
            menu.option_frame,
        ))
    } else {
        match &menu.slot_page {
            Some((state, art)) => canvas_image(art.render(state, &menu.art)),
            None => image(&menu.art, &menu.state),
        }
    };
    let handle = menu.image.clone();
    if let Some(mut target) = world.resource_mut::<Assets<Image>>().get_mut(&handle) {
        *target = image;
    }
}

/// The slider applies to the active menu surface immediately (Q186).
/// Refresh cached backdrop pixels while retaining each panel's selection.
pub fn palette_changed(world: &mut World, values: crate::live_palette::Values, why: &str) {
    if !world.contains_resource::<Menu>() { return; }
    let (handle,cursor) = {
        let mut menu = world.resource_mut::<Menu>();
        let Menu { art,graphics,options,.. } = &mut *menu;
        art.set_palette(values.brightness,values.output);
        if let Some(page) = graphics { page.palette_changed(art); }
        if let Some(page) = options { page.palette_changed(art); }
        (menu.image.clone(),canvas_image(menu.art.cursor()))
    };
    let sample = |world:&World| {
        let menu = world.resource::<Menu>();
        let image = world.resource::<Assets<Image>>().get(&handle)?;
        let rgba = image.data.as_deref()?;
        match menu.graphics.as_ref() {
            Some(page) => Some(crate::live_palette::fingerprint(&page.palette_sample(rgba,image.width(),image.height())?)),
            None => Some(crate::live_palette::fingerprint(rgba)),
        }
    };
    let before = sample(world);
    redraw(world);
    let after = sample(world);
    let cursor = world.resource_mut::<Assets<Image>>().add(cursor);
    if let Ok(window) = world.query_filtered::<Entity,With<PrimaryWindow>>().single(world) {
        world.entity_mut(window).insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage { handle:cursor,hotspot:(0,0),..default() })));
    }
    world.resource::<crate::Shared>().log.lock().unwrap().summary(format!("palette: menu live brightness {}, layout-masked static pixels before {before:?}, after {after:?}; Graphics and Options backdrops refreshed ({why})",values.brightness));
}

fn option_key(world: &mut World, key: Key) {
    let log = world.resource::<crate::Shared>().log.clone();
    let old = world.resource::<Preferences>().settings.clone();
    let mut settings = old.clone();
    let back = {
        let mut menu = world.resource_mut::<Menu>();
        let back = menu.options.as_mut().unwrap().input(key, &mut settings);
        if settings != old {
            settings.apply_groups(&mut menu.audio.groups);
            if old.volume[0] != settings.volume[0] {
                if let Some(output) = &menu.audio.output {
                    output.send([Command::Gains {
                        id: THEME_ID,
                        gains: [menu.audio.groups.loudness(4); 2],
                    }]);
                }
            }
            log.lock().unwrap().summary(format!(
                "options: {key:?} -> {settings:?}, theme gain {:.2}",
                menu.audio.groups.loudness(4)
            ));
        }
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right | Key::Tab => menu.audio.play("click.aud"),
            Key::Click(_, _) | Key::Enter => menu.audio.play("intrface.aud"),
            _ => {}
        }
        back
    };
    if settings != old {
        world.resource_mut::<Preferences>().settings = settings.clone();
    }
    if back == options_menu::Destination::Credits {
        let score = world
            .get_resource::<crate::door_view::DoorSim>()
            .filter(|_| world.resource::<Menu>().state.has_game())
            .and_then(|s| {
                s.active
                    .world
                    .connor_stats
                    .as_ref()
                    .map(|c| c.earned_experience())
            });
        match credits::Screen::open(&world.resource::<Source>().game, score) {
            Ok(screen) => {
                world.resource_mut::<Menu>().credits = Some(screen);
                credit_status(world, "Options");
            }
            Err(e) => log.lock().unwrap().summary(format!("credits: {e}")),
        }
    } else if back == options_menu::Destination::Graphics {
        let source = &world.resource::<Source>().game;
        match graphics_menu::Screen::open(source, &world.resource::<Menu>().art) {
            Ok(screen) => {
                world.resource_mut::<Menu>().graphics = Some(screen);
                log.lock().unwrap().summary("graphics: opened original Graphics.gui");
                let graphics = world.resource::<Preferences>().settings.graphics.clone();
                let output = kq8_data::palette::Output::from_mode(&graphics.mode).unwrap_or(kq8_data::palette::Output::Direct3D);
                crate::live_palette::apply_to_surface(world,crate::live_palette::Values {brightness:graphics.brightness,output},"Graphics setup");
            }
            Err(e) => log.lock().unwrap().summary(format!("graphics: could not open: {e}")),
        }
    } else if back == options_menu::Destination::Keyboard {
        let source = &world.resource::<Source>().game;
        match keyboard_menu::Screen::open(source, &world.resource::<Menu>().art) {
            Ok(screen) => {
                world.resource_mut::<Menu>().keyboard = Some(screen);
                log.lock()
                    .unwrap()
                    .summary("keyboard: opened original movekey.gui (13 actions)");
            }
            Err(e) => log
                .lock()
                .unwrap()
                .summary(format!("keyboard: could not open: {e}")),
        }
    } else if back == options_menu::Destination::Back {
        match world.resource_mut::<Preferences>().store() {
            Ok(()) => log
                .lock()
                .unwrap()
                .summary(format!("options: back to main, {settings:?}")),
            Err(e) => log
                .lock()
                .unwrap()
                .summary(format!("options: settings could not be saved: {e}")),
        }
        world.resource_mut::<Menu>().options = None;
    }
    redraw(world);
}

fn graphics_key(world: &mut World, key: Key) {
    let log = world.resource::<crate::Shared>().log.clone();
    let old = world.resource::<Preferences>().settings.graphics.clone();
    let mut graphics = old.clone();
    let action = {
        let mut menu = world.resource_mut::<Menu>();
        let action = menu.graphics.as_mut().unwrap().input(key, &mut graphics);
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right | Key::Tab => menu.audio.play("click.aud"),
            Key::Click(_, _) | Key::Enter => menu.audio.play("intrface.aud"),
            _ => {},
        }
        action
    };
    if graphics != old {
        world.resource_mut::<Preferences>().settings.graphics = graphics.clone();
        if graphics.dynamic_lighting != old.dynamic_lighting {
            crate::light_view::apply_menu_dynamic(world, graphics.dynamic_lighting);
        }
        if graphics.shadow_mask != old.shadow_mask {
            crate::shadow_view::apply_menu_mask(world, graphics.shadow_mask);
        }
        if graphics.brightness != old.brightness {
            let output = kq8_data::palette::Output::from_mode(&graphics.mode).unwrap_or(kq8_data::palette::Output::Direct3D);
            crate::live_palette::apply_to_surface(world,crate::live_palette::Values { brightness:graphics.brightness,output },"Graphics slider");
        }
        log.lock().unwrap().summary(format!("graphics: {key:?} -> {graphics:?}"));
    }
    if action == graphics_menu::Action::Back {
        match world.resource_mut::<Preferences>().store() {
            Ok(()) => log.lock().unwrap().summary("graphics: returned to Options; saved local preferences"),
            Err(e) => log.lock().unwrap().summary(format!("graphics: could not save preferences: {e}")),
        }
        world.resource_mut::<Menu>().graphics = None;
    }
    redraw(world);
}

fn keyboard_key(world: &mut World, key: Key) {
    use keyboard_menu::Action;
    let log = world.resource::<crate::Shared>().log.clone();
    let mut bindings = world.resource::<Keyboard>().active;
    let mut menu = world.resource_mut::<Menu>();
    let action = menu.keyboard.as_mut().unwrap().input(key, &mut bindings);
    match key {
        Key::Up | Key::Down | Key::Tab => menu.audio.play("click.aud"),
        Key::Enter | Key::Click(_, _) | Key::Physical(_) => menu.audio.play("intrface.aud"),
        _ => {}
    }
    match action {
        Action::Capture => log.lock().unwrap().summary("keyboard: press desired key"),
        Action::Rejected => log.lock().unwrap().summary("keyboard: key in use; retry"),
        Action::Bound(row, v) => {
            world.resource_mut::<Keyboard>().active = bindings;
            log.lock()
                .unwrap()
                .summary(format!("keyboard: assigned row {row} to Windows key {v}"));
        }
        Action::Back => {
            menu.keyboard = None;
            match world.resource_mut::<Keyboard>().store() {
                Ok(()) => log.lock().unwrap().summary("keyboard: returned to Options"),
                Err(e) => log
                    .lock()
                    .unwrap()
                    .summary(format!("keyboard: could not store bindings: {e}")),
            }
        }
        Action::None => {}
    }
    redraw(world);
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
    if world.resource::<Menu>().movie.is_some() {
        play_movie_frame(world);
        return;
    }
    if world
        .resource::<Menu>()
        .shown
        .is_none_or(|at| at.elapsed().as_secs_f32() < 0.2)
    {
        return;
    }
    if world.resource::<Menu>().credits.is_some() {
        let delta = world.resource::<Time<Real>>().delta();
        let mut menu = world.resource_mut::<Menu>();
        let page = menu.credits.as_mut().unwrap();
        let before = page.index;
        if page.tick(delta) {
            end_credits(world);
            return;
        }
        if page.index != before {
            credit_status(world, "timer");
            redraw(world);
        }
    } else if world.resource::<Menu>().options.is_some() || world.resource::<Menu>().graphics.is_some() {
        let frame =
            (world.resource::<Menu>().option_clock.elapsed().as_millis() / 100) as usize % 16;
        if frame != world.resource::<Menu>().option_frame {
            world.resource_mut::<Menu>().option_frame = frame;
            redraw(world);
        }
    }
    let mut input = Vec::new();
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let capturing = world
        .resource::<Menu>()
        .keyboard
        .as_ref()
        .is_some_and(|s| s.capture);
    if capturing {
        for &code in keys.get_just_pressed() {
            let key = if code == KeyCode::Escape {
                Some(Key::Escape)
            } else {
                crate::keyboard_input::virtual_key(code).map(Key::Physical)
            };
            if let Some(key) = key {
                input.push((key, true));
            }
        }
    }
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
        if !capturing && keys.just_pressed(code) {
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
            && (world
                .resource::<Menu>()
                .slot_page
                .as_ref()
                .is_some_and(|(s, _)| s.dragging())
                || world
                    .resource::<Menu>()
                    .options
                    .as_ref()
                    .is_some_and(|s| s.dragging)
                || world
                    .resource::<Menu>()
                    .graphics
                    .as_ref()
                    .is_some_and(|s| s.dragging))
        {
            input.push((Key::Drag(p.x.floor() as i32, p.y.floor() as i32), true));
        }
    }
    if mouse.just_released(MouseButton::Left)
        && (world.resource::<Menu>().slot_page.is_some()
            || world.resource::<Menu>().options.is_some()
            || world.resource::<Menu>().graphics.is_some())
    {
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
        if world.resource::<Menu>().credits.is_some() {
            match key {
                Key::Escape => end_credits(world),
                Key::Enter | Key::Click(_, _) => {
                    let mut menu = world.resource_mut::<Menu>();
                    menu.audio.play("intrface.aud");
                    if menu.credits.as_mut().unwrap().advance() {
                        end_credits(world);
                    } else {
                        credit_status(world, "advance");
                        redraw(world);
                    }
                }
                _ => {}
            }
            continue;
        }
        if world.resource::<Menu>().graphics.is_some() {
            graphics_key(world, key);
            continue;
        }
        if world.resource::<Menu>().keyboard.is_some() {
            keyboard_key(world, key);
            continue;
        }
        if world.resource::<Menu>().options.is_some() {
            option_key(world, key);
            continue;
        }
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
                close_menu(world, true);
                if let Action::Start(level) = action {
                    world.resource_mut::<crate::door_view::HeldItem>().1 = Some(level);
                    crate::world_build::first_world(world);
                } else if let Some(sim) = world.get_resource::<crate::sound_view::SoundSim>() {
                    sim.menu_pause(false);
                }
                return;
            }
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
            Action::Options => match options_menu::Screen::open(
                &world.resource::<Source>().game,
                &world.resource::<Menu>().art,
            ) {
                Ok(page) => {
                    let mut menu = world.resource_mut::<Menu>();
                    menu.options = Some(page);
                    menu.option_clock = std::time::Instant::now();
                    menu.option_frame = 0;
                    redraw(world);
                }
                Err(e) => world
                    .resource::<crate::Shared>()
                    .log
                    .lock()
                    .unwrap()
                    .summary(format!("menu: could not open Options: {e}")),
            },
            Action::Replay => {
                if let Err(e) = open_movie(world, "W32opn_1.dll", MovieReturn::Menu) {
                    world
                        .resource::<crate::Shared>()
                        .log
                        .lock()
                        .unwrap()
                        .summary(format!("movie: {e}"));
                }
                return;
            }
            Action::Stay => {}
        }
    }
}

fn close_menu(world: &mut World, fade: bool) {
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
        if let Some(output) = menu.audio.output.filter(|_| fade) {
            world.insert_resource(FadingTheme {
                output,
                gain: menu.audio.groups.loudness(4),
                started: std::time::Instant::now(),
            });
        }
    }
}
fn open_movie(world: &mut World, name: &str, after: MovieReturn) -> Result<(), String> {
    let source = world.resource::<Source>();
    let log = world.resource::<crate::Shared>().log.clone();
    let new_menu = !world.contains_resource::<Menu>();
    let page = if new_menu {
        Some(Menu::open(
            &source.game,
            Vec::new(),
            source.sound,
            &mut log.lock().unwrap(),
        )?)
    } else {
        None
    };
    let playback = (|| {
        if !["W32opn_1.dll", "W32mdl_1.dll", "W32ndg_1.dll"]
            .iter()
            .any(|film| film.eq_ignore_ascii_case(name))
        {
            return Err(format!("unknown KQ8 movie {name}"));
        }
        let game = GameDir::open(&source.game).map_err(|e| e.to_string())?;
        let path = game
            .find(name)
            .ok_or_else(|| format!("missing movie {name}"))?;
        crate::movie_player::Playback::open(&path)
    })();
    let message = match &playback {
        Ok(m) => format!(
            "opening {name}: {} intervals, {} holds, {} stereo samples at {} Hz",
            m.frames,
            m.hold_count(),
            m.audio.frames(),
            m.audio.rate
        ),
        Err(e) => format!("{name} unavailable: {e}; awaiting acknowledgement"),
    };
    log.lock().unwrap().summary(format!("movie: {message}"));
    if let Some(page) = page {
        world.insert_resource(page);
    }
    // The original closes game sound around its independent AVI soundtrack.
    world.remove_resource::<FadingTheme>();
    if let Some(mut sound) = world.get_resource_mut::<crate::sound_view::SoundSim>() {
        sound.begin_movie();
    }
    let cursor_visible = world
        .query_filtered::<&mut bevy::window::CursorOptions, With<PrimaryWindow>>()
        .iter_mut(world)
        .next()
        .map(|mut c| {
            let before = c.visible;
            c.visible = false;
            before
        });
    let mut menu = world.resource_mut::<Menu>();
    if let Some(out) = &menu.audio.output {
        out.send([Command::StopOneShots, Command::Pause { id: THEME_ID }]);
    }
    menu.movie = Some(MoviePage {
        name: name.to_owned(),
        after,
        playback,
        cursor_visible,
    });
    if new_menu {
        world.run_schedule(DisplayMenu);
    } else {
        redraw(world);
    }
    Ok(())
}

fn leave_movie(world: &mut World, reason: &str) {
    let mut menu = world.resource_mut::<Menu>();
    let page = menu.movie.take().unwrap();
    let counts = page
        .playback
        .as_ref()
        .map(|m| {
            format!(
                " after {} intervals, {} decoded pictures",
                m.shown, m.decoded
            )
        })
        .unwrap_or_default();
    if let Some(out) = &menu.audio.output {
        out.send([Command::Stop { id: MOVIE_ID }]);
    }
    menu.shown = Some(std::time::Instant::now());
    if let Some(visible) = page.cursor_visible {
        if let Some(mut c) = world
            .query_filtered::<&mut bevy::window::CursorOptions, With<PrimaryWindow>>()
            .iter_mut(world)
            .next()
        {
            c.visible = visible;
        }
    }
    let elapsed = page
        .playback
        .as_ref()
        .ok()
        .and_then(|m| m.started)
        .map(|at| at.elapsed().as_millis());
    world
        .resource::<crate::Shared>()
        .log
        .lock()
        .unwrap()
        .summary(format!(
            "movie: {} {reason}{counts}, returning to {:?}, elapsed ms {elapsed:?}",
            page.name, page.after
        ));
    let graphics = world.resource::<Preferences>().settings.graphics.clone();
    let output = kq8_data::palette::Output::from_mode(&graphics.mode).unwrap_or(kq8_data::palette::Output::Direct3D);
    crate::live_palette::apply_to_surface(world,crate::live_palette::Values {brightness:graphics.brightness,output},"return from film");
    match page.after {
        MovieReturn::Gameplay => {
            close_menu(world, false);
            if let Some(sound) = world.get_resource::<crate::sound_view::SoundSim>() {
                sound.end_movie(false);
            }
        }
        MovieReturn::Menu => {
            if let Some(sound) = world.get_resource::<crate::sound_view::SoundSim>() {
                sound.end_movie(true);
            }
            if let Some(out) = &world.resource::<Menu>().audio.output {
                out.send([Command::Resume { id: THEME_ID }]);
            }
            redraw(world);
        }
        MovieReturn::Title => {
            if world.resource::<PresentationQueue>().0.is_empty() {
                world.resource_mut::<Menu>().audio.play("theme.aud");
                redraw(world);
            } else {
                present_next(world);
            }
        }
    }
}

fn play_movie_frame(world: &mut World) {
    use crate::movie_player::Update;
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let escape = keys.just_pressed(KeyCode::Escape);
    let enter = keys.just_pressed(KeyCode::Enter);
    let log = world.resource::<crate::Shared>().log.clone();
    let mut menu = world.resource_mut::<Menu>();
    let page = menu.movie.as_ref().unwrap();
    let failed = page.playback.is_err();
    let started = page.playback.as_ref().is_ok_and(|m| m.started.is_some());
    let queued = match menu.keys.front() {
        Some(Key::Enter) if failed => true,
        Some(Key::Escape) if failed || started => true,
        _ => false,
    };
    if queued {
        menu.keys.pop_front();
    }
    if failed {
        if enter || escape || queued {
            leave_movie(world, "acknowledged");
        }
        return;
    }
    if escape || queued {
        leave_movie(world, "skipped");
        return;
    }
    let page = menu.movie.as_mut().unwrap();
    let movie = page.playback.as_mut().unwrap();
    match movie.step(std::time::Instant::now()) {
        Ok(Update::Wait) => {}
        Ok(Update::Frame(pixels)) => {
            if !started {
                let sound = movie.audio.clone();
                log.lock().unwrap().summary(format!(
                    "movie: {} first picture, PCM start {} stereo samples",
                    page.name,
                    sound.frames()
                ));
                if let Some(out) = &menu.audio.output {
                    out.send([Command::Start {
                        id: MOVIE_ID,
                        sound,
                        looping: false,
                        gains: [1.; 2],
                    }]);
                }
            }
            let image = menu.image.clone();
            if let Some(mut target) = world.resource_mut::<Assets<Image>>().get_mut(&image) {
                target.data = Some(pixels);
            }
        }
        Ok(Update::Finished) => leave_movie(world, "finished"),
        Err(e) => {
            log.lock().unwrap().summary(format!(
                "movie: {} failed: {e}; awaiting acknowledgement",
                page.name
            ));
            page.playback = Err(e);
            if let Some(out) = &menu.audio.output {
                out.send([Command::Stop { id: MOVIE_ID }]);
            }
            redraw(world);
        }
    }
}
