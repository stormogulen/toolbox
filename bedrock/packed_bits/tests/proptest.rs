// // tests/proptest.rs

// #![cfg(test)]

// use proptest::prelude::*;
// use packed_bits::{PackedBitsContainer, FlagsContainer};

// #[cfg(feature = "mmap")]
// use tempfile::NamedTempFile;

// //
// // -----------------------------------------------------------------------------
// // Helper Functions
// // -----------------------------------------------------------------------------

// /// Generate values that fit in N bits
// // fn value_for_bits(n: usize) -> impl Strategy<Value = u32> {
// //     let max_val = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
// //     0..=max_val
// // }

// //
// // -----------------------------------------------------------------------------
// // PackedBitsContainer Properties - Basic Operations
// // -----------------------------------------------------------------------------

// proptest! {
//     #[test]
//     fn prop_push_and_get_roundtrip(values in prop::collection::vec(0u32..4096, 0..1000)) {
//         let mut container = PackedBitsContainer::<12>::new_in_memory().unwrap();

//         for &v in &values {
//             container.push(v).unwrap();
//         }

//         prop_assert_eq!(container.len(), values.len());

//         for (i, &expected) in values.iter().enumerate() {
//             prop_assert_eq!(container.get(i), Some(expected));
//         }
//     }
// }

// proptest! {
//     #[test]
//     fn prop_set_updates_correctly(
//         values in prop::collection::vec(0u32..128, 1..100),
//         update_idx in 0usize..100,
//         new_val in 0u32..128
//     ) {
//         let mut container = PackedBitsContainer::<7>::new_in_memory().unwrap;

//         for &v in &values {
//             container.push(v).unwrap();
//         }

//         if !values.is_empty() {
//             let idx = update_idx % values.len();
//             container.set(idx, new_val).unwrap();
//             prop_assert_eq!(container.get(idx), Some(new_val));

//             // Other values unchanged
//             for (i, &expected) in values.iter().enumerate() {
//                 if i != idx {
//                     prop_assert_eq!(container.get(i), Some(expected));
//                 }
//             }
//         }
//     }
// }

// proptest! {
//     #[test]
//     fn prop_iterator_matches_get(values in prop::collection::vec(0u32..256, 0..500)) {
//         let mut container = PackedBitsContainer::<8>::new_in_memory();

//         for &v in &values {
//             container.push(v).unwrap();
//         }

//         let collected: Vec<_> = container.iter().collect();
//         prop_assert_eq!(collected, values);
//     }
// }

// //
// // -----------------------------------------------------------------------------
// // PackedBitsContainer Properties - Edge Cases
// // -----------------------------------------------------------------------------

// proptest! {
//     #[test]
//     fn prop_n32_full_range(values in prop::collection::vec(any::<u32>(), 0..100)) {
//         let mut container = PackedBitsContainer::<32>::new_in_memory();

//         for &v in &values {
//             container.push(v).unwrap();
//         }

//         for (i, &expected) in values.iter().enumerate() {
//             prop_assert_eq!(container.get(i), Some(expected));
//         }
//     }
// }

// proptest! {
//     #[test]
//     fn prop_n1_boolean_storage(values in prop::collection::vec(any::<bool>(), 0..1000)) {
//         let mut container = PackedBitsContainer::<1>::new_in_memory();

//         for &v in &values {
//             container.push(v as u32).unwrap();
//         }

//         prop_assert_eq!(container.len(), values.len());

//         for (i, &expected) in values.iter().enumerate() {
//             prop_assert_eq!(container.get(i), Some(expected as u32));
//         }
//     }
// }

// proptest! {
//     #[test]
//      fn prop_various_bit_widths(
//         n in 1usize..=32,
//         count in 0usize..100
//     ) {
//         // Generate values that fit in n bits
//         let max_val = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
//         let values: Vec<u32> = (0..count).map(|i| {
//             if n == 32 {
//                 (i as u32).wrapping_mul(1234567) // Use wrapping to avoid overflow for n=32
//             } else {
//                 (i as u32) % (max_val + 1)
//             }
//         }).collect();

