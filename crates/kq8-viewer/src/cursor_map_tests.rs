//! Q110 and map23–25a: supplied coordinates, real-time cursor frames and
//! original-asset CPU drawing. No native window or visual inspection.
use super::*;
use std::time::Duration;

fn assert_cursor_pixels(canvas: &gui_draw::Canvas, bitmap: &Bitmap, palette: &[Rgb; 256], high: bool) {
    let size = [bitmap.width, bitmap.height].map(|n| if high { n * 5 / 4 } else { n });
    assert_eq!([canvas.width, canvas.height], size);
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let sx = (u64::from(x) * u64::from(bitmap.width) / u64::from(canvas.width)) as usize;
            let sy = (u64::from(y) * u64::from(bitmap.height) / u64::from(canvas.height)) as usize;
            let index = bitmap.pixels[sy * bitmap.width as usize + sx] as usize;
            let expected = if index == 0 && bitmap.flags & 1 != 0 { [0; 4] }
                else { let [r, g, b] = palette[index]; [r, g, b, 255] };
            assert_eq!(canvas.at(x, y), expected, "source frame at {x},{y}");
        }
    }
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; grouped original cursor/map CPU and input checks"]
fn original_cursor_map_display_coordinate_and_frame_matrix() {
    use bevy::window::{CursorIcon, CustomCursor, PrimaryWindow};
    use kq8_core::magic_map::START_VIEW;
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = Log::create("cursor-map-matrix").unwrap();
    let mut gui = load_gui(&game, &region, &mut log).unwrap();
    let table = std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap();
    let types = kq8_data::items::ItemTypes::parse(&table).0;
    let held = types.find("Candle").unwrap().cursor.clone().unwrap();
    let tags = [PLAY_POINTER, SWORD_POINTER, BOW_POINTER, "IDBMA_CURS06", "IDBMA_CURS07", &held];
    let pictures: Vec<_> = tags.iter().map(|tag| {
        let file = gui.file_of(tag).unwrap();
        gui.load_file(&file);
        (tag.to_string(), gui.files[&file.to_ascii_lowercase()].as_ref().unwrap().clone())
    }).collect();
    let palette = gui.palette;
    let mut app = App::new();
    app.add_systems(Update, pointer_draw).insert_resource(gui)
        .init_resource::<Assets<Image>>().insert_resource(Time::<Real>::default())
        .insert_resource(ScreenFrame([0.0, 0.0, 640.0, 480.0]));
    let mut window = Window::default();
    window.set_cursor_position(Some(Vec2::new(123.0, 87.0)));
    let entity = app.world_mut().spawn((window, PrimaryWindow)).id();
    let mut last_ms = 0u64;
    for high in [false, true, false] {
        let screen = kq8_core::caption::screen(high);
        app.world_mut().resource_mut::<Gui>().resize(screen);
        app.world_mut().resource_mut::<ScreenFrame>().0 = [0.0, 0.0, screen[0] as f32, screen[1] as f32];
        for (tag, frames) in &pictures {
            app.world_mut().resource_mut::<Gui>().pointer = tag.clone();
            // Exercise the exact100ms edge and looping in the actual system.
            let base = (last_ms / 100 + 1) * 100;
            for now in [base, base + 99, base + 100, base + 200, base + (frames.len() as u64 + 3) * 100] {
                app.world_mut().resource_mut::<Time<Real>>().advance_by(Duration::from_millis(now - last_ms));
                last_ms = now;
                app.world_mut().run_schedule(Update);
                let CursorIcon::Custom(CustomCursor::Image(cursor)) = app.world().get::<CursorIcon>(entity).unwrap() else { panic!("game cursor missing") };
                assert_eq!(cursor.hotspot, (0, 0));
                assert_eq!(app.world().get::<Window>(entity).unwrap().cursor_position(), Some(Vec2::new(123.0, 87.0)), "art selection never shifts click position");
                let image = app.world().resource::<Assets<Image>>().get(&cursor.handle).unwrap();
                let mut actual = gui_draw::Canvas::new(image.width(), image.height());
                actual.rgba.clone_from(image.data.as_ref().unwrap());
                assert_cursor_pixels(&actual, &frames[(now / 100) as usize % frames.len()], &palette, high);
            }
        }
    }
    // Use modified frame metadata to exercise size and drawing-kind changes
    // even if a particular installed pointer has identical frame dimensions.
    let mut synthetic = pictures[0].1[0].clone();
    synthetic.width = 7;
    synthetic.height = 5;
    synthetic.pixels = (0..35).map(|n| (n % 4) as u8).collect();
    for flags in [8, 9] {
        synthetic.flags = flags;
        for high in [false, true] {
            let canvas = cursor_canvas(Pic::of(&synthetic).unwrap(), kq8_core::caption::screen(high), &palette);
            assert_cursor_pixels(&canvas, &synthetic, &palette, high);
        }
    }

    let menu_frames = kq8_data::font::bitmap_array(&std::fs::read(game.find("GAME/8Gui/cursgui.pba").unwrap()).unwrap()).unwrap();
    let menu_palette = kq8_data::palette::as_shown(kq8_data::palette::Palette::parse(&std::fs::read(game.find("GAME/8Gui/menus.ppl").unwrap()).unwrap()).unwrap().colors);
    let mut art = kq8_core::main_menu::Art::open(game.root()).unwrap();
    assert_eq!(art.cursor_frames(), menu_frames.len());
    for high in [false, true, false] {
        art.resize(high);
        for tick in 0..=menu_frames.len() + 1 {
            assert_cursor_pixels(&art.cursor_frame(tick), &menu_frames[tick % menu_frames.len()], &menu_palette, high);
        }
        assert_eq!(art.cursor().rgba, art.cursor_frame(0).rgba);
    }

    let text = kq8_data::kq_text::parse(&std::fs::read(game.find("GAME/Kq/davmap.kq").unwrap()).unwrap());
    let keys = text.iter().find_map(kq8_data::magic_map::MapKeys::from_section).unwrap().unwrap();
    let full = Bitmap::parse(&std::fs::read(game.find(&format!("GAME/Resource/{}", keys.full_map)).unwrap()).unwrap()).unwrap();
    let mut gui = app.world_mut().remove_resource::<Gui>().unwrap();
    gui.load_file(&keys.arrow_pba);
    let arrow_frames = gui.files[&keys.arrow_pba.to_ascii_lowercase()].as_ref().unwrap().clone();
    assert_eq!(arrow_frames.len(), 16);
    assert!(arrow_frames.iter().all(|p| [p.width, p.height] == [10, 10] && p.flags & 1 != 0));
    gui.map = Some(MapShown { active: true, version: Some(1), pixels: full.pixels.clone(), own: full.pixels,
        view: START_VIEW.as_array(), mini_view: START_VIEW.as_array(), pixel: Some((230, 230)), arrow_file: keys.arrow_pba,
        ..Default::default() });
    gui.panels.set(kq8_core::panels::Group::Map, true, "coordinate matrix");
    gui.panels.update(1000.0);
    for high in [false, true, false] {
        gui.resize(kq8_core::caption::screen(high));
        let expected_area = if high { [15.0, 95.0, 385.0, 465.0] } else { [12.0, 76.0, 308.0, 372.0] };
        assert_eq!(gui.map_area_rect().unwrap(), expected_area);
        let (size, specs) = gui.specs(Drawn::Panel(Panel::Map)).unwrap();
        let only_map: Vec<_> = specs.into_iter().filter(|s| matches!(s, Spec::Map { .. })).collect();
        assert_eq!(only_map.len(), 1);
        let Spec::Map { rect, view, arrow, .. } = &only_map[0] else { unreachable!() };
        assert_eq!(*view, START_VIEW.as_array(), "resize preserves stored display-pixel view");
        let rect = *rect;
        let arrow = arrow.clone().unwrap();
        let origin = gui.control(Panel::Map.layout_id()).unwrap().rect;
        let rendered = gui.draw([origin[0], origin[1]], size, &only_map, 1.0);
        let centre = [(rect[0] + rect[2]) / 2, (rect[1] + rect[3]) / 2];
        let map_row = (rect[1] + 20) as u32;
        // At800 the starting picture stays298px wide inside a370px area.
        let columns: Vec<_> = (rect[0] as u32..rect[2] as u32).filter(|&x| rendered.at(x, map_row)[3] != 0).collect();
        if high { assert!(columns.is_empty(), "top of enlarged area remains outside the unscaled picture"); }
        let filled: Vec<_> = (rect[0] as u32..rect[2] as u32).filter(|&x| rendered.at(x, centre[1] as u32)[3] != 0).collect();
        assert_eq!(filled.len(), if high { 298 } else { 294 });
        let without_arrow = [Spec::Map { rect, version: 1, view: START_VIEW.as_array(), arrow: None, own: false }];
        let plain = gui.draw([origin[0], origin[1]], size, &without_arrow, 1.0);
        assert_eq!(arrow.2,[230,230],"drawing receives the map pixel rather than a separately rounded arrow offset");
        // trunc(-152.83) + (230 * trunc(298.27) / 460) - 5 = -8.
        let at = [(centre[0]-8) as u32,(centre[1]-8) as u32];
        let bitmap = &arrow_frames[arrow.1];
        assert_eq!(gui.map_arrow_size(),Some([bitmap.width,bitmap.height]));
        let absolute_arrow = START_VIEW.arrow_position((230,230),expected_area.map(|v|v as i32),[bitmap.width,bitmap.height]);
        assert_eq!(absolute_arrow,[at[0] as i32+origin[0],at[1] as i32+origin[1]],"follow and drawing share the same integer surface point");
        for y in 0..10u32 { for x in 0..10u32 {
            let index = bitmap.pixels[(y * 10 + x) as usize] as usize;
            let expected = if index == 0 { plain.at(at[0] + x, at[1] + y) }
                else { let [r, g, b] = palette[index]; [r, g, b, 255] };
            assert_eq!(rendered.at(at[0] + x, at[1] + y), expected, "arrow stays ten display pixels at either resolution");
        }}
        let mut view = START_VIEW;
        assert!(map_drag_motion(&mut view, true, [40.0, 120.0], [50.0, 108.0]));
        assert!((view.left - START_VIEW.left - 10.0).abs() < 1e-9);
        assert!((view.top - START_VIEW.top + 12.0).abs() < 1e-9);
        let before = view;
        assert!(map_drag_motion(&mut view, false, [50.0, 108.0], [114.0, 999.0]));
        assert!((view.width() - before.width() * 2.0).abs() < 1e-9);
        let before = view;
        assert!(map_drag_motion(&mut view, false, [114.0, 999.0], [114.0, 900.0]));
        assert_eq!(view, before, "vertical right drag does not zoom");
        assert!(!map_drag_motion(&mut view, true, [114.0, 900.0], [114.0, 900.0]));
    }
    println!("CURSOR/MAP PASS: original art,100ms frames,640/800 rounding,top-left hotspot,unchanged click; map display-pixel size/arrow/drag/zoom/follow-area origins");
}
