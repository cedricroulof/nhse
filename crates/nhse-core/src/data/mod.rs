//! Embedded game data: string tables, item metadata and generated lookup tables.
pub mod items;
pub mod strings;
pub mod tables;

pub use strings::{strings, GameStrings, LANGUAGES};
pub use tables::{enum_name, enum_values, tables, Tables};
