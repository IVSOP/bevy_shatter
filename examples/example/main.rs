mod character;
use avian3d::{prelude::*, PhysicsPlugins};
use bevy_shatter::*;
use character::*;
mod menu;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PresentMode, WindowTheme},
};
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
    ))
    .add_systems(Startup, (setup_scene,))
    .add_systems(Update, (grab_mouse,));

    app.run();
}

// grab on left click, release on escape
fn grab_mouse(
    mut window: Single<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
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

    // made by grok to mimic old system shock 2
    let glass_material = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::new(0.1, 0.5, 0.5, 0.3)), // Cyan tint with alpha for transparency
        alpha_mode: AlphaMode::Blend,                                       // Enable transparency
        metallic: 0.0,                                                      // Glass is not metallic
        reflectance: 0.1, // Slight reflectance for a subtle sheen
        emissive: LinearRgba::rgb(0.0, 0.1, 0.1),
        // cull_mode: None, // Render both sides of the glass
        ..default()
    });

    // glass
    commands.spawn((
        AutoGlass {
            translation: Vec3::new(10.0, 3.0, 0.0),
            rotation: Quat::IDENTITY,
            glass: Glass {
                width: 20.0,
                height: 5.0,
                thickness: 0.1,
                shatter_config: ShatterConfig::Density {
                    cells_per_unit: 5.0,
                },
            },
        },
        MeshMaterial3d(glass_material.clone()),
    ));
}
