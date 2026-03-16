
use soa_derive::StructOfArray;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShipClass {
    #[default]
    Sloop,
    Brig,
    Galleon,
}

#[derive(StructOfArray)]
#[soa_derive(Debug, Eq, PartialEq, Clone)]
pub struct ShipStats {
    pub max_health: u32,
    pub move_range: u8,
    pub fire_range: u8,
    pub fire_dmg: u32,
}

impl ShipStats {
    pub fn new(max_health: u32, move_range: u8, fire_range: u8, fire_dmg: u32) -> Self {
        ShipStats {
            max_health,
            move_range,
            fire_range,
            fire_dmg,
        }
    }
}

impl From<ShipClass> for ShipStats {
    fn from(value: ShipClass) -> Self {
        match value {
            ShipClass::Sloop => ShipStats::new(10, 1, 1, 3),
            ShipClass::Brig => ShipStats::new(22, 1, 2, 3),
            ShipClass::Galleon => ShipStats::new(17, 2, 1, 3),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ship_profiles_have_distinct_visual_specs() {
        todo!()
    }
}
