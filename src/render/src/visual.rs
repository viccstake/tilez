use game::ShipClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualSpec {
    pub prefab_key: &'static str,
    pub animation_set: &'static str,
}

impl VisualSpec {
    pub const fn new(prefab_key: &'static str, animation_set: &'static str) -> Self {
        Self {
            prefab_key,
            animation_set,
        }
    }
}

impl From<ShipClass> for VisualSpec {
    fn from(class: ShipClass) -> Self {
        match class {
            ShipClass::Sloop => Self::new("ship.sloop", "anim.sloop"),
            ShipClass::Brig => Self::new("ship.brig", "anim.brig"),
            ShipClass::Galleon => Self::new("ship.galleon", "anim.galleon"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{VisualSpec};
    use game::ShipClass;

    fn ship_visual_spec(class: ShipClass) -> VisualSpec {
        class.into()
    }

    #[test]
    fn ship_classes_map_to_distinct_prefabs() {
        let sloop = ship_visual_spec(ShipClass::Sloop);
        let brig = ship_visual_spec(ShipClass::Brig);
        let galleon = ship_visual_spec(ShipClass::Galleon);

        assert_ne!(sloop.prefab_key, brig.prefab_key);
        assert_ne!(brig.prefab_key, galleon.prefab_key);
        assert_ne!(sloop.prefab_key, galleon.prefab_key);
    }

    #[test]
    fn from_impl_matches_helper() {
        let from_impl: VisualSpec = ShipClass::Brig.into();
        let from_helper = ship_visual_spec(ShipClass::Brig);
        assert_eq!(from_impl, from_helper);
    }
}
