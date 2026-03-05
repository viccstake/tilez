use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("transport I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("failed to deserialize inbound message: {0}")]
    Deserialize(bincode::Error),
    #[error("failed to serialize outbound message: {0}")]
    Serialize(bincode::Error),
}
