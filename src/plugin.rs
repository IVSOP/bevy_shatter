use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::*;

/// This plugin must be added for everything to work properly
pub struct ShatterPlugin;

impl Plugin for ShatterPlugin {
    fn build(&self, app: &mut App) {
        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        let glass_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let glass_collider = Collider::cuboid(1.0, 1.0, 1.0);

        app.insert_resource(GlassMesh(glass_mesh))
            .insert_resource(GlassCollider(glass_collider))
            .add_observer(autoglass_hook)
            .add_observer(shatter_hook);
    }
}
