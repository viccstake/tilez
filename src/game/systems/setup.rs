use bevy::prelude::*;

use crate::game::components::HexTile;
use crate::game::hex::Hex;
use crate::game::resources::HexLayout;

pub fn setup_game(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn a flat-top hex tile sprite for every cell within `radius` of the origin.
pub fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    hex_layout: Res<HexLayout>,
) {
    // Rectangular grid dimensions (in offset coordinates).
    const COLS: i32 = 16;
    const ROWS: i32 = 10;
    let size = hex_layout.size;
    let col_center = COLS / 2;
    let row_center = ROWS / 2;

    // One shared mesh for all tile entities; each tile has its own material color.
    let tile_mesh = meshes.add(RegularPolygon::new(size * 0.95, 6));

    // Build a rectangle in odd-q offset coordinates, then convert to centered axial.
    // This keeps a rectangular board silhouette while still using axial math.
    for col in 0..COLS {
        for row in 0..ROWS {
            let centered_col = col - col_center;
            let centered_row = row - row_center;

            // Odd-q offset (column-staggered) -> axial conversion.
            let q = centered_col;
            let odd = q.rem_euclid(2);
            let r = centered_row - ((q - odd) / 2);
            let hex = Hex::new(q, r);
            let pos = hex.to_world(size);

            // Alternate two shades of ocean-blue for depth.
            let shade = if (col + row).rem_euclid(2) == 0 { 0.26 } else { 0.31 };
            let color = Color::srgb(shade, shade + 0.16, shade + 0.38);
            commands.spawn((
                HexTile,
                Mesh2d(tile_mesh.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ));
        }
    }
}
