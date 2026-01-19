use packed_data::prelude::*;

// --- Define your entity ---
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct Entity {
    x: FixedSmall<32, 16>,
    y: FixedSmall<32, 16>,
    vx: FixedSmall<32, 16>,
    vy: FixedSmall<32, 16>,
    health: u8,
}

unsafe impl Pod for Entity {}
unsafe impl Zeroable for Entity {}

fn main() -> Result<()> {
    println!("=== EntityBuilder Example ===\n");

    // --- Build entities with fixed-point positions ---
    let mut entities: Vec<Entity> = EntityBuilder::new()
        .try_add::<Box<dyn std::error::Error>>(Ok(Entity {
            x: Fixed16_16::from_f32(10.5)?,
            y: Fixed16_16::from_f32(20.25)?,
            vx: Fixed16_16::from_f32(1.5)?,
            vy: Fixed16_16::from_f32(-0.75)?,
            health: 100,
        }))?
        .try_add::<Box<dyn std::error::Error>>(Ok(Entity {
            x: Fixed16_16::from_f32(50.75)?,
            y: Fixed16_16::from_f32(30.5)?,
            vx: Fixed16_16::from_f32(-2.0)?,
            vy: Fixed16_16::from_f32(1.25)?,
            health: 80,
        }))?
        .build();

    println!("Created {} entities", entities.len());
    for (i, e) in entities.iter().enumerate() {
        println!(
            "  Entity {}: pos=({:.2}, {:.2}), vel=({:.2}, {:.2}), hp={}",
            i,
            e.x.to_f32(),
            e.y.to_f32(),
            e.vx.to_f32(),
            e.vy.to_f32(),
            e.health
        );
    }

    // --- Save game state raw ---
    save_raw("game_state.bin", &entities)?;
    println!("\n✓ Game state saved ({} bytes)", std::fs::metadata("game_state.bin")?.len());

    // --- Load game state ---
    let mut loaded: Vec<Entity> = load_raw("game_state.bin")?;
    println!("\nLoaded {} entities", loaded.len());

    // --- Simulate one frame: update positions based on velocity ---
    for entity in &mut loaded {
        entity.x = entity.x.add(entity.vx);
        entity.y = entity.y.add(entity.vy);
    }

    println!("\nAfter one frame:");
    for (i, e) in loaded.iter().enumerate() {
        println!(
            "  Entity {}: pos=({:.2}, {:.2}), hp={}",
            i, e.x.to_f32(), e.y.to_f32(), e.health
        );
    }

    Ok(())
}
