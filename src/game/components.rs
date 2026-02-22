use bevy::prelude::*;
use super::hex::Hex;

#[derive(Component)]
pub struct Ship {
    pub owner_id: u32,
}

#[derive(Component)]
pub struct Position {
    pub hex: Hex,
}

#[derive(Component)]
pub struct TargetPosition {
    pub hex: Hex,
}

#[derive(Component)]
pub struct Health(pub u32);

/// Server-assigned ship ID, used to reconcile ECS entities with snapshots.
#[derive(Component, Clone, Copy)]
pub struct ShipId(pub u32);