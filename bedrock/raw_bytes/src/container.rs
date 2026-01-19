use crate::{ContainerError, Storage};
use bytemuck::Pod;

/// High-level container for typed elements backed by different storage mechanisms.
///
/// `Container<T>` provides a unified interface over in-memory vectors and memory-mapped
/// files, where `T` must implement [`bytemuck::Pod`] (Plain Old Data) for safe
/// zero-copy operations.
///
/// # Storage Backends
///
/// - **In-memory**: Standard heap-allocated vector, supports dynamic growth
/// - **Memory-mapped read-only**: Fast read access to large on-disk datasets
/// - **Memory-mapped read-write**: Persistent storage with in-place updates
///
/// # Examples
///
/// ## Basic In-Memory Usage
///
/// ```
/// use raw_bytes::Container;
/// use bytemuck_derive::{Pod, Zeroable};
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
/// struct Packet {
///     id: u32,
///     value: f32,
/// }
///
/// let mut container = Container::<Packet>::new();
///
/// // Push elements
/// container.push(Packet { id: 1, value: 10.0 }).unwrap();
/// container.push(Packet { id: 2, value: 20.0 }).unwrap();
///
/// // Read elements
/// assert_eq!(container.len(), 2);
/// assert_eq!(container.get(0).unwrap().id, 1);
///
/// // Modify elements
/// container.write(0, Packet { id: 99, value: 99.0 }).unwrap();
/// assert_eq!(container.get(0).unwrap().id, 99);
/// ```
///
/// ## Using Iterators
///
/// ```
/// use raw_bytes::Container;
/// use bytemuck_derive::{Pod, Zeroable};
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Pod, Zeroable)]
/// struct Point { x: f32, y: f32 }
///
/// let data = vec![
///     Point { x: 1.0, y: 2.0 },
///     Point { x: 3.0, y: 4.0 },
/// ];
/// let container = Container::from_slice(&data);
///
/// // Sum all x coordinates
/// let sum_x: f32 = container.iter().map(|p| p.x).sum();
/// assert_eq!(sum_x, 4.0);
/// ```
///
/// ## Memory-Mapped Files
///
/// ```no_run
/// # #[cfg(feature = "mmap")]
/// # {
/// use raw_bytes::Container;
/// use bytemuck_derive::{Pod, Zeroable};
/// use std::io::Write;
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Pod, Zeroable, Debug)]
/// struct Record {
///     id: u32,
///     value: f32,
/// }
///
/// // Create a data file
/// let mut file = std::fs::File::create("data.bin").unwrap();
/// let records = [
///     Record { id: 1, value: 10.0 },
///     Record { id: 2, value: 20.0 },
/// ];
/// file.write_all(bytemuck::cast_slice(&records)).unwrap();
/// drop(file);
///
/// // Open as memory-mapped read-only
/// let container = Container::<Record>::mmap_readonly("data.bin").unwrap();
/// assert_eq!(container.len(), 2);
/// assert_eq!(container.get(0).unwrap().id, 1);
///
/// // Cleanup
/// std::fs::remove_file("data.bin").unwrap();
/// # }
/// ```
#[derive(Debug)]
pub struct Container<T: Pod> {
    storage: Storage<T>,
}

impl<T: Pod> Container<T> {
    /// Creates an empty in-memory container.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Data { value: u32 }
    ///
    /// let mut container = Container::<Data>::new();
    /// assert!(container.is_empty());
    /// container.push(Data { value: 42 }).unwrap();
    /// assert_eq!(container.len(), 1);
    /// ```
    pub fn new() -> Self {
        Container {
            storage: Storage::new_in_memory(),
        }
    }

