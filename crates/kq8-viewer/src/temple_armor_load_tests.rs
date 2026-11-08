use super::*;

#[test]
#[ignore = "requires KQ8_GAME_DIR; original Temple1 model loader with saved maps and retained bank"]
fn original_temple_armor_banks_load_matrix() {
    let game = GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
    let world = World::open(&game, "temple1").unwrap();
    let file =
        kq8_data::kq_text::parse(&std::fs::read(game.find("GAME/Kq/Connor.kq").unwrap()).unwrap());
    let actor = file
        .iter()
        .find(|section| section.name.eq_ignore_ascii_case("Connor"))
        .unwrap();
    let original = kq8_data::characters::ConnorAnimLists::from_file(&file, actor, false);
    let first_person = kq8_data::characters::ConnorAnimLists::from_file(&file, actor, true);
    let mut saved = original.clone();
    saved.stowed = Some("conner2.anm".into());
    saved
        .hand
        .iter_mut()
        .find(|(item, _)| item == "Dagger")
        .unwrap()
        .1 = "contpsd2.anm".into();
    saved
        .ranged
        .iter_mut()
        .find(|(item, _)| item == "SmCrossBow")
        .unwrap()
        .1 = "conicbw2.anm".into();
    assert_eq!(
        changed_third_person_banks(&original, &saved),
        ["conner2.anm", "contpsd2.anm", "conicbw2.anm"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    let arrival = Arrival {
        no_name_from: 0,
        items: Some(ItemLoad::Slot {
            objects: vec![],
            next_name: 0,
            order: 0,
            first_person_banks: vec![],
            third_person_banks: vec!["conner.anm".into()],
        }),
        third_person_lists: Some(saved.clone()),
        exit_movement: None,
        kept_vars: vec![],
        gone: Default::default(),
        came_in: vec![],
        object_copies: vec![],
        states: Default::default(),
        prop_placements: Default::default(),
        kept_maps: Default::default(),
    };
    let mut log = Log::create("armour-cold-loader").unwrap();
    let terrain = load_ground(&game, &world, &mut log).unwrap();
    let objects = load_objects(
        &game,
        &world,
        &terrain.terrain,
        &mut log,
        Some(arrival),
        None,
    );
    let banks = objects.connor_fight.as_ref().expect("Connor fight data");
    assert_eq!(banks.third_person, saved);
    assert_eq!(banks.first_person, first_person);
    let assets = &objects.connor.as_ref().expect("Connor's loaded model").1;
    for bank in ["conner2.anm", "contpsd2.anm", "conicbw2.anm", "conner.anm"] {
        let qualified = kq8_core::connor::special_movement_name(bank, "stop");
        let movement = assets
            .movements
            .iter()
            .find(|movement| movement.name.eq_ignore_ascii_case(&qualified))
            .unwrap_or_else(|| panic!("cold restore did not preload {qualified}"));
        let model = &assets.animations[movement.animation];
        assert!(
            model.frame_count > 0 && model.duration > 0. && model.source.is_some(),
            "{qualified}"
        );
        assert!(!movement.first_person);
    }
    assert!(
        assets
            .movements
            .iter()
            .any(|movement| movement.name == "stop"),
        "legacy bare base retained"
    );
}
