//! Core library for reading and editing Animal Crossing: New Horizons save data.
//!
//! This crate is a Rust re-implementation of the NHSE save editor core.

pub mod data;
pub mod editing;
pub mod error;
pub mod map;
pub mod save;
pub mod structs;
#[doc(hidden)]
pub mod testing;
pub mod util;

pub use error::{Error, Result};
pub use save::HorizonSave;
pub use structs::Item;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
