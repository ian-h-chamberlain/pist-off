use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::ui::FocusPolicy;
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlineBundle, OutlinePlugin, OutlineStencil, OutlineVolume,
};
use bevy_mod_picking::{CustomHighlightPlugin, DefaultHighlighting, PickableBundle, PickableMesh};

use crate::GameState;

pub struct HighlightPlugin;

#[derive(Debug, TypeUuid)]
#[uuid = "cb9ba865-75f8-49bc-8cc0-3660f3f0717c"]
pub enum Highlight {
    Hovered,
    Pressed,
    Selected,
}

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Highlight>()
            .add_plugin(CustomHighlightPlugin::<Highlight> {
                highlighting_default: |mut assets| DefaultHighlighting {
                    hovered: assets.add(Highlight::Hovered),
                    pressed: assets.add(Highlight::Pressed),
                    selected: assets.add(Highlight::Selected),
                },
            })
            .add_plugin(AutoGenerateOutlineNormalsPlugin)
            .add_plugin(OutlinePlugin)
            .add_system(set_highlighted_outlines.in_set(OnUpdate(GameState::Playing)));
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
                    // Try to fix some weird artifacting that only shows up on web.
                    // this doesn't quite fix it but helps a little I guess
                    #[cfg(target_family = "wasm")]
                    offset: 3.0,
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
    // TODO: picking doesn't seem like it quite works for GLTF:
    // https://github.com/aevyrie/bevy_mod_picking/issues/108
    pub pickable_mesh: PickableMesh,
    pub focus_policy: FocusPolicy,
    pub interaction: Interaction,
}

impl Default for UnpickableBundle {
    fn default() -> Self {
        Self {
            outline: default(),
            pickable_mesh: default(),
            interaction: default(),
            focus_policy: FocusPolicy::Block,
        }
    }
}

fn set_highlighted_outlines(
    assets: Res<Assets<Highlight>>,
    mut outlinables: Query<(&Handle<Highlight>, &mut OutlineVolume)>,
) {
    for (highlight, mut outline) in &mut outlinables {
        match assets.get(highlight) {
            Some(Highlight::Hovered) => {
                outline.colour = Color::GREEN;
                outline.visible = true;
            }
            Some(Highlight::Pressed) => {
                outline.colour = Color::RED;
                outline.visible = true;
            }
            _ => {
                outline.visible = false;
            }
        }
    }
}
