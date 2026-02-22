use std::collections::HashMap;

use super::orders::Order;

/// Authoritative ship state used by the server for turn resolution.
/// Kept separate from the Bevy ECS components so it can be used and
/// tested without the `game` feature flag.
pub struct ShipState {
    pub id: u32,
    pub owner_id: u32,
    pub q: i32,
    pub r: i32,
    pub health: u32,
}

/// Simultaneous turn resolution.
///
/// For every `Move` order, the target hex is recorded by ship ID.
/// Ships with contested destinations (two or more ships heading to the
/// same hex) are blocked and remain in place.
/// Ships with no order, or with a `Hold` order, also stay put.
pub fn resolve_turn(ships: &mut [ShipState], pending_orders: &HashMap<u32, Vec<Order>>) {
    // Build ship_id → target hex from all submitted orders.
    let mut targets: HashMap<u32, (i32, i32)> = HashMap::new();
    for orders in pending_orders.values() {
        for order in orders {
            if let Order::Move { entity_id, to } = order {
                targets.insert(*entity_id, (to.q, to.r));
            }
        }
    }

    // Count how many ships are heading to each hex.
    // Ships with no move order contribute their current position.
    let mut occupancy: HashMap<(i32, i32), u32> = HashMap::new();
    for ship in ships.iter() {
        let dest = targets.get(&ship.id).copied().unwrap_or((ship.q, ship.r));
        *occupancy.entry(dest).or_insert(0) += 1;
    }

    // Apply moves — a ship is blocked if its destination is contested.
    for ship in ships.iter_mut() {
        if let Some(&dest) = targets.get(&ship.id) {
            if occupancy[&dest] == 1 {
                ship.q = dest.0;
                ship.r = dest.1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::hex::Hex;

    fn ship(id: u32, q: i32, r: i32) -> ShipState {
        ShipState { id, owner_id: 0, q, r, health: 10 }
    }

    fn mv(entity_id: u32, q: i32, r: i32) -> Order {
        Order::Move { entity_id, to: Hex::new(q, r) }
    }

    // Helper: single-player order map
    fn p0(orders: Vec<Order>) -> HashMap<u32, Vec<Order>> {
        HashMap::from([(0u32, orders)])
    }

    #[test]
    fn uncontested_move_succeeds() {
        let mut ships = vec![ship(0, 0, 0)];
        resolve_turn(&mut ships, &p0(vec![mv(0, 1, 0)]));
        assert_eq!((ships[0].q, ships[0].r), (1, 0));
    }

    #[test]
    fn no_orders_ship_stays() {
        let mut ships = vec![ship(0, 3, 2)];
        resolve_turn(&mut ships, &HashMap::new());
        assert_eq!((ships[0].q, ships[0].r), (3, 2));
    }

    #[test]
    fn hold_order_ship_stays() {
        let mut ships = vec![ship(0, 1, 1)];
        resolve_turn(&mut ships, &p0(vec![Order::Hold { entity_id: 0 }]));
        assert_eq!((ships[0].q, ships[0].r), (1, 1));
    }

    #[test]
    fn collision_both_ships_blocked() {
        let mut ships = vec![ship(0, 0, 0), ship(1, 2, 0)];
        let orders = HashMap::from([
            (0u32, vec![mv(0, 1, 0)]),
            (1u32, vec![mv(1, 1, 0)]),
        ]);
        resolve_turn(&mut ships, &orders);
        assert_eq!((ships[0].q, ships[0].r), (0, 0));
        assert_eq!((ships[1].q, ships[1].r), (2, 0));
    }

    #[test]
    fn partial_collision_unrelated_ship_still_moves() {
        let mut ships = vec![ship(0, 0, 0), ship(1, 2, 0), ship(2, 4, 0)];
        // Ships 0 and 1 both target (1,0); ship 2 moves freely to (5,0).
        let orders = HashMap::from([
            (0u32, vec![mv(0, 1, 0), mv(1, 1, 0)]),
            (1u32, vec![mv(2, 5, 0)]),
        ]);
        resolve_turn(&mut ships, &orders);
        assert_eq!((ships[0].q, ships[0].r), (0, 0)); // blocked
        assert_eq!((ships[1].q, ships[1].r), (2, 0)); // blocked
        assert_eq!((ships[2].q, ships[2].r), (5, 0)); // moved
    }

    #[test]
    fn unknown_ship_id_is_ignored() {
        let mut ships = vec![ship(0, 0, 0)];
        resolve_turn(&mut ships, &p0(vec![mv(99, 5, 5)]));
        assert_eq!((ships[0].q, ships[0].r), (0, 0));
    }

    #[test]
    fn multiple_ships_move_to_different_hexes() {
        let mut ships = vec![ship(0, 0, 0), ship(1, 5, 0)];
        resolve_turn(&mut ships, &p0(vec![mv(0, 1, 0), mv(1, 4, 0)]));
        assert_eq!((ships[0].q, ships[0].r), (1, 0));
        assert_eq!((ships[1].q, ships[1].r), (4, 0));
    }

    #[test]
    fn ship_cannot_displace_stationary_ship() {
        // Ship 1 holds; ship 0 tries to move onto ship 1's hex.
        let mut ships = vec![ship(0, 0, 0), ship(1, 1, 0)];
        resolve_turn(&mut ships, &p0(vec![mv(0, 1, 0)]));
        // Both ship 0 and ship 1 count toward occupancy of (1,0), so ship 0 is blocked.
        assert_eq!((ships[0].q, ships[0].r), (0, 0));
        assert_eq!((ships[1].q, ships[1].r), (1, 0));
    }
}