//         match n {
//             1 => test_container::<1>(&values)?,
//             2 => test_container::<2>(&values)?,
//             3 => test_container::<3>(&values)?,
//             4 => test_container::<4>(&values)?,
//             5 => test_container::<5>(&values)?,
//             6 => test_container::<6>(&values)?,
//             7 => test_container::<7>(&values)?,
//             8 => test_container::<8>(&values)?,
//             9 => test_container::<9>(&values)?,
//             10 => test_container::<10>(&values)?,
//             11 => test_container::<11>(&values)?,
//             12 => test_container::<12>(&values)?,
//             13 => test_container::<13>(&values)?,
//             14 => test_container::<14>(&values)?,
//             15 => test_container::<15>(&values)?,
//             16 => test_container::<16>(&values)?,
//             24 => test_container::<24>(&values)?,
//             32 => test_container::<32>(&values)?,
//             _ => {},
//         }
//     }
// }

// fn test_container<const N: usize>(values: &[u32]) -> Result<(), proptest::test_runner::TestCaseError> {
//     let mut container = PackedBitsContainer::<N>::new_in_memory();

//     for &v in values {
//         container.push(v).unwrap();
//     }

//     for (i, &expected) in values.iter().enumerate() {
//         prop_assert_eq!(container.get(i), Some(expected));
//     }

//     Ok(())
// }

// //
// // -----------------------------------------------------------------------------
// // PackedBitsContainer Properties - Persistence
// // -----------------------------------------------------------------------------

// proptest! {
//     #[test]
//     fn prop_header_roundtrip(values in prop::collection::vec(0u32..1024, 0..200)) {
//         let mut container = PackedBitsContainer::<10>::new_in_memory();

//         for &v in &values {
//             container.push(v).unwrap();
//         }

//         // Serialize
//         let bytes = container.storage().as_slice().to_vec();

//         // Deserialize
//         let storage = raw_bytes::Container::from_slice(&bytes);
//         let restored = PackedBitsContainer::<10>::from_storage(storage).unwrap();

//         prop_assert_eq!(restored.len(), values.len());

//         for (i, &expected) in values.iter().enumerate() {
//             prop_assert_eq!(restored.get(i), Some(expected));
//         }
//     }
// }

// proptest! {
//     #[test]
//     fn prop_clear_empties_container(values in prop::collection::vec(0u32..256, 1..100)) {
//         let mut container = PackedBitsContainer::<8>::new_in_memory();

//         for &v in &values {
//             container.push(v).unwrap();
//         }

//         prop_assert!(container.len() > 0);

//         container.clear().unwrap();

//         prop_assert_eq!(container.len(), 0);
//         prop_assert!(container.is_empty());
//     }
// }

// //
// // -----------------------------------------------------------------------------
// // FlagsContainer Properties
// // -----------------------------------------------------------------------------

// proptest! {
//     #[test]
//     fn prop_flags_set_clear_toggle(
//         initial_flags in prop::collection::vec(0u32..8, 0..100),
//         operations in prop::collection::vec((0usize..100, 0u32..8, 0u8..3), 0..50)
//     ) {
//         let mut fc = FlagsContainer::<3>::new_in_memory();

//         for &flags in &initial_flags {
//             fc.push(flags).unwrap();
//         }

//         for (idx, mask, op) in operations {
//             if fc.is_empty() {
//                 continue;
//             }

//             let i = idx % fc.len();
//             let m = 1u32 << (mask % 3);

//             match op {
//                 0 => { fc.set_mask(i, m).unwrap(); }
//                 1 => { fc.clear_mask(i, m).unwrap(); }
//                 2 => { fc.toggle_mask(i, m).unwrap(); }
//                 _ => {}
//             }
//         }

//         // Container should still be valid
//         prop_assert_eq!(fc.len(), initial_flags.len());
//     }
// }

// proptest! {
//     #[test]
//     fn prop_flags_contains_accurate(
//         flags in prop::collection::vec(0u32..256, 0..100)
//     ) {
//         let mut fc = FlagsContainer::<8>::new_in_memory();

//         for &f in &flags {
//             fc.push(f).unwrap();
//         }

