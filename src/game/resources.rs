use bevy::prelude::*;
use super::orders::Order;

/// The server-assigned ID for this client's player. Set on `Welcome`.
#[derive(Resource, Default)]
pub struct LocalPlayerId(pub Option<u32>);

/// Two-step click selection: holds the selected (Bevy entity, server ship_id).
/// `None` means no ship is currently selected.
#[derive(Resource, Default)]
pub struct Selection(pub Option<(Entity, u32)>);

#[derive(Resource, Default)]
pub struct OrderQueue {
    pub orders: Vec<Order>,
}

#[derive(Resource, Default)]
pub struct CurrentTurn(pub u32);

#[derive(Resource)]
pub struct HexLayout {
    pub size: f32,
}

impl Default for HexLayout {
    fn default() -> Self {
        Self { size: 40.0 }
    }
}