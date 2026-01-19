//! Dynamic field access for MTF-annotated types

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;

use bytemuck::{Pod, from_bytes};
use crate::{FieldDef, MTFError, Result, TypeDef, read_mtf, read_string};

/// A handle to a single field in a struct.
///
/// Provides a builder-style API for modifying field values.
pub struct FieldHandle<'a, T> {
    ptr: Option<NonNull<T>>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> FieldHandle<'a, T> {
    /// Create an empty handle (no field found).
    pub fn none() -> Self {
        Self {
            ptr: None,
            _phantom: PhantomData,
        }
    }

    /// Create a handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid, properly aligned, and point to initialized data.
    unsafe fn from_ptr(p: *mut T) -> Self {
        Self {
            ptr: NonNull::new(p),
            _phantom: PhantomData,
        }
    }

    /// Returns true if the handle points to a valid field.
    pub fn is_some(&self) -> bool {
        self.ptr.is_some()
    }

    /// Get an immutable reference to the field value.
    pub fn get(&self) -> Option<&T> {
        self.ptr.map(|p| unsafe { p.as_ref() })
    }

    /// Get a mutable reference to the field value.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.ptr.map(|mut p| unsafe { p.as_mut() })
    }

    /// Set the field value.
    pub fn set(&mut self, v: T) -> &mut Self {
        if let Some(p) = self.ptr {
            unsafe { *p.as_ptr() = v }
        }
        self
    }

    /// Add to the field value (requires AddAssign).
    pub fn add(&mut self, v: T) -> &mut Self
    where
        T: std::ops::AddAssign + Copy,
    {
        if let Some(mut p) = self.ptr {
            unsafe { *p.as_mut() += v }
        }
        self
    }

    /// Subtract from the field value (requires SubAssign).
    pub fn sub(&mut self, v: T) -> &mut Self
    where
        T: std::ops::SubAssign + Copy,
    {
        if let Some(mut p) = self.ptr {
            unsafe { *p.as_mut() -= v }
        }
        self
    }

    /// Apply a closure to modify the field value.
    pub fn apply<F: FnOnce(&mut T)>(&mut self, f: F) -> &mut Self {
        if let Some(mut p) = self.ptr {
            unsafe { f(p.as_mut()) }
        }
        self
    }
}

/// Dynamic access to a slice of structs with MTF metadata.
///
/// Allows field access by name at runtime, useful for:
/// - Generic tooling and editors
/// - Serialization/deserialization
/// - Dynamic queries
pub struct DynamicContainer {
    data: Vec<u8>,
    type_def: TypeDef,
    strings: Vec<u8>,
    struct_size: usize,
    field_map: HashMap<String, FieldDef>,
}

impl DynamicContainer {
    /// Construct from raw data and a complete MTF blob.
    pub fn from_raw(data: Vec<u8>, blob: &[u8]) -> Result<Self> {
        let (types, strings) = read_mtf(blob)?;

        let type_def = types.into_iter().next().ok_or(MTFError::UnexpectedEof)?;

        let struct_size = (type_def.size_bits as usize).div_ceil(8);

        // Precompute field name -> FieldDef map for fast lookups
        let mut field_map = HashMap::new();
        for f in &type_def.fields {
            let name = read_string(strings, f.name_offset)?;
            field_map.insert(name.to_string(), f.clone());
        }

        Ok(Self {
            data,
            type_def,
            strings: strings.to_vec(),
            struct_size,
            field_map,
        })
    }

    /// Construct directly from a file containing MTF-embedded data.
    ///
    /// Expects format: [DATA][METADATA_SIZE: u32][METADATA]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let len = file.metadata()?.len();

        if len < 4 {
            return Err(MTFError::UnexpectedEof);
        }

        // Read entire file into memory
        file.seek(SeekFrom::Start(0))?;
        let mut all_data = vec![0u8; len as usize];
        file.read_exact(&mut all_data)?;

        // Find MTF magic bytes to locate where metadata starts
        for i in 0..all_data.len() - 4 {
            if &all_data[i..i + 4] == b"MTF\0" {
                // Found it! Check if there's a size field 4 bytes before
                if i >= 4 {
                    let size_pos = i - 4;
                    let metadata_size = u32::from_le_bytes([
                        all_data[size_pos],
                        all_data[size_pos + 1],
                        all_data[size_pos + 2],
                        all_data[size_pos + 3],
                    ]) as usize;
                    
                    // Verify this makes sense
                    if size_pos + 4 + metadata_size == all_data.len() {
                        // This is it!
                        let data = all_data[..size_pos].to_vec();
                        let blob = &all_data[i..];
                        return Self::from_raw(data, blob);
                    }
                }
            }
        }
        
