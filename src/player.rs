use std::f32::consts::TAU;

use bevy::gltf::Gltf;
use bevy::log;
use bevy::prelude::*;
use inline_tweak::tweak;

use crate::actions::Actions;
use crate::loading::GLTFAssets;
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Cuby;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_cube.in_schedule(OnEnter(GameState::Playing)))
            .add_system(rotate_cube.in_set(OnUpdate(GameState::Playing)));
    }
}

fn spawn_cube(mut commands: Commands, gltf_assets: Res<Assets<Gltf>>, gltf: Res<GLTFAssets>) {
    let root = gltf_assets.get(&gltf.cuby).unwrap();

    commands
        .spawn(SceneBundle {
            scene: root.named_scenes["Scene"].clone(),
            ..default()
        })
        .insert(Cuby);

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn rotate_cube(
    time: Res<Time>,
    actions: Res<Actions>,
    mut cube: Query<&mut Transform, With<Cuby>>,
    camera: Query<&Transform, (With<Camera>, Without<Cuby>)>,
) {
    let Some(rotation) = actions.player_rotation else { return };

    let speed = tweak!(0.4);

    log::debug!("rotating cube by {:?}", rotation * speed);

    let rpms = TAU * speed * time.delta_seconds();

    for camera_transform in &camera {
        let camera_y = camera_transform.local_y();
        let camera_x = camera_transform.local_x();

        for mut player_transform in &mut cube {
            player_transform.rotate_axis(camera_y, rotation.x * rpms);
            player_transform.rotate_axis(camera_x, rotation.y * rpms);
        }
    }
}
