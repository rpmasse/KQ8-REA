//! Forced owner exit uses the same actual Talk exit as normal completion,
//! without borrowing normal completion's panel or Connor-state effects.
use super::*;
use crate::active::{AnimObject, Monster};
use crate::character::{Character, State as PartnerState};
use kq8_data::msg::{Line, MessageFile, Tuple};

const SPOT: [f32; 3] = [1000., 1400., 400.];
const PARTNER: [f32; 3] = [400., 600., 200.];
const CAM0: [f32; 3] = [-100., -200., 700.];

fn fixture(enter: bool) -> (Active, Walker) {
    let script = b"if test $2 == TalkExit\n inc Matrix::Exits\nendif\nif test $2 == TalkComplete\n inc Matrix::Completes\nendif\n";
    let mut active = Active::new(kq8_data::world_script::Vars::default(), vec![], vec![],
        ["Connor".to_string(), "Bob".to_string()], [("bob.cs".to_string(), script.to_vec())]);
    let mut player = crate::named_anim::tests::still_then_open("Bob");
    player.movements[0].name = "stop".into();
    player.base.position = PARTNER;
    active.add_anims([AnimObject { player, script: Some("bob.cs".into()), gone: false,
        character: Some(Box::new(Character::new(false, 0x20, 0., false, false, true, 7))), translucency: None }]);
    active.world.monsters.push(Monster { name: "Bob".into(), position: PARTNER, script: Some("bob.cs".into()),
        goodbye: Some(Tuple::new(1000, 61, 0, 7, 1)), welcome_back: None });
    let info = b"[BobInfo]\nclassType=KQConvInfo\nconvLocX=1000\nconvLocY=1400\nconvLocZ=400\nnBox=2\nbox0=0 0 4000 0 4000 4000 0 4000\nbox1=500 500 2000 500 2000 2000 500 2000\nnCam=1\ncam0=-100 -200 700 0 0 0\n";
    active.world.conv_infos.push(ConvInfo::from_section(&kq8_data::kq_text::parse(info)[0]).unwrap());
    let mut lines: Vec<_> = (1..=3).map(|seq| Line { tuple: Tuple::new(1000, 60, 0, 0, seq), talker: 9, raw: "Hm.".into() }).collect();
    lines.push(Line { tuple: Tuple::new(1000, 61, 0, 7, 1), talker: 9, raw: "Bye.".into() });
    active.world.talk.book.add("fixture", MessageFile { kind: 0x0F, version: 5010, last_id: 0, lines }, 1000);
    active.world.connor_at = Some(SPOT);
    active.world.first_person = true;
    active.test_command("KQConner::conversation Bob 1000 60 0 0 1 3 0", "forced Talk matrix");
    let mut walker = Walker::new(0.);
    if enter {
        assert!(active.conversation_prepare(&mut walker));
        active.conversation_admitted(&mut walker, SPOT, &|_| None);
        assert_eq!(active.world.conversation.as_ref().unwrap().stage, Stage::Talking);
        assert!(!walker.talk_exit_open);
        assert!(!active.world.first_person);
    }
    // These later positions must not replace the captured feet/authored spot.
    active.world.anims[0].player.base.position = [-9000., -8000., -7000.];
    active.world.connor_at = Some([8000., 9000., 6000.]);
    active.world.requests.clear();
    active.world.gui_orders.clear();
    active.take_log();
    (active, walker)
}

fn speech(active: &Active) -> serde_json::Value {
    serde_json::to_value(active.world.talk.saved(active.world.now())).unwrap()
}

fn assert_camera(active: &Active) {
    // midpoint(PARTNER, SPOT) + following camera's vertical offset.
    let aim = [700., 1000., 1700.];
    let d = std::array::from_fn(|i| aim[i] - CAM0[i]);
    let (tilt, heading) = crate::camera::view_angles(d);
    assert_eq!(active.world.requests.len(), 2, "only exit placement and following; no Connor Wait or handsOff command");
    match &active.world.requests[0] {
        Request::Move { who, flags, position, turn, .. } => {
            assert_eq!(who, "KQCamera");
            assert_eq!(*flags, 0);
            assert_eq!(*position, CAM0);
            assert_eq!(*turn, [tilt, 0., heading]);
        }
        other => panic!("expected exit camera placement, got {other:?}"),
    }
    assert!(matches!(active.world.requests[1], Request::CameraFollow { connor: true, .. }));
}

