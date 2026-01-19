//! Iterator utilities for packed data

use std::iter::from_fn;
use std::marker::PhantomData;

/// Create an iterator from a fallible function
///
/// Useful for parsing variable-length data or complex formats.
///
/// # Examples
///
/// ```
/// use packed_data::iter::iter_parse;
///
/// let data = vec![1u8, 2, 3, 4, 5];
/// let mut pos = 0;
///
/// let items: Vec<u8> = iter_parse(|| {
///     if pos < data.len() {
///         let val = data[pos];
///         pos += 1;
///         Some(val)
///     } else {
///         None
///     }
/// }).collect();
///
/// assert_eq!(items, vec![1, 2, 3, 4, 5]);
/// ```
pub fn iter_parse<F, T>(mut f: F) -> impl Iterator<Item = T>
where
    F: FnMut() -> Option<T>,
{
    from_fn(move || f())
}

pub trait FixedSizeParse: Sized {
    const SIZE: usize;
    type Error;

    fn parse(bytes: &[u8]) -> Result<Self, Self::Error>;
}

pub struct FixedSizeParseIter<'a, T: FixedSizeParse> {
    bytes: &'a [u8],
    pos: usize,
    _marker: PhantomData<T>,
}

impl<'a, T: FixedSizeParse> FixedSizeParseIter<'a, T> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: FixedSizeParse> Iterator for FixedSizeParseIter<'a, T> {
    type Item = Result<T, T::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + T::SIZE > self.bytes.len() {
            return None;
        }

        let chunk = &self.bytes[self.pos..self.pos + T::SIZE];
        self.pos += T::SIZE;
        Some(T::parse(chunk))
    }
}


// impl<'a, T, E> TryParseIter<'a, T, E> {
//     pub fn new(bytes: &'a [u8]) -> Self {
//         Self {
//             bytes,
//             pos: 0,
//             _marker: PhantomData,
//         }
//     }
// }

// impl<'a, T, E> Iterator for TryParseIter<'a, T, E>
// where
//     T: for<'b> TryFrom<&'b [u8], Error = E>,
// {
//     type Item = Result<T, E>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.pos >= self.bytes.len() {
//             return None;
//         }

//         // Try to parse from remaining bytes
//         let result = T::try_from(&self.bytes[self.pos..]);
        
//         match result {
//             Ok(item) => {
//                 // Assume fixed size for now - could be made more sophisticated
//                 self.pos += std::mem::size_of::<T>();
//                 Some(Ok(item))
//             }
//             Err(e) => Some(Err(e)),
//         }
//     }
// }

/// Extension trait for slices to enable convenient parsing
pub trait SliceParseExt {
    /// Parse items in batches of size N
    fn batches<const N: usize, T: Copy>(&self) -> BatchIter<'_, N, T>;
    
    /// Try to parse items with a conversion function
    fn try_parse<T, E, F>(&self, chunk_size: usize, parser: F) -> TryParseWithIter<'_, T, E, F>
    where
        F: FnMut(&[u8]) -> Result<T, E>;
}

impl SliceParseExt for [u8] {
    fn batches<const N: usize, T: Copy>(&self) -> BatchIter<'_, N, T> {
        BatchIter {
            bytes: self,
            pos: 0,
            _marker: PhantomData,
        }
    }

    fn try_parse<T, E, F>(&self, chunk_size: usize, parser: F) -> TryParseWithIter<'_, T, E, F>
    where
        F: FnMut(&[u8]) -> Result<T, E>,
    {
        TryParseWithIter {
            bytes: self,
            chunk_size,
            pos: 0,
            parser,
            _marker: PhantomData,
        }
    }
}

pub struct BatchIter<'a, const N: usize, T> {
    bytes: &'a [u8],
    pos: usize,
    _marker: PhantomData<T>,
}

impl<'a, const N: usize, T: bytemuck::Pod> Iterator for BatchIter<'a, N, T> {
    type Item = [T; N];

    fn next(&mut self) -> Option<Self::Item> {
        let item_size = std::mem::size_of::<T>();
        let batch_size = item_size * N;
        
        if self.pos + batch_size > self.bytes.len() {
            return None;
        }

        let items: &[T] = bytemuck::cast_slice(&self.bytes[self.pos..self.pos + batch_size]);
        let batch = std::array::from_fn(|i| items[i]);
        
        self.pos += batch_size;
        Some(batch)
    }
}

pub struct TryParseWithIter<'a, T, E, F> {
    bytes: &'a [u8],
    chunk_size: usize,
    pos: usize,
    parser: F,
    _marker: PhantomData<(T, E)>,
}

impl<'a, T, E, F> Iterator for TryParseWithIter<'a, T, E, F>
where
    F: FnMut(&[u8]) -> Result<T, E>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + self.chunk_size > self.bytes.len() {
            return None;
        }

        let chunk = &self.bytes[self.pos..self.pos + self.chunk_size];
        let result = (self.parser)(chunk);
        self.pos += self.chunk_size;
        
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_parse_batches() {
        let bytes: Vec<u8> = (0..16).collect();
        let batches: Vec<[u32; 2]> = bytes.batches().collect();
        
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_try_parse_with() {
        let bytes = vec![1u8, 0, 0, 0, 2, 0, 0, 0];
        let numbers: Vec<u32> = bytes
            .try_parse(4, |chunk| {
                Ok(u32::from_le_bytes(chunk.try_into().unwrap()))
            })
            .collect::<Result<Vec<_>, ()>>()
            .unwrap();
        
        assert_eq!(numbers, vec![1, 2]);
    }
}
