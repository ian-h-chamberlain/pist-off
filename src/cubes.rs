use bevy::gltf::Gltf;
use bevy::log;
use bevy::prelude::*;

use crate::loading::GLTFAssets;
use crate::GameState;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            spawn_cube.in_schedule(OnEnter(GameState::Playing)),
            // .in_base_set(CoreSet::PreUpdate),
        );
    }
}

/// The "holding box" of the cube.
#[derive(Component)]
pub struct CubeFrame;

/// The interactable components of the cube.
#[derive(Component)]
pub struct Block;

const ALL_COLORS: &[Color] = &[
    Color::ALICE_BLUE,
    Color::ANTIQUE_WHITE,
    Color::AQUAMARINE,
    Color::AZURE,
    Color::BEIGE,
    Color::BISQUE,
    Color::BLUE,
    Color::CRIMSON,
    Color::CYAN,
    Color::DARK_GREEN,
    Color::FUCHSIA,
    Color::GOLD,
    Color::GREEN,
    Color::INDIGO,
    Color::LIME_GREEN,
    Color::MAROON,
    Color::MIDNIGHT_BLUE,
    Color::NAVY,
    Color::OLIVE,
    Color::ORANGE,
    Color::ORANGE_RED,
    Color::PINK,
    Color::PURPLE,
    Color::RED,
    Color::SALMON,
    Color::SEA_GREEN,
    Color::SILVER,
    Color::TEAL,
    Color::TOMATO,
    Color::TURQUOISE,
    Color::VIOLET,
    Color::YELLOW,
    Color::YELLOW_GREEN,
];

fn spawn_cube(
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    gltf: Res<GLTFAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let root = gltf_assets.get(&gltf.cuby).unwrap();

    let cube = meshes.add(shape::Cube::default().into());

    // use a parent entity to make it simpler to scale down the inner cubes
    let middleman = commands
        .spawn(SpatialBundle {
            transform: Transform::from_scale(Vec3::splat(0.8)),
            ..default()
        })
        .with_children(|parent| {
            let num_cubes_per_axis = 1_i16;
            let cube_scale = 1.0 / f32::from(num_cubes_per_axis);

            let mut color_idx: usize = rand::random();

            for i in gen_combinations(num_cubes_per_axis) {
                for j in gen_combinations(num_cubes_per_axis) {
                    for k in gen_combinations(num_cubes_per_axis) {
                        let x_pos = i * cube_scale;
                        let y_pos = j * cube_scale;
                        let z_pos = k * cube_scale;

                        let color: Color = ALL_COLORS[color_idx % ALL_COLORS.len()];
                        color_idx += 1;

                        log::debug!("spawning at {:?}", (x_pos, y_pos, z_pos));

                        parent.spawn((
                            Block,
                            MaterialMeshBundle {
                                mesh: cube.clone(),
                                // TODO: reuse color materials maybe?
                                material: materials.add(color.into()),
                                transform: Transform::from_xyz(x_pos, y_pos, z_pos)
                                    // slightly smaller than 100% in lieu of a proper border
                                    .with_scale(Vec3::splat(0.95 * cube_scale)),
                                ..default()
                            },
                        ));
                    }
                }
            }
        })
        .id();

    commands
        .spawn((
            CubeFrame,
            SceneBundle {
                scene: root.named_scenes["Scene"].clone(),
                ..default()
            },
        ))
        .add_child(middleman);
}

fn gen_combinations(cubes_per_axis: i16) -> impl Iterator<Item = f32> {
    (1..=cubes_per_axis)
        .map(|i| -i)
        .chain(1..=cubes_per_axis)
        .map(|i| f32::from(i) - 0.5 * f32::from(i.signum()))
}
