mod character;
use avian3d::{prelude::*, PhysicsPlugins};
use bevy_shatter::*;
use character::*;
mod menu;
use bevy::{
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
        CharacterPlugin,
        ShatterPlugin,
        AtmospherePlugin,
    ))
    .add_observer(dynamic_shards)
    .add_observer(hide_glass)
    .add_systems(Startup, setup_scene)
    .add_systems(FixedUpdate, shatter_on_contact);

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
    commands.spawn((
        AutoGlass {
            translation: Vec3::new(0.0, 3.0, -10.0),
            rotation: Quat::IDENTITY,
            glass: Glass::new_with_density(20.0, 5.0, 0.1, 2.0),
        },
        MeshMaterial3d(glass_material.clone()),
        RigidBody::Static,
        CollisionEventsEnabled, 
    ));
}

// hook to make shards have a dynamic rigid body when created
fn dynamic_shards(
    trigger: Trigger<OnAdd, Shard>,
    mut commands: Commands,
) {
    commands.entity(trigger.target()).insert(RigidBody::Dynamic);
}

// hook to hide the glass when it is shattered
fn hide_glass(
    trigger: Trigger<OnAdd, Shattered>,
    mut commands: Commands,
) {
    commands.entity(trigger.target()).insert(Visibility::Hidden);
}

// function to shatter glass when player collides with it
// this is very ugly, consider using https://idanarye.github.io/bevy-tnua/avian3d/collision/contact_types/struct.Collisions.html
fn shatter_on_contact(
    mut collision_event_reader: EventReader<CollisionStarted>,
    player: Single<Entity, With<Player>>,
    glasses: Populated<Entity, With<Glass>>,
    mut commands: Commands,
) {
    let player_entity = player.into_inner();
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        let entity1 = *entity1;
        let entity2 = *entity2;

        if entity1 == player_entity {
            if glasses.contains(entity2) {
                commands.entity(entity2).insert(Shattered);
            }
        } else if entity2 == player_entity {
            if glasses.contains(entity1) {
                commands.entity(entity1).insert(Shattered);
            }
        }
    }
}
