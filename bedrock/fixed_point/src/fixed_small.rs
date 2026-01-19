use crate::error::FixedPointError;

#[cfg(feature = "std_container")]
use bytemuck::{Pod, Zeroable};

/// A fixed-point number with N total bits and F fractional bits.
///
/// Fixed-point numbers represent real numbers using a fixed number of bits
/// for the integer and fractional parts. This provides a good balance between
/// precision and performance for many applications.
///
/// # Type Parameters
///
/// - `N`: Total number of bits (must be ≤ 32)
/// - `F`: Number of fractional bits (must be < N)
///
/// # Examples
///
/// ```
/// use fixed_point::FixedSmall;
///
/// // Create a 16-bit fixed-point number with 8 fractional bits
/// let x = FixedSmall::<16, 8>::from_f32(3.14159)?;
/// assert!((x.to_f32() - 3.14159).abs() < 0.01);
///
/// // Arithmetic operations
/// let y = FixedSmall::<16, 8>::from_f32(2.0)?;
/// let sum = x.add(y);
/// assert!((sum.to_f32() - 5.14159).abs() < 0.01);
/// # Ok::<(), fixed_point::FixedPointError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FixedSmall<const N: usize, const F: usize> {
    /// The raw integer representation.
    pub raw: i32,
}

// Safety: FixedSmall is just a wrapper around i32
#[cfg(feature = "std_container")]
unsafe impl<const N: usize, const F: usize> Zeroable for FixedSmall<N, F> {}

#[cfg(feature = "std_container")]
unsafe impl<const N: usize, const F: usize> Pod for FixedSmall<N, F> {}

impl<const N: usize, const F: usize> FixedSmall<N, F> {
    const SCALE: f32 = (1 << F) as f32;
    //const MAX_INT: i32 = (1 << (N - 1)) - 1;
    //const MIN_INT: i32 = -(1 << (N - 1));
    // Handle the N=32 edge case properly
    const MAX_INT: i32 = if N >= 32 {
        i32::MAX
    } else {
        (1 << (N - 1)) - 1
    };
    
    const MIN_INT: i32 = if N >= 32 {
        i32::MIN
    } else {
        -(1 << (N - 1))
    };

    /// Creates a fixed-point number from an f32 value.
    ///
    /// # Errors
    ///
    /// Returns `FixedPointError::Overflow` if the value is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedSmall;
    ///
    /// let x = FixedSmall::<16, 8>::from_f32(3.14)?;
    /// assert!((x.to_f32() - 3.14).abs() < 0.01);
    ///
    /// // Out of range
    /// let result = FixedSmall::<8, 4>::from_f32(100.0);
    /// assert!(result.is_err());
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn from_f32(value: f32) -> Result<Self, FixedPointError> {
        let scaled = value * Self::SCALE;
    
        // Check bounds before casting to i32
        if scaled < Self::MIN_INT as f32 || scaled > Self::MAX_INT as f32 {
            return Err(FixedPointError::Overflow {
                value,
                bits: N,
                fractional: F,
            });
        }
        
        let raw = scaled.round() as i32;
        Ok(Self { raw })
    }

    /// Converts the fixed-point number to an f32.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedSmall;
    ///
    /// let x = FixedSmall::<16, 8>::from_f32(3.14)?;
    /// let f = x.to_f32();
    /// assert!((f - 3.14).abs() < 0.01);
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn to_f32(&self) -> f32 {
        self.raw as f32 / Self::SCALE
    }

    /// Creates a fixed-point number from a raw integer value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedSmall;
    ///
    /// // With 8 fractional bits, raw value 256 represents 1.0
    /// let x = FixedSmall::<16, 8>::from_raw(256);
    /// assert_eq!(x.to_f32(), 1.0);
    /// ```
    pub fn from_raw(raw: i32) -> Self {
        Self { raw }
    }

    /// Returns the raw integer representation.
    pub fn raw_value(&self) -> i32 {
        self.raw
    }

    /// Create a fixed-point value of zero
    pub const fn zero() -> Self {
        Self { raw: 0 }
    }

    /// Create a fixed-point value of one
    pub const fn one() -> Self {
        Self { raw: 1 << F }
    }

     /// Returns the maximum representable value.
    pub const fn max_value() -> Self {
        Self { raw: Self::MAX_INT }
    }

    /// Returns the minimum representable value.
    pub const fn min_value() -> Self {
        Self { raw: Self::MIN_INT }
    }
}

// Arithmetic operations
impl<const N: usize, const F: usize> FixedSmall<N, F> {

    /// Adds two fixed-point numbers with saturation.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedSmall;
    ///
    /// let x = FixedSmall::<16, 8>::from_f32(1.5)?;
    /// let y = FixedSmall::<16, 8>::from_f32(2.5)?;
    /// let sum = x.add(y);
    /// assert_eq!(sum.to_f32(), 4.0);
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn add(self, other: Self) -> Self {
        Self {
            raw: self.raw.saturating_add(other.raw),
        }
    }

