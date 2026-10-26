//! Save container: loading, hashing, encrypting and writing back.
pub mod crypto;
pub mod encrypted_int;
pub mod file;
pub mod hash;
#[rustfmt::skip]
pub mod hashes_gen;
#[rustfmt::skip]
pub mod offsets_gen;
pub mod horizon;
pub mod island;
pub mod player;
pub mod postbox;
pub mod provider;
pub mod revision;

pub use encrypted_int::EncryptedInt32;
pub use file::SaveFile;
pub use horizon::{HorizonSave, PlayerFiles};
pub use offsets_gen::{MainOffsets, PersonalOffsets, PostBoxOffsets};
pub use provider::SaveSource;
pub use revision::{DetectedRevision, FileHeaderInfo, REVISIONS};
