use crate::loading::FontAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_mod_picking::PickingCameraBundle;

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(click_play_button.in_set(OnUpdate(GameState::Menu)))
            .add_system(color_buttons.in_base_set(CoreSet::Update))
            .add_system(cleanup_menu.in_schedule(OnExit(GameState::Menu)));
    }
}

#[derive(Resource)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
}

#[derive(Component)]
pub struct PlayButton;

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb_u8(100, 100, 100),
            hovered: Color::rgb_u8(60, 60, 60),
        }
    }
}

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                order: -10,
                is_active: true,
                ..default()
            },
            ..default()
        })
        .insert(PickingCameraBundle::default());

    commands
        .spawn((
            PlayButton,
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(120.0), Val::Px(50.0)),
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
            parent.spawn(TextBundle::from_section("Play", font_assets.button_style()));
        });
}

fn color_buttons(
    button_colors: Res<ButtonColors>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
            _ => {}
        }
    }
}

fn click_play_button(
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
) {
    for interaction in &mut interaction_query {
        if let Interaction::Clicked = *interaction {
            state.set(GameState::Playing);
        }
    }
}

fn cleanup_menu(mut commands: Commands, buttons: Query<Entity, With<Button>>) {
    for button in &buttons {
        commands.entity(button).despawn_recursive();
    }
}
