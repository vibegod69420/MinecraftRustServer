//! Faithful port of `net.minecraft.util.profiling.jfr.callback.ProfiledDuration`.
//!
//! In Minecraft Java, this is a `@FunctionalInterface` callback invoked when a
//! profiled JFR duration ends.

/// Callback invoked when a profiled duration completes.
///
/// Faithful port of the Java functional interface
/// `net.minecraft.util.profiling.jfr.callback.ProfiledDuration`.
pub trait ProfiledDuration {
    // JAVA: net.minecraft.util.profiling.jfr.callback.ProfiledDuration#finish
    fn finish(&mut self, success: bool);
}

impl<F> ProfiledDuration for F
where
    F: FnMut(bool),
{
    // JAVA: net.minecraft.util.profiling.jfr.callback.ProfiledDuration#finish
    fn finish(&mut self, success: bool) {
        self(success);
    }
}

#[cfg(test)]
mod tests {
    use super::ProfiledDuration;

    #[derive(Debug, Default)]
    struct RecordingDuration {
        calls: Vec<bool>,
    }

    impl ProfiledDuration for RecordingDuration {
        // JAVA: net.minecraft.util.profiling.jfr.callback.ProfiledDuration#finish
        fn finish(&mut self, success: bool) {
            self.calls.push(success);
        }
    }

    #[derive(Debug, Default)]
    struct NoOpDuration;

    impl ProfiledDuration for NoOpDuration {
        // JAVA: net.minecraft.util.profiling.jfr.callback.ProfiledDuration#finish
        fn finish(&mut self, _success: bool) {}
    }

    #[test]
    fn struct_records_true() {
        let mut duration = RecordingDuration::default();
        duration.finish(true);
        assert_eq!(duration.calls, vec![true]);
    }

    #[test]
    fn struct_records_false() {
        let mut duration = RecordingDuration::default();
        duration.finish(false);
        assert_eq!(duration.calls, vec![false]);
    }

    #[test]
    fn struct_records_multiple_calls_in_order() {
        let mut duration = RecordingDuration::default();
        duration.finish(true);
        duration.finish(false);
        duration.finish(false);
        duration.finish(true);
        assert_eq!(duration.calls, vec![true, false, false, true]);
    }

    #[test]
    fn closure_impl_receives_both_values() {
        let mut received = Vec::new();
        {
            let mut callback = |success: bool| received.push(success);
            ProfiledDuration::finish(&mut callback, true);
            ProfiledDuration::finish(&mut callback, false);
        }
        assert_eq!(received, vec![true, false]);
    }

    #[test]
    fn fn_mut_closure_can_capture_mutable_state() {
        let mut success_count: i32 = 0;
        let mut failure_count: i32 = 0;
        {
            let mut callback = |success: bool| {
                if success {
                    success_count = success_count.wrapping_add(1);
                } else {
                    failure_count = failure_count.wrapping_add(1);
                }
            };
            callback.finish(true);
            callback.finish(true);
            callback.finish(false);
        }
        assert_eq!(success_count, 2);
        assert_eq!(failure_count, 1);
    }

    #[test]
    fn trait_object_dispatch_mut_ref() {
        let mut duration = RecordingDuration::default();
        let callback: &mut dyn ProfiledDuration = &mut duration;
        callback.finish(true);
        callback.finish(false);
        assert_eq!(duration.calls, vec![true, false]);
    }

    #[test]
    fn trait_object_dispatch_boxed() {
        let mut calls = Vec::new();
        {
            let mut callback: Box<dyn ProfiledDuration> =
                Box::new(|success: bool| calls.push(success));
            callback.finish(false);
            callback.finish(true);
        }
        assert_eq!(calls, vec![false, true]);
    }

    #[test]
    fn no_op_impl_accepts_all_values() {
        let mut duration = NoOpDuration;
        duration.finish(true);
        duration.finish(false);
    }

    #[test]
    fn edge_case_overflowing_integer_inputs() {
        let mut received = Vec::new();
        let mut callback = |success: bool| received.push(success);

        let max_i32 = i32::MAX;
        let min_i32 = i32::MIN;
        let wrapped_add = max_i32.wrapping_add(1);
        let wrapped_sub = min_i32.wrapping_sub(1);
        let wrapped_mul = max_i32.wrapping_mul(2);

        ProfiledDuration::finish(&mut callback, wrapped_add == min_i32);
        ProfiledDuration::finish(&mut callback, wrapped_sub == max_i32);
        ProfiledDuration::finish(&mut callback, wrapped_mul == -2i32);

        let (_value, overflowed_add) = max_i32.overflowing_add(1);
        let (_value, overflowed_sub) = min_i32.overflowing_sub(1);
        ProfiledDuration::finish(&mut callback, overflowed_add);
        ProfiledDuration::finish(&mut callback, overflowed_sub);

        assert_eq!(received, vec![true, true, true, true, true]);
    }

