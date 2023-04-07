use std::collections::VecDeque;

use bevy::log;
use bevy::prelude::*;
use bevy::utils::HashSet;
use indextree::{Arena, NodeId};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;

use crate::GameState;

use super::Block;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PropagateTimer>()
            .add_system(propagate_block_toggles.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct EntityGraph {
    arena: Arena<Entity>,
    head: NodeId,
}

impl EntityGraph {
    fn random_from_entities(mut entities: Vec<Entity>) -> Self {
        let rng = &mut rand::thread_rng();
        entities.shuffle(rng);

        let mut arena = Arena::new();

        // TODO: we should probably attach these NodeIDs as components on the blocks
        // to keep track of stuff
        let head = arena.new_node(
            entities
                .pop()
                .expect("should have at least one entity to start"),
        );

        let mut current = head;
        let mut next = VecDeque::new();
        next.push_back(current);

        let max_children = 3;

        while !entities.is_empty() {
            for _ in 0..rng.gen_range(0..max_children) {
                if let Some(ent) = entities.pop() {
                    current.append_value(ent, &mut arena);
                } else {
                    break;
                }
            }

            let amount = rng.gen_range(0..max_children);
            for random_next in current.children(&arena).choose_multiple(rng, amount) {
                next.push_back(random_next);
            }

            if let Some(next) = next.pop_front() {
                current = next;
            }
        }

        log::debug!("built tree:\n{:?}", head.debug_pretty_print(&arena));

        Self { arena, head }
    }

    fn children(&self, block: &Entity) -> Vec<Entity> {
        // TODO:
        Vec::new()
    }
}

/// Spawn a random graph using the given entities
pub fn build_graph(In(blocks): In<Vec<Entity>>, mut commands: Commands) {
    // TODO: this perhaps could be a non-piped system that runs when blocks are Added
    commands.spawn(EntityGraph::random_from_entities(blocks));
}

#[derive(Resource, Deref, DerefMut)]
pub struct PropagateTimer(Timer);

impl Default for PropagateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Repeating))
    }
}

pub fn propagate_block_toggles(
    time: Res<Time>,
    mut timer: ResMut<PropagateTimer>,
    mut blocks: Query<(Entity, &mut Block)>,
    graph: Query<&EntityGraph>,
) {
    // TODO: figure out how to delay the propagation until animation plays or something?
    //
    // if !timer.tick(time.delta()).just_finished() {
    //     return;
    // }

    let graph = graph.single();

    let mut to_toggle = HashSet::new();

    for (entity, block) in &mut blocks {
        if block.is_added() || !block.is_changed() {
            continue;
        }

        for child in graph.children(&entity) {
            to_toggle.insert(child);
        }
    }

    // TODO: idk if this is a very efficient way to iterate...
    for entity in to_toggle {
        if let Ok((_, mut block)) = blocks.get_mut(entity) {
            log::info!("propagating toggle to {entity:?}");
            // TODO: this should only toggle from InPosition -> OutOfPlace
            block.state.toggle();
        } else {
            log::warn!("couldn't find {entity:?} to propagate toggle");
        }
    }
}
