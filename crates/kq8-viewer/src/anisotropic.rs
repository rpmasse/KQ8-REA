//! Anisotropic samplers for 3D surface textures only. Light maps and
//! interface pictures keep their existing sampling.
use bevy::asset::AssetId;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::{building_light::BuildingMaterial, ground_haze::GroundMaterial,
    object_light::ObjectMaterial, option_settings::Preferences, Shared};

fn filtered_sampler(original: &ImageSampler, level: u16) -> ImageSampler {
    let mut sampler = original.clone();
    if level == 1 {
        sampler.get_or_init_descriptor().anisotropy_clamp = 1;
    } else {
        sampler.get_or_init_descriptor().set_anisotropic_filter(level);
    }
    sampler
}

pub fn sync(
    preferences: Option<Res<Preferences>>,
    objects: Res<Assets<ObjectMaterial>>,
    ground: Res<Assets<GroundMaterial>>,
    buildings: Res<Assets<BuildingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    shared: Res<Shared>,
    mut last: Local<Option<u16>>,
) {
    let Some(preferences) = preferences else { return };
    let level = preferences.anisotropic.level;
    let selection_changed = *last != Some(level);
    if !selection_changed && !objects.is_changed() && !ground.is_changed() && !buildings.is_changed() {
        return;
    }
    *last = Some(level);
    let textures: HashSet<AssetId<Image>> = objects.iter().filter_map(|(_, material)| material.texture.as_ref())
        .chain(ground.iter().filter_map(|(_, material)| material.texture.as_ref()))
        .chain(buildings.iter().filter_map(|(_, material)| material.texture.as_ref()))
        .map(Handle::id).collect();
    let mut changed = 0;
    for id in &textures {
        let Some(image) = images.get(*id) else { continue };
        let sampler = filtered_sampler(&image.sampler, level);
        if sampler != image.sampler {
            images.get_mut(*id).unwrap().sampler = sampler;
            changed += 1;
        }
    }
    if selection_changed || changed > 0 {
        shared.log.lock().unwrap().summary(format!(
            "anisotropic: {} selected; {} of {} 3D surface texture samplers changed (image pixels unchanged)",
            preferences.anisotropic.label(), changed, textures.len()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::image::{ImageAddressMode, ImageFilterMode};

    #[test]
    fn filtering_preserves_texture_pixels_and_addressing() {
        let image = crate::object_light::texture_image(2, 2, vec![17; 16], ImageAddressMode::Repeat);
        let pixels = image.data.clone();
        for level in [1, 4, 8, 16] {
            let sampler = filtered_sampler(&image.sampler, level);
            let ImageSampler::Descriptor(descriptor) = sampler else { panic!("expected descriptor") };
            assert_eq!(descriptor.anisotropy_clamp, level);
            assert_eq!(descriptor.address_mode_u, ImageAddressMode::Repeat);
            assert_eq!(descriptor.address_mode_v, ImageAddressMode::Repeat);
            assert_eq!(descriptor.mag_filter, ImageFilterMode::Linear);
            assert_eq!(descriptor.min_filter, ImageFilterMode::Linear);
            assert_eq!(descriptor.mipmap_filter, ImageFilterMode::Linear);
        }
        let enabled = filtered_sampler(&image.sampler, 16);
        let disabled = filtered_sampler(&enabled, 1);
        assert_eq!(disabled, image.sampler);
        assert_eq!(image.data, pixels);
    }
}
