//! PackedStructContainer: A type-safe container for Pod structs.
//!
//! Provides a high-level interface over raw_bytes::Container for working with
//! arrays of Pod types, supporting both in-memory and memory-mapped storage.
//!
//! This module requires the `container` feature (enabled by default).

use bytemuck::Pod;
use raw_bytes::Container;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A container of packed Pod structs.
///
/// Can be backed by in-memory storage or memory-mapped files.
/// Provides zero-cost abstraction over byte arrays with type safety.
///
/// # Example
/// ```
/// use packed_structs::PackedStructContainer;
/// use bytemuck::{Pod, Zeroable};
/// use bytemuck_derive::{Pod, Zeroable};
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Pod, Zeroable)]
/// struct Point {
///     x: f32,
///     y: f32,
/// }
///
/// let mut container = PackedStructContainer::new();
/// container.push(Point { x: 1.0, y: 2.0 }).unwrap();
/// assert_eq!(container[0].x, 1.0);
/// ```
#[derive(Debug)]
pub struct PackedStructContainer<T: Pod + Copy> {
    storage: Container<T>,
    _marker: PhantomData<T>,
}

impl<T: Pod + Copy> PackedStructContainer<T> {
    /// Create an empty in-memory container.
    pub fn new() -> Self {
        Self {
            storage: Container::new(),
            _marker: PhantomData,
        }
    }

    /// Create an in-memory container with pre-allocated capacity for `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: Container::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    /// Create from a slice (in-memory).
    pub fn from_slice(data: &[T]) -> Self {
        Self::validate_alignment();
        Self {
            storage: Container::from_slice(data),
            _marker: PhantomData,
        }
    }

    /// Create from values (convenience method).
    pub fn from_values(values: &[T]) -> Self {
        Self::from_slice(values)
    }

    /// Open a memory-mapped file read-only.
    #[cfg(feature = "mmap")]
    pub fn open_mmap_read<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, raw_bytes::ContainerError> {
        Self::validate_alignment();
        Ok(Self {
            storage: Container::mmap_readonly(path)?,
            _marker: PhantomData,
        })
    }

    /// Open a memory-mapped file read-write.
    #[cfg(feature = "mmap")]
    pub fn open_mmap_rw<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, raw_bytes::ContainerError> {
        Self::validate_alignment();
        Ok(Self {
            storage: Container::mmap_readwrite(path)?,
            _marker: PhantomData,
        })
    }

    /// Validate that T has proper alignment for byte-level casting.
    fn validate_alignment() {
        // bytemuck already validates this at compile time via Pod trait,
        // but we add a runtime check for extra safety
        assert!(
            std::mem::align_of::<T>() <= 8,
            "Type alignment too strict for safe casting"
        );
    }

    /// Returns the number of elements in the container.
    #[inline]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Returns true if the container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Access as slice of T.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.storage.as_slice()
    }

    /// Access as mutable slice if storage is writable.
    ///
    /// Returns `None` if the storage is read-only (e.g., read-only mmap).
    #[inline]
    pub fn as_slice_mut(&mut self) -> Option<&mut [T]> {
        self.storage.as_mut_slice().ok()
    }

    /// Get element by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.storage.get(index).ok()
    }

    /// Get mutable reference to element by index.
    ///
    /// Returns `None` if index is out of bounds or storage is read-only.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.storage.get_mut(index).ok()
    }

    /// Append new elements (in-memory only).
    ///
    /// # Errors
    /// Returns an error if the storage is read-only or cannot be resized.
    pub fn append(&mut self, new: &[T]) -> Result<(), raw_bytes::ContainerError> {
        self.storage.extend_from_slice(new)
    }

    /// Append a single element.
    ///
    /// # Errors
    /// Returns an error if the storage is read-only or cannot be resized.
    pub fn push(&mut self, value: T) -> Result<(), raw_bytes::ContainerError> {
        self.storage.push(value)
    }

    /// Extend from an iterator.
    ///
    /// # Errors
    /// Returns an error if the storage is read-only or cannot be resized.
    pub fn extend<I>(&mut self, iter: I) -> Result<(), raw_bytes::ContainerError>
    where
        I: IntoIterator<Item = T>,
    {
        let values: Vec<T> = iter.into_iter().collect();
        self.storage.extend_from_slice(&values)
    }

    /// Reserve additional capacity (in-memory only).
    ///
    /// # Errors
    /// Returns an error if the storage is memory-mapped.
    pub fn reserve(&mut self, additional: usize) -> Result<(), raw_bytes::ContainerError> {
        self.storage.reserve(additional)
    }

    /// Clear all elements (in-memory only).
    ///
    /// # Errors
    /// Returns an error if the storage is read-only.
    pub fn clear(&mut self) -> Result<(), raw_bytes::ContainerError> {
        self.storage.clear()
    }

    /// Write a value to the element at the given index.
    ///
    /// # Errors
    /// Returns an error if index is out of bounds or storage is read-only.
    pub fn write(&mut self, index: usize, value: T) -> Result<(), raw_bytes::ContainerError> {
        self.storage.write(index, value)
    }

    /// Expose underlying storage for advanced use.
    pub fn storage(&self) -> &Container<T> {
        &self.storage
    }

    /// Mutable access to underlying storage for advanced use.
    pub fn storage_mut(&mut self) -> &mut Container<T> {
        &mut self.storage
    }

    /// Returns an iterator over the elements.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.storage.iter()
    }

    /// Returns a mutable iterator over the elements.
    ///
    /// Returns `None` if the storage is read-only.
    #[inline]
    pub fn iter_mut(&mut self) -> Option<std::slice::IterMut<'_, T>> {
        self.storage.iter_mut().ok()
    }
}

