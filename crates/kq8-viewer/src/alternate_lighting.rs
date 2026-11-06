//! Original interior alternate lighting (object-lighting.md rule 12).
use std::sync::Arc;
use crate::{building_light::LightAtlas, wall_lights::WallData};

/// A decoded authored lighting baseline. No texture or geometry replacement.
pub struct LightChoice {
    pub file: String,
    pub normal_file: String,
    pub atlas: Arc<LightAtlas>,
    pub data: Arc<WallData>,
}

pub fn same_layout(normal: &kq8_data::dil::BuildingLight, alternate: &kq8_data::dil::BuildingLight) -> bool {
    normal.scale_shift == alternate.scale_shift && normal.surfaces.len() == alternate.surfaces.len() && normal.surfaces.iter().zip(&alternate.surfaces).all(|(a,b)| {
        (a.has_map(), a.width, a.height, a.map_offset) == (b.has_map(), b.width, b.height, b.map_offset)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn original_alternate_interior_maps_match_their_geometry() {
        let game = kq8_data::GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let mut log = crate::log::Log::create("alternate-interior-data").unwrap();
        let mut authored = Vec::new();
        for region in kq8_data::game_dir::WORLDS {
            let world = crate::load::World::open(&game, region).unwrap();
            for (name, _, bytes) in world.entries_ending(".kq") {
                let bytes = bytes.unwrap();
                let sections = kq8_data::kq_text::parse(&bytes);
                if sections.first().is_some_and(|s| s.get("althwLightmapName").is_some()) {
                    authored.push(format!("{region}/{name}").to_ascii_lowercase());
                }
            }
        }
        authored.sort();
        assert_eq!(authored, ["gnome/gnome13.kq", "gnome/gnome14.kq", "temple4/temple.kq"]);
        for (region, file) in [("Gnome", "gnome13.kq"), ("Gnome", "gnome14.kq"), ("temple4", "temple.kq")] {
            let world = crate::load::World::open(&game, region).unwrap();
            let (_, bytes) = world.load(file).unwrap().unwrap();
            let sections = kq8_data::kq_text::parse(&bytes);
            let first = &sections[0];
            let shape = first.get("shapeName").unwrap();
            let base = format!("{}.dil", shape.rsplit_once('.').unwrap().0);
            let alt = first.get("althwLightmapName").unwrap();
            let bytes = |name| world.load(name).unwrap().unwrap().1;
            let building = kq8_data::building::Building::parse(&bytes(shape)).unwrap();
            let normal = kq8_data::dil::BuildingLight::parse(&bytes(&base)).unwrap();
            let alternate = kq8_data::dil::BuildingLight::parse(&bytes(alt)).unwrap();
            for light in [&normal, &alternate] {
                assert_eq!(light.build_id, building.build_id);
                assert_eq!(light.surfaces.len(), building.surfaces.len());
            }
            let different_layouts = normal.surfaces.iter().zip(&alternate.surfaces).filter(|(a, b)| {
                (a.has_map(), a.width, a.height, a.map_offset) != (b.has_map(), b.width, b.height, b.map_offset)
            }).count();
            assert!(super::same_layout(&normal, &alternate));
            let a = crate::building_light::LightAtlas::build(&normal);
            let b = crate::building_light::LightAtlas::build(&alternate);
            assert_eq!((a.width, a.height, &a.places, a.texels_per_light), (b.width, b.height, &b.places, b.texels_per_light));
            assert_ne!(a.rgba, b.rgba);
            let changed = normal.values().zip(alternate.values()).filter(|(a,b)| a != b).count();
            log.summary(format!("{region}/{file}: {base} -> {alt}, {} surfaces; {different_layouts} different layouts; {changed} changed sequential texels", building.surfaces.len()));
        }
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR"]
    fn original_gnome_interiors_load_both_authored_maps_and_register_defaults() {
        let game = kq8_data::GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR")).unwrap();
        let world = crate::load::World::open(&game, "Gnome").unwrap();
        let mut log = crate::log::Log::create("alternate-interior-loader").unwrap();
        let ground = crate::load::load_ground(&game, &world, &mut log).unwrap();
        let objects = crate::load::load_objects(&game, &world, &ground.terrain, &mut log, None, None);
        for name in ["gnome13", "gnome14"] {
            assert_eq!(objects.interior_lighting.get(name), Some(&false));
            assert!(!objects.persistent_stat_objects.contains(name), "permanent interiors reload fresh");
            let (_, model) = objects.shapes.iter().find(|(file, _)| file.eq_ignore_ascii_case(&format!("{name}.dig"))).unwrap();
            let alternate = model.alternate_light.as_ref().unwrap();
            assert_eq!(alternate.file, format!("{name}l.dil"));
            assert_ne!(model.light_atlas.as_ref().unwrap().rgba, alternate.atlas.rgba);
        }
        let p = &objects.doors.iter().find(|object| object.placement.name.eq_ignore_ascii_case("fusebox")).unwrap().placement;
        assert_eq!(p.position, [98527.,127000.,15525.]);
        assert!(objects.scripts.iter().any(|(name, _)| name.eq_ignore_ascii_case("fusebox.cs")));
    }
}
