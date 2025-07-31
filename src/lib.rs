//! # **bevy_shatter**
//!
//! Procedural glass shattering plugin for the [Bevy game engine](https://bevyengine.org/)
//!
//! **Note**: This plugin uses [avian3d](https://github.com/Jondolf/avian) for collider generation, but [rapier3d](https://rapier.rs/) integration should be trivial to add in the future
//!
//! # Usage
//!
//! **Creating glass**
//!
//! Add the [`Glass`] component to an entity, using the [`Transform::scale`] as width, height and thickness. A helper is available in [`AutoGlass`] to add other needed components automatically, such as a mesh and a transform with the correct scale.
//!
//! **Shattering glass**
//!
//! Add the [`Shattered`] component to an entity that has [`Glass`], and glass shards will automatically be created.
//!
//! # Customizing behaviour
//!
//! This plugin prioritizes user control instead of guessing what the user wants to do, at a cost of convenience for the simpler use cases. You are responsible, for example, for adding RigidBody::Dynamic to each shard of glass (if that's what you need), and you can customize the entities using hooks.
//!
//! **Making the original glass entity hidden**
//!
//! This plugin does not assume what you want to do with the original [`Glass`] entity. If you want it to be hidden when the glass shatters, this will have to be done manually by inserting [`Visibility::Hidden`].
//!
//! **Shards**
//!
//! Are entities with the [`Shard`] component.
//! You can use this to, for example, make an OnAdd hook that automatically makes shards have a dynamic rigid body when added.
//!
//! **Shard relationship**
//!
//! Shards and their Glass are related using [`ShardOf`] and [`Shards`]. You can use this to delete all the shards belonging to a glass, make all the shards have the same material as their glass, etc.
//!
//! # Examples
//!
//! See the [`examples/`](https://github.com/ivsop/bevy_shatter) folder.
//!
//! # Compatibility
//!
//! | `bevy_shatter` | `bevy` |
//! | :--            | :--    |
//! | `0.1.0`        | `0.16` |
//!
//! # How it works
//!
//! Currently, the glass is broken into cells using a voronoi diagram. These cells are then extruded to 3D, creating a shard.
//!
//! # Contributing
//!
//! This plugin is in very early development. PRs and forks are welcome. See TODO.md for a list of things that are missing

use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    platform::collections::HashMap,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use voronator::{delaunator::*, VoronoiDiagram};

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
/// The component that marks an entity as glass that can be shattered. No other components are added to the entity, so you should add a material, mesh, etc. Feel free to take the mesh from [`GlassMesh`]. See [`AutoGlass`] for a quick way to spawn glass with some default components.
///
/// **Note:** the [`Transform::scale`] will be used to get the width (x), height (y) and thickness (z) of the glass.
#[derive(Component, Clone, Debug)]
pub struct Glass {
    /// The number of cell points to be used along the width and the height. Either passed in manually or through [`Glass::new_with_density`].
    /// Increasing this number means that more shattered glass pieces will be spawned, with smaller sizes,
    /// increasing computational cost
    pub num_cell_points: UVec2,
}

impl Glass {
    /// Generates glass using a density value (number of cells per unit of distance),
    /// automatically computing the number of cells
    pub fn new_with_density(width: f32, height: f32, cells_per_unit: f32) -> Self {
        let cells_x: u32 = (cells_per_unit * width).floor() as u32;
        let cells_y: u32 = (cells_per_unit * height).floor() as u32;

        Self {
            num_cell_points: UVec2::new(cells_x, cells_y),
        }
    }

    /// Generates glass using an XY grid for the number of cells
    pub fn new(num_cell_points: UVec2) -> Self {
        Self { num_cell_points }
    }

