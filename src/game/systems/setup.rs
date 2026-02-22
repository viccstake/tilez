use bevy::prelude::*;

pub fn setup_game(mut commands: Commands) {
    commands.spawn(Camera2d);
}
