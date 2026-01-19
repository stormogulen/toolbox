

use bytemuck::Pod;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::{fs::File, io::Error as IoError, path::Path};

#[cfg(feature = "mmap")]
use memmap2::{Mmap, MmapMut};

use crate::ContainerError;

/// The low-level storage backend for raw bytes.
///
/// - Always includes in-memory Vec<T>
/// - Includes mmap only when feature = "mmap"
#[derive(Debug)]
pub enum Storage<T: Pod> {
    /// Standard in-memory vector
    InMemory(Vec<T>),

    /// Read-only memory mapped file
    #[cfg(feature = "mmap")]
    MmapReadOnly(Mmap),

    /// Read-write memory mapped file
    #[cfg(feature = "mmap")]
    MmapReadWrite(MmapMut),
}

impl<T: Pod> Storage<T> {
    /// Create empty in-memory storage
    pub fn new_in_memory() -> Self {
        Storage::InMemory(Vec::new())
    }

    /// Return element count
    pub fn len(&self) -> usize {
        match self {
            Storage::InMemory(vec) => vec.len(),

            #[cfg(feature = "mmap")]
            Storage::MmapReadOnly(m) => m.len() / core::mem::size_of::<T>(),

            #[cfg(feature = "mmap")]
            Storage::MmapReadWrite(m) => m.len() / core::mem::size_of::<T>(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push an element — only valid for InMemory
    pub fn push(&mut self, value: T) -> Result<(), ContainerError> {
        match self {
            Storage::InMemory(vec) => {
                vec.push(value);
                Ok(())
            }

            #[cfg(feature = "mmap")]
            Storage::MmapReadOnly(_) | Storage::MmapReadWrite(_) => {
                #[cfg(feature = "std")]
                return Err(ContainerError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot push to mmap storage (fixed size)",
                )));

                #[cfg(not(feature = "std"))]
                return Err(ContainerError::Io(
                    "Cannot push to mmap storage (fixed size)",
                ));
            }
        }
    }

    /// Read a reference to element i
    pub fn get(&self, index: usize) -> Result<&T, ContainerError> {
        if index >= self.len() {
            return Err(ContainerError::OutOfBounds(index));
        }

        let elem_size = core::mem::size_of::<T>();
        let offset = index * elem_size;

        match self {
            Storage::InMemory(vec) => Ok(&vec[index]),

            #[cfg(feature = "mmap")]
            Storage::MmapReadOnly(m) => {
                let ptr = &m[offset] as *const u8 as *const T;
                Ok(unsafe { &*ptr })
            }

            #[cfg(feature = "mmap")]
            Storage::MmapReadWrite(m) => {
                let ptr = &m[offset] as *const u8 as *const T;
                Ok(unsafe { &*ptr })
            }
        }
    }

    /// Read a mutable reference — valid only for InMemory and MmapReadWrite
    pub fn get_mut(&mut self, index: usize) -> Result<&mut T, ContainerError> {
        if index >= self.len() {
            return Err(ContainerError::OutOfBounds(index));
        }

        match self {
            Storage::InMemory(vec) => Ok(&mut vec[index]),

            #[cfg(feature = "mmap")]
            Storage::MmapReadOnly(_) => {
                #[cfg(feature = "std")]
                return Err(ContainerError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cannot mut-access read-only mmap storage",
                )));

                #[cfg(not(feature = "std"))]
                return Err(ContainerError::Io(
                    "Cannot mut-access read-only mmap storage",
                ));
            }

            #[cfg(feature = "mmap")]
            Storage::MmapReadWrite(m) => {
                let elem_size = core::mem::size_of::<T>();
                let offset = index * elem_size;
                let ptr = unsafe { m.as_mut_ptr().add(offset) as *mut T };
                Ok(unsafe { &mut *ptr })
            }
        }
    }

    //  Mmap constructors 

    #[cfg(feature = "mmap")]
    pub fn from_mmap_readonly(path: &Path) -> Result<Self, ContainerError> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        // let elem_size = core::mem::size_of::<T>();
        // if mmap.len() % elem_size != 0 {
        //     return Err(ContainerError::InvalidFileSize { /* ... */ });
        //}
        Ok(Storage::MmapReadOnly(mmap))
    }

    #[cfg(feature = "mmap")]
    pub fn from_mmap_readwrite(path: &Path) -> Result<Self, ContainerError> {
        let file = File::options().read(true).write(true).open(path)?;
        // let elem_size = core::mem::size_of::<T>();
        // if mmap.len() % elem_size != 0 {
        //     return Err(ContainerError::InvalidFileSize { /* ... */ });
        // }
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        Ok(Storage::MmapReadWrite(mmap))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck_derive::Pod;
    use bytemuck_derive::Zeroable;

    // Simple Pod type for testing
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Zeroable, Pod)]
    struct Packet {
        id: u32,
        value: f32,
    }

    #[test]
    fn in_memory_basic_operations() {
        let mut storage = Storage::new_in_memory();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());

        let p1 = Packet { id: 1, value: 10.0 };
        let p2 = Packet { id: 2, value: 20.0 };

        storage.push(p1).unwrap();
        storage.push(p2).unwrap();

        assert_eq!(storage.len(), 2);
        assert!(!storage.is_empty());

        // get
        assert_eq!(storage.get(0).unwrap(), &p1);
        assert_eq!(storage.get(1).unwrap(), &p2);
        assert!(matches!(
            storage.get(2),
            Err(ContainerError::OutOfBounds(2))
        ));

        // get_mut
        let mut_ref = storage.get_mut(0).unwrap();
        mut_ref.value = 42.0;
        assert_eq!(storage.get(0).unwrap().value, 42.0);
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_readonly_operations() {
        use std::io::{Seek, SeekFrom, Write};
        use tempfile::NamedTempFile;

        // create temp file with two Packets
        let mut file = NamedTempFile::new().unwrap();
        let packets = [Packet { id: 1, value: 10.0 }, Packet { id: 2, value: 20.0 }];
        let bytes: &[u8] = bytemuck::cast_slice(&packets);
        file.write_all(bytes).unwrap();
        file.flush().unwrap();

        let storage = Storage::<Packet>::from_mmap_readonly(file.path()).unwrap();
        assert_eq!(storage.len(), 2);

        assert_eq!(storage.get(0).unwrap(), &packets[0]);
        assert_eq!(storage.get(1).unwrap(), &packets[1]);

        // get_mut should fail
        let mut storage = storage;
        assert!(storage.get_mut(0).is_err());
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn mmap_readwrite_operations() {
        use std::io::{Seek, SeekFrom, Write};
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        let packets = [Packet { id: 1, value: 10.0 }, Packet { id: 2, value: 20.0 }];
        let bytes: &[u8] = bytemuck::cast_slice(&packets);
        file.write_all(bytes).unwrap();
        file.flush().unwrap();

        let mut storage = Storage::<Packet>::from_mmap_readwrite(file.path()).unwrap();
        assert_eq!(storage.len(), 2);

        // get
        assert_eq!(storage.get(0).unwrap(), &packets[0]);
        assert_eq!(storage.get(1).unwrap(), &packets[1]);

        // get_mut
        let mut_ref = storage.get_mut(0).unwrap();
        mut_ref.value = 42.0;
        assert_eq!(storage.get(0).unwrap().value, 42.0);
    }

    #[test]
    fn push_error_for_mmap() {
        #[cfg(feature = "mmap")]
        {
            use bytemuck::cast_slice;
            use std::io::Write;
            use tempfile::NamedTempFile;

            let mut file = NamedTempFile::new().unwrap();
            let packets = [Packet { id: 1, value: 1.0 }];
            file.write_all(cast_slice(&packets)).unwrap();
            file.flush().unwrap();

            let mut ro = Storage::<Packet>::from_mmap_readonly(file.path()).unwrap();
            assert!(ro.push(Packet { id: 2, value: 2.0 }).is_err());

            let mut rw = Storage::<Packet>::from_mmap_readwrite(file.path()).unwrap();
            assert!(rw.push(Packet { id: 3, value: 3.0 }).is_err());
        }
    }
}
