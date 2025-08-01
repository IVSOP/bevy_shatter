use avian3d::{prelude::*, PhysicsPlugins};
use bevy_shatter::*;
mod menu;
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
    window::{PresentMode, WindowTheme},
};
use bevy_atmosphere::prelude::*;
use bevy_spectator::*;
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
        SpectatorPlugin,
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
    .add_observer(static_shards)
    .add_observer(hide_glass)
    .add_systems(Startup, (setup_scene, setup_camera))
    .add_systems(Update, (click_shatter, drop_shards));

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
    commands.spawn((
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
        Spectator,
    ));

    // crosshair
    commands
        .spawn((Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },))
        .with_child((
            Node {
                width: Val::Px(8.0),
                height: Val::Px(8.0),
                ..default()
            },
            ImageNode {
                color: Color::LinearRgba(LinearRgba::rgb(2.0, 0.0, 0.0)),
                image: Handle::<Image>::default(),
                ..default()
            },
        ));
}

// hook to make shards have a static rigid body when created
// I also make them have larger gravity for the effect to be more noticeable
fn static_shards(trigger: Trigger<OnAdd, Shard>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(RigidBody::Static)
        .insert(GravityScale(2.0));
}

// helper to store where the glass was hit, relative to the bottom left of the glass
#[derive(Component)]
pub struct GlassHitPoint(Vec2);

// make shards fall if close to the hit point
// this could be improved to make all shards fall at different times, depending on how far
// away they are, making a radial pattern
// but this would make the example even more complex than it already is
fn drop_shards(
    shattered_glasses: Populated<(&GlassHitPoint, &Shards), With<Glass>>,
    mut shards: Query<(&Shard, &mut RigidBody)>,
) {
    for (hit, shard_children) in shattered_glasses.iter() {
        for shard in shard_children.iter() {
            // the shard will obviously belong to the shards query
            let (shard_info, mut shard_body) = shards.get_mut(shard).unwrap();

            let distance = shard_info.pos.distance(hit.0);
            if distance < 2.0 {
                *shard_body = RigidBody::Dynamic;
            }
        }
    }
}

// hook to hide the glass when it is shattered
fn hide_glass(trigger: Trigger<OnAdd, Shattered>, mut commands: Commands) {
    // remove the rigid body too otherwise the debug physics plugin makes this visible again for some reason
    commands
        .entity(trigger.target())
        .remove::<RigidBody>()
        .insert(Visibility::Hidden);
}

fn click_shatter(
    camera: Single<&Transform, With<Spectator>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    glasses: Populated<(Entity, &Transform, &Glass), Without<Shattered>>,
    spatial_query: SpatialQuery,
) {
    if mouse.just_pressed(MouseButton::Right) {
        let cam_transform = camera.into_inner();
        let cam_forward = cam_transform.forward();
        // ray cast to check if intersecting a glass

        // TODO: use query filter instead of checking if it is a glass,
        // got lazy since I would have to add masks to all other entities in this example
        if let Some(hit) = spatial_query.cast_ray(
            cam_transform.translation,
            cam_forward,
            1000.0,
            false,
            &SpatialQueryFilter::default(),
        ) {
            if let Ok((glass_entity, glass_transf, glass)) = glasses.get(hit.entity) {
                let hit_position =
                    cam_transform.translation + (cam_forward.as_vec3() * hit.distance);

                let relative_break_pos = glass.project_to_glass(glass_transf, hit_position);

                commands
                    .entity(glass_entity)
                    .insert(Shattered)
                    .insert(GlassHitPoint(relative_break_pos));
            }
        }
    }
}
