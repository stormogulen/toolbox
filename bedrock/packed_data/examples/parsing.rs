use packed_data::prelude::*;
use packed_data::{batch, try_parse_iter};
use packed_data::iter::{iter_parse, SliceParseExt};
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
struct Packet {
    id: u32,
    timestamp: u32,
    value: u16,
    flags: u16,
}

impl TryFrom<&[u8]> for Packet {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> std::result::Result<Self, Self::Error> {
        if bytes.len() < 12 {
            return Err("packet too short");
        }

        Ok(Packet {
            id: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            timestamp: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            value: u16::from_le_bytes(bytes[8..10].try_into().unwrap()),
            flags: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
        })
    }
}

fn main() -> Result<()> {
    println!("=== Rust Idioms for Parsing ===\n");

    // Create sample packet data
    let mut data = Vec::new();
    for i in 0..5 {
        let id = i as u32;

        data.extend_from_slice(&id.to_le_bytes());
        data.extend_from_slice(&(1000u32 + id).to_le_bytes());
        //data.extend_from_slice(&(i as u32).to_le_bytes());          // id
        //data.extend_from_slice(&(1000 + i).to_le_bytes());         // timestamp
        data.extend_from_slice(&((i * 10) as u16).to_le_bytes());  // value
        data.extend_from_slice(&(0x01u16).to_le_bytes());          // flags
    }

    println!("Raw data: {} bytes", data.len());

    // ------------------------------------------------------------
    // 1. Fixed-size parsing with try_parse_iter
    // ------------------------------------------------------------
    println!("\n1. Using try_parse_iter:");

    let packets: Vec<Packet> =
        try_parse_iter::<Packet, _, 12>(&data)
            .collect::<std::result::Result<_, _>>()?;

    for packet in &packets {
        println!(
            "  Packet {}: ts={}, val={}, flags={:04x}",
            packet.id, packet.timestamp, packet.value, packet.flags
        );
    }

    // ------------------------------------------------------------
    // 2. Using batch for typed fixed-size groups
    // ------------------------------------------------------------
    println!("\n2. Using batch for u32 groups:");

    let words: &[u32] = bytemuck::cast_slice(&data);
    if let Some(first_three) = batch::<3, _>(words, 0) {
        println!("  First 3 u32s: {:?}", first_three);
    }

    // ------------------------------------------------------------
    // 3. Using SliceParseExt::try_parse
    // ------------------------------------------------------------
    println!("\n3. Using slice extension trait:");

    let parsed: Vec<Packet> = data
        .try_parse(12, |chunk| Packet::try_from(chunk))
        .collect::<std::result::Result<_, _>>()?;

    println!("  Parsed {} packets", parsed.len());

    // ------------------------------------------------------------
    // 4. Stateful parsing with iter_parse
    // ------------------------------------------------------------
    println!("\n4. Using iter_parse with state:");

    let mut pos = 0;
    let packets_iter: Vec<Packet> = iter_parse(|| {
        if pos + 12 <= data.len() {
            let pkt = Packet::try_from(&data[pos..pos + 12]).ok()?;
            pos += 12;
            Some(pkt)
        } else {
            None
        }
    })
    .collect();

    println!(
        "  Parsed {} packets via iter_parse",
        packets_iter.len()
    );

    Ok(())
}