    // TODO: how to generate a lot of random numbers as fast as possible?
    // TODO: break this into more than one method
    /// Spawns the entities that make up the shattered glass and makes the old glass invisible.
    fn shatter(
        &self,
        glass_entity: Entity,
        glass_transf: &Transform,
        glass_material: Handle<StandardMaterial>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        // voronator crashes when the cells overlap or are too close
        // when using a lot of cells or a very small glass, this actually becomes a pain
        // you might also just get unlucky with the RNG gods and have the game crash for no apparent reason
        // to ensure this can never happen, I define a safety margin EPSILON, and cells must have at least that distance from each other
        // FIX: also consider the case where it is not possible to conserve this distance, but at that point it's mostly user error
        const EPSILON: f32 = 0.001;

        let width = glass_transf.scale.x;
        let height = glass_transf.scale.y;
        let thickness = glass_transf.scale.z;

        // the full cell width, used to determine the center of each cell
        let cell_width: f32 = width / self.num_cell_points.x as f32;
        let cell_height: f32 = height / self.num_cell_points.y as f32;

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

        let voronoi_diagram =
            VoronoiDiagram::<Point>::from_tuple(&(0., 0.), &(width as f64, height as f64), &cells)
                .expect("Error generating Voronoi diagram");

        // to allow shard baking, this is now done manually by the user
        // // mark original entity as invisible
        // commands.entity(glass_entity).insert(Visibility::Hidden);

        // it is (much) easier to offset the vertices themselves than the transform,
        // so every shard uses this transform which corresponds to the bottom left of the glass
        let shard_transform = glass_transf.with_scale(Vec3::ONE)
            * Transform::from_translation(Vec3::new(-width, -height, thickness) / 2.0);

        // iterate voronoi cells and triangulate them
        // also need to extrude since we want 3D mesh
        // TODO: consider generating the normals myself
        // TODO: extruding vertices was way harder than I expected, I have no idea thy I use negative values like -width and -thickness,
        // if it works it works. try to replace this with some lib that can extrude meshes in the future, I couldn't find anything decent and lightweight
        for (cell_id, cell) in voronoi_diagram.cells().iter().enumerate() {
            let points = cell.points();
            let shard_center = cells[cell_id];

            // points relative to this cell need to be converted to (f64, f64) for whatever reason
            // FIX: try to avoid this
            let cell_points: Vec<(f64, f64)> =
                points.iter().map(|point| (point.x, point.y)).collect();

            let (delaunay, _) = triangulate_from_tuple::<Point>(&cell_points)
                .expect("Error running delaunay triangulation on the cell");

            // Original vertices are used as the bottom (z = 0)
            let mut verts: Vec<Vec3> = points
                .iter()
                .map(|point| Vec3::new(point.x as f32, point.y as f32, 0.0))
                .collect();
            let n = verts.len();

            // Extruded vertices as the top (z = -thickness)
            let mut top_verts: Vec<Vec3> = points
                .iter()
                .map(|point| Vec3::new(point.x as f32, point.y as f32, -thickness))
                .collect();
            verts.append(&mut top_verts);

            // now we have to make edges to join the bottom and top vertices.
            // from here on this was mostly made by grok as I couldn't find any resources on this, and
            // making the triangles have the exact order you need them to have is hard
            let mut edge_count: HashMap<(usize, usize), i32> = HashMap::new();
            for triangle in delaunay.triangles.chunks(3) {
                let edges = [
                    (triangle[0], triangle[1]),
                    (triangle[1], triangle[2]),
                    (triangle[2], triangle[0]),
                ];
                for &(a, b) in edges.iter() {
                    *edge_count.entry((a, b)).or_insert(0) += 1;
                    *edge_count.entry((b, a)).or_insert(0) -= 1;
                }
            }

            // Only keep edges that appear once (boundary edges)
            let boundary_edges: Vec<(usize, usize)> = edge_count
                .iter()
                .filter(|&(&(_, _), &count)| count == 1)
                .map(|(&(a, b), _)| (a, b))
                .collect();

            let mut indices: Vec<u32> = Vec::new();

            // Bottom faces (reversed for outward facing)
            for triangle in delaunay.triangles.chunks(3) {
                indices.extend_from_slice(&[
                    triangle[2] as u32,
                    triangle[1] as u32,
                    triangle[0] as u32,
                ]);
            }

            // Top faces
            for triangle in delaunay.triangles.chunks(3) {
                indices.extend_from_slice(&[
                    (triangle[0] + n) as u32,
                    (triangle[1] + n) as u32,
                    (triangle[2] + n) as u32,
                ]);
            }

            // Side faces with proper winding
            // TODO: calculate normals here??
            for &(a, b) in boundary_edges.iter() {
                indices.extend_from_slice(&[
                    a as u32,
                    b as u32,
                    (b + n) as u32,
                    (b + n) as u32,
                    (a + n) as u32,
                    a as u32,
                ]);
            }

            // Create the mesh
            // I assume I will never need the mesh on the CPU again
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
            // .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(Indices::U32(indices));

            // collider
            let collider =
                // Collider::trimesh_from_mesh(&mesh) // this has abysmal performance for some reason, but works fine in rapier
                Collider::convex_hull_from_mesh(&mesh) // this is probably slow to create but is the only way I can get stable performance with avian
                .expect("Could not make trimesh out of the extrusion mesh for a cell");

            // add the normals. this is VERY inneficient but whatever, had many issues doing it manually
            // also, should this be done before collider??
            mesh = mesh.with_duplicated_vertices().with_computed_flat_normals();

            // spawn the glass shard
            commands.spawn((
                shard_transform,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(glass_material.clone()),
                collider,
                ShardOf(glass_entity),
                Shard {
                    pos: Vec2::new(shard_center.0 as f32, shard_center.1 as f32),
                },
            ));
        }
    }
}

/// Glass shards are children of a glass, through this relationship.
/// This allows you to get information on what glass caused certain shards to spawn.
#[derive(Component)]
#[relationship(relationship_target = Shards)]
pub struct ShardOf(pub Entity);

/// Glasses have shards as their children. This allows you to get all the shards originating in a glass.
/// With this, you could, for example, despawn the glass entity when all its shards have been despawned.
#[derive(Component, Deref)]
#[relationship_target(relationship = ShardOf)]
pub struct Shards(Vec<Entity>);

/// Every glass shard has this component, so you can use it with a hook to customize the shards.
#[derive(Component)]
pub struct Shard {
    /// Position in the glass, relative to the bottom left point
    pub pos: Vec2,
}

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
/// - [`Collider`]
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

#[derive(Resource)]
struct GlassCollider(Collider);

#[derive(Resource)]
pub struct GlassMesh(Handle<Mesh>);

/// Add this component to an entity with the [`Glass`] component to shatter it,
/// which creates all the glass shards.
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
            scale: Vec3::new(ag.width, ag.height, ag.thickness),
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

    let (glass, transform, material) = glasses.get(entity).unwrap();
    glass.shatter(
        entity,
        transform,
        material.0.clone(),
        commands.reborrow(),
        meshes,
    );
}
