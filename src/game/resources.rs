use bevy::prelude::*;
use super::orders::Order;

#[derive(Resource, Default)]
pub struct OrderQueue {
    pub orders: Vec<Order>,
}

#[derive(Resource, Default)]
pub struct CurrentTurn(pub u32);