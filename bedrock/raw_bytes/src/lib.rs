#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
pub use std::{string::String, vec::Vec};

#[cfg(not(feature = "std"))]
pub use alloc::{string::String, vec::Vec};

pub mod container;
pub mod error;
#[doc(hidden)]
pub mod storage;

pub use container::Container;
pub use error::ContainerError;
pub use storage::Storage;
