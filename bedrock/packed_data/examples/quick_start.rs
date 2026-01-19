
use packed_data::prelude::*;

#[derive(MTF, Copy, Clone, Debug)]
#[repr(C)]
struct Player {
    x: i16,
    y: i16,
    health: u8,
    mana: u8,
}

unsafe impl Pod for Player {}
unsafe impl Zeroable for Player {}

fn main() -> Result<()> {
    println!("=== Packed Data Quick Start ===\n");

    // Create some players
    let players = vec![
        Player { x: 100, y: 200, health: 100, mana: 50 },
        Player { x: 150, y: 250, health: 80, mana: 75 },
        Player { x: 200, y: 300, health: 90, mana: 60 },
    ];

    println!("Original players:");
    for (i, p) in players.iter().enumerate() {
        println!("  Player {}: pos=({}, {}), hp={}, mp={}", i, p.x, p.y, p.health, p.mana);
    }

    // Save with metadata
    save_with_metadata("players.mtf", &players)?;
    println!("\n✓ Saved {} players with metadata", players.len());

    // Save raw (more compact, no metadata)
    save_raw("players.raw", &players)?;
    
    let mtf_size = std::fs::metadata("players.mtf")?.len();
    let raw_size = std::fs::metadata("players.raw")?.len();
    println!("✓ Saved raw data");
    println!("  MTF size: {} bytes", mtf_size);
    println!("  Raw size: {} bytes", raw_size);
    println!("  Metadata overhead: {} bytes", mtf_size - raw_size);

    // Load dynamically with MTF
    println!("\n=== Dynamic Loading ===");
    let mut container = load_dynamic("players.mtf")?;
    
    println!("Type: {}", container.type_name()?);
    println!("Fields: {:?}", container.field_names());
    println!("Count: {}", container.len());

    println!("\nReading fields dynamically:");
    for i in container.iter() {
        let health: &u8 = container.field(i, "health").unwrap();
        let mana: &u8 = container.field(i, "mana").unwrap();
        println!("  Player {}: health={}, mana={}", i, health, mana);
    }

    // Modify through dynamic API
    println!("\n=== Modifying Data ===");
    for i in 0..container.len() {
        container.field_mut::<u8>(i, "health").add(10);
        container.field_mut::<u8>(i, "mana").sub(5);
    }

    println!("After modification:");
    for i in container.iter() {
        let health: &u8 = container.field(i, "health").unwrap();
        let mana: &u8 = container.field(i, "mana").unwrap();
        println!("  Player {}: health={}, mana={}", i, health, mana);
    }

    // Load raw (fastest, no overhead)
    println!("\n=== Raw Loading ===");
    let loaded: Vec<Player> = load_raw("players.raw")?;
    println!("Loaded {} players from raw file", loaded.len());
    for (i, p) in loaded.iter().enumerate() {
        println!("  Player {}: {:?}", i, p);
    }

    Ok(())
}