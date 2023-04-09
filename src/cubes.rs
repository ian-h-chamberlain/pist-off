mod activation;
mod graph;
mod highlight;

use bevy::gltf::Gltf;
use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::{CustomHighlightPlugin, DefaultPickingPlugins};
use rand::seq::SliceRandom;

use crate::cubes::highlight::{HighlightableBundle, UnpickableBundle};
use crate::loading::GLTFAssets;
use crate::GameState;

use self::activation::{ActivatePlugin, ToggleTimer};
use self::graph::GraphPlugin;
use self::highlight::HighlightPlugin;

pub use self::activation::ToggleEvent;
pub use self::graph::EntityGraph;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockCount>()
            .add_plugins(
                DefaultPickingPlugins
                    .build()
                    // disable the default material based highlighting
                    .disable::<CustomHighlightPlugin<StandardMaterial>>()
                    .disable::<CustomHighlightPlugin<ColorMaterial>>(),
            )
            .add_plugin(ActivatePlugin)
            .add_plugin(GraphPlugin)
            .add_plugin(HighlightPlugin)
            .add_system(
                spawn_cuby
                    .pipe(activation::prepare_animations)
                    .pipe(graph::build_graph)
                    .in_base_set(CoreSet::PreUpdate)
                    .in_schedule(OnEnter(GameState::Playing)),
            );
    }
}

/// The "holding box" of the cube.
#[derive(Component)]
pub struct CubeFrame;

/// The number of blocks per axis to spawn into the cube.
// TODO: allow this to be a number of blocks per *side* instead, so we could start
// at 1 and work our way up.
#[derive(Resource)]
pub struct BlockCount(pub i16);

impl Default for BlockCount {
    fn default() -> Self {
        Self(1)
    }
}

/// The interactable components of the cube.
#[derive(Component, Default, Debug)]
pub struct Block {
    /// Whether a block is in place or not.
    pub state: BlockState,
    /// Which way the block moves "forward" when it's out of place
    pub out_direction: Vec3,
}

#[derive(Bundle, Default)]
struct BlockBundle {
    pub block: Block,
    pub toggle_timer: ToggleTimer,
    pub highlight: HighlightableBundle,
}

/// Whether a block is in its proper place or not.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockState {
    OutOfPlace,
    InPosition,
}

impl BlockState {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::OutOfPlace => Self::InPosition,
            Self::InPosition => Self::OutOfPlace,
        }
    }
}

impl Default for BlockState {
    fn default() -> Self {
        Self::OutOfPlace
    }
}

const ALL_COLORS: &[Color] = &[
    Color::ALICE_BLUE,
    Color::AQUAMARINE,
    Color::AZURE,
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
    Color::SEA_GREEN,
    Color::TEAL,
    Color::TOMATO,
    Color::TURQUOISE,
    Color::VIOLET,
    Color::YELLOW,
    Color::YELLOW_GREEN,
];

fn spawn_cuby(
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    gltf: Res<GLTFAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_count: Res<BlockCount>,
) -> (Vec<Entity>, f32) {
    let root = gltf_assets.get(&gltf.cuby).unwrap();

    let cube = meshes.add(shape::Cube::default().into());

    let mut blocks = Vec::new();
    let mut block_scale = 0.0;

    // use a parent entity to make it simpler to scale down the inner cubes
    let middleman = commands
        .spawn(SpatialBundle {
            transform: Transform::from_scale(Vec3::splat(0.8)),
            ..default()
        })
        .with_children(|parent| {
            (blocks, block_scale) = spawn_blocks(parent, cube, &mut materials, block_count);
        })
        .id();

    commands
        .spawn((
            CubeFrame,
            SceneBundle {
                scene: root.named_scenes["Scene"].clone(),
                ..default()
            },
            UnpickableBundle::default(),
        ))
        .add_child(middleman);

    (blocks, block_scale)
}

fn spawn_blocks(
    parent: &mut ChildBuilder,
    cube_mesh: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    block_count: Res<BlockCount>,
) -> (Vec<Entity>, f32) {
    let mut ids = Vec::new();

    let num_cubes_per_axis = block_count.0;

    let cube_scale = 1.0 / f32::from(num_cubes_per_axis);
    let mut color_idx: usize = rand::random();

    for i in gen_combinations(num_cubes_per_axis) {
        let i_f = f32::from(i);
        for j in gen_combinations(num_cubes_per_axis) {
            let j_f = f32::from(j);
            for k in gen_combinations(num_cubes_per_axis) {
                let k_f = f32::from(k);

                let mut axes = Vec::new();
                if i.abs() == num_cubes_per_axis {
                    axes.push(Vec3::X * i_f.signum());
                }
                if j.abs() == num_cubes_per_axis {
                    axes.push(Vec3::Y * j_f.signum());
                }
                if k.abs() == num_cubes_per_axis {
                    axes.push(Vec3::Z * k_f.signum());
                }

                let x_pos = (i_f - 0.5 * i_f.signum()) * cube_scale;
                let y_pos = (j_f - 0.5 * j_f.signum()) * cube_scale;
                let z_pos = (k_f - 0.5 * k_f.signum()) * cube_scale;

                let translation = Vec3::new(x_pos, y_pos, z_pos);

                let color: Color = ALL_COLORS[color_idx % ALL_COLORS.len()];
                color_idx += 1;

                let state = if axes.is_empty() {
                    BlockState::InPosition
                } else {
                    BlockState::OutOfPlace
                };

                let out_direction = axes
                    .choose(&mut rand::thread_rng())
                    .copied()
                    .unwrap_or(Vec3::Z);

                // https://github.com/bevyengine/bevy/pull/7817
                let up_direction = out_direction.any_orthonormal_vector();

                let block = Block {
                    state,
                    out_direction,
                };

                let transform = Transform::from_translation(translation)
                    .looking_to(out_direction, up_direction);

                log::debug!("spawning block {block:?} at {transform:?}");

                parent
                    .spawn(
                        // use an intermediate transform bundle so we keep the
                        // "origin" the same but can still animate the block itself
                        SpatialBundle {
                            transform,
                            ..default()
                        },
                    )
                    .with_children(|parent| {
                        let mut block_cmd = parent.spawn((MaterialMeshBundle {
                            mesh: cube_mesh.clone(),
                            // TODO: reuse color materials maybe?
                            material: materials.add(StandardMaterial {
                                metallic: 0.5,
                                reflectance: 0.75,
                                ..color.into()
                            }),
                            // slightly smaller than 100% looks a little nicer
                            transform: Transform::from_scale(Vec3::splat(0.95 * cube_scale)),
                            ..default()
                        },));

                        if axes.is_empty() {
                            block_cmd.insert(UnpickableBundle::default());
                        } else {
                            let block_id =
                                block_cmd.insert(BlockBundle { block, ..default() }).id();

                            ids.push(block_id);
                        }
                    });
            }
        }
    }

    (ids, cube_scale)
}

fn gen_combinations(cubes_per_axis: i16) -> impl Iterator<Item = i16> {
    (1..=cubes_per_axis).map(|i| -i).chain(1..=cubes_per_axis)
}
