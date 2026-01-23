//! # Packed Data
//!
//! A unified, ergonomic API for efficient binary data handling in Rust.
//!
//! ## Features
//!
//! - **Bit-level packing**: Pack booleans and small integers efficiently
//! - **Fixed-point math**: Fast deterministic arithmetic without floating point
//! - **Self-describing formats**: MTF format with runtime reflection
//! - **Compact structs**: #[repr(C, packed)] with bit-field support
//! - **Zero-copy operations**: Direct memory manipulation with bytemuck
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use packed_data::prelude::*;
//!
//! // Define a compact game entity
//! #[derive(PackedStruct, MTF, Copy, Clone)]
//! #[repr(C)]
//! struct Entity {
//!     x: Fixed16,      // 16-bit fixed-point position
//!     y: Fixed16,
//!     health: u8,      // 0-255
//!     team: u8,        // Team ID
//! }
//!
//! // Create and pack
//! let entity = Entity {
//!     x: Fixed16::from_f32(10.5),
//!     y: Fixed16::from_f32(20.75),
//!     health: 100,
//!     team: 1,
//! };
//!
//! // Save with metadata for introspection
//! save_with_metadata("entities.dat", &[entity])?;
//!
//! // Load dynamically
//! let container = load_dynamic("entities.dat")?;
//! for i in container.iter() {
//!     let health: &u8 = container.field(i, "health")?;
//!     println!("Entity {} health: {}", i, health);
//! }
//! ```

pub mod prelude;
pub mod builder;
pub mod convert;
pub mod iter;
pub mod io;

pub use crate::convert::{ToBytes, FromBytes, PackedConvert, batch, try_parse_iter, parse_with};
pub use crate::iter::{iter_parse, SliceParseExt};

// Re-export MTF types
pub use mtf::{MTF, MTFType, MTFError};
pub use mtf::dynamic::{DynamicContainer, FieldHandle, DynamicContainerIter};

// Re-export fixed_point types
pub use fixed_point::{
    scalar_formats::{Fixed10_6, Fixed16_16, Fixed24_8, Fixed4_12, Fixed8_8},
    FixedSmall, FixedPointArray, FixedPointIter,
    FixedPointError, FixedPointContainer, FixedPointContainerTrait,
};

// Re-export packed_bits types
pub use packed_bits::{
    PackedBitsContainer, FlagsContainer,
    flags::FlagsIter,
    PackedBitsError,
};

// Re-export packed_struct types
pub use packed_structs::{
    PackedStructContainer,
    bytemuck,
};

// Re-export raw_bytes types
pub use raw_bytes::{Container, Storage, ContainerError};

// Re-export save system
#[cfg(feature = "verified")]
pub use save::{
    save, load,
    //merkle::MerkleNode, merkle::build_merkle_tree, merkle::verify_merkle_tree,
};

// Re-export for convenience
pub use bytemuck::{Pod, Zeroable};


