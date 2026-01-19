#[cfg(feature = "packed_container")]
use crate::{FixedSmall, FixedPointError};
#[cfg(feature = "packed_container")]
use super::FixedPointContainerTrait;
#[cfg(feature = "packed_container")]
use packed_bits::PackedBits;

/// Packed container using PackedBits for memory-efficient storage
#[cfg(feature = "packed_container")]
#[derive(Debug)]
pub struct FixedPointContainerPacked<const N: usize, const F: usize> {
    bits: PackedBits<N>,
}

#[cfg(feature = "packed_container")]
impl<const N: usize, const F: usize> FixedPointContainerTrait<N, F> 
    for FixedPointContainerPacked<N, F> 
{
    fn new() -> Self {
        Self {
            bits: PackedBits::new()
                .expect("Failed to create PackedBits container"),
        }
    }

    fn with_capacity(cap: usize) -> Self {
        Self {
            bits: PackedBits::with_capacity(cap)
                .expect("Failed to create PackedBits container with capacity"),
        }
    }

    fn push(&mut self, value: FixedSmall<N, F>) -> Result<(), FixedPointError> {
        // Mask to N bits for packed storage
        let packed_value = if N < 32 {
            (value.raw as u32) & ((1u32 << N) - 1)
        } else {
            value.raw as u32
        };
        
        self.bits.push(packed_value)
            .map_err(FixedPointError::PackedBitsError)
    }

    fn len(&self) -> usize {
        self.bits.len()
    }

    fn as_bytes(&self) -> &[u8] {
        self.bits.as_bytes()
    }

    fn get(&self, index: usize) -> Option<FixedSmall<N, F>> {
        self.bits.get(index).map(|packed| {
            // Sign-extend if needed (if high bit is set and N < 32)
            let raw = if N < 32 && (packed & (1 << (N - 1))) != 0 {
                // Negative number - sign extend to i32
                (packed as i32) | (!0i32 << N)
            } else {
                packed as i32
            };
            FixedSmall { raw }
        })
    }
}

#[cfg(feature = "packed_container")]
impl<const N: usize, const F: usize> FixedPointContainerPacked<N, F> {
    pub fn capacity(&self) -> usize {
        self.bits.capacity()
    }

    /// Get the underlying PackedBits for advanced operations
    pub fn as_packed_bits(&self) -> &PackedBits<N> {
        &self.bits
    }

    pub fn as_packed_bits_mut(&mut self) -> &mut PackedBits<N> {
        &mut self.bits
    }
}

#[cfg(feature = "packed_container")]
impl<const N: usize, const F: usize> Default for FixedPointContainerPacked<N, F> {
    fn default() -> Self {
        Self::new()
    }
}