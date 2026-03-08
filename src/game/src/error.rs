use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {


    #[error("turn counter overflowed while advancing the game state")]
    TurnOverflow,


    #[error("primitive action command was invalid")]
    IllegalOrder,

    #[error("")]
    OutOfStamina,
}
