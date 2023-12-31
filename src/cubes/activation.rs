use bevy::log;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_mod_picking::events::{Click, Pointer};

use crate::GameState;

use super::{Block, BlockState};

pub struct ActivatePlugin;

impl Plugin for ActivatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToggleAnimation>()
            .add_event::<ToggleEvent>()
            .add_systems(
                Update,
                (
                    (
                        prepare_animations.after(super::spawn_cuby),
                        animate_toggled_blocks,
                    )
                        .chain(),
                    fire_toggle_timers,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                activate_selected_block.run_if(in_state(GameState::Playing)),
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

#[derive(Default, Resource)]
pub struct ToggleAnimation(Handle<AnimationClip>);

#[derive(Component, DerefMut, Deref)]
pub struct ToggleTimer(Timer);

impl Default for ToggleTimer {
    fn default() -> Self {
        let mut dummy_timer = Timer::from_seconds(0.0, TimerMode::Once);
        dummy_timer.pause();
        Self(dummy_timer)
    }
}

fn prepare_animations(
    mut commands: Commands,
    mut blocks: Query<Entity, Added<Block>>,
    mut toggle_anim: ResMut<ToggleAnimation>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    let block_name = Name::new("block");
    if clips.get(&toggle_anim.0).is_none() {
        const DISTANCE: f32 = 0.75;
        const DURATION: f32 = 0.3;

        let mut clip = AnimationClip::default();
        clip.add_curve_to_path(
            EntityPath {
                parts: vec![block_name.clone()],
            },
            VariableCurve {
                keyframe_timestamps: vec![0.0, DURATION],
                // Just animate going "forward" by one unit, KISS
                keyframes: Keyframes::Translation(vec![Vec3::ZERO, DISTANCE * -Vec3::Z]),
            },
        );

        log::info!("Built animation clips for blocks");

        toggle_anim.0 = clips.add(clip);
    };

    let handle = &toggle_anim.0;
    let clip = clips.get(handle).unwrap();

    for block in &mut blocks {
        log::debug!("starting animation for block {block:?}");

        let mut player = AnimationPlayer::default();
        player.start(handle.clone()).seek_to(clip.duration());

        commands.entity(block).insert((player, block_name.clone()));
    }
}

fn animate_toggled_blocks(
    mut blocks: Query<(Entity, &mut AnimationPlayer, &Block, &mut ToggleTimer), Changed<Block>>,
    toggle_anim: Res<ToggleAnimation>,
    clips: Res<Assets<AnimationClip>>,
) {
    let handle = &toggle_anim.0;
    let Some(clip) = clips.get(handle) else {
        log::debug!("animation clip is not ready yet");
        return;
    };

    for (ent, mut player, block, mut timer) in &mut blocks {
        let handle = handle.clone();

        let (duration, speed) = match block.state {
            BlockState::OutOfPlace => (clip.duration() - player.seek_time(), 1.0),
            BlockState::InPosition => (player.seek_time(), -1.0),
        };

        // TODO: the docs claim seek_time() should always be within [0, duration]
        // but it seems to be returning something higher here... Maybe should
        // file a Bevy bug about it eventually
        let cur_time = player.seek_time().clamp(0.0, clip.duration());

        // We need to restart the animation to get it playing again, which
        // can be done with start() or replay(). In either case, we have to seek
        // back to the original time, since the seek time gets reset.
        player.start(handle).seek_to(cur_time).set_speed(speed);

        log::debug!(
            "playing at speed {speed:?} on {ent:?} current: {cur_time:.2} remaining: {duration:.2}/{:.2}",
            clip.duration(),
        );

        timer.set_duration(Duration::from_secs_f32(duration.abs()));
        timer.reset();
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
