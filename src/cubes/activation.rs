use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, Pointer};

use crate::GameState;

use super::{Block, BlockState};

pub struct ActivatePlugin;

impl Plugin for ActivatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationClips>()
            .add_event::<ToggleEvent>()
            .add_systems(
                PostUpdate,
                activate_selected_block.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    animate_toggled_blocks.after(super::spawn_cuby),
                    fire_toggle_timers,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn activate_selected_block(
    mut blocks: Query<&mut Block>,
    mut selected_events: EventReader<Pointer<Click>>,
) {
    for evt in selected_events.read() {
        let ent = evt.target;
        let Ok(mut block) = blocks.get_mut(ent) else {
            continue;
        };

        // TODO: probably don't allow toggling blocks "out of place", or at least reconsider it
        block.state.toggle();
        log::info!("block {ent:?} toggled to {:?}", block.state);
    }
}

#[derive(Default)]
pub struct AnimationClipHandle {
    pub handle: Handle<AnimationClip>,
    pub duration: f32,
}

#[derive(Default, Resource)]
pub struct AnimationClips {
    pub out_of_place: AnimationClipHandle,
    pub in_position: AnimationClipHandle,
}

pub fn prepare_animations(
    In((blocks, block_scale)): In<(Vec<Entity>, f32)>,
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut anim_clips: ResMut<AnimationClips>,
) -> Vec<Entity> {
    log::info!("setting up animations for {} blocks", blocks.len());

    let block_name = Name::new("block");

    let extrude_distance = 0.75 * block_scale;

    let mut clip = AnimationClip::default();
    clip.add_curve_to_path(
        EntityPath {
            parts: vec![block_name.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 0.3],
            // Just animate going "forward" by one unit, KISS
            keyframes: Keyframes::Translation(vec![Vec3::ZERO, extrude_distance * -Vec3::Z]),
        },
    );

    anim_clips.out_of_place = AnimationClipHandle {
        duration: clip.duration(),
        handle: animations.add(clip),
    };

    let mut clip = AnimationClip::default();
    clip.add_curve_to_path(
        EntityPath {
            parts: vec![block_name.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 0.3],
            // Just animate going "backward" by one unit, KISS
            keyframes: Keyframes::Translation(vec![extrude_distance * -Vec3::Z, Vec3::ZERO]),
        },
    );

    anim_clips.in_position = AnimationClipHandle {
        duration: clip.duration(),
        handle: animations.add(clip),
    };

    for &block in &blocks {
        log::debug!("building animation for block {block:?}");

        let mut player = AnimationPlayer::default();
        player.play(anim_clips.out_of_place.handle.clone());

        commands.entity(block).insert((player, block_name.clone()));
    }

    blocks
}

#[derive(Component, DerefMut, Deref)]
pub struct ToggleTimer(Timer);

impl Default for ToggleTimer {
    fn default() -> Self {
        let mut dummy_timer = Timer::from_seconds(0.0, TimerMode::Once);
        dummy_timer.pause();
        Self(dummy_timer)
    }
}

fn animate_toggled_blocks(
    mut blocks: Query<(Entity, &mut AnimationPlayer, Ref<Block>, &mut ToggleTimer), Changed<Block>>,
    clips: Res<AnimationClips>,
) {
    for (ent, mut player, block, mut timer) in &mut blocks {
        // blocks that just spawned already have their animation playing
        if block.is_added() {
            continue;
        }

        log::debug!("playing anim {:?} on block {ent:?}", block.state);

        let clip = match block.state {
            BlockState::OutOfPlace => &clips.out_of_place,
            BlockState::InPosition => &clips.in_position,
        };

        let elapsed = player.elapsed().min(clip.duration);
        player
            .play(clip.handle.clone())
            .seek_to(clip.duration - elapsed);

        let duration = clip.duration - player.elapsed();
        *timer = ToggleTimer(Timer::from_seconds(duration.max(0.0), TimerMode::Once));
    }
}

/// Indicates a block has finished its trajectory to the given state.
#[derive(Event)]
pub struct ToggleEvent {
    pub state: BlockState,
    pub block: Entity,
}

fn fire_toggle_timers(
    time: Res<Time>,
    mut blocks: Query<(Entity, &Block, &mut ToggleTimer)>,
    mut events: EventWriter<ToggleEvent>,
) {
    for (entity, block, mut timer) in &mut blocks {
        if timer.tick(time.delta()).just_finished() {
            events.send(ToggleEvent {
                state: block.state,
                block: entity,
            });
        }
    }
}
