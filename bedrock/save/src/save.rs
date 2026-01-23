use crate::merkle::merkle_root;
use bytemuck::{Pod, cast_slice, cast_slice_mut};
use bytemuck_derive::Pod;
use bytemuck_derive::Zeroable;
use std::io::{Read, Write};
use std::fs::File;
use std::path::Path;

const MAGIC: u32 = 0x53415645; // "SAVE"
const VERSION: u16 = 1;
const DEFAULT_CHUNK_SIZE: usize = 4096;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SaveHeader {
    pub magic: u32,
    pub version: u16,
    pub element_size: u16,
    pub element_count: u32,
    pub chunk_size: u32,
    pub merkle_root: [u8; 32],
}

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    InvalidMagic,
    InvalidVersion,
    HashMismatch,
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}

use std::fmt;

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "I/O error: {}", e),
            SaveError::InvalidMagic => write!(f, "Invalid SAVE magic"),
            SaveError::InvalidVersion => write!(f, "Unsupported SAVE version"),
            SaveError::HashMismatch => write!(f, "Merkle hash mismatch"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Io(e) => Some(e),
            _ => None,
        }
    }
}


/// Save a slice of POD elements to a writer.
pub fn save<T: Pod, W: Write>(writer: &mut W, data: &[T]) -> Result<(), SaveError> {
    let bytes = cast_slice(data);
    let root = merkle_root(bytes, DEFAULT_CHUNK_SIZE);

    let header = SaveHeader {
        magic: MAGIC,
        version: VERSION,
        element_size: std::mem::size_of::<T>() as u16,
        element_count: data.len() as u32,
        chunk_size: DEFAULT_CHUNK_SIZE as u32,
        merkle_root: *root.as_bytes(),
    };

    writer.write_all(bytemuck::bytes_of(&header))?;
    writer.write_all(bytes)?;
    Ok(())
}

/// Load POD elements from a reader and verify integrity.
pub fn load<T: Pod, R: Read>(reader: &mut R) -> Result<Vec<T>, SaveError> {
    let mut header = SaveHeader {
        magic: 0,
        version: 0,
        element_size: 0,
        element_count: 0,
        chunk_size: 0,
        merkle_root: [0; 32],
    };

    reader.read_exact(bytemuck::bytes_of_mut(&mut header))?;

    if header.magic != MAGIC {
        return Err(SaveError::InvalidMagic);
    }

    if header.version != VERSION {
        return Err(SaveError::InvalidVersion);
    }

    if header.element_size as usize != std::mem::size_of::<T>() {
        return Err(SaveError::InvalidVersion);
    }

    let mut data = vec![T::zeroed(); header.element_count as usize];
    let bytes = cast_slice_mut(&mut data);
    reader.read_exact(bytes)?;

    let root = merkle_root(bytes, header.chunk_size as usize);
    if root.as_bytes() != &header.merkle_root {
        return Err(SaveError::HashMismatch);
    }

    Ok(data)
}

pub fn save_to_file<P: AsRef<Path>, T: Pod>(
    path: P,
    data: &[T],
) -> Result<(), SaveError> {
    let mut file = File::create(path)?;
    save(&mut file, data)
}

pub fn load_from_file<P: AsRef<Path>, T: Pod>(
    path: P,
) -> Result<Vec<T>, SaveError> {
    let mut file = File::open(path)?;
    load(&mut file)
}