fn assert_callback_before_wait(active: &mut Active, callback: bool) {
    assert_eq!(active.scripts.vars.get("Matrix::Exits"), callback.then_some("1"));
    assert_eq!(active.scripts.vars.get("Matrix::Completes"), None, "an interrupted line is not completed");
    assert_eq!(active.world.anims[0].character.as_ref().unwrap().state, PartnerState::Wait);
    let log = active.take_log().join("\n");
    let wait = log.find("Bob: the conversation ended:").expect(&log);
    let exit = log.find("script bob.cs Bob TalkExit");
    if callback { assert!(exit.expect(&log) < wait, "synchronous TalkExit precedes the real partner's Wait: {log}"); }
    else { assert!(exit.is_none(), "the already-sent goodbye owns TalkExit: {log}"); }
}

#[test]
fn forced_talk_exit_order_aim_and_queue_matrix() {
    // No goodbye, currently playing goodbye, and already-paused goodbye all
    // retain the full speech state and presentation events at forced exit.
    for goodbye_phase in 0..3 {
        let (mut active, walker) = fixture(true);
        let now = active.world.now();
        active.world.talk.queue(&Speak { issuer: CONNOR.into(), first: Tuple::new(1000, 60, 0, 0, 1), last: 1, nofade: false }, now);
        if goodbye_phase != 0 {
            assert_eq!(active.world.talk.conv_goodbye("Bob", now, true, &|_| true), Goodbye::Started);
            if goodbye_phase == 2 {
                active.world.talk.update(now + 100., true, &|_| true);
                assert!(active.world.talk.conv_paused("Bob"));
            }
        }
        assert_eq!(active.world.talk.conv_suppresses_talk_exit("bOB"), goodbye_phase != 0);
        active.world.connor_hands_off = true;
        active.world.gui_orders.push(GuiOrder::Status { at: "retained panel command".into() });
        let panels = active.world.gui_orders.clone();
        let queue = speech(&active);
        let events = active.world.talk.events.clone();
        let goodbyes = active.world.talk.goodbyes_ended.clone();
        assert!(active.force_conversation_exit(), "forced exit does not consult the closed ordinary gate");
        assert_eq!(walker.mode, crate::connor::Mode::Talk, "the caller, not the exit operation, replaces Connor's controller");
        assert!(!walker.talk_exit_open);
        assert!(active.world.conversation.is_none());
        assert!(active.world.first_person);
        assert!(active.world.connor_hands_off);
        assert!(active.world.connor_leave.is_none(), "no normal Wait/Move finish marker");
        assert_eq!(active.world.gui_orders, panels);
        assert_eq!(speech(&active), queue);
        assert_eq!(active.world.talk.events, events);
        assert_eq!(active.world.talk.goodbyes_ended, goodbyes);
        assert_camera(&active);
        assert_callback_before_wait(&mut active, goodbye_phase == 0);
        assert!(!active.force_conversation_exit(), "the same owner exits only once");
        assert_eq!(active.world.requests.len(), 2);
    }

    // A start message that was refused has not acquired Talk ownership.
    let (mut asked, _) = fixture(false);
    let queue = speech(&asked);
    assert!(!asked.force_conversation_exit());
    assert_eq!(asked.world.conversation.as_ref().unwrap().stage, Stage::Asked);
    assert_eq!(speech(&asked), queue);
    assert!(asked.world.requests.is_empty() && asked.world.gui_orders.is_empty());
    assert_eq!(asked.scripts.vars.get("Matrix::Exits"), None);

    // Normal completion shares the real exit but still owns its separate
    // panel restoration and subsequent Connor Wait/Move request marker.
    let (mut normal, mut walker) = fixture(true);
    let queue = speech(&normal);
    normal.conversation_leave("matrix normal finish", true);
    assert_camera(&normal);
    assert_callback_before_wait(&mut normal, true);
    assert_eq!(normal.world.connor_leave.as_deref(), Some("matrix normal finish"));
    assert_eq!(normal.world.gui_orders.len(), 1);
    assert!(matches!(normal.world.gui_orders[0], GuiOrder::Conversation { start: false, .. }));
    assert_eq!(speech(&normal), queue);
    assert!(normal.world.first_person);
    assert_eq!(exit_request(&mut walker), Some(StateRequest::Wait));
    assert!(walker.talk_exit_open);
}
