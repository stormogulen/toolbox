//! Property-based tests for Storage<T> (in-memory backend)

use proptest::prelude::*;
use raw_bytes::storage::Storage; // <-- adjust crate if needed

use bytemuck_derive::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Zeroable, Pod)]
struct Packet {
    id: u32,
    value: f32,
}

// Strategy for generating arbitrary Packets
fn packet_strategy() -> impl Strategy<Value = Packet> {
    (any::<u32>(), any::<f32>()).prop_map(|(id, value)| Packet { id, value })
}

proptest! {
    // -------------------------------------------------------------
    // 1. Pushing random values should store them correctly.
    // -------------------------------------------------------------
    #[test]
    fn prop_push_and_get(ref packets in prop::collection::vec(packet_strategy(), 1..200)) {
        let mut storage = Storage::new_in_memory();

        // Push everything
        for p in packets.iter().copied() {
            storage.push(p).unwrap();
        }

        // Length must match
        prop_assert_eq!(storage.len(), packets.len());

        // Every get() must return the exact same packet
        for (i, original) in packets.iter().enumerate() {
            prop_assert_eq!(storage.get(i).unwrap(), original);
        }
    }

    // -------------------------------------------------------------
    // 2. Mutating values via get_mut must be visible via get.
    // -------------------------------------------------------------
    #[test]
    fn prop_mutation_works(
        ref packets in prop::collection::vec(packet_strategy(), 1..200),
        new_value in any::<f32>()
    ) {
        let mut storage = Storage::new_in_memory();

        for p in packets.iter().copied() {
            storage.push(p).unwrap();
        }

        // mutate a random index
        //let idx = (packets.len() - 1)
        //    .min(usize::from(new_value.to_bits() % (packets.len() as u32)) as usize);
        // mutate a random index
        let idx = (new_value.to_bits() as usize) % packets.len();

        let mut_ref = storage.get_mut(idx).unwrap();
        mut_ref.value = new_value;

        // check via get()
        let got = storage.get(idx).unwrap();
        prop_assert_eq!(got.value, new_value);
    }

    // -------------------------------------------------------------
    // 3. Out-of-bounds checks always fail.
    // -------------------------------------------------------------
    #[test]
    fn prop_out_of_bounds(ref packets in prop::collection::vec(packet_strategy(), 0..200)) {
        let mut storage = Storage::new_in_memory();
        for p in packets.iter().copied() {
            storage.push(p).unwrap();
        }

        let len = storage.len();

        if len > 0 {
            prop_assert!(storage.get(len).is_err());
            prop_assert!(storage.get_mut(len).is_err());
        } else {
            // len == 0 -> index 0 must be out of bounds
            prop_assert!(storage.get(0).is_err());
            prop_assert!(storage.get_mut(0).is_err());
        }
    }

    // -------------------------------------------------------------
    // 4. get_mut must give a unique &mut reference.
    // (aliasing safety)
    // -------------------------------------------------------------
    #[test]
    fn prop_unique_mut_refs(
        a in packet_strategy(),
        b in packet_strategy(),
        new in packet_strategy()
    ) {
        let mut storage = Storage::new_in_memory();
        storage.push(a).unwrap();
        storage.push(b).unwrap();

        // mutate index 0
        {
            let m = storage.get_mut(0).unwrap();
            *m = new;
        }

        // index 1 must remain unchanged
        prop_assert_eq!(storage.get(1).unwrap(), &b);

        // index 0 must contain the new value
        prop_assert_eq!(storage.get(0).unwrap(), &new);
    }
}