    /// Creates an in-memory container with pre-allocated capacity.
    ///
    /// This is more efficient when you know the approximate number of elements
    /// you'll be storing, as it avoids multiple reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Point { x: f32, y: f32 }
    ///
    /// let mut container = Container::<Point>::with_capacity(1000);
    ///
    /// // Can push up to 1000 elements without reallocation
    /// for i in 0..1000 {
    ///     container.push(Point { x: i as f32, y: 0.0 }).unwrap();
    /// }
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Container {
            storage: Storage::InMemory(Vec::with_capacity(capacity)),
        }
    }

    /// Creates an in-memory container from a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
    /// struct Value { data: u64 }
    ///
    /// let data = vec![
    ///     Value { data: 1 },
    ///     Value { data: 2 },
    ///     Value { data: 3 },
    /// ];
    ///
    /// let container = Container::from_slice(&data);
    /// assert_eq!(container.len(), 3);
    /// assert_eq!(container.get(1).unwrap(), &Value { data: 2 });
    /// ```
    pub fn from_slice(values: &[T]) -> Self {
        Container {
            storage: Storage::InMemory(values.to_vec()),
        }
    }

    /// Opens a memory-mapped file for read-only access.
    ///
    /// This provides fast, zero-copy access to large datasets stored on disk.
    /// The file must contain a valid sequence of `T` values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist or can't be opened
    /// - The file size isn't a multiple of `size_of::<T>()`
    /// - Memory mapping fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mmap")]
    /// # {
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Sensor { id: u32, reading: f32 }
    ///
    /// // Create data file
    /// let mut file = std::fs::File::create("sensors.bin").unwrap();
    /// let sensors = [Sensor { id: 1, reading: 23.5 }];
    /// file.write_all(bytemuck::cast_slice(&sensors)).unwrap();
    /// drop(file);
    ///
    /// // Memory-map for fast access
    /// let container = Container::<Sensor>::mmap_readonly("sensors.bin").unwrap();
    /// assert_eq!(container.len(), 1);
    ///
    /// # std::fs::remove_file("sensors.bin").unwrap();
    /// # }
    /// ```
    #[cfg(feature = "mmap")]
    pub fn mmap_readonly<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ContainerError> {
        let storage = Storage::from_mmap_readonly(path.as_ref())?;

        // Validate that we can actually cast this memory
        if let Storage::MmapReadOnly(ref m) = storage {
            validate_mmap_layout::<T>(m.as_ref())?;
        }

        Ok(Container { storage })
    }

    /// Opens a memory-mapped file for read-write access.
    ///
    /// Changes made to the container are persisted directly to the file.
    /// The file must already exist and contain valid data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist or can't be opened for writing
    /// - The file size isn't a multiple of `size_of::<T>()`
    /// - Memory mapping fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "mmap")]
    /// # {
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
    /// struct Score { player_id: u32, points: u32 }
    ///
    /// // Create initial data
    /// let mut file = std::fs::File::create("scores.bin").unwrap();
    /// let scores = [Score { player_id: 1, points: 100 }];
    /// file.write_all(bytemuck::cast_slice(&scores)).unwrap();
    /// drop(file);
    ///
    /// // Open for read-write
    /// let mut container = Container::<Score>::mmap_readwrite("scores.bin").unwrap();
    ///
    /// // Modify data (changes persist to file!)
    /// container.write(0, Score { player_id: 1, points: 200 }).unwrap();
    /// assert_eq!(container.get(0).unwrap().points, 200);
    ///
    /// # std::fs::remove_file("scores.bin").unwrap();
    /// # }
    /// ```
    #[cfg(feature = "mmap")]
    pub fn mmap_readwrite<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ContainerError> {
        let storage = Storage::from_mmap_readwrite(path.as_ref())?;

        // Validate that we can cast this memory to T
        if let Storage::MmapReadWrite(ref m) = storage {
            validate_mmap_layout::<T>(m.as_ref())?;
        }

        Ok(Container { storage })
    }

    /// Validates that the mmap bytes can be safely cast to `T`.
    // #[cfg(feature = "mmap")]
    // fn validate_mmap_layout<T: Pod>(bytes: &[u8]) -> Result<(), ContainerError> {
    //     bytemuck::try_cast_slice::<u8, T>(bytes)
    //         .map(|_| ())
    //         .map_err(|e| {
    //             #[cfg(feature = "std")]
    //             return ContainerError::Io(std::io::Error::new(
    //                 std::io::ErrorKind::InvalidData,
    //                 format!("Invalid mmap layout for type {}: {:?}", 
    //                         core::any::type_name::<T>(), e)
    //             ));
    //             #[cfg(not(feature = "std"))]
    //             return ContainerError::Io("Invalid mmap layout");
    //         })
    // }

    /// Returns the number of elements in the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Item { id: u32 }
    ///
    /// let mut c = Container::<Item>::new();
    /// assert_eq!(c.len(), 0);
    ///
    /// c.push(Item { id: 1 }).unwrap();
    /// assert_eq!(c.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Returns `true` if the container contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Item { id: u32 }
    ///
    /// let mut c = Container::<Item>::new();
    /// assert!(c.is_empty());
    ///
    /// c.push(Item { id: 1 }).unwrap();
    /// assert!(!c.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Returns a reference to the element at the given index.
    ///
    /// # Errors
    ///
    /// Returns `ContainerError::OutOfBounds` if `index >= len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
    /// struct Point { x: i32, y: i32 }
    ///
    /// let data = [Point { x: 10, y: 20 }, Point { x: 30, y: 40 }];
    /// let c = Container::from_slice(&data);
    ///
    /// assert_eq!(c.get(0).unwrap(), &Point { x: 10, y: 20 });
    /// assert_eq!(c.get(1).unwrap(), &Point { x: 30, y: 40 });
    /// assert!(c.get(2).is_err());
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Result<&T, ContainerError> {
        self.storage.get(index)
    }

    /// Returns a mutable reference to the element at the given index.
    ///
    /// Only available for in-memory containers and read-write memory-mapped files.
    ///
    /// # Errors
    ///
    /// - `ContainerError::OutOfBounds` if `index >= len()`
    /// - `ContainerError::Io` if storage is read-only
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Counter { count: u32 }
    ///
    /// let mut c = Container::from_slice(&[Counter { count: 5 }]);
    ///
    /// c.get_mut(0).unwrap().count += 10;
    /// assert_eq!(c.get(0).unwrap().count, 15);
    /// ```
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Result<&mut T, ContainerError> {
        self.storage.get_mut(index)
    }

    /// Writes a value to the element at the given index.
    ///
    /// This is a convenience method equivalent to `*container.get_mut(index)? = value`.
    ///
    /// # Errors
    ///
    /// - `ContainerError::OutOfBounds` if `index >= len()`
    /// - `ContainerError::Io` if storage is read-only
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
    /// struct Record { id: u32 }
    ///
    /// let mut c = Container::from_slice(&[Record { id: 1 }, Record { id: 2 }]);
    ///
    /// c.write(0, Record { id: 99 }).unwrap();
    /// assert_eq!(c.get(0).unwrap(), &Record { id: 99 });
    /// ```
    pub fn write(&mut self, index: usize, value: T) -> Result<(), ContainerError> {
        let slot = self.storage.get_mut(index)?;
        *slot = value;
        Ok(())
    }

    /// Appends an element to the back of the container.
    ///
    /// Only available for in-memory containers.
    ///
    /// # Errors
    ///
    /// Returns `ContainerError::Io` if the container is backed by a memory-mapped file.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Event { timestamp: u64 }
    ///
    /// let mut c = Container::<Event>::new();
    ///
    /// c.push(Event { timestamp: 100 }).unwrap();
    /// c.push(Event { timestamp: 200 }).unwrap();
    ///
    /// assert_eq!(c.len(), 2);
    /// ```
    pub fn push(&mut self, value: T) -> Result<(), ContainerError> {
        self.storage.push(value)
    }

    /// Extend with elements from slice (InMemory only)
    pub fn extend_from_slice(&mut self, values: &[T]) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.extend_from_slice(values);
                Ok(())
            }
            #[cfg(feature = "mmap")]
            _ => {
                #[cfg(feature = "std")]
                return Err(ContainerError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot extend mmap storage",
                )));
                #[cfg(not(feature = "std"))]
                return Err(ContainerError::Io("Cannot extend mmap storage"));
            }
        }
    }

    /// Reserve additional capacity (InMemory only)
    pub fn reserve(&mut self, additional: usize) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.reserve(additional);
                Ok(())
            }
            #[cfg(feature = "mmap")]
            _ => {
                #[cfg(feature = "std")]
                return Err(ContainerError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot reserve on mmap storage",
                )));
                #[cfg(not(feature = "std"))]
                return Err(ContainerError::Io("Cannot reserve on mmap storage"));
            }
        }
    }

    /// Clear all elements (InMemory only)
    pub fn clear(&mut self) -> Result<(), ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => {
                vec.clear();
                Ok(())
            }
            #[cfg(feature = "mmap")]
            _ => {
                #[cfg(feature = "std")]
                return Err(ContainerError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot clear mmap storage",
                )));
                #[cfg(not(feature = "std"))]
                return Err(ContainerError::Io("Cannot clear mmap storage"));
            }
        }
    }

    /// Returns an immutable slice view of all elements.
    ///
    /// This is the most efficient way to iterate over elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Value { data: f32 }
    ///
    /// let c = Container::from_slice(&[
    ///     Value { data: 1.0 },
    ///     Value { data: 2.0 },
    ///     Value { data: 3.0 },
    /// ]);
    ///
    /// let sum: f32 = c.as_slice().iter().map(|v| v.data).sum();
    /// assert_eq!(sum, 6.0);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        match &self.storage {
            Storage::InMemory(vec) => vec.as_slice(),

            #[cfg(feature = "mmap")]
            Storage::MmapReadOnly(m) => bytemuck::cast_slice(m.as_ref()),
            
            #[cfg(feature = "mmap")]
            Storage::MmapReadWrite(m) => bytemuck::cast_slice(m.as_ref()),
        }
    }

    /// Get mutable slice (only for InMemory and MmapReadWrite)
    pub fn as_mut_slice(&mut self) -> Result<&mut [T], ContainerError> {
        match &mut self.storage {
            Storage::InMemory(vec) => Ok(vec.as_mut_slice()),
            #[cfg(feature = "mmap")]
            Storage::MmapReadOnly(_) => {
                #[cfg(feature = "std")]
                return Err(ContainerError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Cannot get mutable slice from read-only storage",
                )));
                #[cfg(not(feature = "std"))]
                return Err(ContainerError::Io(
                    "Cannot get mutable slice from read-only storage",
                ));
            }
            #[cfg(feature = "mmap")]
            Storage::MmapReadWrite(m) => Ok(bytemuck::cast_slice_mut(m.as_mut())),
        }
    }

    /// Returns an iterator over elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_bytes::Container;
    /// use bytemuck_derive::{Pod, Zeroable};
    ///
    /// #[repr(C)]
    /// #[derive(Clone, Copy, Pod, Zeroable)]
    /// struct Score { points: u32 }
    ///
    /// let c = Container::from_slice(&[
    ///     Score { points: 100 },
    ///     Score { points: 200 },
    ///     Score { points: 300 },
    /// ]);
    ///
    /// let total: u32 = c.iter().map(|s| s.points).sum();
    /// assert_eq!(total, 600);
    /// ```
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Get mutable iterator (only for InMemory and MmapReadWrite)
    pub fn iter_mut(&mut self) -> Result<core::slice::IterMut<'_, T>, ContainerError> {
        Ok(self.as_mut_slice()?.iter_mut())
    }
}

