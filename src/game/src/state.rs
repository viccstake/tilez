
use soa_derive::StructOfArray;

use crate::grid::*;
use crate::state_store::StateStore;

///
/// MatchState
/// 

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Planning,
    Resolving,
    Animating,
}

impl GameState {
    pub fn next(self) -> Self {
        match self {
            Self::Planning => Self::Resolving,
            Self::Resolving => Self::Animating,
            Self::Animating => Self::Resolving,
        }
    }
}

pub struct WorldState {
    pub board: BoardState,
    states: StateStore,
}

impl WorldState {
    pub fn with_standard_board() -> Self {
        let mut world = Self {
            board: BoardState::new(WIDTH * HEIGHT),
            states: StateStore::default(),
        };
        world.insert_state::<ShipStateVec>(ShipStateVec::new());
        world
    }

    pub fn insert_state<T: 'static + Send + Sync>(&mut self, state: T) -> Option<T> {
        self.states.insert(state)
    }

    pub fn state<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.states.get()
    }

    pub fn state_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.states.get_mut()
    }

    pub fn ships(&self) -> Option<&ShipStateVec> {
        self.state::<ShipStateVec>()
    }

    pub fn ships_mut(&mut self) -> Option<&mut ShipStateVec> {
        self.state_mut::<ShipStateVec>()
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::with_standard_board()
    }
}

#[derive(Default)]
pub struct BoardState {
    pub tiles: TileVec, // indexed by (r * width + q)
}

impl BoardState {
    pub fn new(size: usize) -> Self {
        let mut tiles = TileVec::with_capacity(size);
        for _ in 0..size {
            tiles.push(Tile::default());
        }
        Self { tiles }
    }
}

#[derive(Default, StructOfArray)]
#[soa_derive(Clone, Debug, PartialEq)]
pub struct Tile {
    state: TileTypeState,
    experience: ExperienceState,
}

#[derive(Debug, Clone, PartialEq)]
enum TileTypeState {
    Water(Weather),
    Land(Weather, Earth),
}

impl Default for TileTypeState {
    fn default() -> Self {
        Self::Water(Weather::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
enum Weather {
    #[default]
    Clear,
    Wind(Polar),
    Storm,
    Tornado,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum Earth {
    #[default]
    Grassland,
    Tundra,
    Desert,
    Tropical,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExperienceState {
    multipliers: Vec<i8>,
}

#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
pub struct ShipState {
    pub id: u32,
    pub owner_id: u32,
    #[nested_soa]
    pub hx: Hex,
    pub health: u32,
}

pub struct PlayerState {
    board: BoardState,
    experience: ExperienceState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_code() {
        // 1. Initialize struct of arrays
        let mut world = WorldState::default();
        let ships = world
            .ships_mut()
            .expect("world should provide a default ship state bucket");

        // 2. Push data into it.
        ships.push(ShipState {
            id: 1,
            owner_id: 10,
            hx: Hex::new(0, 0),
            health: 100,
        });

        // 3. Accessing specific columns.
        for health in ships.health.iter_mut() {
            *health -= 10;
        }

        // 4. Accessing a specific row. This returns a `ShipStateRef`.
        let first_ship = ships.get(0).unwrap();
        assert_eq!(*first_ship.health, 90);
    }
}
