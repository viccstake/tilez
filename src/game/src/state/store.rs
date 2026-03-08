
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Type-indexed storage for game state buckets (ships, ports, weather, etc.).
#[derive(Default)]
pub struct StateStore {
    states: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl StateStore {
    pub fn insert<T: Any + Send + Sync>(&mut self, state: T) -> Option<T> {
        self.states
            .insert(TypeId::of::<T>(), Box::new(state))
            .and_then(|old| old.downcast::<T>().ok().map(|boxed| *boxed))
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.states
            .get(&TypeId::of::<T>())
            .and_then(|state| state.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.states
            .get_mut(&TypeId::of::<T>())
            .and_then(|state| state.downcast_mut::<T>())
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.states
            .remove(&TypeId::of::<T>())
            .and_then(|old| old.downcast::<T>().ok().map(|boxed| *boxed))
    }

    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.states.contains_key(&TypeId::of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::StateStore;

    #[derive(Debug, PartialEq, Eq)]
    struct ShipRoster {
        count: usize,
    }

    #[derive(Debug, PartialEq)]
    struct WeatherState {
        wind_speed: f32,
    }

    #[test]
    fn supports_multiple_state_kinds() {
        let mut store = StateStore::default();
        store.insert(ShipRoster { count: 3 });
        store.insert(WeatherState { wind_speed: 8.5 });

        assert_eq!(store.get::<ShipRoster>().map(|v| v.count), Some(3));
        assert_eq!(
            store.get::<WeatherState>().map(|v| v.wind_speed),
            Some(8.5)
        );
    }

    #[test]
    fn insert_returns_previous_value() {
        let mut store = StateStore::default();
        assert!(store.insert(ShipRoster { count: 1 }).is_none());

        let old = store.insert(ShipRoster { count: 2 });
        assert_eq!(old.map(|v| v.count), Some(1));
        assert_eq!(store.get::<ShipRoster>().map(|v| v.count), Some(2));
    }
}
