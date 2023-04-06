use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;

use crate::{tweak, GameState};

use super::{Block, BlockState};

pub struct ActivatePlugin;

impl Plugin for ActivatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationClips>()
            .add_system(
                activate_selected_block
                    .in_base_set(CoreSet::PostUpdate)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(animate_toggled_blocks.in_set(OnUpdate(GameState::Playing)));
    }
}

fn activate_selected_block(
    mut blocks: Query<&mut Block>,
    mut selected_events: EventReader<PickingEvent>,
) {
    for evt in selected_events.iter() {
        if let &PickingEvent::Clicked(ent) = evt {
            let mut block = blocks
                .get_mut(ent)
                .expect("only Blocks should be selectable");

            block.state.toggle();
            log::info!("block {ent:?} toggled to {:?}", block.state);
        }
    }
}

#[derive(Default, Resource)]
pub struct AnimationClips {
    pub out_of_place: Handle<AnimationClip>,
    pub in_position: Handle<AnimationClip>,
}

pub fn prepare_animations(
    In((blocks, block_scale)): In<(Vec<Entity>, f32)>,
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut anim_clips: ResMut<AnimationClips>,
) {
    log::info!("setting up animations for {} blocks", blocks.len());

    let block_name = Name::new("block");

    let extrude_distance = 0.75 * block_scale;

    let mut clip = AnimationClip::default();
    clip.add_curve_to_path(
        EntityPath {
            parts: vec![block_name.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 1.0],
            // Just animate going "forward" by one unit, KISS
            keyframes: Keyframes::Translation(vec![Vec3::ZERO, extrude_distance * -Vec3::Z]),
        },
    );

    anim_clips.out_of_place = animations.add(clip);

    let mut clip = AnimationClip::default();
    clip.add_curve_to_path(
        EntityPath {
            parts: vec![block_name.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 1.0],
            // Just animate going "backward" by one unit, KISS
            keyframes: Keyframes::Translation(vec![extrude_distance * -Vec3::Z, Vec3::ZERO]),
        },
    );

    anim_clips.in_position = animations.add(clip);

    for block in blocks {
        log::trace!("building animation for block {block:?}");

        let mut player = AnimationPlayer::default();
        player.play(anim_clips.out_of_place.clone());

        commands.entity(block).insert((player, block_name.clone()));
    }
}

fn animate_toggled_blocks(
    mut blocks: Query<(&mut AnimationPlayer, &Block, Ref<Block>), Changed<Block>>,
    clips: Res<AnimationClips>,
) {
    let anim_speed = tweak!(3.0);

    for (mut player, block, block_info) in &mut blocks {
        if block_info.is_added() {
            // just so we can tweak on the fly:
            player.set_speed(anim_speed);

            // we don't need to change any animations to start out
            continue;
        }

        let clip = match block.state {
            BlockState::OutOfPlace => clips.out_of_place.clone(),
            BlockState::InPosition => clips.in_position.clone(),
        };

        let elapsed = player.elapsed().min(1.0);
        player
            .play(clip)
            .set_elapsed(1.0 - elapsed)
            .set_speed(anim_speed);
    }
}
