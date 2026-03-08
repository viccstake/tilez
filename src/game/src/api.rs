use crate::{state::*, grid::Hex, Result};


/// User-facing entrypoint for creating a game instance.
pub struct GameBuilder {
    ships: Vec<ShipState>,
    nr_players: u8,
}

impl Default for GameBuilder {
    fn default() -> Self {
        let ships = vec![];
        let nr_players = 2;
        GameBuilder { ships, nr_players}
    }

}

impl GameBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ship(mut self, ship: ShipState) -> Self {
        self.ships.push(ship);
        self
    }

    pub fn with_ships<I>(mut self, ships: I) -> Self
    where
        I: IntoIterator<Item = ShipState>,
    {
        self.ships.extend(ships);
        self
    }

    pub fn build(self) -> Game {
        let mut state = MatchState::new();
        if let Some(ships) = state.state_mut().get_mut::<ShipStateVec>() {
            ships.extend(self.ships);
        }
        Game { state }
    }
}

/// User-facing game handle.
pub struct Game {
    state: MatchState,
}

impl Game {

    pub fn new() -> Self {
        GameBuilder::new().build()
    }

    pub fn builder() -> GameBuilder {
        GameBuilder::new()
    }

    pub fn from_match_state(self, state: MatchState) -> Self {
        Self { state: state }
    }

    pub fn into_match_state(self) -> MatchState {
        self.state
    }

    pub fn turn(&self) -> u64 {
        self.state.turn()
    }

    pub fn phase(&self) -> GameState {
        self.state.phase()
    }

    pub fn advance_turn(&mut self) -> Result<u64> {
        self.state.advance_turn()
    }

    pub fn advance_phase(&mut self) -> GameState {
        self.state.advance_phase()
    }

    pub fn ships(&self) -> Option<&ShipStateVec> {
        self.state.state().get()
    }

    pub fn ships_mut(&mut self) -> Option<&mut ShipStateVec> {
        self.state.state_mut().get_mut()
    }

    pub fn spawn_ship(&mut self, ship: ShipState) {
        if let Some(ships) = self.ships_mut() {
            ships.push(ship);
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_forwards_turn_and_phase_state() {
        let mut game = Game::new();
        assert_eq!(game.turn(), 0);
        assert_eq!(game.phase(), GameState::Planning);

        assert_eq!(game.advance_turn().expect("turn should advance"), 1);
        assert_eq!(game.advance_phase(), GameState::Resolving);
        assert_eq!(game.turn(), 1);
    }
}
