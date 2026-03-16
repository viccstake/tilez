use std::collections::{VecDeque};
use super::world::*;
use super::order::{self, Order};


use crate::{Error, Hex, Result};

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
    players: PlayerQueue,               // Action-taking entities to be interleaved when resolving (AI Opponents, Other players, NPCs)
    phase: GameState,                   // The game exists in discrete phases (turn-based)
    world: World,                       // The visible and playable world containing the state of each tile, world difficulty is embedded in these parameters
}

type PlayerQueue = VecDeque<PlayerState>;

impl MatchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn turn(&self) -> u64 {
        self.turn
    }

    pub(crate) fn phase(&self) -> GameState {
        self.phase
    }

    pub(crate) fn world(&self) -> &World {
        &self.world
    }

    pub(crate) fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub(crate) fn user(&mut self) -> &mut PlayerState {
        self.players.front_mut().expect("empty player deck not possible")
    }

    pub(crate) fn users(&mut self) -> impl Iterator<Item = &mut PlayerState> {
        self.players.iter_mut()
    }

    pub(crate) fn advance_phase(&mut self) -> GameState {
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
            world: World::default(),
            players: VecDeque::from(vec![PlayerState::default()])
        }
    }
}

pub struct PlayerState {
    id: u8,
    gold: u64,
    orders: Vec<Order>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            id: 0,
            gold: 0,
            orders: vec![],
        }
    }

    pub fn with_id(id: u8) -> Self {
        Self {
            id,
            gold: 0,
            orders: vec![],
        }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn gold(&self) -> u64 {
        self.gold
    }

    pub fn submit_order(&mut self, ship_id: u32, action: order::Action) {
        self.orders.push(Order::from_action(ship_id, action));
    }

    pub(crate) fn spawn<F, T, P>(&self, f: F, param: P) -> T 
    where 
        F: Fn(&Self, P) -> T 
    {
        let o = f(self, param);
        println!("Spawned somthing");
        return o;
    }

    pub(crate) fn set_id(&mut self, id: u8) {
        self.id = id;
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
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
