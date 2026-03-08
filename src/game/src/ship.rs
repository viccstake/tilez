#[allow(dead_code)]


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShipClass {
    #[default]
    Sloop,
    Brig,
    Galleon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipStats {
    pub max_health: u32,
    pub move_range: u8,
    pub fire_range: u8,
    pub fire_dmg: u32
}
impl ShipStats {
    pub fn new(    
        max_health: u32,
        move_range: u8,
        fire_range: u8,
        fire_dmg: u32
    ) -> Self {
        ShipStats {
            max_health, move_range, fire_range, fire_dmg
        }
    }
}

impl Into<ShipStats> for ShipClass {
    fn into(self) -> ShipStats {
        match self {
            Self::Sloop => ShipStats::new(10, 1, 1, 3),
            Self::Brig => ShipStats::new(22, 1, 2, 3),
            Self::Galleon => ShipStats::new(17, 2, 1, 3),
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
