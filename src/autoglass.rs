use bevy::prelude::*;

use crate::*;

// TODO: is this cursed? is there a better way?
/// Allows simpler glass spawning, by automatically adding the necessary components.
/// Requires [`ShatterPlugin`] to have been added.
///
/// Removed components:
/// - [`AutoGlass`]
///
/// Added components:
/// - [`Glass`]
/// - [`Transform`], with the correct scale
/// - [`Mesh3d`], as a cuboid
/// - [`Collider`], as a cuboid
///
/// Note: no material or rigid body are added.
/// You can completely ignore this and do things manually for more control. Keep in mind this function
/// uses meshes and colliders of size 1x1x1, being only resized by their transform's scale
#[derive(Component, Debug)]
pub struct AutoGlass {
    pub glass: Glass,
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
    pub translation: Vec3,
    pub rotation: Quat,
}

// The entire thing is cursed but works
// FIX: can't I read the AutoGlass struct straigt from the trigger???????????? that way I don't need the query or the unwrap
/// Hook to add [`AutoGlass`] functionality when it is added to an entity
pub(super) fn autoglass_hook(
    trigger: Trigger<OnAdd, AutoGlass>,
    mut commands: Commands,
    collider: Res<GlassCollider>,
    mesh: Res<GlassMesh>,
    autoglasses: Populated<&AutoGlass>,
) {
    let entity = trigger.target();
    let mut entitycmd = commands.entity(entity);

    let ag = autoglasses.get(entity).unwrap();
    let glass = &ag.glass;

    entitycmd.insert((
        Transform {
            translation: ag.translation,
            rotation: ag.rotation,
            scale: Vec3::new(ag.width, ag.height, ag.thickness),
        },
        Mesh3d(mesh.0.clone()),
        collider.0.clone(), // TODO: test if this is faster than recomputing the collider
        glass.clone(),
    ));
    entitycmd.remove::<AutoGlass>();
}
