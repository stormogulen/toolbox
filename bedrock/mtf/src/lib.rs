//! MTF: Minimal Type Format
//!
//! Self-describing binary format for packed structs with bit-level precision.

use std::io::{self, Write};
use thiserror::Error;

// Re-export the derive macro
#[cfg(feature = "derive")]
pub use mtf_derive::MTF;

// Re-export bytemuck for users
pub use bytemuck;

const MTF_MAGIC: &[u8; 4] = b"MTF\0";
const MTF_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDef {
    pub name_offset: u32,
    pub offset_bits: u32,
    pub size_bits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDef {
    pub name_offset: u32,
    pub size_bits: u32,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Error)]
pub enum MTFError {
    #[error("Invalid magic bytes (expected MTF\\0)")]
    InvalidMagic,
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),
    #[error("Unexpected end of data")]
    UnexpectedEof,
    #[error("Invalid UTF-8 in string table")]
    InvalidUtf8,
    #[error("String offset {0} out of bounds")]
    InvalidStringOffset(u32),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, MTFError>;

pub trait MTFType {
    fn mtf_type_blob() -> &'static [u8];
    fn mtf_string_table() -> &'static [u8];
}

/// Write MTF metadata blob: [MAGIC][VERSION][TYPE_COUNT][TYPES][STRING_TABLE_SIZE][STRING_TABLE]
pub fn write_mtf(types: &[TypeDef], strings: &[u8], mut out: impl Write) -> Result<()> {
    out.write_all(MTF_MAGIC)?;
    out.write_all(&MTF_VERSION.to_le_bytes())?;

    let count = types.len() as u32;
    out.write_all(&count.to_le_bytes())?;

    for t in types {
        out.write_all(&t.name_offset.to_le_bytes())?;
        out.write_all(&t.size_bits.to_le_bytes())?;
        let fcount = t.fields.len() as u32;
        out.write_all(&fcount.to_le_bytes())?;
        for f in &t.fields {
            out.write_all(&f.name_offset.to_le_bytes())?;
            out.write_all(&f.offset_bits.to_le_bytes())?;
            out.write_all(&f.size_bits.to_le_bytes())?;
        }
    }

    let string_len = strings.len() as u32;
    out.write_all(&string_len.to_le_bytes())?;
    out.write_all(strings)?;

    Ok(())
}

/// Read MTF blob, returning type definitions and string table.
pub fn read_mtf(data: &[u8]) -> Result<(Vec<TypeDef>, &[u8])> {
    let mut pos = 0;

    if data.len() < 12 {
        return Err(MTFError::UnexpectedEof);
    }

    if &data[pos..pos + 4] != MTF_MAGIC {
        return Err(MTFError::InvalidMagic);
    }
    pos += 4;

    let version = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
    pos += 4;
    if version != MTF_VERSION {
        return Err(MTFError::UnsupportedVersion(version));
    }

    let count = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    let mut types = Vec::with_capacity(count);

    for _ in 0..count {
        if pos + 12 > data.len() {
            return Err(MTFError::UnexpectedEof);
        }
        let name_offset = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;
        let size_bits = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;
        let fcount = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        let mut fields = Vec::with_capacity(fcount);
        for _ in 0..fcount {
            if pos + 12 > data.len() {
                return Err(MTFError::UnexpectedEof);
            }
            let no = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;
            let off = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;
            let sz = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;
            fields.push(FieldDef {
                name_offset: no,
                offset_bits: off,
                size_bits: sz,
            });
        }

        types.push(TypeDef {
            name_offset,
            size_bits,
            fields,
        });
    }

    if pos + 4 > data.len() {
        return Err(MTFError::UnexpectedEof);
    }
    let string_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    if pos + string_len > data.len() {
        return Err(MTFError::UnexpectedEof);
    }
    let strings = &data[pos..pos + string_len];

    Ok((types, strings))
}

pub fn read_string(strings: &[u8], offset: u32) -> Result<&str> {
    let start = offset as usize;
    if start >= strings.len() {
        return Err(MTFError::InvalidStringOffset(offset));
    }
    let remaining = &strings[start..];
    let end = remaining
        .iter()
        .position(|&b| b == 0)
        .ok_or(MTFError::UnexpectedEof)?;
    std::str::from_utf8(&remaining[..end]).map_err(|_| MTFError::InvalidUtf8)
}

/// Build a string table from list of strings
pub fn build_string_table(strings: &[&str]) -> (Vec<u8>, std::collections::HashMap<String, u32>) {
    let mut table = Vec::new();
    let mut offsets = std::collections::HashMap::new();
    for s in strings {
        let off = table.len() as u32;
        offsets.insert(s.to_string(), off);
        table.extend_from_slice(s.as_bytes());
        table.push(0);
    }
    (table, offsets)
}

/// Write a slice of MTF types with embedded metadata.
///
/// Format: [DATA][METADATA_SIZE: u32][METADATA: complete MTF blob]
pub fn write_slice_with_mtf<T: MTFType + bytemuck::Pod>(
    mut out: impl Write,
    slice: &[T],
) -> Result<()> {
    // Write the actual data first
    let raw = bytemuck::cast_slice(slice);
    out.write_all(raw)?;

    // Get the complete MTF blob (includes magic, version, types, strings)
    let blob = T::mtf_type_blob();

    // Write metadata size so readers know where it starts
    let metadata_size = blob.len() as u32;
    out.write_all(&metadata_size.to_le_bytes())?;

    // Write metadata
    out.write_all(blob)?;

    Ok(())
}

// Re-export dynamic module
pub mod dynamic;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtf_magic() {
        assert_eq!(MTF_MAGIC, b"MTF\0");
        assert_eq!(MTF_VERSION, 1);
    }

    #[test]
    fn test_build_string_table() {
        let (table, offsets) = build_string_table(&["foo", "bar", "baz"]);
        
        assert_eq!(offsets.get("foo"), Some(&0));
        assert_eq!(offsets.get("bar"), Some(&4));
        assert_eq!(offsets.get("baz"), Some(&8));
        
        assert_eq!(&table[0..4], b"foo\0");
        assert_eq!(&table[4..8], b"bar\0");
        assert_eq!(&table[8..12], b"baz\0");
    }

    #[test]
    fn test_read_string() {
        let strings = b"hello\0world\0test\0";
        
        assert_eq!(read_string(strings, 0).unwrap(), "hello");
        assert_eq!(read_string(strings, 6).unwrap(), "world");
        assert_eq!(read_string(strings, 12).unwrap(), "test");
    }

    #[test]
    fn test_write_and_read_mtf() {
        let type_def = TypeDef {
            name_offset: 0,
            size_bits: 64,
            fields: vec![
                FieldDef {
                    name_offset: 5,
                    offset_bits: 0,
                    size_bits: 32,
                },
                FieldDef {
                    name_offset: 7,
                    offset_bits: 32,
                    size_bits: 32,
                },
            ],
        };

        let strings = b"Test\0x\0y\0";
        let mut output = Vec::new();
        
        write_mtf(&[type_def.clone()], strings, &mut output).unwrap();
        
        let (parsed_types, parsed_strings) = read_mtf(&output).unwrap();
        
        assert_eq!(parsed_types.len(), 1);
        assert_eq!(parsed_types[0], type_def);
        assert_eq!(parsed_strings, strings);
    }
}