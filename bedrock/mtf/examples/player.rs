use mtf::{MTF, MTFType, write_slice_with_mtf};
use mtf::dynamic::DynamicContainer;
use std::io::Write;

#[derive(MTF, Copy, Clone, Debug)]
#[repr(C)]
struct Player {
    x: f32,
    y: f32,
    health: u32,
    score: u32,
}

unsafe impl bytemuck::Pod for Player {}
unsafe impl bytemuck::Zeroable for Player {}

fn main() {
    let players = vec![
        Player { x: 1.0, y: 2.0, health: 100, score: 50 },
        Player { x: 3.0, y: 4.0, health: 80, score: 120 },
    ];

    println!("Original players: {:#?}\n", players);

    // Write to file
    let mut file = std::fs::File::create("players.mtf").unwrap();
    write_slice_with_mtf(&mut file, &players).unwrap();
    file.flush().unwrap();
    drop(file); // Ensure file is closed
    
    // Check file size
    let metadata = std::fs::metadata("players.mtf").unwrap();
    println!("File size: {} bytes", metadata.len());
    println!("Expected data size: {} bytes", std::mem::size_of::<Player>() * 2);
    println!("MTF blob size: {} bytes\n", Player::mtf_type_blob().len());

    // Read dynamically
    let mut container = DynamicContainer::from_file("players.mtf").unwrap();
    
    println!("Type: {}", container.type_name().unwrap());
    println!("Fields: {:?}", container.field_names());
    println!("Count: {}\n", container.len());
    
    // Read fields (immutable iteration)
    for i in container.iter() {
        let x: &f32 = container.field(i, "x").unwrap();
        let y: &f32 = container.field(i, "y").unwrap();
        let health: &u32 = container.field(i, "health").unwrap();
        let score: &u32 = container.field(i, "score").unwrap();
        println!("Player {} - pos: ({}, {}), health: {}, score: {}", 
                 i, x, y, health, score);
    }
    
    // Modify fields (separate loop)
    println!("\nBoosting all scores by 10...");
    for i in 0..container.len() {
        container.field_mut::<u32>(i, "score").add(10);
    }
    
    println!("\nAfter score boost:");
    for i in 0..container.len() {
        let score: &u32 = container.field(i, "score").unwrap();
        println!("Player {} - new score: {}", i, score);
    }
    
    // Demonstrate builder-style chaining
    println!("\nDamaging player 0 and adjusting position:");
    container.field_mut::<u32>(0, "health")
        .sub(20)
        .apply(|h| println!("  Health after damage: {}", h));
    
    container.field_mut::<f32>(0, "x").add(5.0);
    container.field_mut::<f32>(0, "y").add(3.0);
    
    let x: &f32 = container.field(0, "x").unwrap();
    let y: &f32 = container.field(0, "y").unwrap();
    println!("  New position: ({}, {})", x, y);
}