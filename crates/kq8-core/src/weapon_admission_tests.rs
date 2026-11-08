//! Shared lower weapon-request rules, from ranged-weapons.md rules 2–9.
//! Minimal item-table fixtures exercise the real parser and carried-item lookup;
//! animation ownership and assignment-side selection clearing belong to the viewer.
use super::{Ask, Request, WeaponMode, WorldClick};
use crate::{active::Active, item::{Items, Place}};
use kq8_data::items::ItemTypes;

fn active(ranged: Option<&str>) -> Active {
    let rows = [
        ("Fists", "ShortWeapon", "SwordMode"),
        ("NoRanged", "LongWeapon", "NoModeChange"),
        ("SmCrossbow", "LongWeapon", "BowMode"),
        ("Candle", "QuestItem", "NoModeChange"),
    ];
    let mut table = format!("[InventoryItemTypeList]\nnumItems={}\n", rows.len());
    for (n, (name, category, mode)) in rows.into_iter().enumerate() {
        table.push_str(&format!("item{n}=\tKQInventoryItemType\tINVITEM_{name}\t{n}\t0\t0\t0\t0\t0\t0\t{category}\tIDBMP_TEST\t1 0 0 1\t{mode}\t0\t1\tnone\tnone\tnone\ttest.kqs\tnone\t1\t256\n"));
    }
    let (types, problems) = ItemTypes::parse(table.as_bytes());
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(types.types.len(), 4);
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], Vec::<(String, Vec<u8>)>::new());
    active.world.items = Items::new(types, []);
    for kind in [Some("Fists"), ranged, Some("Candle")].into_iter().flatten() {
        let item = active.world.items.make(kind, 1, Place::Carried("Connor".into())).unwrap();
        if kind == "Candle" { active.world.hand = Some(item.name.clone()); }
    }
    active
}

fn request(ask: Ask) -> Request { Request { ask, how: "lower weapon admission matrix".into() } }

fn unchanged(active: &Active) -> (WeaponMode, Option<String>, Option<String>, bool) {
    (active.world.weapon_mode, active.world.weapon_ready.clone(), active.world.hand.clone(), active.world.last_away_ranged)
}

