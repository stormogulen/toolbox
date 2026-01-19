use proptest::prelude::*;
use fixed_point::{FixedSmall, FixedPointError};

/// Strategy generator for any f32 safely representable by FixedSmall<N, F>.
fn any_representable<const N: usize, const F: usize>() -> impl Strategy<Value = f32> {
    // Range is MIN_INT / SCALE  .. MAX_INT / SCALE
    let max = ((1 << (N - 1)) - 1) as f32 / (1 << F) as f32;
    let min = (-(1 << (N - 1))) as f32 / (1 << F) as f32;

    // Generate values slightly outside range too, to test overflow errors.
    // proptest will shrink them toward boundaries automatically.
    (min * 2.0)..=(max * 2.0)
}

/// Helper that converts f32 → raw fixed-point integer reliably
fn f32_to_raw_fallback<const F: usize>(value: f32) -> i32 {
    (value * (1 << F) as f32).round() as i32
}

proptest! {

    // --- from_f32 <-> to_f32 roundtrip ---
    #[test]
    fn roundtrip_from_f32_to_f32_is_reasonable(v in any_representable::<16,8>()) {
        let res = FixedSmall::<16,8>::from_f32(v);
        match res {
            Ok(fx) => {
                let f = fx.to_f32();
                // Precision tolerance: 1 LSB of fractional precision
                let eps = 1.0 / (1 << 8) as f32;
                prop_assert!((f - v).abs() <= eps * 2.0);
            }
            Err(FixedPointError::Overflow{..}) => {
                // Only values outside range should overflow
                let raw = f32_to_raw_fallback::<8>(v);
                let max = ((1 << (16 - 1)) - 1);
                let min = -(1 << (16 - 1));
                prop_assert!(raw > max || raw < min);
            }
            //Err(_) => prop_assert!(false, "Unexpected error"),
            Err(other) => unreachable!("Unexpected error: {:?}", other),
        }
    }

    // --- raw roundtrip ---
    #[test]
    fn from_raw_gives_correct_raw_back(raw in any::<i32>()) {
        let fx = FixedSmall::<16,8>::from_raw(raw);
        prop_assert_eq!(fx.raw_value(), raw);
    }

    // --- Addition matches raw addition with saturation ---
    #[test]
    fn add_saturates_correctly(a in any::<i32>(), b in any::<i32>()) {
        let x = FixedSmall::<16,8>::from_raw(a);
        let y = FixedSmall::<16,8>::from_raw(b);

        let sum = x.add(y);

        let expected = a.saturating_add(b);
        prop_assert_eq!(sum.raw_value(), expected);
    }

    // --- Subtraction matches raw subtraction with saturation ---
    #[test]
    fn sub_saturates_correctly(a in any::<i32>(), b in any::<i32>()) {
        let x = FixedSmall::<16,8>::from_raw(a);
        let y = FixedSmall::<16,8>::from_raw(b);

        let diff = x.sub(y);

        let expected = a.saturating_sub(b);
        prop_assert_eq!(diff.raw_value(), expected);
    }

    // --- Multiplication property ---
    #[test]
    fn mul_matches_scaled_integer_math(a in any::<i32>(), b in any::<i32>()) {
        const N: usize = 16;
        const F: usize = 8;

        let x = FixedSmall::<N,F>::from_raw(a);
        let y = FixedSmall::<N,F>::from_raw(b);

        let prod = x.mul(y);

        // Expected raw result = (a*b) >> F, with saturation
        let full = (a as i64 * b as i64) >> F;
        let max = ((1 << (N - 1)) - 1) as i64;
        let min = -(1 << (N - 1)) as i64;
        let expected = full.clamp(min, max) as i32;

        prop_assert_eq!(prod.raw_value(), expected);
    }

    // --- Negation ---
    #[test]
    fn neg_matches_saturating_neg(raw in any::<i32>()) {
        let x = FixedSmall::<16,8>::from_raw(raw);
        let expected = raw.saturating_neg();
        prop_assert_eq!(x.neg().raw_value(), expected);
    }

    // --- Abs ---
    #[test]
    fn abs_matches_i32_abs(raw in any::<i32>()) {
        let x = FixedSmall::<16,8>::from_raw(raw);
        let expected = raw.abs();
        prop_assert_eq!(x.abs().raw_value(), expected);
    }
}
