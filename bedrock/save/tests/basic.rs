//use crate save2::*;
use save::save::{load, save};
use save::save::{save_to_file, load_from_file};

use save::SaveError;
use packed_structs::PackedStructContainer;
use bytemuck_derive::{Pod, Zeroable};
//use bytemuck::Pod;
use std::fs;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
struct SaveData {
    player_id: u32,
    score: u32,
    level: u32,
}

impl SaveData {
    fn new(player_id: u32, score: u32, level: u32) -> Self {
        SaveData { player_id, score, level }
    }
}

#[test]
fn round_trip_save_load() {
    let container = PackedStructContainer::from_slice(&[
        SaveData::new(1, 9999, 7),
        SaveData::new(2, 1234, 2),
    ]);

    let path = "test_save.bin";
    save_to_file(path, &container).unwrap();

    let loaded = load_from_file::<_, SaveData>(path).unwrap();

    assert_eq!(loaded.len(), 2);
    let loaded_slice = loaded.as_slice();
    assert_eq!(loaded_slice[0], SaveData::new(1, 9999, 7));
    assert_eq!(loaded_slice[1], SaveData::new(2, 1234, 2));

    fs::remove_file(path).unwrap();
}


#[test]
fn detect_corrupt_save() {
    let container = PackedStructContainer::from_slice(&[
        SaveData::new(1, 9999, 7),
    ]);

    let path = "corrupt_test_save.bin";
    save_to_file(path, &container).unwrap();

    // Corrupt the payload (past header)
    let mut bytes = fs::read(path).unwrap();
    bytes[40] ^= 0xFF;
    fs::write(path, &bytes).unwrap();

    let result = load_from_file::<_, SaveData>(path);
    assert!(matches!(result, Err(SaveError::HashMismatch)));

    fs::remove_file(path).unwrap();
}


#[test]
fn reject_wrong_element_size() {
    let container = PackedStructContainer::from_slice(&[
        SaveData::new(1, 2, 3),
    ]);

    let path = "wrong_type.bin";
    save_to_file(path, &container).unwrap();

    let result = load_from_file::<_, u64>(path);
    assert!(matches!(result, Err(SaveError::InvalidVersion)));

    fs::remove_file(path).unwrap();
}

#[test]
fn empty_save_roundtrip() {
    let container = PackedStructContainer::<SaveData>::from_slice(&[]);

    let path = "empty.bin";
    save_to_file(path, &container).unwrap();
    let loaded = load_from_file::<_, SaveData>(path).unwrap();

    assert_eq!(loaded.len(), 0);
    fs::remove_file(path).unwrap();
}
