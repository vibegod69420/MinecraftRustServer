//! Faithful port of `net.minecraft.util.ToFloatFunction` (Minecraft Java Edition 1.21.11).
//!
//! This module provides the [`ToFloatFunction`] trait, a functional interface that maps
//! an input value of type `T` to an `f32`, matching Java's
//! `@FunctionalInterface net.minecraft.util.ToFloatFunction<T>`.

/// A functional interface that produces an `f32` from a value of type `T`.
///
/// This is the Rust equivalent of Java's `net.minecraft.util.ToFloatFunction<T>`.
/// Any `Fn(T) -> f32` automatically implements this trait.
pub trait ToFloatFunction<T> {
    // JAVA: net.minecraft.util.ToFloatFunction#applyAsFloat
    fn apply_as_float(&self, value: T) -> f32;
}

impl<T, F> ToFloatFunction<T> for F
where
    F: Fn(T) -> f32,
{
    // JAVA: net.minecraft.util.ToFloatFunction#applyAsFloat
    fn apply_as_float(&self, value: T) -> f32 {
        (self)(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Scale(i32);

    impl ToFloatFunction<i32> for Scale {
        // JAVA: net.minecraft.util.ToFloatFunction#applyAsFloat
        fn apply_as_float(&self, value: i32) -> f32 {
            value.wrapping_mul(self.0) as f32
        }
    }

    #[derive(Debug)]
    struct ConstFloat(f32);

    impl ToFloatFunction<()> for ConstFloat {
        // JAVA: net.minecraft.util.ToFloatFunction#applyAsFloat
        fn apply_as_float(&self, _value: ()) -> f32 {
            self.0
        }
    }

    #[test]
    fn struct_impl_basic() {
        let f = Scale(2);
        assert_eq!(f.apply_as_float(3).to_bits(), 6.0f32.to_bits());
        assert_eq!(f.apply_as_float(-4).to_bits(), (-8.0f32).to_bits());
        assert_eq!(f.apply_as_float(0).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn closure_impl_basic() {
        let f = |x: f32| x * 2.0;
        assert_eq!(
            ToFloatFunction::apply_as_float(&f, 1.5f32).to_bits(),
            3.0f32.to_bits()
        );
        assert_eq!(
            ToFloatFunction::apply_as_float(&f, -2.25f32).to_bits(),
            (-4.5f32).to_bits()
        );
    }

    #[test]
    fn fn_pointer_impl() {
        fn negate(x: i32) -> f32 {
            -(x as f32)
        }

        let f: fn(i32) -> f32 = negate;
        assert_eq!(f.apply_as_float(5).to_bits(), (-5.0f32).to_bits());
        assert_eq!(f.apply_as_float(-7).to_bits(), 7.0f32.to_bits());
        assert_eq!(f.apply_as_float(0).to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn negative_zero_bits_preserved() {
        let f = ConstFloat(-0.0f32);
        let result = f.apply_as_float(());
        assert_eq!(result.to_bits(), (-0.0f32).to_bits());
        assert!(result.is_sign_negative());
        // IEEE-754: -0.0 == 0.0, but bit patterns differ.
        assert_eq!(result, 0.0f32);
        assert_ne!(result.to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn nan_bits_preserved() {
        let nan_bits: u32 = 0x7FC0_0001;
        let f = move |_x: u32| f32::from_bits(nan_bits);
        let result = f.apply_as_float(0);
        assert!(result.is_nan());
        assert_eq!(result.to_bits(), nan_bits);
    }

    #[test]
    fn signaling_nan_bits_preserved() {
        let snan_bits: u32 = 0x7F80_0001;
        let f = |_x: ()| f32::from_bits(snan_bits);
        let result = f.apply_as_float(());
        assert_eq!(result.to_bits(), snan_bits);
    }

    #[test]
    fn infinity_values() {
        let pos = |_x: i32| f32::INFINITY;
        let neg = |_x: i32| f32::NEG_INFINITY;
        assert_eq!(ToFloatFunction::apply_as_float(&pos, 0), f32::INFINITY);
        assert_eq!(ToFloatFunction::apply_as_float(&neg, 0), f32::NEG_INFINITY);
        assert_eq!(pos.apply_as_float(123).to_bits(), f32::INFINITY.to_bits());
        assert_eq!(
            neg.apply_as_float(-456).to_bits(),
            f32::NEG_INFINITY.to_bits()
        );
    }

    #[test]
    fn subnormal_values() {
        let sub = f32::from_bits(1u32); // smallest positive subnormal
        let neg_sub = f32::from_bits(0x8000_0001u32); // smallest negative subnormal

        let f_pos = |_x: ()| sub;
        let f_neg = |_x: ()| neg_sub;

        let pos_result = f_pos.apply_as_float(());
        let neg_result = f_neg.apply_as_float(());

        assert_eq!(pos_result.to_bits(), 1u32);
        assert_eq!(neg_result.to_bits(), 0x8000_0001u32);
        assert!(pos_result.is_subnormal());
        assert!(neg_result.is_subnormal());
    }

    #[test]
    fn min_max_f32_values() {
        let f_max = |_x: i8| f32::MAX;
        let f_min = |_x: i8| f32::MIN;
        let f_min_pos = |_x: i8| f32::MIN_POSITIVE;
        let f_epsilon = |_x: i8| f32::EPSILON;

        assert_eq!(f_max.apply_as_float(0).to_bits(), f32::MAX.to_bits());
        assert_eq!(f_min.apply_as_float(0).to_bits(), f32::MIN.to_bits());
        assert_eq!(
            f_min_pos.apply_as_float(0).to_bits(),
            f32::MIN_POSITIVE.to_bits()
        );
        assert_eq!(
            f_epsilon.apply_as_float(0).to_bits(),
            f32::EPSILON.to_bits()
        );
    }

    #[test]
    fn negative_and_extreme_integer_inputs() {
        let f = |x: i32| x as f32;
        assert_eq!(f.apply_as_float(0).to_bits(), 0.0f32.to_bits());
        assert_eq!(f.apply_as_float(-1).to_bits(), (-1.0f32).to_bits());
        assert_eq!(
            f.apply_as_float(i32::MIN).to_bits(),
            (i32::MIN as f32).to_bits()
        );
        assert_eq!(
            f.apply_as_float(i32::MAX).to_bits(),
            (i32::MAX as f32).to_bits()
        );
    }

    #[test]
    fn wrapping_integer_semantics_in_input_computation() {
        // Demonstrates Java-like wrapping behavior can be expressed through the interface.
        let f = |x: i32| x.wrapping_mul(2) as f32;
        assert_eq!(
            f.apply_as_float(i32::MAX).to_bits(),
            (i32::MAX.wrapping_mul(2) as f32).to_bits()
        );
        assert_eq!(
            f.apply_as_float(i32::MIN).to_bits(),
            (i32::MIN.wrapping_mul(2) as f32).to_bits()
        );
        assert_eq!(f.apply_as_float(-1).to_bits(), (-2.0f32).to_bits());
    }

    #[test]
    fn wrapping_add_sub_overflow_edge_cases() {
        let add = |x: i32| x.wrapping_add(i32::MAX) as f32;
        let sub = |x: i32| x.wrapping_sub(i32::MIN) as f32;

        assert_eq!(
            add.apply_as_float(1).to_bits(),
            (1i32.wrapping_add(i32::MAX) as f32).to_bits()
        );
        assert_eq!(
            add.apply_as_float(i32::MAX).to_bits(),
            (i32::MAX.wrapping_add(i32::MAX) as f32).to_bits()
        );
        assert_eq!(
            sub.apply_as_float(-1).to_bits(),
            ((-1i32).wrapping_sub(i32::MIN) as f32).to_bits()
        );
        assert_eq!(
            sub.apply_as_float(i32::MIN).to_bits(),
            (i32::MIN.wrapping_sub(i32::MIN) as f32).to_bits()
        );
    }

    #[test]
    fn reference_input_type() {
        let f = |s: &str| s.len() as f32;
        assert_eq!(
            ToFloatFunction::apply_as_float(&f, "hello").to_bits(),
            5.0f32.to_bits()
        );
        assert_eq!(f.apply_as_float("").to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn generic_helper_accepts_to_float_function() {
        fn apply_twice<T: Copy, F: ToFloatFunction<T>>(f: &F, value: T) -> f32 {
            f.apply_as_float(value) + f.apply_as_float(value)
        }

        let f = |x: i32| x as f32;
        let result = apply_twice(&f, 3);
        assert_eq!(result.to_bits(), 6.0f32.to_bits());

        let neg = |x: i32| -(x as f32);
        let neg_result = apply_twice(&neg, -4);
        assert_eq!(neg_result.to_bits(), 8.0f32.to_bits());
    }

    #[test]
    fn trait_object_works() {
        let f = |x: i32| (x as f32) * 0.5;
        let obj: &dyn ToFloatFunction<i32> = &f;
        assert_eq!(obj.apply_as_float(10).to_bits(), 5.0f32.to_bits());
        assert_eq!(obj.apply_as_float(-3).to_bits(), (-1.5f32).to_bits());
        assert_eq!(obj.apply_as_float(0).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn boxed_trait_object_works() {
        let f: Box<dyn ToFloatFunction<i64>> = Box::new(|x: i64| x as f32);
        assert_eq!(f.apply_as_float(i64::MIN).to_bits(), (i64::MIN as f32).to_bits());
        assert_eq!(f.apply_as_float(-1i64).to_bits(), (-1.0f32).to_bits());
        assert_eq!(f.apply_as_float(42i64).to_bits(), 42.0f32.to_bits());
    }

    #[test]
    fn exact_float_bit_patterns() {
        // Verify specific IEEE-754 bit patterns are preserved through the interface.
        let patterns: [u32; 8] = [
            0x0000_0000, // +0.0
            0x8000_0000, // -0.0
            0x0000_0001, // smallest positive subnormal
            0x007F_FFFF, // largest subnormal
            0x0080_0000, // smallest positive normal
            0x7F7F_FFFF, // f32::MAX
            0xFF7F_FFFF, // f32::MIN (-f32::MAX)
            0x3F80_0000, // 1.0
        ];

        for bits in patterns {
            let f = move |_x: u8| f32::from_bits(bits);
            let result = f.apply_as_float(0);
            assert_eq!(
                result.to_bits(),
                bits,
                "bit pattern 0x{bits:08X} was not preserved"
            );
        }
    }
}
