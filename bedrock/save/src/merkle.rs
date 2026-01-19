//! Merkle-compatible hashing utilities.
//!
//! Computes a Merkle root over a byte slice without allocating
//! or storing tree nodes. Hashes are compatible with standard
//! Merkle tree constructions.

use blake3::Hasher;

/// Hash a leaf chunk.
#[inline]
fn hash_leaf(bytes: &[u8]) -> blake3::Hash {
    let mut h = Hasher::new();
    h.update(bytes);
    h.finalize()
}

/// Hash two child hashes into a parent.
#[inline]
fn hash_parent(left: &blake3::Hash, right: &blake3::Hash) -> blake3::Hash {
    let mut h = Hasher::new();
    h.update(left.as_bytes());
    h.update(right.as_bytes());
    h.finalize()
}

/// Compute a Merkle-compatible root hash over `data`.
///
/// - `chunk_size` defines leaf granularity
/// - no heap allocation
/// - no stored tree
///
/// If the number of leaves is odd, the last hash is duplicated
/// (standard Merkle behavior).
pub fn merkle_root(data: &[u8], chunk_size: usize) -> blake3::Hash {
    assert!(chunk_size > 0);

    let mut level = [blake3::Hash::from([0u8; 32]); 64];
    let mut level_len = [0usize; 64];

    // --- Leaf hashing ---
    for chunk in data.chunks(chunk_size) {
        let mut h = hash_leaf(chunk);
        let mut depth = 0;

        loop {
            if level_len[depth] == 0 {
                level[depth] = h;
                level_len[depth] = 1;
                break;
            }

            h = hash_parent(&level[depth], &h);
            level_len[depth] = 0;
            depth += 1;
        }
    }

    // --- Fold remaining nodes ---
    let mut root: Option<blake3::Hash> = None;

    for depth in 0..64 {
        if level_len[depth] == 1 {
            root = Some(match root {
                None => level[depth],
                Some(r) => hash_parent(&level[depth], &r),
            });
        }
    }

    root.unwrap_or_else(|| hash_leaf(&[]))
}
