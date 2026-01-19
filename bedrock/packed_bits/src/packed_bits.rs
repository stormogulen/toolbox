//! PackedBits: simple in-memory bit-packing (no mmap)

use crate::PackedBitsError;
//use alloc::vec::Vec;

pub struct PackedBits<const N: usize> {
    data: Vec<u8>,
    len: usize,
}

impl<const N: usize> PackedBits<N> {
    pub fn new() -> Self {
        assert!(N > 0 && N <= 32, "N must be 1..=32");
        Self { data: Vec::new(), len: 0 }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push(&mut self, value: u32) -> Result<(), PackedBitsError> {
        let max = if N == 32 { u32::MAX } else { (1 << N) - 1 };
        if value > max {
            return Err(PackedBitsError::ValueOverflow(value, N));
        }

        let bit_pos = self.len * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let total_bits = bit_pos + N;
        let needed = (total_bits + 7) / 8;
        if self.data.len() < needed {
            self.data.resize(needed, 0);
        }

        let mut v = (value as u64) << bit_offset;
        let bytes = (N + bit_offset + 7) / 8;

        for i in 0..bytes {
            self.data[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
        }

        self.len += 1;
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.len { return None; }

        let bit_pos = index * N;
        let byte_pos = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let mut val = 0u64;
        let bytes = (N + bit_offset + 7) / 8;

        for i in 0..bytes {
            if byte_pos + i < self.data.len() {
                val |= (self.data[byte_pos + i] as u64) << (i * 8);
            }
        }

        val >>= bit_offset;

        let mask = if N == 32 { u32::MAX as u64 } else { (1u64 << N) - 1 };
        Some((val & mask) as u32)
    }

    #[inline]
    fn elem_bit_index(index: usize) -> BitIndex {
        BitIndex::from(ElemIndex(index))
    }
}
