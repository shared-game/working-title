use bevy::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    FirstPerson,
    MenuOpen,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>();
    }
}