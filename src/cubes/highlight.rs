use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::ui::FocusPolicy;
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlineBundle, OutlinePlugin, OutlineStencil, OutlineVolume,
};
use bevy_mod_picking::highlight::{GlobalHighlight, HighlightPlugin as PickingHighlightPlugin};
use bevy_mod_picking::picking_core::Pickable;
use bevy_mod_picking::PickableBundle;

use crate::GameState;

use super::activation::ToggleTimer;
use super::{Block, BlockState};

pub struct HighlightPlugin;

#[derive(Debug, TypeUuid, Asset, TypePath)]
#[uuid = "cb9ba865-75f8-49bc-8cc0-3660f3f0717c"]
pub enum Highlight {
    Hovered,
    Pressed,
    Selected,
}

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Highlight>()
            .add_plugins((
                PickingHighlightPlugin::<Highlight> {
                    highlighting_default: |mut assets| GlobalHighlight {
                        hovered: assets.add(Highlight::Hovered),
                        pressed: assets.add(Highlight::Pressed),
                        selected: assets.add(Highlight::Selected),
                    },
                },
                AutoGenerateOutlineNormalsPlugin,
                OutlinePlugin,
            ))
            .add_systems(
                Update,
                set_highlighted_outlines.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Bundle)]
pub struct HighlightableBundle {
    pub pickable: PickableBundle,
    pub outline: OutlineBundle,
    pub highlight_type: Handle<Highlight>,
}

impl Default for HighlightableBundle {
    fn default() -> Self {
        Self {
            outline: OutlineBundle {
                outline: OutlineVolume {
                    width: 5.0,
                    ..default()
                },
                stencil: OutlineStencil {
                    enabled: true,
                    ..default()
                },
                ..default()
            },
            pickable: default(),
            highlight_type: default(),
        }
    }
}

#[derive(Bundle)]
pub struct UnpickableBundle {
    // TODO: it would be nice to keep the outlines from rendering "above" the GLTF
    // frame, but I haven't figured out how to do it with bevy_mod_outline yet.
    // OutlineBundle doesn't seem to be quite enough
    pub outline: OutlineBundle,
    pub pickable: Pickable,
    pub focus_policy: FocusPolicy,
    pub interaction: Interaction,
}

impl Default for UnpickableBundle {
    fn default() -> Self {
        Self {
            outline: default(),
            pickable: default(),
            interaction: default(),
            focus_policy: FocusPolicy::Block,
        }
    }
}

fn set_highlighted_outlines(
    assets: Res<Assets<Highlight>>,
    mut outlinables: Query<(&Handle<Highlight>, &mut OutlineVolume, &Block, &ToggleTimer)>,
) {
    for (highlight, mut outline, block, timer) in &mut outlinables {
        match assets.get(highlight) {
            Some(Highlight::Selected | Highlight::Hovered)
                if !timer.paused() && !timer.finished() =>
            {
                outline.colour = Color::BLUE;
                outline.visible = true;
            }
            Some(Highlight::Hovered) => {
                outline.colour = match block.state {
                    BlockState::OutOfPlace => Color::RED,
                    BlockState::InPosition => Color::GREEN,
                };
                outline.visible = true;
            }
            Some(Highlight::Pressed) => {
                outline.colour = Color::BLUE;
                outline.visible = true;
            }
            _ => {
                outline.visible = false;
            }
        }
    }
}
