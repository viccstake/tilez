use super::hex::Hex;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Order {
    Move { entity_id: u32, to: Hex },
    Hold { entity_id: u32 },
}