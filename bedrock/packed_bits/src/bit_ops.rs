//     #[inline(always)]
//     pub(crate) fn set_bits(slice: &mut [u8], bit_pos: usize, n: usize, value: u32) {
//         let byte_pos = bit_pos / 8;
//         let bit_offset = bit_pos % 8;
//         let mut v = value as u64;
//         v <<= bit_offset;
//         let num_bytes = (n + bit_offset + 7) / 8;
//         for i in 0..num_bytes {
//             if byte_pos + i < slice.len() {
//                 let mask = (((1u64 << n) - 1) << bit_offset) >> (i * 8);
//                 slice[byte_pos + i] &= !(mask as u8);
//                 slice[byte_pos + i] |= ((v >> (i * 8)) & 0xFF) as u8;
//             }
//         }
//     }

//     #[inline(always)]
//    pub(crate) fn get_bits(
//     slice: &[u8],
//     bit_pos: usize,
//     n: usize,
// ) -> u32 {
//     let byte_pos = bit_pos / 8;
//     let bit_offset = bit_pos % 8;

//     let bytes = (n + bit_offset).div_ceil(8);
//     let mut val = 0u64;

//     for i in 0..bytes {
//         if byte_pos + i < slice.len() {
//             val |= (slice[byte_pos + i] as u64) << (i * 8);
//         }
//     }

//     let mask = if n == 32 {
//         u32::MAX as u64
//     } else {
//         (1u64 << n) - 1
//     };

//     ((val >> bit_offset) & mask) as u32
// }

// slow but hopefully correct version
pub fn set_bits(slice: &mut [u8], bit_offset: usize, bit_width: usize, value: u64) {
    let masked = value & ((1u64 << bit_width) - 1);

    for i in 0..bit_width {
        let bit = (masked >> i) & 1;
        let pos = bit_offset + i;
        let byte = pos / 8;
        let bit_in_byte = pos % 8;

        if bit == 1 {
            slice[byte] |= 1 << bit_in_byte;
        } else {
            slice[byte] &= !(1 << bit_in_byte);
        }
    }
}

pub fn get_bits(slice: &[u8], bit_offset: usize, bit_width: usize) -> u64 {
    let mut value = 0u64;

    for i in 0..bit_width {
        let pos = bit_offset + i;
        let byte = pos / 8;
        let bit_in_byte = pos % 8;

        let bit = (slice[byte] >> bit_in_byte) & 1;
        value |= (bit as u64) << i;
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_bits() {
        let mut buf = [0u8; 8];
        set_bits(&mut buf, 3, 5, 0b10101);
        assert_eq!(get_bits(&buf, 3, 5), 0b10101);
    }
}
