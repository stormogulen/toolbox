//! PackedStruct: Safe byte buffers and typed containers for Pod types.
//!
//! This crate provides two main components:
//! - `PackedBytes`: A fixed-size byte buffer that can be safely cast to/from Pod types
//! - `PackedStructContainer`: A type-safe container for arrays of Pod types (requires `container` feature)
//!
//! ## Features
//! - `std` (default): Enable standard library support
//! - `container` (default): Enable `PackedStructContainer` with `raw_bytes::Container` backend
//! - `mmap`: Enable memory-mapped file support (requires `std` and `container`)
//!
//! ## no_std Support
//! To use in a `no_std` environment:
//! ```toml
//! [dependencies]
//! packed_struct = { version = "0.1", default-features = false }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

//use bytemuck::{Pod, Zeroable};

// Re-export bytemuck for convenience
pub use bytemuck;

// Core module - always available
//mod packed_bits;
#[cfg(feature = "bits")]
pub use packed_bits::*;

// Container module - only with container feature
#[cfg(feature = "container")]
pub mod container;
#[cfg(feature = "container")]
pub use container::*;
