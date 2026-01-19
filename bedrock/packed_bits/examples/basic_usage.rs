use packed_bits::{FlagsContainer, PackedBitsContainer};

fn main() {
    println!("=== Packed Bits Examples ===\n");

    // Example 1: Storing small integers
    let _ = example_small_integers();

    // Example 2: Flag permissions
    let _ = example_permissions();

    // Example 3: Memory comparison
    let _ = example_memory_savings();
}

fn example_small_integers() -> Result<(), packed_bits::PackedBitsError> {
    println!("Example 1: Storing RGB color indices (5 bits each)");

    let mut colors = PackedBitsContainer::<5>::new_in_memory()?;

    // Store palette indices (0-31)
    colors.push(15).unwrap(); // Red shade
    colors.push(8).unwrap(); // Green shade
    colors.push(23).unwrap(); // Blue shade

    println!("  Stored {} colors", colors.len());
    println!("  Color 0: {}", colors.get(0).unwrap());
    println!("  Color 1: {}", colors.get(1).unwrap());
    println!("  Color 2: {}", colors.get(2).unwrap());
    println!();

    Ok(())
}

fn example_permissions() {
    println!("Example 2: File permissions using flags");

    const READ: u32 = 1 << 0;
    const WRITE: u32 = 1 << 1;
    const EXECUTE: u32 = 1 << 2;

    let mut perms = FlagsContainer::<3>::new_in_memory().unwrap();

    // File 0: read-only
    perms.push(READ).unwrap();

    // File 1: read+write
    perms.push(READ | WRITE).unwrap();

    // File 2: all permissions
    perms.push(READ | WRITE | EXECUTE).unwrap();

    println!(
        "  File 0 - Read: {}, Write: {}, Execute: {}",
        perms.contains(0, READ),
        perms.contains(0, WRITE),
        perms.contains(0, EXECUTE)
    );

    println!("  File 2 permissions:");
    for flag in perms.iter_flags(2).unwrap() {
        println!("    - Flag: 0b{:03b}", flag.trailing_zeros());
    }
    println!();
}

fn example_memory_savings() -> Result<(), packed_bits::PackedBitsError> {
    println!("Example 3: Memory savings comparison");

    let count = 10_000;

    // Standard Vec<u32>
    let standard_bytes = count * 4;

    // PackedBitsContainer<12> (values 0-4095)
    let mut packed = PackedBitsContainer::<12>::new_in_memory()?;
    for i in 0..count {
        packed.push(i % 4096).unwrap();
    }
    let packed_bytes = packed.storage().as_slice().len();

    let savings = 100.0 * (1.0 - (packed_bytes as f64 / standard_bytes as f64));

    println!("  Storing {} 12-bit values:", count);
    println!("  Vec<u32>: {} bytes", standard_bytes);
    println!("  Packed:   {} bytes", packed_bytes);
    println!("  Savings:  {:.1}%", savings);

    Ok(())
}