impl<T: Pod + Copy> Default for PackedStructContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Deref to slice for ergonomic access.
///
/// Allows using the container like a slice: `container[i]`, `container.len()`, etc.
impl<T: Pod + Copy> Deref for PackedStructContainer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// DerefMut for mutable slice access.
///
/// # Panics
/// Panics if the storage is read-only (e.g., read-only memory-mapped file).
/// Use `as_slice_mut()` for non-panicking access.
impl<T: Pod + Copy> DerefMut for PackedStructContainer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
            .expect("Cannot mutably dereference read-only storage")
    }
}

/// Iterator support - iterate over references to elements.
impl<'a, T: Pod + Copy> IntoIterator for &'a PackedStructContainer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Point {
        x: f32,
        y: f32,
    }

    // Safety: Point is repr(C) with only f32 fields, which are Pod
    unsafe impl bytemuck::Zeroable for Point {}
    unsafe impl bytemuck::Pod for Point {}

    #[test]
    fn test_new_and_push() {
        let mut container = PackedStructContainer::new();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());

        container.push(Point { x: 1.0, y: 2.0 }).unwrap();
        container.push(Point { x: 3.0, y: 4.0 }).unwrap();

        assert_eq!(container.len(), 2);
        assert_eq!(container[0].x, 1.0);
        assert_eq!(container[1].y, 4.0);
    }

    #[test]
    fn test_from_slice() {
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
        let container = PackedStructContainer::from_slice(&points);

        assert_eq!(container.len(), 2);
        assert_eq!(container[0], points[0]);
    }

    #[test]
    fn test_append() {
        let mut container = PackedStructContainer::new();
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];

        container.append(&points).unwrap();
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_extend() {
        let mut container = PackedStructContainer::new();
        let points = vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];

        container.extend(points).unwrap();
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_deref() {
        let mut container = PackedStructContainer::from_slice(&[
            Point { x: 1.0, y: 2.0 },
            Point { x: 3.0, y: 4.0 },
        ]);

        // Use as slice
        assert_eq!(container.len(), 2);

        // Mutable access via DerefMut
        container[0].x = 10.0;
        assert_eq!(container[0].x, 10.0);
    }

    #[test]
    fn test_iterator() {
        let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
        let container = PackedStructContainer::from_slice(&points);

        let collected: Vec<_> = container.iter().cloned().collect();
        assert_eq!(collected, points);

        // Also test IntoIterator
        let collected2: Vec<_> = (&container).into_iter().cloned().collect();
        assert_eq!(collected2, points);
    }

    #[test]
    fn test_get_mut() {
        let mut container = PackedStructContainer::from_slice(&[Point { x: 1.0, y: 2.0 }]);

        if let Some(point) = container.get_mut(0) {
            point.x = 100.0;
        }

        assert_eq!(container[0].x, 100.0);
    }

    #[test]
    fn test_with_capacity() {
        let mut container = PackedStructContainer::<Point>::with_capacity(100);
        assert_eq!(container.len(), 0);

        // Add some elements to verify it works
        for i in 0..10 {
            container
                .push(Point {
                    x: i as f32,
                    y: i as f32 * 2.0,
                })
                .unwrap();
        }
        assert_eq!(container.len(), 10);
    }

    #[test]
    fn test_clear() {
        let mut container = PackedStructContainer::from_slice(&[
            Point { x: 1.0, y: 2.0 },
            Point { x: 3.0, y: 4.0 },
        ]);

        assert_eq!(container.len(), 2);
        container.clear().unwrap();
        assert_eq!(container.len(), 0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_write() {
        let mut container = PackedStructContainer::from_slice(&[
            Point { x: 1.0, y: 2.0 },
            Point { x: 3.0, y: 4.0 },
        ]);

        container.write(0, Point { x: 99.0, y: 88.0 }).unwrap();
        assert_eq!(container[0].x, 99.0);
        assert_eq!(container[0].y, 88.0);
    }

    #[test]
    fn test_reserve() {
        let mut container = PackedStructContainer::<Point>::new();
        container.reserve(100).unwrap();
        
        // Should be able to add elements without reallocation
        for i in 0..50 {
            container.push(Point { x: i as f32, y: 0.0 }).unwrap();
        }
        assert_eq!(container.len(), 50);
    }
}