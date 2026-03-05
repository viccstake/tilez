



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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_profiles_have_distinct_visual_specs() {
        todo!()
    }
}
