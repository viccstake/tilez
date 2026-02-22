use serde::{Deserialize, Serialize};

use crate::game::orders::Order;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { player_name: String },
    SubmitOrders { turn: u32, orders: Vec<Order> },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome { player_id: u32 },
    TurnStarted { turn: u32 },
    TurnResolved { turn: u32 },
    /// bincode-encoded `GameSnapshot`
    StateSnapshot(Vec<u8>),
    Error(String),
    Pong,
}

// ── game snapshot (sent as StateSnapshot payload) ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipSnapshot {
    pub id: u32,
    pub owner_id: u32,
    pub q: i32,
    pub r: i32,
    pub health: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub turn: u32,
    pub ships: Vec<ShipSnapshot>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::hex::Hex;
    use crate::game::orders::Order;

    fn roundtrip<T>(v: T) -> T
    where
        T: Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let bytes = bincode::serialize(&v).expect("serialize failed");
        let decoded: T = bincode::deserialize(&bytes).expect("deserialize failed");
        assert_eq!(v, decoded);
        decoded
    }

    #[test]
    fn client_ping() {
        roundtrip(ClientMessage::Ping);
    }

    #[test]
    fn client_join() {
        roundtrip(ClientMessage::Join { player_name: "Alice".into() });
    }

    #[test]
    fn client_submit_orders_with_move_and_hold() {
        roundtrip(ClientMessage::SubmitOrders {
            turn: 7,
            orders: vec![
                Order::Move { entity_id: 0, to: Hex::new(1, -1) },
                Order::Hold { entity_id: 2 },
            ],
        });
    }

    #[test]
    fn client_submit_empty_orders() {
        roundtrip(ClientMessage::SubmitOrders { turn: 1, orders: vec![] });
    }

    #[test]
    fn server_welcome() {
        roundtrip(ServerMessage::Welcome { player_id: 42 });
    }

    #[test]
    fn server_turn_started() {
        roundtrip(ServerMessage::TurnStarted { turn: 3 });
    }

    #[test]
    fn server_turn_resolved() {
        roundtrip(ServerMessage::TurnResolved { turn: 3 });
    }

    #[test]
    fn server_pong() {
        roundtrip(ServerMessage::Pong);
    }

    #[test]
    fn server_error() {
        roundtrip(ServerMessage::Error("something broke".into()));
    }

    #[test]
    fn state_snapshot_outer_roundtrip() {
        let bytes = bincode::serialize(&GameSnapshot {
            turn: 2,
            ships: vec![ShipSnapshot { id: 0, owner_id: 1, q: -2, r: 1, health: 10 }],
        })
        .unwrap();
        roundtrip(ServerMessage::StateSnapshot(bytes));
    }

    #[test]
    fn game_snapshot_fields_survive_roundtrip() {
        let original = GameSnapshot {
            turn: 5,
            ships: vec![
                ShipSnapshot { id: 0, owner_id: 0, q: -3, r: 2, health: 10 },
                ShipSnapshot { id: 1, owner_id: 1, q: 3, r: -2, health: 7 },
            ],
        };
        let bytes = bincode::serialize(&original).unwrap();
        let decoded: GameSnapshot = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded.turn, 5);
        assert_eq!(decoded.ships.len(), 2);
        assert_eq!(decoded.ships[1].health, 7);
        assert_eq!(decoded.ships[0].q, -3);
    }

    #[test]
    fn negative_hex_coords_survive_roundtrip() {
        roundtrip(ClientMessage::SubmitOrders {
            turn: 1,
            orders: vec![Order::Move { entity_id: 0, to: Hex::new(-5, -3) }],
        });
    }
}
