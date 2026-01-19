pub mod std_container;

#[cfg(feature = "packed_container")]
pub mod packed_container;

use crate::{FixedSmall, FixedPointError};

/// Common trait for all fixed-point container implementations
pub trait FixedPointContainerTrait<const N: usize, const F: usize>: Sized {
    fn new() -> Self;
    fn with_capacity(cap: usize) -> Self;
    fn push(&mut self, value: FixedSmall<N, F>) -> Result<(), FixedPointError>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn as_bytes(&self) -> &[u8];
    
    // Optional indexed access
    fn get(&self, index: usize) -> Option<FixedSmall<N, F>>;
    fn as_slice(&self) -> Option<&[FixedSmall<N, F>]> {
        None
    }
    fn as_mut_slice(&mut self) -> Option<&mut [FixedSmall<N, F>]> {
        None
    }
}

// Type alias for the selected container implementation
#[cfg(feature = "std_container")]
pub type FixedPointContainer<const N: usize, const F: usize> = 
    std_container::FixedPointContainerStd<N, F>;

#[cfg(feature = "packed_container")]
pub type FixedPointContainer<const N: usize, const F: usize> = 
    packed_container::FixedPointContainerPacked<N, F>;

