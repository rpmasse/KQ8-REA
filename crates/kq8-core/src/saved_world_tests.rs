use super::*;

fn world(names: &[&str], script: &str) -> Active {
    let objects = names
        .iter()
        .map(|n| {
            let mut c =
                Clickable::plain(n, "KQObject", [-1.; 3], [1.; 3], Placement::at([0.; 3]), 32);
            c.script = Some("queue.cs".into());
            c
        })
        .collect();
    Active::new(
        Vars::default(),
        vec![],
        objects,
        [],
        [("queue.cs".into(), script.as_bytes().to_vec())],
    )
}

#[test]
fn saved_world_listener_registration_and_receiver_matrix() {
    for (saved_listener, init_listener, enable, late_a, expected) in [
        (true, "B", true, true, Some("A")),
        (true, "B", true, false, None),
        (true, "B", false, true, None),
        (true, "Missing", false, true, Some("A")),
        (false, "B", true, true, Some("B")),
    ] {
        let mut original = world(&["from", "A"], "");
        if saved_listener {
            assert_eq!(
                original.test_command("getEndLoop from A 1", "saved"),
                Some("0".into())
            );
        }
        // Saved and new lookup events must both survive, not deduplicate.
        original
            .world
            .queue
            .send(0., 0., "from", "KQLookupEvent", &[], "saved lookup");
        let saved: SavedEvents =
            serde_json::from_slice(&serde_json::to_vec(&original.world.saved_events()).unwrap())
                .unwrap();
        let mut loaded = world(&["from", "B"], "");
        loaded.world.prepare_saved_world(&saved).unwrap();
        let result = loaded.test_command(
            &format!("getEndLoop from {init_listener} {}", u8::from(enable)),
            "Init",
        );
        assert_eq!(
            result,
            Some(if init_listener == "Missing" { "1" } else { "0" }.into())
        );
        loaded.world.finish_saved_object_init("from");
        if late_a {
            loaded.world.known.insert("a".into());
            loaded.world.register_event_receiver("A");
        }
        loaded.world.merge_saved_events(saved).unwrap();
        let expected_deliveries = if saved_listener && (enable || init_listener == "Missing") {
            2
        } else {
            1
        };
        assert_eq!(loaded.service_startup_queue().unwrap(), expected_deliveries);
        assert_eq!(
            loaded
                .world
                .end_loop_listeners
                .get("from")
                .map(String::as_str),
            expected
        );
        assert!(loaded.world.pending_end_loop_listeners.is_empty());
        // A duplicate lookup after consumption cannot replace a later registration.
        loaded.test_command("getEndLoop from B 1", "later");
        loaded.world.queue.send(
            loaded.world.now(),
            0.,
            "from",
            "KQLookupEvent",
            &[],
            "duplicate lookup",
        );
        loaded.service_startup_queue().unwrap();
        assert_eq!(
            loaded
                .world
                .end_loop_listeners
                .get("from")
                .map(String::as_str),
            Some("B")
        );
        loaded.world.remove_event_receiver("B");
        loaded.world.register_event_receiver("B");
        assert!(!loaded.world.end_loop_listeners.contains_key("from"));
    }
    let mut source = world(&["target"], "set Delivered yes\n");
    source
        .world
        .queue
        .send(0., 1., "target", "KQCueEvent", &[], "saved");
    let saved = source.world.saved_events();
    let mut loaded = world(&["target"], "set Delivered yes\n");
    loaded.world.prepare_saved_world(&saved).unwrap();
    loaded.world.merge_saved_events(saved).unwrap();
    loaded.test_command("deleteObject target", "remove");
    assert!(loaded.world.queue.is_empty());
    loaded.world.register_event_receiver("target");
    loaded.world.deleted.remove("target");
    loaded.world.set_current_time(2.).unwrap();
    loaded.service_startup_queue().unwrap();
    assert_eq!(loaded.scripts.vars.get("Delivered"), None);
}

#[test]
fn saved_world_clock_merge_and_cascading_service_matrix() {
    let script="if test $2 == Cue\n if test $3 == 1\n  sendEvent 0.01 receiver KQCueEvent 2\n  sendEvent 0.2 receiver KQCueEvent 9\n else if test $3 == 2\n  set OverdueCascade yes\n else if test $3 == 3\n  sendEvent 0.01 receiver KQCueEvent 4\n  sendEvent 0.2 receiver KQCueEvent 8\n else if test $3 == 4\n  set TimedCascade yes\n endif\n endif\n endif\n endif\nendif\n";
    let mut a = world(&["receiver"], script);
    a.world.set_current_time(10.).unwrap();
    a.world
        .queue
        .send(9., 0., "receiver", "KQCueEvent", &["1".into()], "overdue");
    a.world.queue.send(
        10.,
        0.02,
        "receiver",
        "KQCueEvent",
        &["3".into()],
        "on time",
    );
    assert_eq!(a.service_startup_queue().unwrap(), 4);
    assert_eq!(
        a.world.updates, 0,
        "queue service is not a normal simulation tick"
    );
    assert_eq!(a.world.now(), (10f32 + STARTUP_SERVICE_SECONDS) as f64);
    assert_eq!(a.scripts.vars.get("OverdueCascade"), Some("yes"));
    assert_eq!(a.scripts.vars.get("TimedCascade"), Some("yes"));
    let future1 = a.world.queue.take_due(20.).unwrap();
    let future2 = a.world.queue.take_due(20.).unwrap();
    assert_eq!(future1.due, 10.2f32 as f64);
    assert_eq!(future2.due, 10.22f32 as f64);
    let before = a.world.now() as f32;
    a.update();
    assert_eq!(a.world.updates, 1);
    assert_eq!(a.world.now(), (before + 1f32 / 15.) as f64);

    // Shared core envelope only: production disk/regional entry paths need
    // their own ordered-loader integration check, not two labels on this test.
    {
        let path = "saved source";
        let mut source = world(&["receiver"], "");
        source.world.set_current_time(103.).unwrap();
        source.world.queue.send(
            source.world.now(),
            4.,
            "receiver",
            "KQCueEvent",
            &["7".into()],
            path,
        );
        let saved: SavedEvents =
            serde_json::from_slice(&serde_json::to_vec(&source.world.saved_events()).unwrap())
                .unwrap();
        let mut loaded = world(&["receiver"], "");
        loaded
            .world
            .queue
            .send(0., 0., "receiver", "KQCueEvent", &["1".into()], "Init due");
        loaded.world.queue.send(
            0.,
            4.,
            "receiver",
            "KQCueEvent",
            &["7".into()],
            "Init duplicate",
        );
        loaded.world.prepare_saved_world(&saved).unwrap();
        loaded.world.merge_saved_events(saved).unwrap();
        assert_eq!(loaded.service_startup_queue().unwrap(), 1);
        let first = loaded.world.queue.take_due(107.).unwrap();
        let second = loaded.world.queue.take_due(107.).unwrap();
        assert_eq!((first.due, second.due), (107., 107.));
        assert_eq!(first.from, "Init duplicate");
        assert_eq!(second.from, path);
        assert!(loaded.world.now() < 107.);
        assert_eq!(loaded.world.updates, 0);
    }
}
