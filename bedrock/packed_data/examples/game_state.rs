
use packed_data::prelude::*;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct Entity {
    x: FixedSmall<32, 16>,
    y: FixedSmall<32, 16>,
    velocity_x: FixedSmall<32, 16>,
    velocity_y: FixedSmall<32, 16>,
    health: u8,
    team: u8,
}

unsafe impl Pod for Entity {}
unsafe impl Zeroable for Entity {}

fn main() -> Result<()> {
    println!("=== Game State Management Example ===\n");

    // Create entities with fixed-point positions
    let entities = EntityBuilder::new()
        .add(Entity {
            x: Fixed16_16::from_f32(10.5)?,
            y: Fixed16_16::from_f32(20.25)?,
            velocity_x: Fixed16_16::from_f32(1.5)?,
            velocity_y: Fixed16_16::from_f32(-0.75)?,
            health: 100,
            team: 0,
        })
        .add(Entity {
            x: Fixed16_16::from_f32(50.75)?,
            y: Fixed16_16::from_f32(30.5)?,
            velocity_x: Fixed16_16::from_f32(-2.0)?,
            velocity_y: Fixed16_16::from_f32(1.25)?,
            health: 80,
            team: 1,
        })
        .build();

    println!("Created {} entities", entities.len());
    for (i, e) in entities.iter().enumerate() {
        println!("  Entity {}: pos=({:.2}, {:.2}), vel=({:.2}, {:.2}), hp={}, team={}",
            i,
            e.x.to_f32(),
            e.y.to_f32(),
            e.velocity_x.to_f32(),
            e.velocity_y.to_f32(),
            e.health,
            e.team
        );
    }

    // Save game state (raw binary - fastest for game saves)
    save_raw("game_state.bin", &entities)?;
    println!("\n✓ Game state saved ({} bytes)", 
             std::fs::metadata("game_state.bin")?.len());

    // Load game state
    println!("\n=== Loading and Simulating ===");
    let mut loaded: Vec<Entity> = load_raw("game_state.bin")?;
    println!("Loaded {} entities", loaded.len());

    // Simulate one frame: update positions based on velocity
    for entity in &mut loaded {
        entity.x = entity.x.add(entity.velocity_x);
        entity.y = entity.y.add(entity.velocity_y);
    }

    println!("\nAfter one frame:");
    for (i, e) in loaded.iter().enumerate() {
        println!("  Entity {}: pos=({:.2}, {:.2}), hp={}",
            i, e.x.to_f32(), e.y.to_f32(), e.health);
    }

    // Optionally save with verification for extra safety
    #[cfg(feature = "verified")]
    {
        use packed_data::io::save_verified;
        println!("\n=== Verified Save ===");
        save_verified("game_state.verified", &loaded)?;
        println!("✓ Saved with Merkle verification");
    }

    Ok(())
}