        Err(MTFError::InvalidMagic)
    } 
    

    /// Returns the number of structs in the container.
    pub fn len(&self) -> usize {
        if self.struct_size == 0 {
            0
        } else {
            self.data.len() / self.struct_size
        }
    }

    /// Returns true if the container is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the type name.
    pub fn type_name(&self) -> Result<&str> {
        read_string(&self.strings, self.type_def.name_offset)
    }

    /// List all field names.
    pub fn field_names(&self) -> Vec<String> {
        self.field_map.keys().cloned().collect()
    }

    /// Immutable access to a field of a struct at index.
    pub fn field<T: Pod>(&self, index: usize, field_name: &str) -> Option<&T> {
        if index >= self.len() {
            return None;
        }

        let field = self.field_map.get(field_name)?;

        let field_size = (field.size_bits as usize).div_ceil(8);
        if field_size != std::mem::size_of::<T>() {
            return None;
        }

        let field_offset = (field.offset_bits / 8) as usize;
        if field_offset % std::mem::align_of::<T>() != 0 {
            return None;
        }

        let struct_start = index * self.struct_size;
        let field_start = struct_start + field_offset;
        let field_end = field_start + field_size;

        let field_slice = self.data.get(field_start..field_end)?;

        Some(from_bytes(field_slice))
    }

    /// Mutable access to a field of a struct at index.
    pub fn field_mut<T: Pod>(&mut self, index: usize, field_name: &str) -> FieldHandle<'_, T> {
        if index >= self.len() {
            return FieldHandle::none();
        }

        let field = match self.field_map.get(field_name) {
            Some(f) => f,
            None => return FieldHandle::none(),
        };

        let field_size = (field.size_bits as usize).div_ceil(8);
        if field_size != std::mem::size_of::<T>() {
            return FieldHandle::none();
        }

        let field_offset = (field.offset_bits / 8) as usize;
        if field_offset % std::mem::align_of::<T>() != 0 {
            return FieldHandle::none();
        }

        let struct_start = index * self.struct_size;
        let field_start = struct_start + field_offset;
        let field_end = field_start + field_size;

        let field_slice = match self.data.get_mut(field_start..field_end) {
            Some(s) => s,
            None => return FieldHandle::none(),
        };

        let ptr = field_slice.as_mut_ptr() as *mut T;
        unsafe { FieldHandle::from_ptr(ptr) }
    }

    /// Get raw byte data.
    pub fn raw(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable raw byte data.
    pub fn raw_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Iterator over struct indices.
    pub fn iter(&self) -> DynamicContainerIter<'_> {
        DynamicContainerIter {
            container: self,
            index: 0,
        }
    }
}

/// Iterator over the container structs (yields indices).
pub struct DynamicContainerIter<'a> {
    container: &'a DynamicContainer,
    index: usize,
}

impl<'a> Iterator for DynamicContainerIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.container.len() {
            let idx = self.index;
            self.index += 1;
            Some(idx)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.container.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for DynamicContainerIter<'a> {}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_blob() -> Vec<u8> {
        let mut blob = Vec::new();
        blob.extend_from_slice(b"MTF\0");
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.extend_from_slice(&0u32.to_le_bytes());
        blob.extend_from_slice(&64u32.to_le_bytes());
        blob.extend_from_slice(&2u32.to_le_bytes());

        blob.extend_from_slice(&5u32.to_le_bytes());
        blob.extend_from_slice(&0u32.to_le_bytes());
        blob.extend_from_slice(&32u32.to_le_bytes());

        blob.extend_from_slice(&7u32.to_le_bytes());
        blob.extend_from_slice(&32u32.to_le_bytes());
        blob.extend_from_slice(&32u32.to_le_bytes());

        blob.extend_from_slice(&9u32.to_le_bytes());
        blob.extend_from_slice(b"Test\0x\0y\0");

        blob
    }

    #[test]
    fn test_dynamic_container_creation() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let blob = create_test_blob();

        let container = DynamicContainer::from_raw(data, &blob).unwrap();
        assert_eq!(container.len(), 1);
        assert_eq!(container.type_name().unwrap(), "Test");
        assert!(!container.is_empty());
    }

    #[test]
    fn test_field_names() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let blob = create_test_blob();

        let container = DynamicContainer::from_raw(data, &blob).unwrap();
        let names = container.field_names();
        
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"y".to_string()));
    }

    #[test]
    fn test_field_access() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let blob = create_test_blob();

        let mut container = DynamicContainer::from_raw(data, &blob).unwrap();

        let x: &u32 = container.field(0, "x").unwrap();
        assert_eq!(*x, 0x04030201);

        container.field_mut(0, "y").set(0xDEADBEEF_u32);

        let y: &u32 = container.field(0, "y").unwrap();
        assert_eq!(*y, 0xDEADBEEF);
    }

    #[test]
    fn test_field_handle_operations() {
        let data = vec![0x0A, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00];
        let blob = create_test_blob();

        let mut container = DynamicContainer::from_raw(data, &blob).unwrap();

        // Test add
        container.field_mut::<u32>(0, "x").add(5);
        let x: &u32 = container.field(0, "x").unwrap();
        assert_eq!(*x, 15);

        // Test sub
        container.field_mut::<u32>(0, "y").sub(4);
        let y: &u32 = container.field(0, "y").unwrap();
        assert_eq!(*y, 16);

        // Test apply
        container.field_mut::<u32>(0, "x").apply(|v| *v *= 2);
        let x: &u32 = container.field(0, "x").unwrap();
        assert_eq!(*x, 30);
    }

    #[test]
    fn test_field_handle_none() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let blob = create_test_blob();

        let mut container = DynamicContainer::from_raw(data, &blob).unwrap();

        // Non-existent field
        let handle = container.field_mut::<u32>(0, "nonexistent");
        assert!(!handle.is_some());
        assert!(handle.get().is_none());

        // Out of bounds index
        let handle = container.field_mut::<u32>(99, "x");
        assert!(!handle.is_some());
    }

    #[test]
    fn test_iterator() {
        let data = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
        ];
        let blob = create_test_blob();

        let container = DynamicContainer::from_raw(data, &blob).unwrap();
        
        assert_eq!(container.len(), 3);
        
        let indices: Vec<usize> = container.iter().collect();
        assert_eq!(indices, vec![0, 1, 2]);
        
        assert_eq!(container.iter().len(), 3);
    }
}
