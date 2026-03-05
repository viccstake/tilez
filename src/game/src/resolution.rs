/// Simultaneous turn resolution.
///
/// For every `Move` order, the target hex is recorded by ship ID.
/// Ships with contested destinations (two or more ships heading to the
/// same hex) are blocked and remain in place.
/// Ships with no order, or with a `Hold` order, also stay put.
pub fn resolve_turn() {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hex;

    fn ship(id: u32, q: i32, r: i32) -> () {
        todo!()
    }

    fn mv(entity_id: u32, q: i32, r: i32) {
        todo!()
    }

    #[test]
    fn uncontested_move_succeeds() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn no_orders_ship_stays() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn hold_order_ship_stays() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn collision_both_ships_blocked() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn partial_collision_unrelated_ship_still_moves() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn unknown_ship_id_is_ignored() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn multiple_ships_move_to_different_hexes() {
        // ...
        resolve_turn();
        // ...
    }

    #[test]
    fn ship_cannot_displace_stationary_ship() {
        // ...
        resolve_turn();
        // ...
    }
}