//         for (i, &f) in flags.iter().enumerate() {
//             for bit in 0..8 {
//                 let mask = 1u32 << bit;
//                 let expected = (f & mask) != 0;
//                 prop_assert_eq!(fc.contains(i, mask), expected);
//             }
//         }
//     }
// }

// proptest! {
//     #[test]
//     fn prop_flags_iter_flags_correct(flags in 0u32..256) {
//         let mut fc = FlagsContainer::<8>::new_in_memory();
//         fc.push(flags).unwrap();

//         let collected: Vec<_> = fc.iter_flags(0).unwrap().collect();

//         // Check that all returned flags are set
//         for &flag in &collected {
//             prop_assert!((flags & flag) != 0);
//         }

//         // Check that we found all set flags
//         let mut expected_count = 0;
//         for bit in 0..8 {
//             if (flags & (1 << bit)) != 0 {
//                 expected_count += 1;
//             }
//         }
//         prop_assert_eq!(collected.len(), expected_count);
//     }
// }

// //
// // -----------------------------------------------------------------------------
// // Invariants - No Data Corruption
// // -----------------------------------------------------------------------------

// proptest! {
//     #[test]
//     fn prop_no_cross_contamination(
//         values1 in prop::collection::vec(0u32..128, 10..50),
//         values2 in prop::collection::vec(0u32..128, 10..50)
//     ) {
//         let mut c1 = PackedBitsContainer::<7>::new_in_memory();
//         let mut c2 = PackedBitsContainer::<7>::new_in_memory();

//         for &v in &values1 {
//             c1.push(v).unwrap();
//         }

//         for &v in &values2 {
//             c2.push(v).unwrap();
//         }

//         // Verify c1 hasn't been affected by c2
//         for (i, &expected) in values1.iter().enumerate() {
//             prop_assert_eq!(c1.get(i), Some(expected));
//         }

//         // Verify c2 hasn't been affected by c1
//         for (i, &expected) in values2.iter().enumerate() {
//             prop_assert_eq!(c2.get(i), Some(expected));
//         }
//     }
// }

// proptest! {
//     #[test]
//    fn prop_boundary_values(n in 1usize..=16) {
//         let max_val = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
//         let boundary_values = vec![0, 1, max_val / 2, max_val - 1, max_val];

//         match n {
//             1 => test_boundaries::<1>(&boundary_values)?,  // Added ?
//             2 => test_boundaries::<2>(&boundary_values)?,
//             3 => test_boundaries::<3>(&boundary_values)?,
//             4 => test_boundaries::<4>(&boundary_values)?,
//             5 => test_boundaries::<5>(&boundary_values)?,
//             6 => test_boundaries::<6>(&boundary_values)?,
//             7 => test_boundaries::<7>(&boundary_values)?,
//             8 => test_boundaries::<8>(&boundary_values)?,
//             9 => test_boundaries::<9>(&boundary_values)?,
//             10 => test_boundaries::<10>(&boundary_values)?,
//             11 => test_boundaries::<11>(&boundary_values)?,
//             12 => test_boundaries::<12>(&boundary_values)?,
//             13 => test_boundaries::<13>(&boundary_values)?,
//             14 => test_boundaries::<14>(&boundary_values)?,
//             15 => test_boundaries::<15>(&boundary_values)?,
//             16 => test_boundaries::<16>(&boundary_values)?,
//             _ => {},
//         }
//     }
// }

// fn test_boundaries<const N: usize>(values: &[u32]) -> Result<(), proptest::test_runner::TestCaseError> {
//     let max_val = if N == 32 { u32::MAX } else { (1u32 << N) - 1 };
//     let valid_values: Vec<_> = values.iter().filter(|&&v| v <= max_val).copied().collect();

//     let mut container = PackedBitsContainer::<N>::new_in_memory();

//     for &v in &valid_values {
//         container.push(v).unwrap();
//     }

//     for (i, &expected) in valid_values.iter().enumerate() {
//         prop_assert_eq!(container.get(i), Some(expected));
//     }

//     Ok(())
// }

// //
// // -----------------------------------------------------------------------------
// // Capacity and Memory Properties
// // -----------------------------------------------------------------------------

