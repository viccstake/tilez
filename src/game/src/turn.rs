use crate::error::{Error, Result};
use crate::state::*;

pub struct MatchState {
    turn: u64,
    phase: GameState,
    world: WorldState,
}

impl MatchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn turn(&self) -> u64 {
        self.turn
    }

    pub fn phase(&self) -> GameState {
        self.phase
    }

    pub fn world(&self) -> &WorldState {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut WorldState {
        &mut self.world
    }

    pub fn advance_phase(&mut self) -> GameState {
        self.phase = self.phase.next();
        self.phase
    }

    pub fn advance_turn(&mut self) -> Result<u64> {
        self.turn = self.turn.checked_add(1).ok_or(Error::TurnOverflow)?;
        Ok(self.turn)
    }
}

impl Default for MatchState {
    fn default() -> Self {
        Self {
            turn: 0,
            phase: GameState::default(),
            world: WorldState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MatchState;

    #[test] 
    fn match_state_starts_at_zero() {
        let state = MatchState::new();
        assert_eq!(state.turn(), 0);
    }

    #[test]
    fn match_state_advance_turn_is_monotonic() {
        let mut state = MatchState::new();

        assert_eq!(state.advance_turn().expect("turn 1 should advance"), 1);
        assert_eq!(state.advance_turn().expect("turn 2 should advance"), 2);
        assert_eq!(state.advance_turn().expect("turn 3 should advance"), 3);
        assert_eq!(state.turn(), 3);
    }
}
