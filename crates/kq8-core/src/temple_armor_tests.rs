//! Inventory rule 8: persistent animation mappings, not an armour-present flag.
use super::ConnorFight;
use crate::{
    active::Active,
    connor_inventory,
    item::{Items, Place},
    journey::Journey,
    stats::{ConnorStats, Stats},
    weapons::WeaponMode,
};
use kq8_data::{characters::ConnorAnimLists, items::ItemTypes, GameDir};

fn actor(
    types: &ItemTypes,
    third_person: &ConnorAnimLists,
    first_person: &ConnorAnimLists,
) -> Active {
    let mut active = Active::new(Default::default(), vec![], vec![], ["Connor".into()], []);
    active.world.items = Items::new(types.clone(), []);
    active.set_connor_stats(ConnorStats {
        stats: Stats([10, 10, 0, 0, 0]),
        level: 1,
    });
    active.world.connor_fight = Some(ConnorFight {
        ouch_sound: "conrow3.aud".into(),
        min_damage_factor: 0.75,
        max_damage_factor: 4.,
        third_person: third_person.clone(),
        first_person: first_person.clone(),
        list_refresh_pending: false,
        shoot_offsets: vec![],
        start_projectile: None,
        start_offset: [0.; 3],
    });
    active
}

fn receive(active: &mut Active, kind: &str) -> String {
    let name = active
        .world
        .items
        .make(
            kind,
            1,
            Place::World {
                position: [4., 5., 6.],
                angles: [0.; 3],
            },
        )
        .unwrap()
        .name
        .clone();
    connor_inventory::receive(
        &mut active.world,
        &name,
        Some([4., 5., 6.]),
        "armour matrix",
    );
    name
}