// proptest! {
//     #[test]
//     fn prop_capacity_grows_appropriately(count in 0usize..500) {
//         let mut container = PackedBitsContainer::<12>::new_in_memory();

//         for i in 0..count {
//             container.push(i as u32 % 4096).unwrap();
//             prop_assert!(container?.capacity() >= container?.len());
//         }
//     }
// }

// proptest! {
//     #[test]
//     fn prop_with_capacity_preallocates(capacity in 10usize..500) {
//         let container = PackedBitsContainer::<8>::with_capacity(capacity);

//         prop_assert!(container?.capacity() >= capacity);
//         prop_assert_eq!(container?.len(), 0);
//     }
// }

// tests/proptest.rs

#![cfg(test)]

use packed_bits::{FlagsContainer, PackedBitsContainer};
use proptest::prelude::*;

#[cfg(feature = "mmap")]
use tempfile::NamedTempFile;

//
// -----------------------------------------------------------------------------
// PackedBitsContainer Properties - Basic Operations
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_push_and_get_roundtrip(values in prop::collection::vec(0u32..4096, 0..1000)) {
        let mut container = PackedBitsContainer::<12>::new_in_memory().unwrap();

        for &v in &values {
            container.push(v).unwrap();
        }

        prop_assert_eq!(container.len(), values.len());

        for (i, &expected) in values.iter().enumerate() {
            prop_assert_eq!(container.get(i), Some(expected));
        }
    }
}

proptest! {
    #[test]
    fn prop_set_updates_correctly(
        values in prop::collection::vec(0u32..128, 1..100),
        update_idx in 0usize..100,
        new_val in 0u32..128
    ) {
        let mut container = PackedBitsContainer::<7>::new_in_memory().unwrap();

        for &v in &values {
            container.push(v).unwrap();
        }

        if !values.is_empty() {
            let idx = update_idx % values.len();
            container.set(idx, new_val).unwrap();
            prop_assert_eq!(container.get(idx), Some(new_val));

            for (i, &expected) in values.iter().enumerate() {
                if i != idx {
                    prop_assert_eq!(container.get(i), Some(expected));
                }
            }
        }
    }
}

proptest! {
    #[test]
    fn prop_iterator_matches_get(values in prop::collection::vec(0u32..256, 0..500)) {
        let mut container = PackedBitsContainer::<8>::new_in_memory().unwrap();

        for &v in &values {
            container.push(v).unwrap();
        }

        let collected: Vec<_> = container.iter().collect();
        prop_assert_eq!(collected, values);
    }
}

//
// -----------------------------------------------------------------------------
// Edge Cases
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_n32_full_range(values in prop::collection::vec(any::<u32>(), 0..100)) {
        let mut container = PackedBitsContainer::<32>::new_in_memory().unwrap();

        for &v in &values {
            container.push(v).unwrap();
        }

        for (i, &expected) in values.iter().enumerate() {
            prop_assert_eq!(container.get(i), Some(expected));
        }
    }
}

proptest! {
    #[test]
    fn prop_n1_boolean_storage(values in prop::collection::vec(any::<bool>(), 0..1000)) {
        let mut container = PackedBitsContainer::<1>::new_in_memory().unwrap();

        for &v in &values {
            container.push(v as u32).unwrap();
        }

        prop_assert_eq!(container.len(), values.len());

        for (i, &expected) in values.iter().enumerate() {
            prop_assert_eq!(container.get(i), Some(expected as u32));
        }
    }
}

//
// -----------------------------------------------------------------------------
// Various Bit Widths
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_various_bit_widths(
        n in 1usize..=32,
        count in 0usize..100
    ) {
        let max_val = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
        let values: Vec<u32> = (0..count)
            .map(|i| (i as u32) & max_val)
            .collect();

        match n {
            1 => test_container::<1>(&values),
            2 => test_container::<2>(&values),
            3 => test_container::<3>(&values),
            4 => test_container::<4>(&values),
            5 => test_container::<5>(&values),
            6 => test_container::<6>(&values),
            7 => test_container::<7>(&values),
            8 => test_container::<8>(&values),
            9 => test_container::<9>(&values),
            10 => test_container::<10>(&values),
            11 => test_container::<11>(&values),
            12 => test_container::<12>(&values),
            13 => test_container::<13>(&values),
            14 => test_container::<14>(&values),
            15 => test_container::<15>(&values),
            16 => test_container::<16>(&values),
            24 => test_container::<24>(&values),
            32 => test_container::<32>(&values),
            _ => Ok(()),
        }?;


    }
}

