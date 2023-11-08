use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use working_title_core::CorePlugin;
use working_title_menu::MenuPlugin;
use working_title_player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            CorePlugin,
            MenuPlugin,
            PlayerPlugin,
        ))
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup_test_world)
        .run();
}

fn setup_test_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_size = 200.1;
    let ground_height = 0.1;

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 1000.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            transform: Transform::from_xyz(0.0, -ground_height, 0.0),
            mesh: meshes.add(
                shape::Box::new(ground_size * 2.0, ground_height * 2.0, ground_size * 2.0).into(),
            ),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));

    let num = 8;
    let rad = 1.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
    let mut color = 0;
    let colors = [
        Color::hsl(220.0, 1.0, 0.3),
        Color::hsl(180.0, 1.0, 0.3),
        Color::hsl(260.0, 1.0, 0.7),
    ];

    for j in 0usize..20 {
        for i in 0..num {
            for k in 0usize..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery + 3.0;
                let z = k as f32 * shift - centerz + offset;
                color += 1;

                commands.spawn((
                    PbrBundle {
                        transform: Transform::from_xyz(x, y, z)
                            .with_rotation(Quat::from_rotation_x(0.2)),
                        mesh: meshes.add(shape::Box::new(rad * 2.0, rad * 2.0, rad * 2.0).into()),
                        material: materials.add(colors[color % 3].into()),
                        ..Default::default()
                    },
                    RigidBody::Dynamic,
                    ColliderMassProperties::Density(50.0),
                    Collider::cuboid(rad, rad, rad),
                ));
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }
}
