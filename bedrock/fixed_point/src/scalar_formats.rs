//! Common fixed-point format type aliases.
//!
//! This module provides convenient type aliases for commonly used fixed-point
//! formats. The naming convention is `FixedI_F` where I is the number of
//! integer bits and F is the number of fractional bits.

/// 8.8 fixed-point format (8 integer bits, 8 fractional bits).
///
/// Range: [-128.0, 127.99609375]
/// Precision: ~0.00390625
pub type Fixed8_8 = crate::FixedSmall<16, 8>;   // 8.8 format

/// 16.16 fixed-point format (16 integer bits, 16 fractional bits).
///
/// Range: [-32768.0, 32767.999984741]
/// Precision: ~0.000015259
pub type Fixed16_16 = crate::FixedSmall<32, 16>; // 16.16 format

/// 4.12 fixed-point format (4 integer bits, 12 fractional bits).
///
/// Range: [-8.0, 7.999755859]
/// Precision: ~0.000244141
pub type Fixed4_12 = crate::FixedSmall<16, 12>;  // 4.12 format

/// 10.6 fixed-point format (10 integer bits, 6 fractional bits).
///
/// Range: [-512.0, 511.984375]
/// Precision: ~0.015625
pub type Fixed10_6 = crate::FixedSmall<16, 6>;   // 10.6 format

/// 24.8 fixed-point format (24 integer bits, 8 fractional bits).
///
/// Range: [-8388608.0, 8388607.99609375]
/// Precision: ~0.00390625
pub type Fixed24_8 = crate::FixedSmall<32, 8>;