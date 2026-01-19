
//! I/O utilities for saving and loading packed data

use std::fs::File;
use std::io::{self, Write, Read};
use std::path::Path;
use mtf::{MTFType, write_slice_with_mtf};
use mtf::dynamic::DynamicContainer;
use save::{save, load, SaveError};
use bytemuck::Pod;

/// Save data with MTF metadata for runtime introspection
pub fn save_with_metadata<T, P>(path: P, data: &[T]) -> io::Result<()>
where
    T: MTFType + Pod,
    P: AsRef<Path>,
{
    let mut file = File::create(path)?;
    //write_slice_with_mtf(&mut file, data)?;
    write_slice_with_mtf(&mut file, data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    file.flush()?;
    Ok(())
}

/// Load data dynamically with MTF metadata
pub fn load_dynamic<P: AsRef<Path>>(path: P) -> mtf::Result<DynamicContainer> {
    DynamicContainer::from_file(path)
}

/// Save raw bytes without metadata (most compact)
pub fn save_raw<T, P>(path: P, data: &[T]) -> io::Result<()>
where
    T: Pod,
    P: AsRef<Path>,
{
    let mut file = File::create(path)?;
    let bytes = bytemuck::cast_slice(data);
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

/// Load raw bytes into typed slice
pub fn load_raw<T, P>(path: P) -> io::Result<Vec<T>>
where
    T: Pod,
    P: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    
    let count = bytes.len() / std::mem::size_of::<T>();
    let (aligned, _) = bytes.split_at(count * std::mem::size_of::<T>());
    
    Ok(bytemuck::cast_slice(aligned).to_vec())
}

/// Save with Merkle tree verification (requires 'verified' feature)
/// 
/// Adds a 32-byte hash prefix for integrity checking on load.
#[cfg(feature = "verified")]
pub fn save_verified<T, P>(path: P, data: &[T]) -> io::Result<()>
where
    T: Pod + Copy,
    P: AsRef<Path>,
{
    let container = packed_structs::PackedStructContainer::from_slice(data);
    save::save(path, &container)
}

/// Load with Merkle tree verification (requires 'verified' feature)
/// 
/// Verifies the 32-byte hash prefix matches the data integrity.
#[cfg(feature = "verified")]
pub fn load_verified<T, P>(path: P) -> io::Result<Vec<T>>
where
    T: Pod + Copy,
    P: AsRef<Path>,
{
    let container = save::load(path)?;
    Ok(container.as_slice().to_vec())
}

/// Efficient streaming writer for large datasets
pub struct PackedWriter<W: Write, T> {
    writer: W,
    _phantom: std::marker::PhantomData<T>,
}

impl<W: Write, T: Pod> PackedWriter<W, T> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn write(&mut self, item: &T) -> io::Result<()> {
        let bytes = bytemuck::bytes_of(item);
        self.writer.write_all(bytes)
    }

    pub fn write_batch(&mut self, items: &[T]) -> io::Result<()> {
        let bytes = bytemuck::cast_slice(items);
        self.writer.write_all(bytes)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Efficient streaming reader for large datasets
pub struct PackedReader<R: Read, T> {
    reader: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<R: Read, T: Pod> PackedReader<R, T> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn read(&mut self) -> io::Result<Option<T>> {
        let mut bytes = vec![0u8; std::mem::size_of::<T>()];
        match self.reader.read_exact(&mut bytes) {
            Ok(()) => Ok(Some(*bytemuck::from_bytes(&bytes))),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn read_batch(&mut self, count: usize) -> io::Result<Vec<T>> {
        let mut bytes = vec![0u8; std::mem::size_of::<T>() * count];
        self.reader.read_exact(&mut bytes)?;
        Ok(bytemuck::cast_slice(&bytes).to_vec())
    }
}
