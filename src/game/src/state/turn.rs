use crate::state::PlayerState;
use crate::{Error, HEIGHT, Result, WIDTH};
use super::world::TileVec;
use super::store::{StateStore};


impl GameState {
    pub fn next(self) -> Self {
        match self {
            Self::Planning => Self::Resolving,
            Self::Resolving => Self::Animating,
            Self::Animating => Self::Resolving,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Planning,
    Resolving,
    Animating,
}

pub struct MatchState {
    turn: u64,
    user: PlayerState,
    phase: GameState,
    world: TileVec,
    states: StateStore,
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

    pub fn world(&self) -> &TileVec {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut TileVec {
        &mut self.world
    }

    pub fn state(&self) -> &StateStore {
        &self.states
    }

    pub fn state_mut(&mut self) -> &mut StateStore {
        &mut self.states
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
            world: TileVec::with_capacity(WIDTH*HEIGHT),
            states: StateStore::default(),
            user: PlayerState::new(),
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
