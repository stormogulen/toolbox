//! # fixed_point
//!
//! A fixed-point arithmetic library with support for multiple storage backends.
//!
//! ## Features
//!
//! - Generic fixed-point numbers with compile-time precision specification
//! - Multiple storage backends:
//!   - `std_container`: Standard Vec-based storage (default)
//!   - `packed_container`: Bit-packed storage for memory efficiency
//! - Zero-copy byte access for serialization
//! - Iterator support
//! - Type-safe arithmetic operations
//!
//! ## Examples
//!
//! ```
//! use fixed_point::{FixedPointArray, FixedSmall};
//!
//! // Create a fixed-point array with 16 total bits, 8 fractional bits
//! let mut array: FixedPointArray<16, 8> = FixedPointArray::new();
//!
//! // Add values
//! let value = FixedSmall::<16, 8>::from_f32(3.14159)?;
//! array.push(value)?;
//!
//! // Convert back to f32
//! if let Some(v) = array.get(0) {
//!     println!("Value: {}", v.to_f32());
//! }
//!
//! // Create from iterator
//! let values = vec![1.0, 2.5, -3.75];
//! let array = FixedPointArray::<16, 8>::from_iter(values)?;
//! # Ok::<(), fixed_point::FixedPointError>(())
//! ```

pub mod fixed_small;
pub mod scalar_formats;
pub mod error;
pub mod container;

pub use fixed_small::FixedSmall;
pub use container::{FixedPointContainer, FixedPointContainerTrait};
pub use error::FixedPointError;

// A dynamic array of fixed-point numbers with configurable precision.
///
/// `FixedPointArray` provides a convenient interface for working with collections
/// of fixed-point values. The storage backend is selected at compile time via
/// feature flags.
///
/// # Type Parameters
///
/// - `N`: Total number of bits (including sign bit)
/// - `F`: Number of fractional bits
///
/// # Examples
///
/// ```
/// use fixed_point::{FixedPointArray, FixedSmall};
///
/// let mut array: FixedPointArray<16, 8> = FixedPointArray::new();
/// array.push(FixedSmall::from_f32(1.5)?)?;
/// array.push(FixedSmall::from_f32(-2.25)?)?;
///
/// assert_eq!(array.len(), 2);
/// assert_eq!(array.get(0).unwrap().to_f32(), 1.5);
/// # Ok::<(), fixed_point::FixedPointError>(())
/// ```
#[derive(Debug)]
pub struct FixedPointArray<const N: usize, const F: usize> {
    container: FixedPointContainer<N, F>,
}

impl<const N: usize, const F: usize> FixedPointArray<N, F> {

