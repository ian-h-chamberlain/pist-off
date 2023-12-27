use bevy::log;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use indextree::{Arena, NodeId};

use crate::{tweak, GameState};

use super::activation::ToggleEvent;
use super::{Block, BlockState};

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            propagate_block_toggles.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct EntityGraph {
    arena: Arena<Entity>,
    nodes: HashMap<Entity, NodeId>,
}

impl EntityGraph {
    fn random_from_entities(entities: Vec<Entity>) -> Self {
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

        Self { arena, nodes }
    }

    fn children(&self, block: Entity) -> Vec<Entity> {
        let Some(node) = self.nodes.get(&block) else {
            return Vec::new();
        };
        node.children(&self.arena)
            .map(|node| self.arena.get(node).unwrap().get())
            .copied()
            .collect()
    }

    fn parent(&self, block: Entity) -> Option<Entity> {
        self.nodes
            .get(&block)
            .and_then(|node| {
                node.ancestors(&self.arena)
                    .nth(1)
                    .map(|node| self.arena.get(node).unwrap().get())
            })
            .copied()
    }
}

/// Spawn a random graph using the given entities
pub fn build_graph(In(blocks): In<Vec<Entity>>, mut commands: Commands) {
    // TODO: this perhaps could be a non-piped system that runs when blocks are Added
    commands.spawn(EntityGraph::random_from_entities(blocks));
}

#[derive(Debug, Copy, Clone)]
pub enum PropagateMode {
    /// Easier difficulty, since a mistake along the way is more likely to end up
    /// costing less.
    Children,
    /// Harder difficulty. A mistake could end up resetting most of the cube.
    Ancestors,
}

impl PropagateMode {
    fn new(hard_mode: bool) -> Self {
        if hard_mode {
            Self::Ancestors
        } else {
            Self::Children
        }
    }
}

pub fn propagate_block_toggles(
    mut events: ParamSet<(EventReader<ToggleEvent>, EventWriter<ToggleEvent>)>,
    mut blocks: Query<(Entity, &mut Block)>,
    graph: Query<&EntityGraph>,
) {
    let graph = graph.single();

    let mode = PropagateMode::new(tweak!(false));

    let mut to_toggle = HashSet::new();

    let mut event_reader = events.p0();

    for toggled in event_reader.read() {
        // moving a block out of place doesn't affect anything
        if toggled.state == BlockState::InPosition {
            let affected_blocks = match mode {
                PropagateMode::Children => graph.children(toggled.block),
                PropagateMode::Ancestors => graph.parent(toggled.block).into_iter().collect(),
            };

            for child in affected_blocks {
                to_toggle.insert(child);
            }
        }
    }

    if !to_toggle.is_empty() {
        log::debug!("propagating toggles to {to_toggle:?}");
    }

    // idk if this is a very efficient way to iterate, but it seems to work okay.
    for entity in to_toggle {
        if let Ok((_, mut block)) = blocks.get_mut(entity) {
            // only move blocks out of position, not into position
            if block.state == BlockState::InPosition {
                block.state.toggle();
            } else {
                // HACK: basically, we're recursing into this system again on
                // the next tick to force this block to get updated again. It
                // isn't really in position, but it will be treated like it is
                // and propagate to its parent/child blocks as well.
                events.p1().send(ToggleEvent {
                    state: BlockState::InPosition,
                    block: entity,
                });

                // It might work better to have an intermediate event type, that
                // only gets sent when
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
