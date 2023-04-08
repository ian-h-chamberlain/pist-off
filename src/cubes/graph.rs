use std::collections::{BTreeMap, VecDeque};

use bevy::log;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use indextree::{Arena, NodeId};
use rand::distributions::Slice;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;

use crate::GameState;

use super::{Block, BlockState};

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
    head: Entity,
    nodes: HashMap<Entity, NodeId>,
}

impl EntityGraph {
    fn random_from_entities(entities: Vec<Entity>) -> Self {
        let rng = &mut rand::thread_rng();

        let mut arena = Arena::new();

        let nodes: HashMap<Entity, NodeId> = entities
            .iter()
            .map(|&ent| (ent, arena.new_node(ent)))
            .collect();

        let avg_children = 3;
        let digraph = graphalgs::generate::random_digraph(
            entities.len(),
            entities.len() * (entities.len() - 1) / avg_children,
        )
        .expect("should always be a valid number of edges");

        for (parent, child) in digraph {
            let child = &entities[child];
            let parent = &entities[parent];

            if let Err(e1) = nodes[parent].checked_append(nodes[child], &mut arena) {
                if let Err(e2) = nodes[child].checked_append(nodes[parent], &mut arena) {
                    log::warn!("failed to create any edge {child:?} <-> {parent:?}: {e1}, {e2}");
                }
            }
        }

        debug_assert!(
            arena.count() == entities.len(),
            "failed to add some blocks to the graph",
        );
        #[cfg(not(debug_assertions))]
        if arena.count() != entities.len() {
            log::error!("failed to add some blocks to the graph! this puzzle could be unsolvable");
        }

        // there might be a better way to find the root node, but this ought to work I think??
        let head_node = nodes[&entities[0]].ancestors(&arena).last().unwrap();
        log::debug!("built tree:\n{:?}", head_node.debug_pretty_print(&arena));

        let head = *arena.get(head_node).unwrap().get();

        Self { arena, head, nodes }
    }

    fn children(&self, block: Entity) -> Vec<Entity> {
        let Some(node) = self.nodes.get(&block) else { return Vec::new() };

        node.children(&self.arena)
            .map(|node| self.arena.get(node).unwrap().get())
            .copied()
            .collect()
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
    // TODO: want to delay the propagation until animation plays or something?
    // it's actually fairly reasonable that it animates immediately at the moment.
    // cascading effect might be nicer with delays though
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

        // moving a block out of place doesn't affect anything
        if block.state == BlockState::InPosition {
            for child in graph.children(entity) {
                to_toggle.insert(child);
            }
        }
    }

    // idk if this is a very efficient way to iterate, but it seems to work okay.
    for entity in to_toggle {
        if let Ok((_, mut block)) = blocks.get_mut(entity) {
            if block.state == BlockState::InPosition {
                log::info!("propagating toggle to {entity:?}");
                block.state.toggle();
            }
        } else {
            log::warn!("couldn't find {entity:?} to propagate toggle");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This isn't comprehensive, but hopefully gives a sense of whether the graph
    // generation is likely to fail in general.
    #[test]
    fn generate_tree() {
        // it's random, so let's do it a lot
        for _ in 0..100 {
            for entity_count in 8..120 {
                let mut app = App::new();

                let ents: Vec<Entity> = (0..entity_count)
                    .map(|_| app.world.spawn_empty().id())
                    .collect();

                let _ = EntityGraph::random_from_entities(ents);
            }
        }
    }
}
