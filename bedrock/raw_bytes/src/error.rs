#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use thiserror::Error;

/// Container errors
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug)]
pub enum ContainerError {
    /// IO error (only available in std builds)
    #[cfg(feature = "std")]
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Fallback IO error for no_std builds
    #[cfg(not(feature = "std"))]
    Io(&'static str),

    /// Out-of-bounds access
    #[cfg(feature = "std")]
    #[error("Index {0} out of bounds")]
    OutOfBounds(usize),

    #[cfg(not(feature = "std"))]
    OutOfBounds(usize),
}