// Implement Index for convenient access
impl<T: Pod> core::ops::Index<usize> for Container<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T: Pod> core::ops::IndexMut<usize> for Container<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

// Default implementation
impl<T: Pod> Default for Container<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "mmap")]
fn validate_mmap_layout<T: Pod>(bytes: &[u8]) -> Result<(), ContainerError> {
    bytemuck::try_cast_slice::<u8, T>(bytes)
        .map(|_| ())
        .map_err(|e| {
            #[cfg(feature = "std")]
            return ContainerError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid mmap layout for type {}: {:?}", 
                        core::any::type_name::<T>(), e)
            ));
            #[cfg(not(feature = "std"))]
            return ContainerError::Io("Invalid mmap layout");
        })
}

impl<T: Pod> Container<T> {
    // ... rest of your impl
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck_derive::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
    struct Packet {
        id: u32,
        value: f32,
    }

    #[test]
    fn in_memory_basic_operations() -> Result<(), ContainerError> {
        let mut c = Container::<Packet>::new();
        assert!(c.is_empty());

        let p1 = Packet { id: 1, value: 10.0 };
        let p2 = Packet { id: 2, value: 20.0 };

        c.push(p1)?;
        c.push(p2)?;
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(0)?, &p1);
        assert_eq!(c.get(1)?, &p2);

