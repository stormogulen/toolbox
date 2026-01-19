//! Simple binary save format with integrity hashing.
//!
//! The format supports packed POD structs and computes a Merkle-compatible
//! root hash over the payload for integrity verification.

pub mod merkle;
pub mod save;

pub use save::{SaveError, SaveHeader};
