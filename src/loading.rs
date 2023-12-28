use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::GameState;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu),
        )
        .configure_loading_state(
            LoadingStateConfig::new(GameState::Loading)
                .load_collection::<FontAssets>()
                .load_collection::<GLTFAssets>(),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/Suwannaphum-Regular.ttf")]
    pub suwannaphum: Handle<Font>,
}

impl FontAssets {
    pub fn text_style(&self) -> TextStyle {
        TextStyle {
            font: self.suwannaphum.clone(),
            font_size: 50.0,
            color: Color::rgb_u8(74, 49, 33),
        }
    }

    pub fn button_style(&self) -> TextStyle {
        TextStyle {
            color: Color::BEIGE,
            ..self.text_style()
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct GLTFAssets {
    #[asset(path = "models/cuby.gltf")]
    pub cuby: Handle<Gltf>,
}