        // Modify using write
        let p3 = Packet { id: 3, value: 30.0 };
        c.write(1, p3)?;
        assert_eq!(c.get(1)?, &p3);

        // Modify using get_mut
        c.get_mut(0)?.value = 99.0;
        assert_eq!(c.get(0)?.value, 99.0);

        Ok(())
    }

    #[test]
    fn index_operations() {
        let mut c = Container::<Packet>::from_slice(&[
            Packet { id: 1, value: 10.0 },
            Packet { id: 2, value: 20.0 },
        ]);

        assert_eq!(c[0].id, 1);
        assert_eq!(c[1].value, 20.0);

        c[1].value = 42.0;
        assert_eq!(c[1].value, 42.0);
    }

    #[test]
    fn slice_operations() {
        let data = vec![
            Packet { id: 1, value: 10.0 },
            Packet { id: 2, value: 20.0 },
            Packet { id: 3, value: 30.0 },
        ];

        let c = Container::<Packet>::from_slice(&data);

        let slice = c.as_slice();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[1].id, 2);
    }

    #[test]
    fn iterator_operations() {
        let data = vec![Packet { id: 1, value: 10.0 }, Packet { id: 2, value: 20.0 }];

        let c = Container::<Packet>::from_slice(&data);

        let sum: f32 = c.iter().map(|p| p.value).sum();
        assert_eq!(sum, 30.0);
    }

    #[test]
    fn extend_and_reserve() -> Result<(), ContainerError> {
        let mut c = Container::<Packet>::with_capacity(10);

        c.push(Packet { id: 1, value: 10.0 })?;

        c.extend_from_slice(&[Packet { id: 2, value: 20.0 }, Packet { id: 3, value: 30.0 }])?;

        assert_eq!(c.len(), 3);
        c.reserve(10)?;

        Ok(())
    }

    #[test]
    fn clear_operation() -> Result<(), ContainerError> {
        let mut c = Container::<Packet>::from_slice(&[
            Packet { id: 1, value: 10.0 },
            Packet { id: 2, value: 20.0 },
        ]);

        assert_eq!(c.len(), 2);
        c.clear()?;
        assert_eq!(c.len(), 0);
        assert!(c.is_empty());

        Ok(())
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_readonly_operations() -> Result<(), ContainerError> {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new()?;
        let packets = [Packet { id: 1, value: 10.0 }, Packet { id: 2, value: 20.0 }];
        let bytes: &[u8] = bytemuck::cast_slice(&packets);
        file.write_all(bytes)?;
        file.flush()?;

        let mut c = Container::<Packet>::mmap_readonly(file.path())?;
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(0)?, &packets[0]);

        // These should all fail on readonly
        assert!(
            c.write(
                0,
                Packet {
                    id: 99,
                    value: 99.0
                }
            )
            .is_err()
        );
        assert!(c.get_mut(0).is_err());
        assert!(c.as_mut_slice().is_err());

        Ok(())
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_readwrite_operations() -> Result<(), ContainerError> {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new()?;
        let packets = [Packet { id: 1, value: 10.0 }, Packet { id: 2, value: 20.0 }];
        let bytes: &[u8] = bytemuck::cast_slice(&packets);
        file.write_all(bytes)?;
        file.flush()?;

        let mut c = Container::<Packet>::mmap_readwrite(file.path())?;
        assert_eq!(c.len(), 2);

        // Can read
        assert_eq!(c.get(0)?, &packets[0]);

        // Can write
        c.write(
            0,
            Packet {
                id: 99,
                value: 99.0,
            },
        )?;
        assert_eq!(c.get(0)?.id, 99);

        // Can get mutable slice
        let slice = c.as_mut_slice()?;
        slice[1].value = 42.0;
        assert_eq!(c.get(1)?.value, 42.0);

        // Cannot push/extend
        assert!(c.push(Packet { id: 3, value: 3.0 }).is_err());

        Ok(())
    }
}
