
//! Convenience re-exports for common usage

pub use crate::builder::{PackedDataBuilder, EntityBuilder};
pub use crate::convert::{ToBytes, FromBytes, PackedConvert};
pub use crate::io::{save_with_metadata, load_dynamic, save_raw, load_raw};

// MTF types
pub use mtf::{MTF, MTFType};
pub use mtf::dynamic::{DynamicContainer, FieldHandle};

// Fixed-point types
pub use fixed_point::scalar_formats::{
    Fixed16_16, Fixed24_8, Fixed8_8, // Most common formats
};
pub use fixed_point::{FixedSmall, FixedPointArray};


// Packed types
pub use packed_bits::{PackedBitsContainer, FlagsContainer};
pub use packed_structs::PackedStructContainer;

// Raw bytes
pub use raw_bytes::Container;

// Bytemuck utilities
pub use bytemuck::{Pod, Zeroable, cast_slice, cast_slice_mut};

// Re-export verified save functions
#[cfg(feature = "verified")]
pub use save::{save, load};

// Common result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
