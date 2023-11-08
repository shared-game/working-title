use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use working_title_core::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (handle_movement, handle_mouse, handle_other_input)
                    .run_if(in_state(GameState::FirstPerson)),
            )
            .add_systems(OnEnter(GameState::FirstPerson), grab_cursor)
            .add_systems(OnExit(GameState::FirstPerson), release_cursor);
    }
}

pub fn grab_cursor(mut query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = query.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
}

pub fn release_cursor(mut query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = query.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

pub fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            TransformBundle::from_transform(Transform::from_xyz(0.0, 1.0, 0.0)),
            RigidBody::Dynamic,
            Collider::capsule_y(0.5, 0.25),
            Velocity::zero(),
            ColliderMassProperties::Density(50.0),
            GravityScale(1.0),
            Sleeping::disabled(),
            Ccd::enabled(),
            ExternalImpulse::default(),
            LockedAxes::ROTATION_LOCKED,
            // Friction {
            //     coefficient: 0.0,
            //     combine_rule: CoefficientCombineRule::Min,
            // },
            // Damping {
            //     linear_damping: 0.0,
            //     angular_damping: 0.0,
            // },
            // Restitution {
            //     coefficient: 0.0,
            //     combine_rule: CoefficientCombineRule::Min,
            // },
            ReadMassProperties::default(),
            PlayerController::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 0.25, 0.0),
                    camera: Camera {
                        // is_active: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                PlayerCam,
            ));
        });
}

#[derive(Component, Default)]
pub struct PlayerController {
    last_target_velocity: Vec3,
    camera_rotation: Quat,
}

pub fn handle_movement(
    mut query: Query<(&mut PlayerController, &Velocity, &mut ExternalImpulse)>,
    camera_query: Query<&Transform, With<PlayerCam>>,
    input: Res<Input<KeyCode>>,
    ctx: Res<RapierContext>,
) {
    query.for_each_mut(|(mut controller, velocity, mut impulse)| {
        let mut movement_direction = Vec3::ZERO;

        let (forward, right) = if let Ok(transform) = camera_query.get_single() {
            let local_z = -transform.local_z();
            let forward = Vec3::new(local_z.x, 0.0, local_z.z).normalize();
            let right = Vec3::new(-local_z.z, 0.0, local_z.x).normalize();
            (forward, right)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };

        if input.pressed(KeyCode::W) {
            movement_direction += forward;
        }

        if input.pressed(KeyCode::S) {
            movement_direction -= forward;
        }

        if input.pressed(KeyCode::D) {
            movement_direction += right;
        }

        if input.pressed(KeyCode::A) {
            movement_direction -= right;
        }

        let max_speed = if input.pressed(KeyCode::ShiftLeft) {
            10.0
        } else {
            5.0
        };

        let accel = 50.0;
        let max_accel_force = 10.0;

        let target_velocity = max_speed * movement_direction;

        let target_velocity = Vec3::lerp(
            controller.last_target_velocity,
            target_velocity,
            f32::min(accel * ctx.integration_parameters.dt, 1.0),
        );

        let mut needed_accel =
            (target_velocity - velocity.linvel).clamp_length_max(max_accel_force);
        needed_accel.y = 0.0;

        impulse.impulse += needed_accel;

        if input.just_pressed(KeyCode::Space) {
            impulse.impulse += Vec3::Y * 50.0;
        }

        controller.last_target_velocity = target_velocity;
    })
}

#[derive(Component, Default)]
pub struct PlayerCam;

pub fn handle_mouse(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut player: Query<&mut PlayerController>,
    mut query: Query<(&mut Transform, &Parent), With<PlayerCam>>,
    mut mouse_evr: EventReader<MouseMotion>,
) {
    if let Ok(window) = window_query.get_single() {
        for ev in mouse_evr.read() {
            query.for_each_mut(|(mut transform, parent)| {
                if let Ok(mut player) = player.get_mut(parent.get()) {
                    let mouse_ratio = 0.00020;

                    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

                    let window_scale = f32::min(window.height(), window.width());

                    yaw -= (ev.delta.x * mouse_ratio * window_scale).to_radians();
                    pitch -= (ev.delta.y * mouse_ratio * window_scale).to_radians();

                    pitch = pitch.clamp(-1.54, 1.54);

                    let rotation =
                        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
                    transform.rotation = rotation;
                    player.camera_rotation = rotation;
                }
            });
        }
    }
}

pub fn handle_other_input(
    input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Tab) {
        next_state.set(GameState::MenuOpen)
    }
}