fn test_container<const N: usize>(
    values: &[u32],
) -> Result<(), proptest::test_runner::TestCaseError> {
    let mut container = PackedBitsContainer::<N>::new_in_memory().unwrap();

    for &v in values {
        container.push(v).unwrap();
    }

    for (i, &expected) in values.iter().enumerate() {
        prop_assert_eq!(container.get(i), Some(expected));
    }

    Ok(())
}

//
// -----------------------------------------------------------------------------
// Persistence
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_header_roundtrip(values in prop::collection::vec(0u32..1024, 0..200)) {
        let mut container = PackedBitsContainer::<10>::new_in_memory().unwrap();

        for &v in &values {
            container.push(v).unwrap();
        }

        let bytes = container.storage().as_slice().to_vec();
        let storage = raw_bytes::Container::from_slice(&bytes);
        let restored = PackedBitsContainer::<10>::from_storage(storage).unwrap();

        prop_assert_eq!(restored.len(), values.len());

        for (i, &expected) in values.iter().enumerate() {
            prop_assert_eq!(restored.get(i), Some(expected));
        }
    }
}

//
// -----------------------------------------------------------------------------
// FlagsContainer
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_flags_set_clear_toggle(
        initial_flags in prop::collection::vec(0u32..8, 0..100),
        operations in prop::collection::vec((0usize..100, 0u32..8, 0u8..3), 0..50)
    ) {
        let mut fc = FlagsContainer::<3>::new_in_memory().unwrap();

        for &flags in &initial_flags {
            fc.push(flags).unwrap();
        }

        for (idx, mask, op) in operations {
            if fc.is_empty() {
                continue;
            }

            let i = idx % fc.len();
            let m = 1u32 << (mask % 3);

            match op {
                0 => fc.set_mask(i, m).unwrap(),
                1 => fc.clear_mask(i, m).unwrap(),
                2 => fc.toggle_mask(i, m).unwrap(),
                _ => {}
            }
        }

        prop_assert_eq!(fc.len(), initial_flags.len());
    }
}

proptest! {
    #[test]
    fn prop_flags_contains_accurate(
        flags in prop::collection::vec(0u32..256, 0..100)
    ) {
        let mut fc = FlagsContainer::<8>::new_in_memory().unwrap();

        for &f in &flags {
            fc.push(f).unwrap();
        }

        for (i, &f) in flags.iter().enumerate() {
            for bit in 0..8 {
                let mask = 1u32 << bit;
                let expected = (f & mask) != 0;
                prop_assert_eq!(fc.contains(i, mask), expected);
            }
        }
    }
}

proptest! {
    #[test]
    fn prop_flags_iter_flags_correct(flags in 0u32..256) {
        let mut fc = FlagsContainer::<8>::new_in_memory().unwrap();
        fc.push(flags).unwrap();

        let collected: Vec<_> = fc.iter_flags(0).unwrap().collect();

        for &flag in &collected {
            prop_assert!((flags & flag) != 0);
        }

        let expected_count = (0..8).filter(|b| (flags & (1 << b)) != 0).count();
        prop_assert_eq!(collected.len(), expected_count);
    }
}

//
// -----------------------------------------------------------------------------
// Capacity
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_capacity_grows_appropriately(count in 0usize..500) {
        let mut container = PackedBitsContainer::<12>::new_in_memory().unwrap();

        for i in 0..count {
            container.push(i as u32 % 4096).unwrap();
            prop_assert!(container.capacity() >= container.len());
        }
    }
}

proptest! {
    #[test]
    fn prop_with_capacity_preallocates(capacity in 10usize..500) {
        let container = PackedBitsContainer::<8>::with_capacity(capacity).unwrap();
        prop_assert!(container.capacity() >= capacity);
        prop_assert_eq!(container.len(), 0);
    }
}