    /// Subtracts two fixed-point numbers with saturation.    
    pub fn sub(self, other: Self) -> Self {
        Self {
            raw: self.raw.saturating_sub(other.raw),
        }
    }

    /// Multiplies two fixed-point numbers with saturation.
    ///
    /// # Examples
    ///
    /// ```
    /// use fixed_point::FixedSmall;
    ///
    /// let x = FixedSmall::<16, 8>::from_f32(2.0)?;
    /// let y = FixedSmall::<16, 8>::from_f32(3.0)?;
    /// let product = x.mul(y);
    /// assert_eq!(product.to_f32(), 6.0);
    /// # Ok::<(), fixed_point::FixedPointError>(())
    /// ```
    pub fn mul(self, other: Self) -> Self {
        let result = (self.raw as i64 * other.raw as i64) >> F;
        Self {
            raw: result.clamp(Self::MIN_INT as i64, Self::MAX_INT as i64) as i32,
        }
    }

    /// Negates the fixed-point number.
    pub fn neg(self) -> Self {
        Self {
            raw: self.raw.saturating_neg(),
        }
    }

    /// Returns the absolute value.
    pub fn abs(self) -> Self {
        Self {
            raw: self.raw.abs(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_f32_basic() {
        let x = FixedSmall::<16, 8>::from_f32(3.14).unwrap();
        assert!((x.to_f32() - 3.14).abs() < 0.01);
    }

    #[test]
    fn test_from_f32_negative() {
        let x = FixedSmall::<16, 8>::from_f32(-5.5).unwrap();
        assert!((x.to_f32() - (-5.5)).abs() < 0.01);
    }

    #[test]
    fn test_overflow() {
        let result = FixedSmall::<8, 4>::from_f32(100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_and_one() {
        let zero = FixedSmall::<16, 8>::zero();
        let one = FixedSmall::<16, 8>::one();
        
        assert_eq!(zero.to_f32(), 0.0);
        assert_eq!(one.to_f32(), 1.0);
    }

    #[test]
    fn test_add() {
        let x = FixedSmall::<16, 8>::from_f32(1.5).unwrap();
        let y = FixedSmall::<16, 8>::from_f32(2.5).unwrap();
        let sum = x.add(y);
        assert!((sum.to_f32() - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_sub() {
        let x = FixedSmall::<16, 8>::from_f32(5.0).unwrap();
        let y = FixedSmall::<16, 8>::from_f32(3.0).unwrap();
        let diff = x.sub(y);
        assert!((diff.to_f32() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_mul() {
        let x = FixedSmall::<16, 8>::from_f32(2.0).unwrap();
        let y = FixedSmall::<16, 8>::from_f32(3.0).unwrap();
        let product = x.mul(y);
        assert!((product.to_f32() - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_neg() {
        let x = FixedSmall::<16, 8>::from_f32(3.5).unwrap();
        let neg_x = x.neg();
        assert!((neg_x.to_f32() - (-3.5)).abs() < 0.01);
    }

    #[test]
    fn test_abs() {
        let x = FixedSmall::<16, 8>::from_f32(-3.5).unwrap();
        let abs_x = x.abs();
        assert!((abs_x.to_f32() - 3.5).abs() < 0.01);
    }

    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_roundtrip_8_4(value in -7.0f32..7.0f32) {
                let fixed = FixedSmall::<8, 4>::from_f32(value).unwrap();
                let recovered = fixed.to_f32();
                prop_assert!((recovered - value).abs() < 0.1);
            }

            #[test]
            fn test_roundtrip_16_8(value in -127.0f32..127.0f32) {
                let fixed = FixedSmall::<16, 8>::from_f32(value).unwrap();
                let recovered = fixed.to_f32();
                prop_assert!((recovered - value).abs() < 0.01);
            }

            #[test]
            fn test_add_commutative(a in -10.0f32..10.0f32, b in -10.0f32..10.0f32) {
                let x = FixedSmall::<16, 8>::from_f32(a).unwrap();
                let y = FixedSmall::<16, 8>::from_f32(b).unwrap();
                prop_assert_eq!(x.add(y), y.add(x));
            }

            #[test]
            fn test_mul_commutative(a in -5.0f32..5.0f32, b in -5.0f32..5.0f32) {
                let x = FixedSmall::<16, 8>::from_f32(a).unwrap();
                let y = FixedSmall::<16, 8>::from_f32(b).unwrap();
                let xy = x.mul(y).to_f32();
                let yx = y.mul(x).to_f32();
                prop_assert!((xy - yx).abs() < 0.1);
            }
        }
    }
}

