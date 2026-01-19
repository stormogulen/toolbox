use thiserror::Error;

/// Errors for fixed-point operations
#[derive(Debug, Error)]
pub enum FixedPointError {
    #[error("Value {value} out of range for fixed-point format {bits}.{fractional}")]
    Overflow {
        value: f32,
        bits: usize,
        fractional: usize,
    },
    
    /// Error from the underlying PackedBits container.
    #[cfg(feature = "packed_container")]
    #[error("PackedBits operation failed: {0}")]
    PackedBitsError(#[from] packed_bits::PackedBitsError),
}


#[cfg(feature = "packed_container")]
impl From<packed_bits::PackedBitsError> for FixedPointError {
    fn from(err: packed_bits::PackedBitsError) -> Self {
        FixedPointError::PackedBitsError(err.to_string())
    }
}