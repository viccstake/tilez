use bevy::prelude::*;
use crate::game::resources::OrderQueue;
use crate::game::orders::Order;
use crate::game::hex::Hex;

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut orders: ResMut<OrderQueue>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        orders.orders.push(Order::Move {
            entity_id: 0,
            to: Hex::new(1, 0),
        });
    }
}