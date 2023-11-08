use bevy::prelude::*;
use working_title_core::GameState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_input.run_if(in_state(GameState::MenuOpen)));
    }
}

pub fn handle_input(input: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if input.just_pressed(KeyCode::Tab) {
        next_state.set(GameState::FirstPerson)
    }
}
