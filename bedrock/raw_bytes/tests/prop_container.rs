//! Property-based tests for raw_bytes container & storage.

use proptest::prelude::*;
use tempfile::NamedTempFile;
//use bytemuck_derive::Pod;
//use bytemuck_derive::Zeroable;

use raw_bytes::Container;

use std::io::Write;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct P {
    a: u32,
    b: u32,
}

fn write_u32_vec_to_file(values: &[u32], file: &NamedTempFile) {
    let mut f = file.reopen().unwrap();
    for v in values {
        f.write_all(&v.to_ne_bytes()).unwrap();
    }
    f.flush().unwrap();
}

/// A sample POD type for mmap roundtrip
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct Pt(u32);

//
// -----------------------------------------------------------------------------
// In-Memory Properties
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_in_memory_push_read(values: Vec<(u32,u32)>) {
        let mut c = Container::<P>::new();

        for (a,b) in &values {
            c.push(P{ a:*a, b:*b }).unwrap();
        }

        prop_assert_eq!(c.len(), values.len());

        for (i, (a,b)) in values.iter().enumerate() {
            let v = c.get(i).unwrap();
            prop_assert_eq!(v.a, *a);
            prop_assert_eq!(v.b, *b);
        }
    }
}

proptest! {
    #[test]
    fn prop_in_memory_random_write(values: Vec<u32>, index in 0usize..1000, new_val: u32) {
        let mut c = Container::<Pt>::new();

        for v in &values {
            c.push(Pt(*v)).unwrap();
        }

        if !values.is_empty() {
            let i = index % values.len();
            c.write(i, Pt(new_val)).unwrap();
            prop_assert_eq!(c.get(i).unwrap().0, new_val);
        }
    }
}

//
// -----------------------------------------------------------------------------
// Mmap Read-Only Properties
// -----------------------------------------------------------------------------

#[cfg(feature = "mmap")]
proptest! {
    #[test]
    fn prop_mmap_readonly_roundtrip(values: Vec<u32>) {
        // Create file
        let file = NamedTempFile::new().unwrap();
        write_u32_vec_to_file(&values, &file);

        // Load mmap
        let c = Container::<u32>::mmap_readonly(file.path()).unwrap();

        // Check length
        prop_assert_eq!(c.len(), values.len());

        // Check data
        for (i, v) in values.iter().enumerate() {
            prop_assert_eq!(*c.get(i).unwrap(), *v);
        }
    }
}

//
// -----------------------------------------------------------------------------
// Mmap Read-Write Properties
// -----------------------------------------------------------------------------

#[cfg(feature = "mmap")]
proptest! {
    #[test]
    fn prop_mmap_readwrite_write_persists(values: Vec<u32>, new_val: u32) {
        if values.is_empty() {
            return Ok(());
        }

        // Setup file
        let file = NamedTempFile::new().unwrap();
        write_u32_vec_to_file(&values, &file);

        // Write with mmap read-write
        {
            let mut c = Container::<u32>::mmap_readwrite(file.path()).unwrap();
            c.write(0, new_val).unwrap();
        } // drop → flush mmap

        // Reload read-only
        let c2 = Container::<u32>::mmap_readonly(file.path()).unwrap();
        prop_assert_eq!(*c2.get(0).unwrap(), new_val);
        //Ok(())
    }
}

//
// -----------------------------------------------------------------------------
// Memory Safety Invariants
// -----------------------------------------------------------------------------

// Invariant: get() must always return aligned references
proptest! {
    #[test]
    fn prop_in_memory_alignment(values: Vec<(u32,u32)>) {
        //use std::ptr;

        let mut c = Container::<P>::new();
        for (a,b) in &values {
            c.push(P{ a:*a, b:*b }).unwrap();
        }

        for i in 0..c.len() {
            let ptr = c.get(i).unwrap() as *const P as usize;
            // aligned to 4 bytes (u32) or 8 bytes (two u32) depending on struct
            let alignment = std::mem::align_of::<P>();
            prop_assert_eq!(ptr % alignment, 0);
        }
    }
}

// Invariant: get_mut() must never alias mutable references
proptest! {
    #[test]
    fn prop_in_memory_no_alias(values: Vec<u32>) {
        let mut c = Container::<Pt>::new();
        for v in &values {
            c.push(Pt(*v)).unwrap();
        }

        if c.len() > 1 {
            let a_ptr = c.get_mut(0).unwrap() as *mut Pt;
            let b_ptr = c.get_mut(1).unwrap() as *mut Pt;
            prop_assert!(a_ptr != b_ptr);
        }
    }
}

//
// -----------------------------------------------------------------------------
// Stability: get() always returns the same reference for same index (no realloc)
// -----------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_in_memory_stable_references(values: Vec<u32>) {
        //use std::ptr;

        let mut c = Container::<Pt>::new();
        for v in &values {
            c.push(Pt(*v)).unwrap();
        }

        if c.len() > 0 {
            let first_ref = c.get(0).unwrap() as *const Pt;
            let second_ref = c.get(0).unwrap() as *const Pt;

            // in-memory vec guarantees stable reference so long as we do not grow
            prop_assert_eq!(first_ref, second_ref);
        }
    }
}
