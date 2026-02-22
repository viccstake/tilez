use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Planning,     // players issue commands
    Resolving,    // authoritative resolution
    Animating,    // smooth movement transitions
}