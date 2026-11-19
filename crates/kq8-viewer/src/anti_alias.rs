//! Apply the Graphics page's exclusive anti-aliasing choice to each 3D camera.
use bevy::anti_alias::{fxaa::Fxaa, smaa::Smaa};
use bevy::prelude::*;
use bevy::render::view::Msaa;
use kq8_core::modern_display::AntiAliasing;

use crate::option_settings::Preferences;

pub fn sync(world: &mut World) {
    let Some(choice) = world.get_resource::<Preferences>().map(|p| p.anti_aliasing) else {
        return;
    };
    let mut query = world.query_filtered::<Entity, With<Camera3d>>();
    let cameras: Vec<_> = query.iter(world).collect();
    let mut applied = Vec::new();
    for id in cameras {
        let mut camera = world.entity_mut(id);
        let mut changed = false;
        if camera.get::<Msaa>() != Some(&Msaa::Off) {
            camera.insert(Msaa::Off);
            changed = true;
        }
        match choice {
            AntiAliasing::Off => {
                if camera.get::<Fxaa>().is_some() { camera.remove::<Fxaa>(); changed = true; }
                if camera.get::<Smaa>().is_some() { camera.remove::<Smaa>(); changed = true; }
            }
            AntiAliasing::Smaa => {
                if camera.get::<Fxaa>().is_some() { camera.remove::<Fxaa>(); changed = true; }
                if camera.get::<Smaa>().is_none() { camera.insert(Smaa::default()); changed = true; }
            }
            AntiAliasing::Fxaa => {
                if camera.get::<Smaa>().is_some() { camera.remove::<Smaa>(); changed = true; }
                if camera.get::<Fxaa>().is_none() { camera.insert(Fxaa::default()); changed = true; }
            }
        }
        if changed { applied.push(format!("anti-aliasing: camera {id:?} uses {} (MSAA off)", choice.label())); }
    }
    if let Some(shared) = world.get_resource::<crate::Shared>() {
        let mut log = shared.log.lock().unwrap();
        for line in applied { log.summary(line); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_core::options_menu::Settings;
    use std::path::Path;

    #[test]
    fn choice_switches_live_camera_and_new_camera_without_stacking_effects() {
        let mut world = World::new();
        world.insert_resource(Preferences::isolated_for_test(Path::new("."), Settings::default()));
        let first = world.spawn(Camera3d::default()).id();
        sync(&mut world);
        assert_eq!(world.entity(first).get::<Msaa>(), Some(&Msaa::Off));
        assert!(world.entity(first).get::<Fxaa>().is_none());
        assert!(world.entity(first).get::<Smaa>().is_none());

        world.resource_mut::<Preferences>().anti_aliasing = AntiAliasing::Smaa;
        sync(&mut world);
        let second = world.spawn(Camera3d::default()).id();
        sync(&mut world);
        for id in [first, second] {
            assert_eq!(world.entity(id).get::<Msaa>(), Some(&Msaa::Off));
            assert!(world.entity(id).get::<Smaa>().is_some());
            assert!(world.entity(id).get::<Fxaa>().is_none());
        }

        world.resource_mut::<Preferences>().anti_aliasing = AntiAliasing::Fxaa;
        sync(&mut world);
        for id in [first, second] {
            assert!(world.entity(id).get::<Fxaa>().is_some());
            assert!(world.entity(id).get::<Smaa>().is_none());
        }

        world.resource_mut::<Preferences>().anti_aliasing = AntiAliasing::Off;
        sync(&mut world);
        for id in [first, second] {
            assert!(world.entity(id).get::<Fxaa>().is_none());
            assert!(world.entity(id).get::<Smaa>().is_none());
        }
    }
}