    /// Creates a new empty fixed-point array.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedPointArray;
    ///
    /// let array: FixedPointArray<16, 8> = FixedPointArray::new();
    /// assert_eq!(array.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { 
            container: FixedPointContainer::new() 
        }
    }

    // Creates a new empty fixed-point array with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedPointArray;
    ///
    /// let array: FixedPointArray<16, 8> = FixedPointArray::with_capacity(100);
    /// assert_eq!(array.len(), 0);
    /// ```
    pub fn with_capacity(cap: usize) -> Self {
        Self { 
            container: FixedPointContainer::with_capacity(cap) 
        }
    }

    /// Appends a fixed-point value to the array.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying container fails to allocate space.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::{FixedPointArray, FixedSmall};
    ///
    /// let mut array: FixedPointArray<16, 8> = FixedPointArray::new();
    /// array.push(FixedSmall::from_f32(3.14)?)?;
    /// assert_eq!(array.len(), 1);
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn push(&mut self, value: FixedSmall<N, F>) -> Result<(), FixedPointError> {
        self.container.push(value)
    }
    /// Returns the number of elements in the array.
    pub fn len(&self) -> usize {
        self.container.len()
    }

    /// Returns `true` if the array contains no elements.
    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }

    /// Returns the element at the given index, or `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::{FixedPointArray, FixedSmall};
    ///
    /// let mut array: FixedPointArray<16, 8> = FixedPointArray::new();
    /// array.push(FixedSmall::from_f32(2.5)?)?;
    ///
    /// assert_eq!(array.get(0).unwrap().to_f32(), 2.5);
    /// assert!(array.get(1).is_none());
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn get(&self, index: usize) -> Option<FixedSmall<N, F>> {
        self.container.get(index)
    }

    /// Returns a slice of the underlying data, if available.
    ///
    /// Note: This is only available with the `std_container` feature.
    pub fn as_slice(&self) -> Option<&[FixedSmall<N, F>]> {
        self.container.as_slice()
    }

    /// Returns a mutable slice of the underlying data, if available.
    ///
    /// Note: This is only available with the `std_container` feature.
    pub fn as_mut_slice(&mut self) -> Option<&mut [FixedSmall<N, F>]> {
        self.container.as_mut_slice()
    }

    /// Returns the raw bytes of the array for serialization.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::{FixedPointArray, FixedSmall};
    ///
    /// let mut array: FixedPointArray<16, 8> = FixedPointArray::new();
    /// array.push(FixedSmall::from_f32(1.0)?)?;
    ///
    /// let bytes = array.as_bytes();
    /// assert!(!bytes.is_empty());
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.container.as_bytes()
    }

    /// Creates a fixed-point array from an iterator of f32 values.
    ///
    /// # Errors
    ///
    /// Returns an error if any value is out of range for the fixed-point format.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedPointArray;
    ///
    /// let values = vec![1.0, 2.5, -3.75, 0.125];
    /// let array = FixedPointArray::<16, 8>::from_iter(values)?;
    /// assert_eq!(array.len(), 4);
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn from_iter<I>(iter: I) -> Result<Self, FixedPointError>
    where
        I: IntoIterator<Item = f32>,
    {
        let mut array = Self::new();
        for value in iter {
            let fixed = FixedSmall::from_f32(value)?;
            array.push(fixed)?;
        }
        Ok(array)
    }

    /// Converts the array to a vector of f32 values.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedPointArray;
    ///
    /// let values = vec![1.0, 2.5, -3.75];
    /// let array = FixedPointArray::<16, 8>::from_iter(values.clone())?;
    /// let result = array.to_f32_vec();
    ///
    /// assert_eq!(result.len(), 3);
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn to_f32_vec(&self) -> Vec<f32> {
        (0..self.len())
            .filter_map(|i| self.get(i))
            .map(|v| v.to_f32())
            .collect()
    }

    /// Returns an iterator over the fixed-point values.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedPointArray;
    ///
    /// let array = FixedPointArray::<16, 8>::from_iter(vec![1.0, 2.0, 3.0])?;
    ///
    /// for (i, value) in array.iter().enumerate() {
    ///     println!("Value {}: {}", i, value.to_f32());
    /// }
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn iter(&self) -> FixedPointIter<'_, N, F> {
        FixedPointIter {
            container: &self.container,
            index: 0,
            len: self.len(),
        }
    }
}

impl<const N: usize, const F: usize> Default for FixedPointArray<N, F> {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over fixed-point values in an array.
pub struct FixedPointIter<'a, const N: usize, const F: usize> {
    container: &'a FixedPointContainer<N, F>,
    index: usize,
    len: usize,
}

impl<'a, const N: usize, const F: usize> Iterator for FixedPointIter<'a, N, F> {
    type Item = FixedSmall<N, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = self.container.get(self.index);
            self.index += 1;
            item
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, const N: usize, const F: usize> ExactSizeIterator for FixedPointIter<'a, N, F> {
    fn len(&self) -> usize {
        self.len - self.index
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_creation() {
        let array: FixedPointArray<16, 8> = FixedPointArray::new();
        assert_eq!(array.len(), 0);
        assert!(array.is_empty());
    }

    #[test]
    fn test_array_push_and_get() {
        let mut array: FixedPointArray<16, 8> = FixedPointArray::new();
        let value = FixedSmall::from_f32(3.14).unwrap();
        
        array.push(value).unwrap();
        assert_eq!(array.len(), 1);
        
        let retrieved = array.get(0).unwrap();
        assert!((retrieved.to_f32() - 3.14).abs() < 0.01);
    }

    #[test]
    fn test_from_iter() {
        let values = vec![1.0, 2.5, -3.75, 0.125];
        let array = FixedPointArray::<16, 8>::from_iter(values.clone()).unwrap();
        
        assert_eq!(array.len(), 4);
        for (i, &expected) in values.iter().enumerate() {
            let actual = array.get(i).unwrap().to_f32();
            assert!((actual - expected).abs() < 0.01);
        }
    }

    #[test]
    fn test_iterator() {
        let values = vec![1.0, 2.0, 3.0];
        let array = FixedPointArray::<16, 8>::from_iter(values).unwrap();
        
        let collected: Vec<f32> = array.iter().map(|v| v.to_f32()).collect();
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn test_to_f32_vec() {
        let values = vec![1.0, 2.5, -3.75];
        let array = FixedPointArray::<16, 8>::from_iter(values.clone()).unwrap();
        let result = array.to_f32_vec();
        
        assert_eq!(result.len(), values.len());
        for (a, b) in result.iter().zip(values.iter()) {
            assert!((a - b).abs() < 0.01);
        }
    }
}
