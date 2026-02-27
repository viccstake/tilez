use bevy::prelude::*;
use bevy::math::Vec2;
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

/// Marker for hex grid background tile entities.
#[derive(Component)]
pub struct HexTile;

/// Per-ship lerp animation state. Added on the first frame of `Animating`,
/// removed when the lerp completes.
#[derive(Component)]
pub struct AnimTimer {
    pub elapsed: f32,
    pub duration: f32,
    /// World-space position when the animation began (lerp start).
    pub start_world: Vec2,
}