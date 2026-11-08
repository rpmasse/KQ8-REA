//! Original popup geometry through production specs, CPU drawing and hits.
//! world-change.md rule 26; in-game-help.md. No native display is opened.
use super::*;
use kq8_core::{active::Active, keyboard_menu::Bindings};
use std::sync::{Arc, Mutex};

fn window_point(gui: &Gui, frame: [f32; 4], local: [i32; 2]) -> Vec2 {
    let panel = gui.control(Panel::Popup.layout_id()).unwrap();
    let scale = frame[2] / gui.screen[0] as f32;
    Vec2::new(frame[0] + (panel.rect[0] + local[0]) as f32 * scale,
        frame[1] + (panel.rect[1] + local[1]) as f32 * scale)
}

fn portal(gui: &mut Gui, sim: &DoorSim) {
    gui.popup = Some(PopupShown {
        lines: kq8_core::portal::popup_lines(&message_500(sim, 114).unwrap()),
        labels: vec![message_500(sim, 29).unwrap(), message_500(sim, 14).unwrap()],
        key_cursor: None, help: None, highlight: 0, pressed: None,
    });
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original popup geometry, CPU rendering and pointer matrix"]
fn original_popup_geometry_draw_hit_and_keyboard_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let region = crate::load::World::open(&game, "Daventry").unwrap();
    let mut log = Log::create("popup-geometry-matrix").unwrap();
    let mut gui = load_gui(&game, &region, &mut log).unwrap();
    let bytes = std::fs::read(game.find("GAME/ENGLISH/500.MSG").unwrap()).unwrap();
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], []);
    active.world.talk.book.add("GAME/ENGLISH", kq8_data::msg::MessageFile::parse(500, &bytes).unwrap(), 500);
    let sim = DoorSim::for_test(active, log);
    let bindings = Bindings::read(&std::fs::read(game.find("movekeys.kq").unwrap()).unwrap()).unwrap();
    let shared = Shared { log: Arc::new(Mutex::new(Log::create("popup-geometry-viewer").unwrap())),
        stats: Arc::new(Mutex::new(crate::Stats::default())) };
    // Independently stated original row positions; the 800-mode offset
    // from each 640-mode row is 24 + 4*k, never a second 1.25 scale.
    let low_rows: [&[i32]; 7] = [&[64], &[48, 74], &[32, 58, 84], &[16, 42, 68, 94],
        &[16, 38, 60, 82, 104], &[14, 34, 54, 74, 94, 114], &[12, 30, 48, 66, 84, 102, 120]];
    let mut rendered = 0;
    for high in [false, true] {
        // Each mode opens fresh popups. This does not claim original
        // live-resize timing with an already visible popup.
        gui.popup = None;
        gui.resize(if high { [800, 600] } else { [640, 480] });
        let parent = gui.control(Panel::Popup.layout_id()).unwrap().rect;
        assert_eq!(parent, if high { [175, 150, 637, 400] } else { [140, 120, 510, 320] });
        let width = parent[2] - parent[0];
        let y_shift = if high { 44 } else { 0 };
        for kind in 0..5 {
            match kind {
                0 => portal(&mut gui, &sim),
                1 => open_help(&mut gui, &sim, &bindings, HelpPage::Tutorial(127), &shared),
                2 => open_help(&mut gui, &sim, &bindings, HelpPage::Catalog(0), &shared),
                3 => open_help(&mut gui, &sim, &bindings, HelpPage::MapGo, &shared),
                // A tutorial >=170 has ordinary OK but no checkbox.
                _ => {
                    open_help(&mut gui, &sim, &bindings, HelpPage::MapGo, &shared);
                    gui.popup.as_mut().unwrap().help.as_mut().unwrap().page = HelpPage::Tutorial(170);
                }
            }
            let initial = gui.popup.clone().unwrap();
            assert!(!initial.lines.is_empty());
            let checkbox = kind == 1;
            let lefts: &[i32] = match (high, kind) {
                (false, 0) => &[79, 202], (true, 0) => &[110, 264],
                (false, 2) => &[48, 140, 232], (true, 2) => &[71, 186, 301],
                (false, _) => &[141], (true, _) => &[187],
            };
            assert_eq!(lefts.len(), initial.labels.len());
            for highlight in 0..initial.labels.len() {
                for pressed in [None, Some(highlight), Some((highlight + 1) % initial.labels.len())] {
                    let mut popup = initial.clone();
                    popup.highlight = highlight;
                    popup.pressed = pressed;
                    gui.popup = Some(popup.clone());
                    let (size, specs) = gui.specs(Drawn::Panel(Panel::Popup)).unwrap();
                    assert_eq!(size, [width, parent[3] - parent[1]]);
                    for (k, label) in popup.labels.iter().enumerate() {
                        let (top, height) = if checkbox { (142, 25) }
                            else if k == highlight { (145, 38) } else { (146, 36) };
                        let expected = [lefts[k], top + y_shift, lefts[k] + 88, top + height + y_shift];
                        assert_eq!(gui.popup_button_rect(&popup, k), Some(expected));
                        let down = pressed == Some(k);
                        assert!(specs.iter().any(|spec| match spec {
                            Spec::Stretch { rect, file, frame } if k == highlight =>
                                *rect == expected && file == POPUP_HIGHLIGHT_PICTURE && *frame == usize::from(down),
                            Spec::Pic { rect, file, frame, stretch, .. } if k != highlight =>
                                *rect == expected && file == POPUP_BUTTON_PICTURE && *frame == usize::from(down) && *stretch,
                            _ => false,
                        }), "missing button {k}: high={high} kind={kind} highlight={highlight}");
                        let shift = i32::from(down);
                        let label_rect = expected.map(|v| v + shift);
                        assert!(specs.iter().any(|spec| matches!(spec,
                            Spec::Text { rect, text, justification: 0xC, .. } if *rect == label_rect && text == label)));
                        // Both native-size and offset/scaled windows use the
                        // exact same final rectangles as production drawing.
                        for frame in [[0., 0., gui.screen[0] as f32, gui.screen[1] as f32],
                            [13., 17., gui.screen[0] as f32 * 2., gui.screen[1] as f32 * 2.]] {
                            if k == highlight {
                                let target = [(expected[0] + expected[2]) / 2,
                                    (expected[1] + expected[3]) / 2 + 16];
                                assert_eq!(gui.popup_key_target(&popup, frame), Some(window_point(&gui, frame, target)));
                            }
                            for point in [[expected[0], expected[1]], [expected[2] - 1, expected[3] - 1]] {
                                assert_eq!(gui.popup_button_at(frame, window_point(&gui, frame, point)), Some(k));
                            }
                            for point in [[expected[0] - 1, expected[1]], [expected[2], expected[1]],
                                [expected[0], expected[1] - 1], [expected[0], expected[3]]] {
                                assert_ne!(gui.popup_button_at(frame, window_point(&gui, frame, point)), Some(k));
                            }
                        }
                    }
                    let canvas = gui.draw([parent[0], parent[1]], size, &specs, 1.0);
                    assert_eq!(canvas.rgba.len(), (size[0] * size[1] * 4) as usize);
                    assert!(canvas.rgba.chunks_exact(4).any(|p| p[3] != 0));
                    assert!(gui.fonts.get("console.pft").and_then(Option::as_ref).is_some());
                    for file in [POPUP_HIGHLIGHT_PICTURE, POPUP_BUTTON_PICTURE] {
                        if file == POPUP_BUTTON_PICTURE && popup.labels.len() == 1 { continue; }
                        assert!(gui.files.get(file).and_then(Option::as_ref).is_some_and(|frames| frames.len() >= 2));
                    }
                    rendered += 1;
                }
            }
            // All seven shared row layouts use each converted authored
            // control's own x edges and the original dynamic y values.
            for (row, tops) in low_rows.iter().enumerate() {
                let n = row + 1;
                gui.popup.as_mut().unwrap().lines = (0..n).map(|k| initial.lines[k % initial.lines.len()].clone()).collect();
                let (_, specs) = gui.specs(Drawn::Panel(Panel::Popup)).unwrap();
                for (k, &low_top) in tops.iter().enumerate() {
                    let control = gui.control(&format!("IDCTL_POPUP_TEXT{}", k + 1)).unwrap();
                    let top = low_top + if high { 24 + 4 * k as i32 } else { 0 };
                    let expected = [control.rect[0], top, control.rect[2], top + if high { 22 } else { 18 }];
                    assert_eq!(gui.popup_line_rect(n, k), Some(expected));
                    let text = &gui.popup.as_ref().unwrap().lines[k];
                    assert!(specs.iter().any(|spec| matches!(spec,
                        Spec::Text { rect, text: value, justification: 0xC, .. } if *rect == expected && value == text)));
                }
            }
            // Checkbox remains in generic converted layout, outside the
            // dynamic button/text override, and exists only for tutorials.
            let frame = [0., 0., gui.screen[0] as f32, gui.screen[1] as f32];
            let check = gui.control("IDCTL_POPUP_CHECKBOX").unwrap().rect;
            let at = window_point(&gui, frame, [(check[0] + check[2]) / 2, (check[1] + check[3]) / 2]);
            assert_eq!(gui.popup_button_at(frame, at), checkbox.then_some(3));
            if checkbox {
                let label = gui.control("IDCTL_POPUP_CHECKBOX_TEXT").unwrap().rect;
                let at = window_point(&gui, frame, [(label[0] + label[2]) / 2, (label[1] + label[3]) / 2]);
                assert_eq!(gui.popup_button_at(frame, at), Some(3));
            }
            println!("popup geometry high={high} kind={kind} parent={parent:?} buttons={lefts:?} rows=1..7");
        }
    }
    println!("popup CPU renders={rendered}; both resolutions/all popup kinds and shared rows checked");
}
