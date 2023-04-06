mod activation;
mod highlight;

use bevy::gltf::Gltf;
use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::{CustomHighlightPlugin, DefaultPickingPlugins};
use rand::seq::SliceRandom;

use crate::cubes::highlight::HighlightableBundle;
use crate::loading::GLTFAssets;
use crate::GameState;

use self::activation::ActivatePlugin;
use self::highlight::HighlightPlugin;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPickingPlugins
                .build()
                // disable the default material based highlighting
                .disable::<CustomHighlightPlugin<StandardMaterial>>()
                .disable::<CustomHighlightPlugin<ColorMaterial>>(),
        )
        .add_plugin(ActivatePlugin)
        .add_plugin(HighlightPlugin)
        .add_system(spawn_cuby.in_schedule(OnEnter(GameState::Playing)));
    }
}

/// The "holding box" of the cube.
#[derive(Component)]
pub struct CubeFrame;

/// The interactable components of the cube.
#[derive(Component, Default, Debug)]
pub struct Block {
    /// Whether a block is in place or not.
    pub state: BlockState,
    /// Which way the block moves "forward" when it's out of place
    pub out_direction: Vec3,
    /// This is the "in-place" position of the block.
    pub original_translation: Vec3,
}

/// Whether a block is in its proper place or not.
#[derive(Debug)]
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
) {
    let root = gltf_assets.get(&gltf.cuby).unwrap();

    let cube = meshes.add(shape::Cube::default().into());

    // use a parent entity to make it simpler to scale down the inner cubes
    let middleman = commands
        .spawn(SpatialBundle {
            transform: Transform::from_scale(Vec3::splat(0.8)),
            ..default()
        })
        .with_children(|parent| spawn_blocks(parent, cube, &mut materials))
        .id();

    commands
        .spawn((
            CubeFrame,
            SceneBundle {
                scene: root.named_scenes["Scene"].clone(),
                ..default()
            },
            // TODO: it might look better to keep the outlines from rendering "above" the frame,
            // but I haven't figured out how to do it with bevy_mod_outline yet
        ))
        .add_child(middleman);
}

fn spawn_blocks(
    parent: &mut ChildBuilder,
    cube_mesh: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let num_cubes_per_axis = 2_i16;

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

                let original_translation = Vec3::new(x_pos, y_pos, z_pos);

                let color: Color = ALL_COLORS[color_idx % ALL_COLORS.len()];
                color_idx += 1;

                // TODO: spawn "in-place" for any cubes that aren't on
                // an outside edge (and make them unselectable, I guess?)
                // this sort of works already, but disabling highlight and making them
                // explicitly "fixed" probably makes more sense
                let state = if axes.is_empty() {
                    BlockState::InPosition
                } else {
                    BlockState::OutOfPlace
                };

                let out_direction = axes
                    .choose(&mut rand::thread_rng())
                    .copied()
                    .unwrap_or_default();

                let block = Block {
                    state,
                    out_direction,
                    original_translation,
                };

                log::info!("spawning block {block:?} at {:?}", (x_pos, y_pos, z_pos));

                parent.spawn((
                    block,
                    MaterialMeshBundle {
                        mesh: cube_mesh.clone(),
                        // TODO: reuse color materials maybe?
                        material: materials.add(color.into()),
                        transform: Transform::from_translation(original_translation)
                            // slightly smaller than 100% looks slightly nicer
                            .with_scale(Vec3::splat(0.95 * cube_scale)),
                        ..default()
                    },
                    HighlightableBundle::default(),
                ));
            }
        }
    }
}

fn gen_combinations(cubes_per_axis: i16) -> impl Iterator<Item = i16> {
    (1..=cubes_per_axis).map(|i| -i).chain(1..=cubes_per_axis)
}
