use avian3d::{prelude::*, PhysicsPlugins};
use bevy_shatter::*;
mod menu;
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
    window::{PresentMode, WindowTheme},
};
use bevy_atmosphere::prelude::*;
use menu::*;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // title: "I am a window!".into(),
                // name: Some("bevy.app".into()),
                // resolution: (500., 300.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells Wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                // prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                // enabled_buttons: bevy::window::EnabledButtons {
                //     maximize: false,
                //     ..Default::default()
                // },
                // This will spawn an invisible window
                // The window will be made visible in the make_visible() system after 3 frames.
                // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                // visible: false,
                ..default()
            }),
            ..default()
        }),
        MenuPlugin,
        PhysicsPlugins::default(),
        PhysicsDebugPlugin::default(),
        ShatterPlugin,
        AtmospherePlugin,
    ))
    .insert_gizmo_config(
        PhysicsGizmos::default(),
        GizmoConfig {
            enabled: false,
            ..default()
        },
    )
    .insert_gizmo_config(
        DefaultGizmoConfigGroup::default(),
        GizmoConfig {
            enabled: false,
            ..default()
        },
    )
    .add_observer(display_shards)
    .add_observer(hide_shards)
    .add_systems(Startup, (setup_scene, setup_camera));

    app.run();
}

fn setup_scene(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::new(0.0, 2.0, 0.0, 1.0)),
        ..default()
    });

    let plane = meshes.add(Plane3d::new(Vec3::NEG_Z, Vec2::splat(0.5)));

    // floor
    commands.spawn((
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(90.0_f32.to_radians()),
            scale: Vec3::new(100.0, 100.0, 1.0),
        },
        Mesh3d(plane.clone()),
        MeshMaterial3d(ground_material.clone()),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 0.01),
    ));

    let glass_material = materials.add(StandardMaterial {
        // color that contrasts with the sky
        base_color: Color::LinearRgba(LinearRgba::new(2.5, 0.0, 0.0, 0.65)),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.0,
        reflectance: 0.1,
        emissive: LinearRgba::rgb(0.0, 0.1, 0.1),
        // cull_mode: None, // Render both sides of the glass
        ..default()
    });

    // glass
    let width = 20.0;
    let height = 5.0;
    let thickness = 0.1;
    let mut glass = commands.spawn((
        AutoGlass {
            width,
            height,
            thickness,
            translation: Vec3::new(0.0, 3.0, -10.0),
            rotation: Quat::IDENTITY,
            glass: Glass::new_with_density(width, height, 2.0),
        },
        MeshMaterial3d(glass_material.clone()),
        RigidBody::Static,
        CollisionEventsEnabled,
    ));

    // we want to shatter the glass as soon as it spawns
    // the glass needs to already exist in order to be shattered, so this needs to be done in a separate spawn
    glass.insert(Shattered);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(),
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            ..default()
        },
        Camera {
            hdr: true,
            order: 0,
            is_active: true,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Projection::from(PerspectiveProjection {
            fov: 80.0_f32.to_radians(),
            ..default()
        }),
    ));
}

// custom component added to the glass to trigger the display_shards hook
#[derive(Component)]
pub struct DisplayShards;

// makes the glass invisible and its shards visible, as well as having a dynamic rigid body
fn display_shards(
    _trigger: Trigger<OnAdd, DisplayShards>,
    glass: Single<(Entity, &Shards), With<Glass>>,
    mut commands: Commands,
) {
    // I already know there is only a single glass
    let (glass_entity, child_shards) = glass.into_inner();

    // hide the glass
    // remove the rigid body too otherwise the debug physics plugin makes this visible again for some reason
    commands
        .entity(glass_entity)
        .remove::<RigidBody>()
        .insert(Visibility::Hidden);

    for shard in child_shards.iter() {
        commands
            .entity(shard)
            .insert((Visibility::Visible, RigidBody::Dynamic));
    }
}

// makes shards invisible when spawned
fn hide_shards(trigger: Trigger<OnAdd, Shard>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(Visibility::Hidden);
}
