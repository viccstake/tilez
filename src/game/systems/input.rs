use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::components::{Position, Ship, ShipId};
use crate::game::hex::Hex;
use crate::game::orders::Order;
use crate::game::resources::{HexLayout, LocalPlayerId, OrderQueue, Selection};
use crate::game::render::rendering::owner_color;

/// Two-step click input (Planning state only).
///
/// First click on one of the local player's ships → selects it (white highlight).
/// Second click anywhere → queues a `Move` order to that hex, deselects.
/// Escape → cancels selection.
pub fn handle_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    ships: Query<(Entity, &Ship, &ShipId, &Position)>,
    mut sprites: Query<&mut Sprite, With<Ship>>,
    mut selection: ResMut<Selection>,
    mut orders: ResMut<OrderQueue>,
    local_id: Res<LocalPlayerId>,
    hex_layout: Res<HexLayout>,
) {
    // Escape cancels any active selection.
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some((sel_entity, _)) = selection.0.take() {
            restore_color(sel_entity, &ships, &mut sprites);
        }
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = cursor_world_pos(&windows, &cameras) else {
        return;
    };
    let clicked_hex = Hex::pixel_to_hex(world_pos, hex_layout.size);

    if let Some((sel_entity, sel_ship_id)) = selection.0.take() {
        // Second click: queue a Move to the clicked hex then deselect.
        orders.orders.push(Order::Move { entity_id: sel_ship_id, to: clicked_hex });
        restore_color(sel_entity, &ships, &mut sprites);
    } else {
        // First click: select a ship belonging to the local player at that hex.
        let Some(local_pid) = local_id.0 else { return };
        for (entity, ship, ship_id, pos) in &ships {
            if ship.owner_id == local_pid && pos.hex == clicked_hex {
                if let Ok(mut sprite) = sprites.get_mut(entity) {
                    sprite.color = Color::WHITE;
                }
                selection.0 = Some((entity, ship_id.0));
                break;
            }
        }
    }
}

/// Runs once on `OnEnter(GameState::Animating)`.
/// If the player had a ship selected but hadn't placed an order, clear the
/// selection and restore the ship's normal color so it doesn't stay white.
pub fn clear_selection_on_animate(
    mut selection: ResMut<Selection>,
    ships: Query<(Entity, &Ship, &ShipId, &Position)>,
    mut sprites: Query<&mut Sprite, With<Ship>>,
) {
    if let Some((entity, _)) = selection.0.take() {
        restore_color(entity, &ships, &mut sprites);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cursor_world_pos(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn restore_color(
    entity: Entity,
    ships: &Query<(Entity, &Ship, &ShipId, &Position)>,
    sprites: &mut Query<&mut Sprite, With<Ship>>,
) {
    if let (Ok((_, ship, _, _)), Ok(mut sprite)) =
        (ships.get(entity), sprites.get_mut(entity))
    {
        sprite.color = owner_color(ship.owner_id);
    }
}
