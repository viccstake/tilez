use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientMessage {
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerMessage {
    TurnAdvanced { turn: u64 },
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_message_bincode_roundtrip() {
        let original = ClientMessage::Ping;
        let encoded = bincode::serialize(&original).expect("client message should serialize");
        let decoded: ClientMessage =
            bincode::deserialize(&encoded).expect("client message should deserialize");

        assert_eq!(decoded, original);
    }

    #[test]
    fn server_message_bincode_roundtrip() {
        let original = ServerMessage::TurnAdvanced { turn: 42 };
        let encoded = bincode::serialize(&original).expect("server message should serialize");
        let decoded: ServerMessage =
            bincode::deserialize(&encoded).expect("server message should deserialize");

        assert_eq!(decoded, original);
    }
}
