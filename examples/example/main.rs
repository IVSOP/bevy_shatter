mod character;
use avian3d::{PhysicsPlugins, prelude::*};
use character::*;
mod menu;
use menu::*;
use bevy::{prelude::*, window::{CursorGrabMode, PresentMode, WindowTheme}};

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
        ))
        .add_systems(
            Startup,
            (
                setup_scene,
            )
        ).add_systems(
            Update,
            (
                grab_mouse,
            )
        );

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
}
