use std::f32::consts::TAU;

use bevy::gltf::Gltf;
use bevy::log;
use bevy::prelude::*;

use crate::actions::Actions;
use crate::loading::GLTFAssets;
use crate::tweak;
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Cuby;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_scene.in_schedule(OnEnter(GameState::Playing)))
            .add_system(rotate_cube.in_set(OnUpdate(GameState::Playing)));
    }
}

fn spawn_scene(
    mut commands: Commands,
    camera: Query<(Entity, &Transform), With<Camera3d>>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf: Res<GLTFAssets>,
) {
    let root = gltf_assets.get(&gltf.cuby).unwrap();

    commands
        .spawn(SceneBundle {
            scene: root.named_scenes["Scene"].clone(),
            ..default()
        })
        .insert(Cuby);

    for (camera, camera_transform) in &camera {
        commands
            .entity(camera)
            // Without this, the child can't be considered visible either
            .insert(VisibilityBundle::default())
            .with_children(|parent| {
                // For a point light we don't care about scale/rotation
                let translation = Vec3::new(4.0, 8.0, 4.0) - camera_transform.translation;

                // Add the point light as a child of the camera, to give the illusion
                // we are rotating the cube relative to the scene, but actually we're
                // just moving/rotating the camera around the cube.
                parent.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 1500.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_translation(translation),
                    ..default()
                });
            });
    }
}

fn rotate_cube(
    time: Res<Time>,
    actions: Res<Actions>,
    cube: Query<&Transform, With<Cuby>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Cuby>)>,
) {
    let Some(rotation) = actions.player_rotation else { return };

    let speed = tweak!(0.4);

    log::debug!("rotating camera by {:?}", rotation * speed);

    let rpms = TAU * speed * time.delta_seconds();

    for cube_transform in &cube {
        for mut camera_transform in &mut camera {
            // TODO: option for inverting the arrow controls? Click+drag would
            // be much easier at the end of the day
            let rotation = Quat::from_axis_angle(camera_transform.local_y(), -rotation.x * rpms)
                * Quat::from_axis_angle(camera_transform.local_x(), rotation.y * rpms);

            // We could probably just rotate around the origin, but if the cube ever moves
            // this should handle it better I think
            camera_transform.rotate_around(cube_transform.translation, rotation);
        }
    }
}
