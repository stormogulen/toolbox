//! # packed_bits
//!
//! A `no_std` compatible bit-packing library.
//!
//! ```rust
//! use packed_bits::PackedBitsContainer;
//!
//! // Store 12-bit values (0-4095)
//! let mut container = PackedBitsContainer::<12>::new_in_memory().expect("Failed to create container");
//! container.push(0xABC).unwrap();
//! container.push(0x123).unwrap();
//!
//! assert_eq!(container.get(0), Some(0xABC));
//! assert_eq!(container.get(1), Some(0x123));
//! ```
//!
//! ## Memory Savings Example
//!
//! ```rust
//! use packed_bits::PackedBitsContainer;
//!
//! // Standard Vec<u32>: 1000 elements × 4 bytes = 4000 bytes
//! let standard: Vec<u32> = (0..1000).collect();
//!
//! // PackedBitsContainer<10>: 1000 elements × 10 bits = 1250 bytes
//! let mut packed = PackedBitsContainer::<10>::new_in_memory().expect("Failed to create container");
//! for i in 0..1000 {
//!     packed.push(i % 1024).unwrap(); // values 0-1023 fit in 10 bits
//! }
//!
//! // 68.75% memory savings!
//! ```
//!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "container", not(feature = "std")))]
extern crate alloc;

pub mod error;
pub use error::PackedBitsError;

mod bit_ops;

#[cfg(feature = "container")]
pub mod container;

#[cfg(feature = "container")]
pub mod flags;

#[cfg(feature = "container")]
pub use container::PackedBitsContainer;

#[cfg(feature = "container")]
pub use flags::FlagsContainer;