#[test]
#[ignore = "requires KQ8_GAME_DIR; original item categories and animation mapping catalogue"]
fn temple_armor_mapping_persistence_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let types =
        ItemTypes::parse(&std::fs::read(game.find("GAME/Kq/Invinfo.itl").unwrap()).unwrap()).0;
    let file =
        kq8_data::kq_text::parse(&std::fs::read(game.find("GAME/Kq/Connor.kq").unwrap()).unwrap());
    let section = file
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case("Connor"))
        .unwrap();
    let third = ConnorAnimLists::from_file(&file, section, false);
    let first = ConnorAnimLists::from_file(&file, section, true);

    for (short, long) in [
        ("Dagger", "SmCrossBow"),
        ("TempleSword", "IceCrossBow"),
        ("Fists", "NoRanged"),
    ] {
        let mut a = actor(&types, &third, &first);
        receive(&mut a, short);
        receive(&mut a, long);
        let held = a
            .world
            .items
            .make("Candle", 1, Place::Carried("Connor".into()))
            .unwrap()
            .name
            .clone();
        a.world.hand = Some(held.clone());
        a.world.weapon_mode = WeaponMode::Hand;
        a.world.weapon_ready = Some(format!("INVITEM_{short}"));
        a.world.weapon_swapped = false;
        a.world.rope_first_person_lock = true;
        a.world.required_weapon_mode = (1, Some("weapon-ready.cs".into()));
        let armor = receive(&mut a, "TempleArmor");
        let maps = &a.world.connor_fight.as_ref().unwrap().third_person;
        let mut expected = third.clone();
        expected.stowed = Some("conner2.anm".into());
        expected
            .hand
            .iter_mut()
            .find(|(name, _)| name.eq_ignore_ascii_case(short))
            .unwrap()
            .1 = "contpsd2.anm".into();
        expected
            .ranged
            .iter_mut()
            .find(|(name, _)| name.eq_ignore_ascii_case(long))
            .unwrap()
            .1 = "conicbw2.anm".into();
        assert_eq!(maps, &expected, "actual equipped types {short}/{long}");
        assert_eq!(a.world.connor_fight.as_ref().unwrap().first_person, first);
        assert!(a.world.connor_fight.as_ref().unwrap().list_refresh_pending);
        assert_eq!(a.world.hand.as_deref(), Some(held.as_str()));
        assert_eq!(a.world.weapon_mode, WeaponMode::Hand);
        assert!(!a.world.weapon_swapped);
        assert!(a.world.rope_first_person_lock);

        // Ordinary removal keeps the changed entries; it is not a per-frame
        // selection conditional on whether the armour is still present.
        assert_eq!(
            a.console_command(
                "KQMonster::dropItem Connor TempleArmor 7 8 9 0 0 0",
                "remove armour",
            ).as_deref(),
            Some("0"),
        );
        assert!(!a.world.items.get(&armor).unwrap().carried_by("Connor"));
        assert_eq!(
            a.world.connor_fight.as_ref().unwrap().third_person,
            expected
        );

        for pending in [false, true] {
            a.world.connor_fight.as_mut().unwrap().list_refresh_pending = pending;
            let saved = Journey::take(&a.world, &a.scripts.vars, "temple1");
            let restored: Journey =
                serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
            let mut b = actor(&types, &third, &first);
            restored.give(&mut b.world, &mut b.scripts.vars);
            assert_eq!(
                b.world.connor_fight.as_ref().unwrap().third_person,
                expected
            );
            assert_eq!(b.world.connor_fight.as_ref().unwrap().first_person, first);
            assert_eq!(
                b.world.connor_fight.as_ref().unwrap().list_refresh_pending,
                pending
            );
            assert_eq!(b.world.hand.as_deref(), Some(held.as_str()));
            assert_eq!(b.world.required_weapon_mode, a.world.required_weapon_mode);
            assert!(
                b.world.requests.is_empty(),
                "restore must not replay pickup sounds or notifications"
            );
            assert_eq!(b.world.connor_stats, a.world.connor_stats);
            let mut destination = actor(&types, &third, &first);
            restored.give_after_flags(&mut destination.world, &mut destination.scripts.vars);
            assert!(
                destination
                    .world
                    .connor_fight
                    .as_ref()
                    .unwrap()
                    .list_refresh_pending,
                "a newly built destination must select the carried map before advancing"
            );
        }

        // A later operation edits the types carried then, preserving earlier
        // substitutions and every first-person mapping.
        receive(&mut a, "SmallAx");
        receive(&mut a, "CrossBow");
        assert!(a
            .world
            .helmetize("nextshort.anm", "nextlong.anm", "repeat operation"));
        expected
            .hand
            .iter_mut()
            .find(|(name, _)| name == "SmallAx")
            .unwrap()
            .1 = "nextshort.anm".into();
        expected
            .ranged
            .iter_mut()
            .find(|(name, _)| name == "CrossBow")
            .unwrap()
            .1 = "nextlong.anm".into();
        assert_eq!(
            a.world.connor_fight.as_ref().unwrap().third_person,
            expected
        );
        assert_eq!(a.world.connor_fight.as_ref().unwrap().first_person, first);

        let saved = Journey::take(&a.world, &a.scripts.vars, "temple1");
        let mut legacy = serde_json::to_value(saved).unwrap();
        legacy.as_object_mut().unwrap().remove("third_person_lists");
        legacy
            .as_object_mut()
            .unwrap()
            .remove("animation_list_refresh_pending");
        legacy
            .as_object_mut()
            .unwrap()
            .remove("required_weapon_mode");
        let legacy: Journey = serde_json::from_value(legacy).unwrap();
        let mut b = actor(&types, &third, &first);
        legacy.give(&mut b.world, &mut b.scripts.vars);
        assert_eq!(b.world.connor_fight.as_ref().unwrap().third_person, third);
        assert!(!b.world.connor_fight.as_ref().unwrap().list_refresh_pending);
        assert_eq!(b.world.required_weapon_mode, (-1, None));
    }
}
