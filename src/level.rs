use bevy::prelude::*;

use crate::cubes::{Block, BlockCount, BlockState, CubeFrame, EntityGraph};
use crate::loading::FontAssets;
use crate::menu::ButtonColors;
use crate::GameState;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(win_condition.in_set(OnUpdate(GameState::Playing)))
            .add_system(reset_level.in_schedule(OnEnter(GameState::Reset)))
            .add_system(cleanup_continue.in_schedule(OnExit(GameState::Reset)))
            .add_system(setup_continue.in_schedule(OnEnter(GameState::Reset)))
            .add_system(click_continue.in_set(OnUpdate(GameState::Reset)));
    }
}

fn win_condition(blocks: Query<&Block>, mut next_state: ResMut<NextState<GameState>>) {
    if blocks
        .iter()
        .all(|block| block.state == BlockState::InPosition)
    {
        next_state.set(GameState::Reset);
    }
}

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

fn setup_continue(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(120.0), Val::Px(100.0)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },

            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Nice work! Keep going?",
                font_assets.text_style(),
            ));

            parent
                .spawn((
                    ContinueButton,
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(150.0), Val::Px(50.0)),
                            margin: UiRect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: button_colors.normal.into(),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Continue",
                        TextStyle {
                            font: font_assets.fira_sans.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
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
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
) {
    for interaction in &mut interaction_query {
        if let Interaction::Clicked = *interaction {
            state.set(GameState::Playing);
        }
    }
}

fn cleanup_continue(mut commands: Commands, ui_nodes: Query<Entity, With<Node>>) {
    for node in &ui_nodes {
        commands.entity(node).despawn_recursive();
    }
}
