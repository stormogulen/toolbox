
use crate::{FixedSmall, FixedPointError};
use super::FixedPointContainerTrait;

#[cfg(feature = "std_container")]
use bytemuck;

/// Standard container using Vec<FixedSmall<N, F>>
#[derive(Debug, Clone)]
pub struct FixedPointContainerStd<const N: usize, const F: usize> {
    data: Vec<FixedSmall<N, F>>,
}

impl<const N: usize, const F: usize> FixedPointContainerTrait<N, F> 
    for FixedPointContainerStd<N, F> 
{
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn with_capacity(cap: usize) -> Self {
        Self { data: Vec::with_capacity(cap) }
    }

    fn push(&mut self, value: FixedSmall<N, F>) -> Result<(), FixedPointError> {
        self.data.push(value);
        Ok(())
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    #[cfg(feature = "std_container")]
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }

    #[cfg(not(feature = "std_container"))]
    fn as_bytes(&self) -> &[u8] {
        &[]
    }

    fn get(&self, index: usize) -> Option<FixedSmall<N, F>> {
        self.data.get(index).copied()
    }

    fn as_slice(&self) -> Option<&[FixedSmall<N, F>]> {
        Some(&self.data)
    }

    fn as_mut_slice(&mut self) -> Option<&mut [FixedSmall<N, F>]> {
        Some(&mut self.data)
    }
}

impl<const N: usize, const F: usize> FixedPointContainerStd<N, F> {
    pub fn into_vec(self) -> Vec<FixedSmall<N, F>> {
        self.data
    }

    pub fn from_vec(data: Vec<FixedSmall<N, F>>) -> Self {
        Self { data }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

impl<const N: usize, const F: usize> Default for FixedPointContainerStd<N, F> {
    fn default() -> Self {
        Self::new()
    }
}
