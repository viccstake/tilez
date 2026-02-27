use bevy::prelude::*;

use crate::game::components::{Position, Ship};
use crate::game::resources::HexLayout;

/// Reactively adds a `Sprite` + `Transform` to every newly spawned ship entity.
/// Fires once per ship via the `Added<Ship>` filter.
pub fn spawn_ship_visuals(
    mut commands: Commands,
    query: Query<(Entity, &Ship, &Position), Added<Ship>>,
    hex_layout: Res<HexLayout>,
) {
    for (entity, ship, pos) in &query {
        let world_pos = pos.hex.to_world(hex_layout.size);
        let color = owner_color(ship.owner_id);
        commands.entity(entity).insert((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(hex_layout.size * 0.55)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
        ));
    }
}

pub fn owner_color(owner_id: u32) -> Color {
    match owner_id % 6 {
        0 => Color::srgb(0.2, 0.85, 0.35),
        1 => Color::srgb(0.85, 0.2, 0.2),
        2 => Color::srgb(0.95, 0.82, 0.1),
        3 => Color::srgb(0.2, 0.45, 0.95),
        4 => Color::srgb(0.9, 0.45, 0.1),
        _ => Color::srgb(0.72, 0.2, 0.88),
    }
}
