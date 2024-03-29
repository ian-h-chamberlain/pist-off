use bevy::{log, prelude::*};

#[cfg(not(target_family = "wasm"))]
use bevy::app::AppExit;

use crate::cubes::{
    Block, BlockCount, BlockState, CubeFrame, EntityGraph, ToggleEvent, ToggleTimer,
};
use crate::loading::FontAssets;
use crate::menu::ButtonColors;
use crate::GameState;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                win_condition.run_if(in_state(GameState::Playing)),
                click_continue.run_if(in_state(GameState::Reset)),
            ),
        )
        .add_systems(OnEnter(GameState::Reset), (reset_level, setup_buttons))
        .add_systems(OnExit(GameState::Reset), cleanup_continue);

        // "quitting" kinda just crashes the app in wasm, let's not do that
        #[cfg(not(target_family = "wasm"))]
        app.add_systems(Update, click_quit.run_if(in_state(GameState::Reset)));
    }
}

fn win_condition(
    blocks: Query<(&Block, &AnimationPlayer)>,
    mut events: EventReader<ToggleEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if events.read().any(|evt| evt.state != BlockState::InPosition) {
        return;
    }

    if blocks.is_empty() {
        return;
    }

    if blocks.iter().all(|(block, player)| {
        // player.is_finished() would be nice but doesn't seem to work in reverse here.
        // (maybe should file a bug report for ath?)
        //
        // < 0.01 should always work, since the player will stop animating *after* the
        // time reaches zero, so it should normally be a little bit less than zero.
        if block.state == BlockState::InPosition {
            log::debug!("seek time: {:?}", player.seek_time());
            player.seek_time() < 0.01
        } else {
            false
        }
    }) {
        next_state.set(GameState::Reset);
    }
}

#[cfg(debug_assertions)]
pub(crate) fn skip_level(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::N) {
        next_state.set(GameState::Reset);
    }
}

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct QuitButton;

fn setup_buttons(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    let style = Style {
        margin: UiRect::all(Val::Auto),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(350.0),
                height: Val::Px(125.0),
                flex_direction: FlexDirection::Column,
                ..style.clone()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Nice work! Keep going?",
                font_assets.text_style(),
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(350.0),
                        height: Val::Px(50.0),
                        flex_direction: FlexDirection::Row,
                        ..style
                    },
                    ..default()
                })
                .with_children(|parent| spawn_buttons(parent, font_assets, button_colors));
        });
}

fn spawn_buttons(
    parent: &mut ChildBuilder,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    let button = ButtonBundle {
        style: Style {
            width: Val::Px(150.0),
            height: Val::Px(50.0),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        background_color: button_colors.normal.into(),
        ..Default::default()
    };

    parent
        .spawn((ContinueButton, button.clone()))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Continue",
                font_assets.button_style(),
            ));
        });

    #[cfg(not(target_family = "wasm"))]
    parent.spawn((QuitButton, button)).with_children(|parent| {
        parent.spawn(TextBundle::from_section("Quit", font_assets.button_style()));
    });
}

type IsGameEntity = Or<(With<CubeFrame>, With<EntityGraph>)>;

fn reset_level(
    game_entities: Query<Entity, IsGameEntity>,
    mut commands: Commands,
    mut block_count: ResMut<BlockCount>,
) {
    for entity in &game_entities {
        commands.entity(entity).despawn_recursive();
    }

    block_count.0 += 1;
}

fn click_continue(
    mut state: ResMut<NextState<GameState>>,
    mut continue_interaction: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
) {
    for interaction in &mut continue_interaction {
        if let Interaction::Pressed = *interaction {
            state.set(GameState::Playing);
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn click_quit(
    mut quit_interaction: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut quit: EventWriter<AppExit>,
) {
    for interaction in &mut quit_interaction {
        if let Interaction::Pressed = *interaction {
            quit.send(AppExit);
        }
    }
}

fn cleanup_continue(mut commands: Commands, ui_nodes: Query<Entity, With<Node>>) {
    for node in &ui_nodes {
        commands.entity(node).despawn_recursive();
    }
}
