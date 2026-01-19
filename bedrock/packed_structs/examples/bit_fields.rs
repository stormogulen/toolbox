//use packed_structs::PackedBytes;
use packed_bits::PackedBits;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PackedBytes Bit-Field Examples ===\n");

    // Example 1: Simple bit flags
    example_1_bit_flags()?;

    // Example 2: Network packet header (IPv4-like)
    example_2_network_header()?;

    // Example 3: Hardware control register
    example_3_hardware_register()?;

    // Example 4: Advanced bit operations with BitView
    #[cfg(feature = "bits")]
    example_4_bit_view()?;

    Ok(())
}

fn example_1_bit_flags() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 1: Bit Flags ---");

    let mut flags = PackedBytes::<1>::new();

    // Set individual flag bits
    flags.set_bit(0, true)?; // Enable
    flags.set_bit(1, false)?; // Interrupt disabled
    flags.set_bit(2, true)?; // Debug mode
    flags.set_bit(3, true)?; // Auto-restart

    println!("Flags byte: 0b{:08b}", flags.as_bytes()[0]);
    println!("  Enable:       {}", flags.get_bit(0).unwrap());
    println!("  Interrupt:    {}", flags.get_bit(1).unwrap());
    println!("  Debug:        {}", flags.get_bit(2).unwrap());
    println!("  Auto-restart: {}", flags.get_bit(3).unwrap());

    // Flip a bit
    flags.flip_bit(1)?;
    println!("\nAfter enabling interrupt:");
    println!("  Interrupt:    {}", flags.get_bit(1).unwrap());

    println!();
    Ok(())
}

fn example_2_network_header() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 2: IPv4-like Header ---");

    let mut header = PackedBytes::<20>::new();

    // Byte 0: Version (4 bits) + IHL (4 bits)
    header.set_bits(0, 4, 4)?; // IPv4
    header.set_bits(4, 4, 5)?; // Header length: 5 * 4 = 20 bytes

    // Byte 1: DSCP (6 bits) + ECN (2 bits)
    header.set_bits(8, 6, 0)?; // DSCP = 0
    header.set_bits(14, 2, 0)?; // ECN = 0

    // Bytes 2-3: Total length (16 bits)
    header.set_bits(16, 16, 60)?; // Total packet length

    // Read back values
    let version = header.get_bits(0, 4).unwrap();
    let ihl = header.get_bits(4, 4).unwrap();
    let total_length = header.get_bits(16, 16).unwrap();

    println!("IPv4 Header:");
    println!("  Version:       {}", version);
    println!("  IHL:           {} (header size: {} bytes)", ihl, ihl * 4);
    println!("  Total Length:  {} bytes", total_length);

    // Display first few bytes
    println!("\nRaw bytes: {:02X?}", &header.as_bytes()[0..4]);

    println!();
    Ok(())
}

fn example_3_hardware_register() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 3: Hardware Control Register ---");

    // 32-bit control register with bit fields
    let mut control_reg = PackedBytes::<4>::new();

    // Bits 0-2: Mode (3 bits, values 0-7)
    control_reg.set_bits(0, 3, 5)?; // Mode 5

    // Bit 3: Enable flag
    control_reg.set_bit(3, true)?;

    // Bits 4-7: Channel selection (4 bits, values 0-15)
    control_reg.set_bits(4, 4, 12)?; // Channel 12

    // Bits 8-15: Threshold (8 bits, 0-255)
    control_reg.set_bits(8, 8, 128)?; // Threshold = 128

    // Bits 16-31: Counter (16 bits)
    control_reg.set_bits(16, 16, 1000)?; // Counter = 1000

    // Read back values
    println!("Control Register:");
    println!("  Mode:      {}", control_reg.get_bits(0, 3).unwrap());
    println!("  Enabled:   {}", control_reg.get_bit(3).unwrap());
    println!("  Channel:   {}", control_reg.get_bits(4, 4).unwrap());
    println!("  Threshold: {}", control_reg.get_bits(8, 8).unwrap());
    println!("  Counter:   {}", control_reg.get_bits(16, 16).unwrap());

    println!("\nRaw register value: 0x{:08X}", u32::from_le_bytes(*control_reg.as_bytes()));

    println!();
    Ok(())
}

#[cfg(feature = "bits")]
fn example_4_bit_view() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 4: Advanced BitView Operations ---");

    let mut buffer = PackedBytes::<8>::new();

    // Use BitView for advanced operations
    {
        let mut view = buffer.bit_view_mut();

        // Set some bits
        view.set(0, true)?;
        view.set(10, true)?;
        view.set(20, true)?;
        view.set(30, true)?;

        println!("Set bits at positions: 0, 10, 20, 30");
        println!("  Total ones: {}", view.count_ones());
        println!("  Total zeros: {}", view.count_zeros());
        println!("  First set bit: {:?}", view.find_first_set());
        println!("  First clear bit: {:?}", view.find_first_clear());
    }

    // Iterate over bits
    {
        let view = buffer.bit_view();
        println!("\nFirst 32 bits:");
        for (i, bit) in view.iter().enumerate().take(32) {
            if i % 8 == 0 {
                print!("\n  ");
            }
            print!("{}", if bit { "1" } else { "0" });
        }
        println!();
    }

    // Fill operation
    {
        let mut view = buffer.bit_view_mut();
        view.fill(true);
        println!("\nAfter fill(true): {} ones", view.count_ones());
    }

    println!();
    Ok(())
}

// Type-safe bit-field wrapper
struct ControlFlags {
    buffer: PackedBytes<2>,
}

impl ControlFlags {
    pub fn new() -> Self {
        Self {
            buffer: PackedBytes::new(),
        }
    }

    // Bit 0: Enable
    pub fn is_enabled(&self) -> bool {
        self.buffer.get_bit(0).unwrap_or(false)
    }

    pub fn set_enabled(&mut self, value: bool) {
        self.buffer.set_bit(0, value).ok();
    }

    // Bits 1-3: Mode (3 bits, values 0-7)
    pub fn mode(&self) -> u8 {
        self.buffer.get_bits(1, 3).unwrap_or(0) as u8
    }

    pub fn set_mode(&mut self, mode: u8) {
        assert!(mode < 8, "Mode must be 0-7");
        self.buffer.set_bits(1, 3, mode as u64).ok();
    }

    // Bits 4-7: Channel (4 bits, values 0-15)
    pub fn channel(&self) -> u8 {
        self.buffer.get_bits(4, 4).unwrap_or(0) as u8
    }

    pub fn set_channel(&mut self, channel: u8) {
        assert!(channel < 16, "Channel must be 0-15");
        self.buffer.set_bits(4, 4, channel as u64).ok();
    }

    // Bits 8-15: Priority (8 bits, 0-255)
    pub fn priority(&self) -> u8 {
        self.buffer.get_bits(8, 8).unwrap_or(0) as u8
    }

    pub fn set_priority(&mut self, priority: u8) {
        self.buffer.set_bits(8, 8, priority as u64).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_flags_wrapper() {
        let mut flags = ControlFlags::new();

        flags.set_enabled(true);
        flags.set_mode(5);
        flags.set_channel(12);
        flags.set_priority(200);

        assert_eq!(flags.is_enabled(), true);
        assert_eq!(flags.mode(), 5);
        assert_eq!(flags.channel(), 12);
        assert_eq!(flags.priority(), 200);
    }
}