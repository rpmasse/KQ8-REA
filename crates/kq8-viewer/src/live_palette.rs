//! Live indexed-image rebuilds from immutable palette/index inputs (Q186).
use bevy::{image::ImageAddressMode, prelude::*};
use kq8_data::palette::{Output, Rgb};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Values {
    pub brightness: u8,
    pub output: Output,
}

#[derive(Debug, Clone, Copy)]
pub enum AlphaTransform {
    Mesh,
    Dither(u8),
}

#[derive(Clone)]
struct Recipe {
    source: Arc<crate::load::IndexedTexture>,
    width: u32,
    height: u32,
    transforms: Vec<AlphaTransform>,
}

impl Recipe {
    fn rgba(&self, values: Values) -> Vec<u8> {
        let palette =
            kq8_data::palette::for_output(*self.source.palette, values.brightness, values.output);
        let mut rgba: Vec<u8> = self
            .source
            .pixels
            .iter()
            .flat_map(|&index| {
                let [r, g, b] = palette[index as usize];
                [
                    r,
                    g,
                    b,
                    if index == 0 && self.source.transparent_zero {
                        0
                    } else {
                        255
                    },
                ]
            })
            .collect();
        for transform in &self.transforms {
            match *transform {
                AlphaTransform::Mesh => {
                    for y in 0..self.height {
                        for x in 0..self.width {
                            if (x + 2 * y) % 4 != 0 {
                                rgba[((y * self.width + x) * 4 + 3) as usize] = 0;
                            }
                        }
                    }
                }
                AlphaTransform::Dither(mode) => {
                    crate::potion_view::dither_rgba(&mut rgba, self.width as usize, mode)
                }
            }
        }
        rgba
    }
}

pub struct WorldPalette {
    pub values: Values,
    pub haze_source: Rgb,
    entries: Mutex<HashMap<AssetId<Image>, Recipe>>,
    serial: u64,
}

#[derive(Debug)]
pub struct Rebuild {
    pub serial: u64,
    pub images: usize,
    pub changed: usize,
    pub before: u64,
    pub after: u64,
}

pub fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

impl WorldPalette {
    pub fn source_key(source: &Option<Arc<crate::load::IndexedTexture>>) -> u64 {
        source.as_ref().map_or(0, |s| {
            s.palette
                .iter()
                .flatten()
                .fold(0xcbf29ce484222325, |h, &b| {
                    (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
                })
        })
    }

    pub fn new(values: Values, haze_source: Rgb) -> Self {
        Self {
            values,
            haze_source,
            entries: Default::default(),
            serial: 0,
        }
    }

    pub fn add(
        &self,
        images: &mut Assets<Image>,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        source: Option<Arc<crate::load::IndexedTexture>>,
        address: ImageAddressMode,
    ) -> Handle<Image> {
        let recipe = source.map(|source| Recipe {
            source,
            width,
            height,
            transforms: Vec::new(),
        });
        let rgba = recipe.as_ref().map_or(rgba, |r| r.rgba(self.values));
        let image = images.add(crate::object_light::texture_image(
            width, height, rgba, address,
        ));
        if let Some(recipe) = recipe {
            self.entries.lock().unwrap().insert(image.id(), recipe);
        }
        image
    }

    pub fn look(
        &self,
        look: crate::load::Look,
        images: &mut Assets<Image>,
        address: ImageAddressMode,
    ) -> Option<Handle<Image>> {
        let crate::load::Look::Texture {
            width,
            height,
            rgba,
            indexed,
            ..
        } = look
        else {
            return None;
        };
        Some(self.add(images, width, height, rgba, indexed, address))
    }

    pub fn derived(
        &self,
        original: &Handle<Image>,
        target: &Handle<Image>,
        transform: AlphaTransform,
    ) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(mut recipe) = entries.get(&original.id()).cloned() {
            recipe.transforms.push(transform);
            entries.insert(target.id(), recipe);
        }
    }

