use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("turn counter overflowed while advancing the game state")]
    TurnOverflow,

    #[error("primitive action command was invalid")]
    IllegalOrder,

    #[error("player id space exhausted (u8 overflow)")]
    PlayerIdExhausted,

    #[error("ship id space exhausted (u32 overflow)")]
    ShipIdExhausted,

    #[error("out of stamina")]
    OutOfStamina,

    #[error("index of array was out of bounds")]
    OutOfBounds,

    #[error("found no value at the given index")]
    NoEntityAtIndex,
}
