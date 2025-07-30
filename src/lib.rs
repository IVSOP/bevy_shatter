use avian3d::prelude::*;
use bevy::prelude::*;
use fastrand::*;
use voronator::{
    delaunator::{self, *},
    VoronoiDiagram,
};

/// This plugin must be added for everything to work properly
pub struct ShatterPlugin;

impl Plugin for ShatterPlugin {
    fn build(&self, app: &mut App) {
        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        let glass_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let glass_collider = Collider::cuboid(1.0, 1.0, 1.0);

        app.insert_resource(GlassMesh(glass_mesh))
            .insert_resource(GlassCollider(glass_collider))
            .add_observer(autoglass)
            .add_observer(shatter_hook);
    }
}

// TODO: store num_cell_points as floats??
/// The component that marks an entity as glass that can be shattered.
/// See [`AutoGlass`] for a helper to automate spawning glass entities with all necessary components
#[derive(Component, Clone)]
pub struct Glass {
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
    /// The number of cell points to be used along the width and the height. Either passed in manually or through [`Glass::new_with_density`].
    /// Increasing this number means that more shattered glass pieces will be spawned, with smaller sizes,
    /// increasing computational cost
    pub num_cell_points: UVec2,
}

impl Glass {
    /// Generates glass using a density value (number of cells per unit of distance),
    /// automatically computing the number of cells
    pub fn new_with_density(width: f32, height: f32, thickness: f32, cells_per_unit: f32) -> Self {
        let cells_x: u32 = (cells_per_unit * width).floor() as u32;
        let cells_y: u32 = (cells_per_unit * height).floor() as u32;

        Self {
            width,
            height,
            thickness,
            num_cell_points: UVec2::new(cells_x, cells_y),
        }
    }

    /// Generates glass using an XY grid for the number of cells
    pub fn new(width: f32, height: f32, thickness: f32, num_cell_points: UVec2) -> Self {
        Self {
            width,
            height,
            thickness,
            num_cell_points,
        }
    }

    // TODO: how to generate a lot of random numbers as fast as possible?
    // TODO: break this into more than one method
    /// Spawns the entities that make up the shattered glass and makes the old glass invisible.
    fn shatter(
        &self,
        glass_entity: Entity,
        glass_transf: &Transform,
        glass_material: Handle<StandardMaterial>,
        world_break_pos: Vec3,
        mut commands: Commands,
        meshes: ResMut<Assets<Mesh>>,
    ) {
        // voronator crashes when the cells overlap or are too close
        // when using a lot of cells or a very small glass, this actually becomes a pain
        // you might also just get unlucky with the RNG gods and have the game crash for no apparent reason
        // to ensure this can never happen, I define a safety margin EPSILON, and cells must have at least that distance from each other
        // FIX: also consider the case where it is not possible to conserve this distance, but at that point it's mostly user error
        const EPSILON: f32 = 0.001;

        // the full cell width, used to determine the center of each cell
        let cell_width: f32 = self.width / self.num_cell_points.x as f32;
        let cell_height: f32 = self.height / self.num_cell_points.y as f32;

        // the max offset a point can be in from the center of the cell
        let cell_offset = Vec2::new((cell_width / 2.0) - EPSILON, (cell_height / 2.0) - EPSILON);

        // 2*cell_offset, representing the max offset from one edge of the cell to the other, instead of just to the center
        let full_cell_offset = cell_offset * 2.0;

        let mut cells: Vec<(f64, f64)> = Vec::new();

        // build the cells from bottom left to top right
        for y in 0..self.num_cell_points.y {
            for x in 0..self.num_cell_points.x {
                let cell_center = Vec2::new(x as f32 * cell_width, y as f32 * cell_height);

                let bottom_left = cell_center - cell_offset;

                // generate a random float inside the cell, using cell_offset_2
                // then offset it by the bottom left position
                let rand = Vec2::new(fastrand::f32(), fastrand::f32());

                let position = (rand * full_cell_offset) + bottom_left;

                cells.push((position.x as f64, position.y as f64));
            }
        }

        let voronoi_diagram = VoronoiDiagram::<Point>::from_tuple(
            &(0., 0.),
            &(self.width as f64, self.height as f64),
            &cells,
        )
        .expect("Error generating Voronoi diagram");

        // delete the old entity
        commands.entity(glass_entity).despawn();
    }
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

/// Add this component to an entity with the [`Glass`] component to shatter it
#[derive(Component)]
pub struct Shattered;

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

fn shatter_hook(
    trigger: Trigger<OnAdd, Shattered>,
    glasses: Populated<(&Glass, &Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
) {
    let entity = trigger.target();
    let mut entitycmd = commands.entity(entity);

    let (glass, transform, material) = glasses.get(entity).unwrap();
    glass.shatter(entity, transform, material.0.clone(), Vec3::ZERO, commands.reborrow(), meshes);
}

