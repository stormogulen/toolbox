
//! Conversion traits and utilities

use bytemuck::{Pod, cast_slice, cast_slice_mut};
use std::convert::TryFrom;

/// Convert types to byte slices
pub trait ToBytes {
    fn to_bytes(&self) -> &[u8];
}

impl<T: Pod> ToBytes for [T] {
    fn to_bytes(&self) -> &[u8] {
        cast_slice(self)
    }
}

impl<T: Pod> ToBytes for Vec<T> {
    fn to_bytes(&self) -> &[u8] {
        cast_slice(self.as_slice())
    }
}

/// Convert byte slices to typed slices
pub trait FromBytes<T> {
    fn from_bytes(bytes: &[u8]) -> &[T];
    fn from_bytes_mut(bytes: &mut [u8]) -> &mut [T];
}

impl<T: Pod> FromBytes<T> for [T] {
    fn from_bytes(bytes: &[u8]) -> &[T] {
        cast_slice(bytes)
    }

    fn from_bytes_mut(bytes: &mut [u8]) -> &mut [T] {
        cast_slice_mut(bytes)
    }
}

/// Unified conversion trait for packed data
pub trait PackedConvert: Sized {
     /// Append this value to a byte buffer
    fn pack_into(&self, buffer: &mut Vec<u8>);

    /// Unpack a value from the start of a buffer
    fn unpack_prefix(buffer: &[u8]) -> Option<Self>;

    /// Size in bytes when packed
    fn packed_size() -> usize;
}

// Implement for Pod types automatically
impl<T: Pod + Copy> PackedConvert for T {
    fn pack_into(&self, buffer: &mut Vec<u8>) {
        //let bytes = bytemuck::bytes_of(self);
        //buffer.extend_from_slice(bytes);
        buffer.extend_from_slice(bytemuck::bytes_of(self));
    }

    fn unpack_prefix(buffer: &[u8]) -> Option<Self> {
        let size = std::mem::size_of::<T>();
        if buffer.len() < size {
            return None;
        }
        Some(*bytemuck::from_bytes(&buffer[..size]))
    }

    fn packed_size() -> usize {
        std::mem::size_of::<T>()
    }
}

/// Extract a fixed-size batch from a slice
///
/// # Examples
///
/// ```
/// use packed_data::convert::batch;
///
/// let data = vec![1u32, 2, 3, 4, 5, 6];
/// let first_four: Option<[u32; 4]> = batch(&data, 0);
/// assert_eq!(first_four, Some([1, 2, 3, 4]));
///
/// let out_of_bounds: Option<[u32; 4]> = batch(&data, 4);
/// assert_eq!(out_of_bounds, None); // Not enough elements
/// ```
pub fn batch<const N: usize, T: Copy>(slice: &[T], start: usize) -> Option<[T; N]> {
    if slice.len() < start + N {
        return None;
    }
    Some(std::array::from_fn(|i| slice[start + i]))
}


/// Try to parse fixed-size items from a byte slice
///
/// Returns an iterator that yields `Result<T, E>` for each item.
///
/// # Examples
///
/// ```
/// use packed_data::convert::try_parse_iter;
///
/// #[derive(Debug, PartialEq)]
/// struct Header {
///     id: u32,
///     value: u32,
/// }
///
/// impl TryFrom<&[u8]> for Header {
///     type Error = &'static str;
///     
///     fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
///         if bytes.len() < 8 {
///             return Err("insufficient bytes");
///         }
///         Ok(Header {
///             id: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
///             value: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
///         })
///     }
/// }
///
/// let bytes = vec![1u8, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0];
/// let headers: Vec<Header> = try_parse_iter::<Header, _, 8>(&bytes)
///     .collect::<Result<Vec<_>, _>>()
///     .unwrap();
///
/// assert_eq!(headers.len(), 2);
/// assert_eq!(headers[0].id, 1);
/// assert_eq!(headers[1].id, 3);
/// ```
pub fn try_parse_iter<'a, T, E, const SIZE: usize>(
    bytes: &'a [u8]
) -> impl Iterator<Item = Result<T, E>> + 'a
where
    T: TryFrom<&'a [u8], Error = E>,
{
    bytes.chunks_exact(SIZE).map(|chunk| T::try_from(chunk))
}

/// Parse items with a fallible closure
///
/// # Examples
///
/// ```
/// use packed_data::convert::parse_with;
///
/// let bytes = vec![1u8, 0, 0, 0, 2, 0, 0, 0];
/// let numbers: Vec<u32> = parse_with(&bytes, 4, |chunk| -> Result<u32, ()> {
///     Ok(u32::from_le_bytes(chunk.try_into().unwrap()))
/// }).collect::<Result<Vec<_>, _>>().unwrap();
///
/// assert_eq!(numbers, vec![1, 2]);
/// ```
pub fn parse_with<'a, T, E, F>(
    bytes: &'a [u8],
    chunk_size: usize,
    mut parser: F,
) -> impl Iterator<Item = Result<T, E>> + 'a
where
    F: FnMut(&[u8]) -> Result<T, E> + 'a,
{
    bytes.chunks_exact(chunk_size).map(move |chunk| parser(chunk))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch() {
        let data = vec![1u32, 2, 3, 4, 5, 6];
        
        let first: Option<[u32; 4]> = batch(&data, 0);
        assert_eq!(first, Some([1, 2, 3, 4]));
        
        let middle: Option<[u32; 3]> = batch(&data, 2);
        assert_eq!(middle, Some([3, 4, 5]));
        
        let none: Option<[u32; 4]> = batch(&data, 4);
        assert_eq!(none, None);
    }

    #[test]
    fn test_try_parse_iter() {
        #[derive(Debug, PartialEq)]
        struct Pair(u32, u32);
        
        impl TryFrom<&[u8]> for Pair {
            type Error = &'static str;
            
            fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                if bytes.len() < 8 {
                    return Err("need 8 bytes");
                }
                let a = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
                let b = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
                Ok(Pair(a, b))
            }
        }
        
        let bytes = vec![
            1, 0, 0, 0, 2, 0, 0, 0,
            3, 0, 0, 0, 4, 0, 0, 0,
        ];
        
        let pairs: Vec<Pair> = try_parse_iter::<Pair, _, 8>(&bytes)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        
        assert_eq!(pairs, vec![Pair(1, 2), Pair(3, 4)]);
    }
}
