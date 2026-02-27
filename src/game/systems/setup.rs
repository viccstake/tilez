use bevy::prelude::*;

use crate::game::components::HexTile;
use crate::game::hex::Hex;
use crate::game::resources::HexLayout;

pub fn setup_game(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn a flat-top hex tile sprite for every cell within `radius` of the origin.
pub fn setup_board(mut commands: Commands, hex_layout: Res<HexLayout>) {
    let radius: i32 = 7;
    let size = hex_layout.size;
    // Tile sprite slightly smaller than the hex pitch to leave visible seams.
    let sprite_size = Vec2::new(size * 1.42, size * f32::sqrt(3.0) * 0.95);

    for q in -radius..=radius {
        let r_min = (-radius).max(-q - radius);
        let r_max = radius.min(-q + radius);
        for r in r_min..=r_max {
            let hex = Hex::new(q, r);
            let pos = hex.to_world(size);
            // Alternate two shades of ocean-blue for depth.
            let shade = if (q - r).rem_euclid(2) == 0 { 0.26 } else { 0.31 };
            let color = Color::srgb(shade, shade + 0.16, shade + 0.38);
            commands.spawn((
                HexTile,
                Sprite {
                    color,
                    custom_size: Some(sprite_size),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ));
        }
    }
}
