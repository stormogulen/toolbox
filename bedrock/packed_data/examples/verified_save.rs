
#[cfg(feature = "verified")]
use packed_data::prelude::*;
#[cfg(feature = "verified")]
use packed_data::io::{save_verified, load_verified};

#[cfg(feature = "verified")]
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct SaveData {
    level: u32,
    score: u32,
    timestamp: u64,
    checksum: u32,
}

#[cfg(feature = "verified")]
unsafe impl Pod for SaveData {}
#[cfg(feature = "verified")]
unsafe impl Zeroable for SaveData {}

#[cfg(feature = "verified")]
fn main() -> Result<()> {
    println!("=== Merkle-Verified Save System ===\n");

    // Create save data
    let saves = vec![
        SaveData {
            level: 5,
            score: 12500,
            timestamp: 1234567890,
            checksum: 0xDEADBEEF,
        },
        SaveData {
            level: 10,
            score: 50000,
            timestamp: 1234567900,
            checksum: 0xCAFEBABE,
        },
    ];

    println!("Original save data:");
    for (i, save) in saves.iter().enumerate() {
        println!("  Save {}: level={}, score={}, ts={}", 
                 i, save.level, save.score, save.timestamp);
    }

    // Save with Merkle verification
    save_verified("game.save", &saves)?;
    
    let file_size = std::fs::metadata("game.save")?.len();
    let data_size = std::mem::size_of::<SaveData>() * saves.len();
    let hash_size = 32; // SHA-256
    
    println!("\n✓ Saved with Merkle verification");
    println!("  File size: {} bytes", file_size);
    println!("  Data size: {} bytes", data_size);
    println!("  Hash size: {} bytes", hash_size);
    println!("  Overhead: {} bytes ({:.1}%)", 
             file_size as i64 - data_size as i64,
             ((file_size as f64 - data_size as f64) / data_size as f64) * 100.0);

    // Load and verify
    println!("\n=== Loading and Verifying ===");
    match load_verified::<SaveData, _>("game.save") {
        Ok(loaded) => {
            println!("✓ Integrity verified!");
            println!("Loaded {} saves:", loaded.len());
            for (i, save) in loaded.iter().enumerate() {
                println!("  Save {}: level={}, score={}, ts={}", 
                         i, save.level, save.score, save.timestamp);
            }
        }
        Err(e) => {
            println!("✗ Verification failed: {}", e);
            return Err(e.into());
        }
    }

    // Demonstrate corruption detection
    println!("\n=== Testing Corruption Detection ===");
    
    // Corrupt the file
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};
    
    {
        let mut file = OpenOptions::new().write(true).open("game.save")?;
        file.seek(SeekFrom::Start(40))?; // Skip hash, corrupt data
        file.write_all(&[0xFF, 0xFF, 0xFF, 0xFF])?;
    }
    
    println!("Corrupted file at offset 40...");
    
    match load_verified::<SaveData, _>("game.save") {
        Ok(_) => {
            println!("✗ ERROR: Corruption was not detected!");
        }
        Err(e) => {
            println!("✓ Corruption detected: {}", e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "verified"))]
fn main() {
    println!("This example requires the 'verified' feature.");
    println!("Run with: cargo run --example verified_save --features verified");
}