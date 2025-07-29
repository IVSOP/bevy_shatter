use avian3d::prelude::*;
use bevy::prelude::*;

/// This plugin must be added for everything to work properly
pub struct ShatterPlugin;

impl Plugin for ShatterPlugin {
    fn build(&self, app: &mut App) {
        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        let glass_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let glass_collider = Collider::cuboid(1.0, 1.0, 1.0);

        app.insert_resource(GlassMesh(glass_mesh))
            .insert_resource(GlassCollider(glass_collider))
            .add_observer(autoglass);
    }
}

/// Describes how many cells to use in the X and Y directions,
/// and allows computing them depending on the size of the glass
#[derive(Clone)]
pub enum ShatterConfig {
    /// Manually specify how many cells to use along the width and height of the glass
    Grid { cells_w: f32, cells_y: f32 },
    /// How many cells there should be per each unit of distance (1.0)
    Density { cells_per_unit: f32 },
}

#[derive(Component, Clone)]
pub struct Glass {
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
    pub shatter_config: ShatterConfig,
}

/// Glass shards are [`ShardOf`] a certain glass.
/// This way we can leverage ECS to get all the shards belonging to a glass
#[derive(Component)]
#[relationship(relationship_target = Shards)]
struct ShardOf(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = ShardOf)]
struct Shards(Vec<Entity>);

// TODO: is this cursed? is there a better way?
/// Allows more automation of glass spawning.
/// Requires [`ShatterPlugin`] to have been added.
/// When an entity with this component is inserted:
///
/// - [`AutoGlass`] component gets removed
/// - glass gets added as its own [`Glass`] component
/// - a [`Transform`] component is added
/// - a [`Mesh3d`] component is added
/// - a [`RigidBody::Static`] component is added
/// - a [`Collider`] component is added
///
/// Note: no material component is added or removed
///
/// You can completely ignore this and do things manually for more control. Keep in mind this function
/// uses meshes and colliders of size 1x1x1, being only resized by their transform's scale
#[derive(Component)]
pub struct AutoGlass {
    pub glass: Glass,
    pub translation: Vec3,
    pub rotation: Quat,
}

#[derive(Resource)]
struct GlassCollider(Collider);

#[derive(Resource)]
struct GlassMesh(Handle<Mesh>);

// The entire thing is cursed but works
fn autoglass(
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
            scale: Vec3::new(glass.width, glass.height, glass.thickness),
        },
        Mesh3d(mesh.0.clone()),
        collider.0.clone(), // TODO: test if this is faster than recomputing the collider
        glass.clone(),
    ));
    entitycmd.remove::<AutoGlass>();
}
