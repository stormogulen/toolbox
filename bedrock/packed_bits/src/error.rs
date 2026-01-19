use raw_bytes::ContainerError;
#[cfg(feature = "std")]
use thiserror::Error;

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug)]
pub enum PackedBitsError {
    #[cfg_attr(
        feature = "std",
        error("Bit width N must be in the range 1..=32, got {0}")
    )]
    InvalidBitWidth(usize),

    #[cfg_attr(feature = "std", error("Value {0} does not fit in {1} bits"))]
    ValueOverflow(u32, usize),

    #[cfg_attr(feature = "std", error("Index {0} is out of bounds for length {1}"))]
    IndexOutOfBounds(usize, usize),

    #[cfg_attr(feature = "std", error("Insufficient bytes for {0} elements"))]
    InsufficientBytes(usize),

    #[cfg_attr(feature = "std", error("invalid magic bytes in storage"))]
    InvalidMagic,

    #[cfg_attr(
        feature = "std",
        error("N mismatch: expected {expected}, found {found}")
    )]
    InvalidN { expected: usize, found: u32 },

    #[cfg_attr(feature = "std", error("storage too small for header"))]
    StorageTooSmall,

    #[cfg_attr(feature = "std", error("storage is read-only"))]
    StorageReadOnly,

    #[cfg_attr(feature = "std", error("failed to resize storage"))]
    ResizeFailed,

    #[cfg_attr(feature = "std", error("storage error: {0}"))]
    Container(#[from] ContainerError),

    #[cfg_attr(feature = "std", error("Unexpected error"))]
    Unexpected,
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for PackedBitsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PackedBitsError::InvalidBitWidth(n) => {
                write!(f, "Bit width N must be in the range 1..=32, got {}", n)
            }
            PackedBitsError::ValueOverflow(v, n) => {
                write!(f, "Value {} does not fit in {} bits", v, n)
            }
            PackedBitsError::IndexOutOfBounds(i, l) => {
                write!(f, "Index {} is out of bounds for length {}", i, l)
            }
            PackedBitsError::InsufficientBytes(n) => {
                write!(f, "Insufficient bytes for {} elements", n)
            }
            PackedBitsError::InvalidMagic => write!(f, "invalid magic bytes in storage"),
            PackedBitsError::InvalidN { expected, found } => {
                write!(f, "N mismatch: expected {}, found {}", expected, found)
            }
            PackedBitsError::StorageTooSmall => write!(f, "storage too small for header"),
            PackedBitsError::StorageReadOnly => write!(f, "storage is read-only"),
            PackedBitsError::ResizeFailed => write!(f, "failed to resize storage"),
            PackedBitsError::Storage(e) => write!(f, "storage error: {}", e),
            PackedBitsError::Unexpected => write!(f, "Unexpected error"),
        }
    }

    fn from(err: ContainerError) -> Self {
        PackedBitsError::Storage(err)
    }
}
