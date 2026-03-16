use std::any::TypeId;

use crate::grid::*;
use crate::ship::*;

use soa_derive::StructOfArray;


#[derive(StructOfArray)]
#[soa_derive(Debug, PartialEq)]
pub struct ShipState {
    pub id: u32,
    pub owner_id: u8,
    #[nested_soa]
    pub hx: Hex,
    pub health: u32,
    pub stamina: u8,
    pub class: ShipClass,
}

impl ShipState {
    pub fn new(owner_id: u8, class: ShipClass, hx: Hex) -> ShipState {
        let stats = ShipStats::from(class);
        Self {
            id: 0,
            owner_id,
            hx,
            health: stats.max_health,
            stamina: stats.move_range,
            class,
        }
    }
}

pub struct World {
    map: TileVec,
    ships: ShipStateVec,
}

impl Default for World {
    fn default() -> Self {
        World{ 
            map: TileVec::with_capacity(WIDTH*HEIGHT),
            ships: ShipStateVec::default()
        }
    }
}


impl World {
    pub fn ships_len(&self) -> usize {
        self.ships.len()
    }

    pub fn ships(&self) -> impl Iterator<Item = ShipStateRef<'_>> {
        self.ships.iter()
    }

    pub fn ships_mut(&mut self) -> impl Iterator<Item = ShipStateRefMut<'_>> {
        self.ships.iter_mut()
    }

    pub fn ship_index(&self, ship_id: u32) -> Option<usize> {
        self.ships.id.iter().position(|id| *id == ship_id)
    }

    pub fn ship(&self, ship_id: u32) -> Option<ShipStateRef<'_>> {
        self.ship_index(ship_id).map(|index| self.ships.index(index))
    }

    pub fn ship_mut(&mut self, ship_id: u32) -> Option<ShipStateRefMut<'_>> {
        let index = self.ship_index(ship_id)?;
        Some(self.ships.index_mut(index))
    }

    pub fn ship_position(&self, ship_id: u32) -> Option<Hex> {
        self.ship(ship_id).map(|ship| ship.hx.into())
    }

    pub fn ship_move_range(&self, ship_id: u32) -> Option<u8> {
        self.ship(ship_id).map(|ship| *ship.stamina)
    }

    pub fn push_ship(&mut self, ship: ShipState) {
        self.ships.push(ship);
    }

    pub fn move_ship(&mut self, ship_id: u32, to: Hex) -> Option<Hex> {
        let index = self.ship_index(ship_id)?;
        let from = Hex {
            q: self.ships.hx.q[index],
            r: self.ships.hx.r[index],
        };
        self.ships.hx.q[index] = to.q;
        self.ships.hx.r[index] = to.r;
        Some(from)
    }

    pub fn drain_stamina(&mut self, ship_id: u32, amount: u8) -> Option<u8> {
        let index = self.ship_index(ship_id)?;
        let current = self.ships.stamina[index];
        let next = current.saturating_sub(amount);
        self.ships.stamina[index] = next;
        Some(next)
    }

    pub fn apply_damage(&mut self, ship_id: u32, amount: u32) -> Option<u32> {
        let index = self.ship_index(ship_id)?;
        let current = self.ships.health[index];
        let next = current.saturating_sub(amount);
        self.ships.health[index] = next;
        Some(next)
    }

    pub fn reset(&mut self, hx: Hex) -> Option<Tile> {
        let i = hx.index();
        let ref_t = self.map.get_mut(i);
        return ref_t.map(|mut t| t.replace(Tile::default()))
    }
}





#[derive(Default, Clone, StructOfArray)]
#[soa_derive(Clone, Debug, PartialEq)]
pub struct Tile {
    state: TileTypeState,
    occupant: Option<TypeId>
}

impl Tile {
    pub fn state(&self) -> TileTypeState { 
        self.state.clone()
    }
    pub fn reset(&mut self) -> Option<TypeId> {
        let t = self.occupant;
        self.occupant = None;
        return t;
    }
    pub fn set_occupant(&mut self, id: TypeId) {
        self.occupant = Some(id)
    }
    pub fn get_occupant(&self) -> Option<TypeId> {
        self.occupant
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileTypeState {
    Water(Weather),
    Land(Weather, Earth),
}

impl Default for TileTypeState {
    fn default() -> Self { Self::Water(Weather::default()) }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Weather {
    #[default]
    Clear,
    Wind(Polar),
    Storm,
    Tornado,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Earth {
    #[default]
    Grassland,
    Tundra,
    Desert,
    Tropical,
}