    #[test]
    fn edge_case_negative_and_boundary_integer_inputs() {
        let mut received = Vec::new();
        let mut callback = |success: bool| received.push(success);

        let values: [i32; 7] = [i32::MIN, -2, -1, 0, 1, 2, i32::MAX];
        for value in values {
            ProfiledDuration::finish(&mut callback, value.is_negative());
        }

        let unsigned_values: [u32; 5] = [0u32, 1u32, u32::MAX - 1, u32::MAX, u32::MIN];
        for value in unsigned_values {
            let wrapped = value.wrapping_sub(1);
            ProfiledDuration::finish(&mut callback, wrapped > value);
        }

        assert_eq!(
            received,
            vec![
                true, true, true, false, false, false, false, // signed negatives
                true, false, false, false, true, // unsigned wrapping_sub underflow indicator
            ]
        );
    }

    #[test]
    fn edge_case_float32_bits_inputs() {
        let mut received = Vec::new();
        let mut callback = |success: bool| received.push(success);

        let nan = f32::NAN;
        let pos_inf = f32::INFINITY;
        let neg_inf = f32::NEG_INFINITY;
        let neg_zero = f32::from_bits(0x8000_0000u32);
        let pos_zero = f32::from_bits(0x0000_0000u32);
        let max_f32 = f32::from_bits(0x7F7F_FFFFu32);
        let min_subnormal = f32::from_bits(0x0000_0001u32);
        let neg_one = -1.0f32;

        ProfiledDuration::finish(&mut callback, nan.is_nan());
        ProfiledDuration::finish(&mut callback, nan.to_bits() != nan.to_bits()); // false: same NaN value has same bits
        ProfiledDuration::finish(&mut callback, pos_inf.is_infinite() && pos_inf.is_sign_positive());
        ProfiledDuration::finish(&mut callback, neg_inf.is_infinite() && neg_inf.is_sign_negative());
        ProfiledDuration::finish(&mut callback, neg_zero.to_bits() == 0x8000_0000u32);
        ProfiledDuration::finish(&mut callback, pos_zero.to_bits() == 0x0000_0000u32);
        ProfiledDuration::finish(&mut callback, neg_zero == pos_zero); // IEEE equality
        ProfiledDuration::finish(&mut callback, max_f32.to_bits() == 0x7F7F_FFFFu32);
        ProfiledDuration::finish(&mut callback, min_subnormal.is_subnormal());
        ProfiledDuration::finish(&mut callback, neg_one.is_sign_negative());
        ProfiledDuration::finish(&mut callback, (neg_one.to_bits() & 0x8000_0000u32) != 0);

        assert_eq!(
            received,
            vec![true, false, true, true, true, true, true, true, true, true, true]
        );
    }

    #[test]
    fn edge_case_float64_bits_inputs() {
        let mut received = Vec::new();
        let mut callback = |success: bool| received.push(success);

        let nan = f64::NAN;
        let pos_inf = f64::INFINITY;
        let neg_inf = f64::NEG_INFINITY;
        let neg_zero = f64::from_bits(0x8000_0000_0000_0000u64);
        let pos_zero = f64::from_bits(0x0000_0000_0000_0000u64);
        let max_f64 = f64::from_bits(0x7FEF_FFFF_FFFF_FFFFu64);
        let min_subnormal = f64::from_bits(0x0000_0000_0000_0001u64);
        let neg_max = -f64::MAX;

        ProfiledDuration::finish(&mut callback, nan.is_nan());
        ProfiledDuration::finish(&mut callback, pos_inf.to_bits() == 0x7FF0_0000_0000_0000u64);
        ProfiledDuration::finish(&mut callback, neg_inf.to_bits() == 0xFFF0_0000_0000_0000u64);
        ProfiledDuration::finish(&mut callback, neg_zero.to_bits() == 0x8000_0000_0000_0000u64);
        ProfiledDuration::finish(&mut callback, pos_zero.to_bits() == 0x0000_0000_0000_0000u64);
        ProfiledDuration::finish(&mut callback, neg_zero == pos_zero);
        ProfiledDuration::finish(&mut callback, max_f64.to_bits() == 0x7FEF_FFFF_FFFF_FFFFu64);
        ProfiledDuration::finish(&mut callback, min_subnormal.is_subnormal());
        ProfiledDuration::finish(&mut callback, neg_max.is_sign_negative());

        assert_eq!(
            received,
            vec![true, true, true, true, true, true, true, true, true]
        );
    }

    #[test]
    fn edge_case_shift_masked_bool_selection() {
        let mut received = Vec::new();
        let mut callback = |success: bool| received.push(success);

        // Java shift semantics use masked shift counts (& 31 for int, & 63 for long).
        let value: i32 = 1;
        let shift32: i32 = 32;
        let shift33: i32 = 33;
        let shifted32 = value.wrapping_shl((shift32 as u32) & 31);
        let shifted33 = value.wrapping_shl((shift33 as u32) & 31);
        ProfiledDuration::finish(&mut callback, shifted32 == 1);
        ProfiledDuration::finish(&mut callback, shifted33 == 2);

        let value64: i64 = 1;
        let shift64: i64 = 64;
        let shift65: i64 = 65;
        let shifted64 = value64.wrapping_shl((shift64 as u32) & 63);
        let shifted65 = value64.wrapping_shl((shift65 as u32) & 63);
        ProfiledDuration::finish(&mut callback, shifted64 == 1);
        ProfiledDuration::finish(&mut callback, shifted65 == 2);

        assert_eq!(received, vec![true, true, true, true]);
    }
}
