use bevy::prelude::*;

use crate::game::state::GameState;

/// Animates ship movement between turns.
/// Phase 4 will lerp ship Transforms via TargetPosition over a fixed duration.
/// Until then, the Animating state is instantaneous.
pub fn animate_transitions(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Planning);
}
