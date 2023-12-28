#![allow(clippy::needless_pass_by_value)]

mod actions;
mod cubes;
mod level;
mod loading;
mod macros;
mod menu;
mod player;

use self::actions::ActionsPlugin;
use self::cubes::CubePlugin;
use self::loading::LoadingPlugin;
use self::menu::MenuPlugin;
use self::player::PlayerPlugin;

use bevy::app::App;
use bevy::prelude::*;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use level::LevelPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    /// Here the menu is drawn and waiting for player interaction
    Menu,
    /// During this State the actual game logic is executed
    Playing,
    /// The level is complete and being prepared for the next level.
    Reset,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            LevelPlugin,
            ActionsPlugin,
            CubePlugin,
            PlayerPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
            ))
            // TODO: pause and/or proper quit menu
            .add_systems(Update, (bevy::window::close_on_esc, level::skip_level));
        }
    }
}
