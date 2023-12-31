use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use working_title_core::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ObjPlugin)
            .add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    handle_movement_input,
                    handle_mouse,
                    handle_other_input,
                    handle_shooting,
                )
                    .run_if(in_state(GameState::FirstPerson)),
            )
            .add_systems(Update, bullet_collision)
            .add_systems(PreUpdate, reset_movement_input)
            .add_systems(PostUpdate, handle_movement_physics)
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

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
            InheritedVisibility::VISIBLE,
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Camera3dBundle {
                        transform: Transform::from_xyz(0.0, 0.25, 0.0),
                        camera: Camera {
                            // is_active: false,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    PlayerCam,
                    InheritedVisibility::VISIBLE,
                ))
                .with_children(|builder| {
                    builder.spawn(PbrBundle {
                        transform: Transform::from_xyz(0.25, -0.15, -0.4)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                        mesh: asset_server.load("models/mosin.gltf#Mesh0/Primitive0"),
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(asset_server.load("models/2_1.BMP")),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                });
        });
}

#[derive(Component, Default)]
pub struct PlayerController {
    movement_direction: Vec3,
    jumping: bool,
    last_target_velocity: Vec3,
    camera_rotation: Quat,
}

pub fn reset_movement_input(mut query: Query<&mut PlayerController>) {
    query.for_each_mut(|mut controller| {
        controller.movement_direction = Vec3::ZERO;
        controller.jumping = false;
    });
}

pub fn handle_movement_input(
    mut query: Query<&mut PlayerController>,
    camera_query: Query<&Transform, With<PlayerCam>>,
    input: Res<Input<KeyCode>>,
) {
    query.for_each_mut(|mut controller| {
        let (forward, right) = if let Ok(transform) = camera_query.get_single() {
            let local_z = -transform.local_z();
            let forward = Vec3::new(local_z.x, 0.0, local_z.z).normalize();
            let right = Vec3::new(-local_z.z, 0.0, local_z.x).normalize();
            (forward, right)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };

        if input.pressed(KeyCode::W) {
            controller.movement_direction += forward;
        }

        if input.pressed(KeyCode::S) {
            controller.movement_direction -= forward;
        }

        if input.pressed(KeyCode::D) {
            controller.movement_direction += right;
        }

        if input.pressed(KeyCode::A) {
            controller.movement_direction -= right;
        }

        if input.just_pressed(KeyCode::Space) {
            controller.jumping = true;
        }
    })
}

fn handle_movement_physics(
    mut query: Query<(
        Entity,
        &mut PlayerController,
        &GlobalTransform,
        &Velocity,
        &mut ExternalImpulse,
    )>,
    ctx: Res<RapierContext>,
    mut collisions: Local<Vec<(Entity, Toi)>>,
) {
    query.for_each_mut(|(entity, mut controller, global, velocity, mut impulse)| {
        let max_speed = 5.0;

        let accel = 50.0;
        let max_accel_force = 10.0;

        let target_velocity = max_speed * controller.movement_direction;

        let target_velocity = Vec3::lerp(
            controller.last_target_velocity,
            target_velocity,
            f32::min(accel * ctx.integration_parameters.dt, 1.0),
        );

        let mut needed_accel =
            (target_velocity - velocity.linvel).clamp_length_max(max_accel_force);
        needed_accel.y = 0.0;

        impulse.impulse += needed_accel;

        let ground_cast = {
            intersections(
                &ctx,
                ShapeDesc {
                    shape_pos: global.transform_point(Vec3::ZERO),
                    shape_rot: global.to_scale_rotation_translation().1,
                    shape_vel: -Vec3::Y,
                    shape: &Collider::ball(0.45),
                },
                1.0,
                QueryFilter::new()
                    .exclude_sensors()
                    .predicate(&|collider| collider != entity),
                &mut collisions,
            );

            collisions.iter().find(|(_, i)| {
                i.status != TOIStatus::Penetrating
                    && i.details
                        .map(|det| {
                            det.normal1.angle_between(Vec3::Y)
                                <= (45.0 * (std::f32::consts::PI / 180.0))
                        })
                        .unwrap_or(true)
            })
        };

        let float_offset = ground_cast.map(|(_, toi)| toi.toi - 0.55);

        let grounded = float_offset
            .map(|offset| (-0.3..=0.05).contains(&offset))
            .unwrap_or(false);

        if grounded && controller.jumping {
            impulse.impulse += Vec3::Y * 100.0;
        }

        controller.last_target_velocity = target_velocity;
    });
}

fn handle_shooting(
    mouse: Res<Input<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCam>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_transform = camera_query.get_single().unwrap();

    let mut transform = camera_transform.compute_transform();

    let forward = transform.forward();

    transform.translation += forward;

    if mouse.just_pressed(MouseButton::Left) {
        commands.spawn((
            Bullet,
            RigidBody::Dynamic,
            Collider::ball(0.025),
            Velocity::linear(forward * 200.0),
            Ccd::enabled(),
            PbrBundle {
                transform,
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: 0.025,
                        ..Default::default()
                    }
                    .into(),
                ),
                material: materials.add(Color::RED.into()),
                ..Default::default()
            },
        ));
    }
}

#[derive(Component)]
struct Bullet;

fn bullet_collision(
    query: Query<Entity, With<Bullet>>,
    ctx: Res<RapierContext>,
    mut commands: Commands,
) {
    query.for_each(|entity| {
        if ctx.contacts_with(entity).next().is_some() {
            commands.entity(entity).despawn();
        }
    });
}

struct ShapeDesc<'a> {
    shape_pos: Vec3,
    shape_rot: Quat,
    shape_vel: Vec3,
    shape: &'a Collider,
}

fn intersections(
    ctx: &RapierContext,
    shape: ShapeDesc,
    max_toi: f32,
    filter: QueryFilter,
    collisions: &mut Vec<(Entity, Toi)>,
) {
    collisions.clear();

    let predicate = filter.predicate;

    loop {
        let predicate = |entity| {
            !collisions.iter().any(|&(e, _)| e == entity)
                && predicate.map(|pred| pred(entity)).unwrap_or(true)
        };

        let filter = filter.predicate(&predicate);

        let ShapeDesc {
            shape_pos,
            shape_rot,
            shape_vel,
            shape,
        } = shape;

        if let Some(collision) = ctx.cast_shape(
            shape_pos, shape_rot, shape_vel, shape, max_toi, true, filter,
        ) {
            collisions.push(collision);
        } else {
            break;
        }
    }
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