    pub fn rebuild(&mut self, values: Values, images: &mut Assets<Image>) -> Option<Rebuild> {
        if values == self.values {
            return None;
        }
        self.values = values;
        self.serial += 1;
        let mut result = Rebuild {
            serial: self.serial,
            images: 0,
            changed: 0,
            before: 0,
            after: 0,
        };
        self.entries.lock().unwrap().retain(|id, recipe| {
            let Some(mut image) = images.get_mut(*id) else {
                return false;
            };
            assert_eq!(
                (
                    image.texture_descriptor.size.width,
                    image.texture_descriptor.size.height
                ),
                (recipe.width, recipe.height)
            );
            let rgba = recipe.rgba(values);
            let Some(data) = image.data.as_mut() else {
                return false;
            };
            assert_eq!(
                data.len(),
                rgba.len(),
                "a palette rebuild cannot resize an image"
            );
            result.images += 1;
            result.changed += usize::from(*data != rgba);
            result.before = result.before.wrapping_add(fingerprint(data));
            result.after = result.after.wrapping_add(fingerprint(&rgba));
            *data = rgba;
            true
        });
        Some(result)
    }
}

/// Apply notification from the slider or surface restoration; storing a
/// preference alone does not call this (Q186 rules 1/3/4/16).
pub fn apply_to_surface(world: &mut World, values: Values, why: &str) {
    let mut stable = None;
    let result = if world.contains_resource::<crate::object_light::ObjectLight>() {
        let (result, colour, before, after) =
            world.resource_scope(|world, mut light: Mut<crate::object_light::ObjectLight>| {
                let before = light.palette_fingerprint(world.resource::<Assets<Image>>());
                let result = light
                    .palette
                    .rebuild(values, &mut world.resource_mut::<Assets<Image>>());
                if result.is_some() {
                    light.haze.colour = kq8_data::palette::hardware_fog(
                        light.palette.haze_source,
                        values.brightness,
                        values.output,
                    )
                    .map(|n| f32::from(n) / 255.0);
                }
                let after = light.palette_fingerprint(world.resource::<Assets<Image>>());
                (
                    result,
                    Vec3::from(light.haze.colour).extend(0.0),
                    before,
                    after,
                )
            });
        stable = Some((before, after));
        if result.is_some() {
            for (_, m) in world
                .resource_mut::<Assets<crate::object_light::ObjectMaterial>>()
                .iter_mut()
            {
                m.settings.haze_colour = colour;
            }
            for (_, m) in world
                .resource_mut::<Assets<crate::ground_haze::GroundMaterial>>()
                .iter_mut()
            {
                m.settings.haze_colour = colour;
            }
            for (_, m) in world
                .resource_mut::<Assets<crate::building_light::BuildingMaterial>>()
                .iter_mut()
            {
                m.settings.haze_colour = colour;
            }
        }
        result
    } else {
        None
    };
    if let Some(mut gui) = world.get_resource_mut::<crate::gui_view::Gui>() {
        gui.apply_palette(values);
    }
    if let Some(mut style) = world.get_resource_mut::<crate::talk_view::CaptionStyle>() {
        style.apply_palette(values);
    }
    crate::startup_menu::palette_changed(world, values, why);
    let menu = world.contains_resource::<crate::startup_menu::Menu>();
    let flush = result.as_ref().map(|r| r.serial);
    world.resource::<crate::Shared>().log.lock().unwrap().summary(format!(
        "palette: live brightness {} gain {:.9}, output {:?}, menu open {menu}, flush {flush:?}; {result:?}; stable cache {stable:?}; source matching and texture dimensions retained ({why})",
        values.brightness,kq8_data::palette::brightness_gain(values.brightness),values.output));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn values(brightness: u8) -> Values {
        Values {
            brightness,
            output: Output::Direct3D,
        }
    }

    #[test]
    fn live_rebuilds_preserve_original_indices_dimensions_alpha_and_unindexed_images() {
        let mut source = [[0; 3]; 256];
        source[10] = [102, 153, 204];
        let source = Arc::new(crate::load::IndexedTexture {
            pixels: vec![0, 10, 10, 10].into(),
            palette: Arc::new(source),
            transparent_zero: true,
        });
        let mut images = Assets::<Image>::default();
        let mut live = WorldPalette::new(values(50), [102, 153, 204]);
        let own = live.add(
            &mut images,
            2,
            2,
            vec![],
            Some(source.clone()),
            ImageAddressMode::Repeat,
        );
        let original = images.get(&own).unwrap().data.clone().unwrap();
        assert_eq!(&original[..8], &[0, 0, 0, 0, 102, 153, 204, 255]);
        let true_colour = live.add(
            &mut images,
            1,
            1,
            vec![22, 44, 66, 255],
            None,
            ImageAddressMode::ClampToEdge,
        );
        let lightmap = images.add(crate::object_light::texture_image(
            1,
            1,
            vec![15, 30, 45, 255],
            ImageAddressMode::ClampToEdge,
        ));
        let dim = live.rebuild(values(0), &mut images).unwrap();
        let dark = images.get(&own).unwrap().data.clone().unwrap();
        assert_eq!(&dark[4..8], &[30, 45, 61, 255]);
        assert_eq!((dim.images, dim.changed, dim.serial), (1, 1, 1));
        live.rebuild(values(100), &mut images).unwrap();
        assert_eq!(
            &images.get(&own).unwrap().data.as_ref().unwrap()[4..8],
            &[127, 191, 255, 255]
        );
        live.rebuild(values(0), &mut images).unwrap();
        assert_eq!(
            images.get(&own).unwrap().data.as_ref(),
            Some(&dark),
            "0/100/0 starts from the original palette each time"
        );
        assert!(live.rebuild(values(0), &mut images).is_none());
        assert_eq!(
            images.get(&true_colour).unwrap().data.as_deref(),
            Some(&[22, 44, 66, 255][..])
        );
        assert_eq!(
            images.get(&lightmap).unwrap().data.as_deref(),
            Some(&[15, 30, 45, 255][..])
        );
        assert_eq!(
            (
                images.get(&own).unwrap().width(),
                images.get(&own).unwrap().height()
            ),
            (2, 2)
        );
        assert_eq!(&*source.pixels, &[0, 10, 10, 10]);
        assert_eq!(source.palette[10], [102, 153, 204]);
        let later = live.add(
            &mut images,
            2,
            2,
            original,
            Some(source),
            ImageAddressMode::Repeat,
        );
        assert_eq!(
            images.get(&later).unwrap().data.as_ref(),
            Some(&dark),
            "a newly created runtime image uses the active palette"
        );
    }

    #[test]
    fn derived_dither_recomputes_colour_key_after_palette_changes() {
        let mut source = [[0; 3]; 256];
        source[11] = [1, 2, 3];
        let source = Arc::new(crate::load::IndexedTexture {
            pixels: vec![11].into(),
            palette: Arc::new(source),
            transparent_zero: false,
        });
        let mut images = Assets::<Image>::default();
        let mut live = WorldPalette::new(values(50), [0; 3]);
        let own = live.add(
            &mut images,
            1,
            1,
            vec![],
            Some(source),
            ImageAddressMode::Repeat,
        );
        let mut image = images.get(&own).unwrap().clone();
        crate::potion_view::dither_rgba(image.data.as_mut().unwrap(), 1, 2);
        let copy = images.add(image);
        live.derived(&own, &copy, AlphaTransform::Dither(2));
        live.rebuild(values(0), &mut images).unwrap();
        assert_eq!(
            images.get(&copy).unwrap().data.as_deref(),
            Some(&[0, 0, 0, 0][..])
        );
        live.rebuild(values(100), &mut images).unwrap();
        assert_eq!(
            images.get(&copy).unwrap().data.as_deref(),
            Some(&[1, 3, 5, 255][..])
        );
    }
}
