
use crate::state::*;
use uid::{IdU8, IdU32};

struct PlayerUidNamespace;
type PlayerUid = IdU8<PlayerUidNamespace>;

type ShipUid = IdU32<PlayerUid>;

/// User-facing entrypoint for creating a game instance.
#[derive(Default)]
pub struct GameBuilder {
    rng_1: PlayerUid,
    ships: Vec<ShipState>,
}

impl GameBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> Game {
        todo!()
    }
}

/// User-facing game handle.
pub struct Game {
    state: MatchState,
    player_gen: IdU8<u8>
}

impl Game {
    pub fn new() -> Self {
        GameBuilder::new().build()
    }

    pub fn builder() -> GameBuilder {
        GameBuilder::new()
    }

    pub fn from_match_state(self, state: MatchState) -> Self {
        Self {
            state,
            player_gen: self.player_gen
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
        todo!()
    }
}
