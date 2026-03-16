pub mod api;
pub mod error;
pub mod grid;
pub mod ship;
pub mod resolution;

pub use api::*;
pub use grid::*;
pub use ship::*;

pub use error::*;

pub mod state;

struct ServerAuthoritative;
struct ClientRelay;