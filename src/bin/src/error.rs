use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("network protocol error: {0}")]
    Net(#[from] net::Error),
    #[error("game state error: {0}")]
    Game(#[from] game::Error),
    #[error("server bind failed on {address}: {source}")]
    Bind {
        address: String,
        source: std::io::Error,
    },
    #[error("failed to connect to {address}: {source}")]
    Connect {
        address: String,
        source: std::io::Error,
    },
}
