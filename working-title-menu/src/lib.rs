use bevy::prelude::*;
use working_title_core::GameState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_menu)
            .add_systems(OnEnter(GameState::MenuOpen), show_menu)
            .add_systems(OnExit(GameState::MenuOpen), hide_menu)
            .add_systems(Update, handle_input.run_if(in_state(GameState::MenuOpen)));
    }
}

#[derive(Debug, Component)]
pub struct Menu;

pub fn create_menu(mut commands: Commands) {
    commands.spawn((
        Menu,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::rgb(0.33, 0.33, 0.33).into(),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
    ));
}

pub fn show_menu(mut query: Query<&mut Visibility, With<Menu>>) {
    query.for_each_mut(|mut visibility| {
        *visibility = Visibility::Visible;
    });
}

pub fn hide_menu(mut query: Query<&mut Visibility, With<Menu>>) {
    query.for_each_mut(|mut visibility| {
        *visibility = Visibility::Hidden;
    });
}

pub fn handle_input(input: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if input.just_pressed(KeyCode::Tab) {
        next_state.set(GameState::FirstPerson)
    }
}
