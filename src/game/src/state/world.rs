use crate::grid::*;
use crate::ship::{ShipStats, ShipClass};

use soa_derive::StructOfArray;

const WIDTH: usize = 100;
const HEIGHT: usize = 60;


pub struct PlayerState {
    id: u8,
    gold: u64,
}

impl PlayerState {
    pub fn new() -> Self {
        Self { id: 0, gold: 0 }
    }

    pub fn spawn_ship(&self, class: ShipClass, hx: Hex) -> ShipState {
        ShipState::new(self.id, class, hx)
    }
}


#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
pub struct ShipState {
    pub id: u32,
    pub owner_id: u8,
    #[nested_soa]
    pub hx: Hex,
    pub health: u32,
    pub class: ShipClass,
    pub stats: ShipStats
}

impl ShipState {
    pub fn new(owner_id: u8, class: ShipClass, hx: Hex) -> ShipState {
        let stats: ShipStats = class.into();
        Self { id: 0, owner_id, hx, health: stats.max_health, class, stats }
    }
}


#[derive(Default, StructOfArray)]
#[soa_derive(Clone, Debug, PartialEq)]
pub struct Tile {
    state: TileTypeState,
    // ...
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