#[test]
fn weapon_request_lower_gate_matrix() {
    use WeaponMode::{Hand, None as Empty, Ranged, Raw};

    // Planning must neither change the current weapon nor clear the held item.
    // NoRanged is a carried LongWeapon: the shared lower gate accepts it, while
    // Delete's hand-to-ranged branch and NoModeChange bar column do not ask.
    let mut a = active(Some("NoRanged"));
    a.world.commit_weapon_change(Hand, false);
    let before = unchanged(&a);
    for ask in [
        Ask::Script(Ranged),
        Ask::ScriptMovement { mode: Ranged, movement: "CuStOm".into() },
        Ask::Required(Ranged),
        Ask::Slot { item: "NoRanged".into(), column: "BowMode".into() },
    ] {
        let planned = a.world.plan_weapon_request(&request(ask), Ok(()), false).unwrap();
        assert_eq!((planned.from, planned.to, planned.to_item.as_deref()), (Hand, Ranged, Some("INVITEM_NoRanged")));
        assert_eq!(unchanged(&a), before);
    }
    for ask in [Ask::Draw, Ask::Slot { item: "NoRanged".into(), column: "NoModeChange".into() }] {
        assert!(a.world.plan_weapon_request(&request(ask), Ok(()), false).is_none());
        assert_eq!(unchanged(&a), before);
    }
    let mut real = active(Some("SmCrossbow"));
    real.world.commit_weapon_change(Hand, false);
    assert_eq!(real.world.plan_weapon_request(&request(Ask::Draw), Ok(()), false).unwrap().to_item.as_deref(), Some("INVITEM_SmCrossbow"));
    let mut absent = active(None);
    for ask in [Ask::Script(Ranged), Ask::Required(Ranged), Ask::Slot { item: "missing".into(), column: "BowMode".into() }] {
        let before = unchanged(&absent);
        assert!(absent.world.plan_weapon_request(&request(ask), Ok(()), false).is_none(), "missing LongWeapon must not fall back to Hand");
        assert_eq!(unchanged(&absent), before);
    }

    // The caller supplies the actual remaining controller after any script
    // preparation. Scripts bypass handsOff, never that controller's refusal.
    for owner in ["Jump", "Fall", "Talk", "Special", "SwapWeapons"] {
        for ask in [Ask::Script(Ranged), Ask::ScriptMovement { mode: Ranged, movement: "out".into() }, Ask::Required(Ranged)] {
            assert!(a.world.plan_weapon_request(&request(ask), Err(owner.into()), true).is_none());
            assert_eq!(unchanged(&a), before);
        }
    }
    for ask in [Ask::Draw, Ask::Away, Ask::Slot { item: "Fists".into(), column: "SwordMode".into() }] {
        assert!(a.world.plan_weapon_request(&request(ask), Ok(()), true).is_none());
        assert_eq!(unchanged(&a), before);
    }
    assert!(a.world.plan_weapon_request(&request(Ask::Script(Ranged)), Ok(()), true).is_some());
    a.world.connor_roping = true;
    assert!(a.world.plan_weapon_request(&request(Ask::Script(Ranged)), Ok(()), false).is_none());
    a.world.connor_roping = false;
    assert_eq!(unchanged(&a), before);
    for ask in [Ask::Script(Hand), Ask::ScriptMovement { mode: Hand, movement: "away".into() }] {
        assert!(a.world.plan_weapon_request(&request(ask), Ok(()), false).is_none(), "same-mode script is a no-op, not a toggle");
    }
    assert_eq!(a.world.plan_weapon_request(&request(Ask::Slot { item: "Fists".into(), column: "SwordMode".into() }), Ok(()), false).unwrap().to, Empty);
    a.world.commit_weapon_change(Empty, false);
    assert!(a.world.plan_weapon_request(&request(Ask::Away), Ok(()), false).is_none());

    // The real command parser retains optional text and exact raw integers.
    // Assignment here tests the core mode/remembered flag only; the viewer
    // separately proves neutral pointer, retained list and selection clearing.
    for raw in [-19, 2, i64::MAX] {
        assert_eq!(a.test_command(&format!("KQConner::setDoMode {raw} CuStOm"), "matrix").as_deref(), Some("0"));
        let queued = a.world.weapon_requests.pop().unwrap();
        assert_eq!(queued.ask, Ask::ScriptMovement { mode: Raw(raw), movement: "CuStOm".into() });
        assert_eq!(queued.ask.movement(), Some("CuStOm"));
        let before = unchanged(&a);
        assert_eq!(a.world.plan_weapon_request(&queued, Ok(()), false).unwrap().to, Raw(raw));
        assert_eq!(unchanged(&a), before);
        a.world.commit_weapon_change(Raw(raw), false);
        assert_eq!(a.test_command("KQConner::getDoMode", "matrix"), Some(raw.to_string()));
        assert_eq!(a.world.world_click_action(false, false), WorldClick::Interact);
        assert!(a.world.plan_weapon_request(&queued, Ok(()), false).is_none());
        let saved = serde_json::to_string(&a.world.weapon_mode).unwrap();
        assert_eq!(serde_json::from_str::<WeaponMode>(&saved).unwrap(), Raw(raw));
    }
    // Without an item masking the click, Raw is still an interaction mode.
    a.world.hand = None;
    assert_eq!(a.world.world_click_action(false, false), WorldClick::Interact);
    for (old, remembered, modifier, expected) in [
        (Ranged, false, false, true), (Hand, true, false, false),
        (Ranged, false, true, false), (Hand, true, true, true),
    ] {
        a.world.commit_weapon_change(old, false);
        a.world.last_away_ranged = remembered;
        a.world.commit_weapon_change(Raw(9), modifier);
        assert_eq!((a.world.weapon_mode, a.world.last_away_ranged), (Raw(9), expected));
    }
    for mode in [Hand, Ranged, Empty] {
        a.world.commit_weapon_change(Raw(9), false);
        let planned = a.world.plan_weapon_request(&request(Ask::Script(mode)), Ok(()), false).unwrap();
        assert_eq!((planned.from, planned.to), (Raw(9), mode));
    }

    // Required mode is a distinct direct request. Disabling and same-current
    // commands retain the old callback; a refused request still records the
    // requirement and a supplied replacement callback.
    let mut a = active(Some("NoRanged"));
    assert_eq!(a.world.required_weapon_mode, (-1, None));
    assert_eq!(a.test_command("KQConner::setRequiredWeaponMode 0 first.cs", "matrix").as_deref(), Some("0"));
    let queued = a.world.weapon_requests.pop().unwrap();
    assert_eq!(queued.ask, Ask::Required(Hand));
    assert!(a.world.plan_weapon_request(&queued, Err("Jump".into()), false).is_none());
    assert_eq!(a.world.required_weapon_mode, (0, Some("first.cs".into())));
    a.world.commit_weapon_change(Hand, false);
    for (command, requirement, callback, asks) in [
        ("KQConner::setRequiredWeaponMode 0 ignored.cs", 0, "first.cs", false),
        ("KQConner::setRequiredWeaponMode -1 ignored.cs", -1, "first.cs", false),
        ("KQConner::setRequiredWeaponMode 1", 1, "first.cs", true),
        ("KQConner::setRequiredWeaponMode 1 second.cs", 1, "second.cs", true),
    ] {
        assert_eq!(a.test_command(command, "matrix").as_deref(), Some("0"));
        assert_eq!(a.world.required_weapon_mode, (requirement, Some(callback.into())));
        assert_eq!(a.world.weapon_requests.len(), usize::from(asks));
        if asks { assert_eq!(a.world.weapon_requests.pop().unwrap().ask, Ask::Required(Ranged)); }
        assert_eq!(a.world.weapon_mode, Hand, "commands do not assign the mode before admission");
    }
}
