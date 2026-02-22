// Always compiled — no Bevy dependency
pub mod hex;
pub mod orders;
pub mod resolution;

// Bevy-dependent submodules
#[cfg(feature = "game")]
pub mod components;
#[cfg(feature = "game")]
pub mod resources;
#[cfg(feature = "game")]
pub mod state;
#[cfg(feature = "game")]
pub mod systems;
