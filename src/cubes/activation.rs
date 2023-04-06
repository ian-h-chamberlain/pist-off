use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::{PickingEvent, SelectionEvent};

use crate::{tweak, GameState};

use super::{Block, BlockState};

pub struct ActivatePlugin;

impl Plugin for ActivatePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
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

            log::info!("block {ent:?} toggled");

            block.state.toggle();
        }
    }
}

fn animate_toggled_blocks(timer: Res<Time>, mut blocks: Query<(&mut Transform, &Block)>) {
    for (mut transform, block) in &mut blocks {
        let destination = match block.state {
            BlockState::OutOfPlace => block.original_translation + block.out_direction,
            BlockState::InPosition => block.original_translation,
        };

        let speed = tweak!(2.0);

        // TODO: this is jank as fuck, we should try using bevy_animation keyframes probs
        // https://github.com/bevyengine/bevy/blob/main/examples/animation/animated_transform.rs
        let distance = transform.translation.distance(destination);

        if distance > tweak!(0.01) {
            transform.translation = transform
                .translation
                .lerp(destination, timer.delta_seconds() * speed / distance);
        }
    }
}
