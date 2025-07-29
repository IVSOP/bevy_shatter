// this is a simple module for managing the character and controls, so I can have a more interactive example
// tnua is used as the character controller

use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    input::mouse::AccumulatedMouseMotion,
    math::*,
    prelude::*,
};

use avian3d::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

#[derive(Debug, Component)]
pub struct Player {
    pub mov: Vec3,
    pub look_dir: Dir3, // the player's body moves behind the actual camera a bit
    pub jumping: bool,
    pub sprinting: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            mov: Vec3::ZERO,
            look_dir: Dir3::NEG_Z,
            jumping: false,
            sprinting: false,
        }
    }
}

const CHARACTER_HEIGHT: f32 = 1.75;
const CHARACTER_RADIUS: f32 = 0.5;

#[derive(Component)]
pub struct PlayerCamera;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // physics run on FixedPostUpdate
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_systems(
            Startup,
            (
                spawn,
                snap_camera, // run this once at the start to make sure it is in the correct place
            )
                .chain(),
        )
        .add_systems(Update, (accumulate_inputs, snap_camera, rotate_camera))
        .add_systems(
            Update,
            (
                character_controller,
                // these only work if camera does not have the FreeCam component
            )
                .in_set(TnuaUserControlsSystemSet),
        );
    }
}

fn spawn(mut commands: Commands) {
    let transform = Transform {
        // translation: Vec3::new(0.0, 2.0, 14.0),
        translation: Vec3::new(0.0, 3.0, -10.0),
        rotation: Quat::from_rotation_y(-90.0_f32.to_radians()),
        ..default()
    };

    commands.spawn((
        transform,
        Player::default(),
        TransformInterpolation,
        RigidBody::Dynamic,
        Collider::capsule(CHARACTER_RADIUS, CHARACTER_HEIGHT),
        TnuaController::default(),
    ));

    commands.spawn((
        PlayerCamera,
        // transform,
        Transform {
            // will get snapped to the player's position
            ..default()
        },
        Camera3d::default(),
        Camera {
            hdr: true,
            order: 0,
            is_active: true,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Projection::from(PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            ..default()
        }),
    ));
}

/// stores what movement to execute the next time the movement system runs
fn accumulate_inputs(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Populated<&mut Player, Without<PlayerCamera>>, // Single???????
    camera: Single<&Transform, With<PlayerCamera>>,
) {
    let transform = camera.into_inner();
    let mut player = player.single_mut().unwrap();

    player.mov = Vec3::ZERO;

    let front: Vec3 = (transform.forward().as_vec3() * Vec3::new(1.0, 0.0, 1.0)).normalize();
    let right: Vec3 = (transform.right().as_vec3() * Vec3::new(1.0, 0.0, 1.0)).normalize();
    // let up: Vec3 = Vec3::new(0.0, 1.0, 0.0); // transform.up().as_vec3();

    if input.pressed(KeyCode::KeyW) {
        player.mov += front;
    }
    if input.pressed(KeyCode::KeyS) {
        player.mov -= front;
    }
    if input.pressed(KeyCode::KeyD) {
        player.mov += right;
    }
    if input.pressed(KeyCode::KeyA) {
        player.mov -= right;
    }
    if input.pressed(KeyCode::Space) {
        player.jumping = true;
    } else {
        player.jumping = false;
    }
    if input.pressed(KeyCode::ShiftLeft) {
        player.sprinting = true;
    } else {
        player.sprinting = false;
    }

    player.mov = player.mov.normalize_or_zero();
}

/// snaps the camera to the player's position, assumes it is already interpolated
fn snap_camera(
    player: Single<&Transform, (With<Player>, Without<PlayerCamera>)>,
    camera: Single<&mut Transform, (Without<Player>, With<PlayerCamera>)>,
) {
    let player_transform = player.into_inner();
    let mut cam_transform = camera.into_inner();
    cam_transform.translation = player_transform.translation;
}

/// rotates the camera and records the new look direction
fn rotate_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut cam_transform: Single<&mut Transform, With<PlayerCamera>>,
    mut player: Single<&mut Player>,
) {
    let camera_sensitivity = Vec2::splat(0.00052);
    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        // Note that we are not multiplying by delta_time here.
        // The reason is that for mouse movement, we already get the full movement that happened since the last frame.
        // This means that if we multiply by delta_time, we will get a smaller rotation than intended by the user.
        // This situation is reversed when reading e.g. analog input from a gamepad however, where the same rules
        // as for keyboard input apply. Such an input should be multiplied by delta_time to get the intended rotation
        // independent of the framerate.
        let delta_yaw = -delta.x * camera_sensitivity.x;
        let delta_pitch = -delta.y * camera_sensitivity.y;

        let (yaw, pitch, roll) = cam_transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        // If the pitch was ±¹⁄₂ π, the camera would look straight up or down.
        // When the user wants to move the camera back to the horizon, which way should the camera face?
        // The camera has no way of knowing what direction was "forward" before landing in that extreme position,
        // so the direction picked will for all intents and purposes be arbitrary.
        // Another issue is that for mathematical reasons, the yaw will effectively be flipped when the pitch is at the extremes.
        // To not run into these issues, we clamp the pitch to a safe range.
        const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        // for now the player does not pitch
        // transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, roll);
        cam_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

        // TODO: make this have faster math. I made this out of lazyness and since I didn't want the body to pitch
        let mut t = cam_transform.clone();
        t.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, roll);
        player.look_dir = t.forward();
    }
}

pub fn character_controller(player: Single<(&mut TnuaController, &Player)>) {
    let (mut controller, player) = player.into_inner();

    let speed = if player.sprinting { 10.0 } else { 4.5 };

    controller.basis(TnuaBuiltinWalk {
        // Move in the direction the player entered, at a speed of 10.0:
        desired_velocity: player.mov * speed,

        // Turn the character in the movement direction:
        desired_forward: Some(player.look_dir),

        // Must be larger than the height of the entity's center from the bottom of its
        // collider, or else the character will not float and Tnua will not work properly:
        float_height: 2.0,

        // TnuaBuiltinWalk has many other fields that can be configured:
        ..Default::default()
    });

    if player.jumping {
        // The jump action must be fed as long as the player holds the button.
        controller.action(TnuaBuiltinJump {
            // The full height of the jump, if the player does not release the button:
            height: 4.0,

            // TnuaBuiltinJump too has other fields that can be configured:
            ..Default::default()
        });
    }
}
