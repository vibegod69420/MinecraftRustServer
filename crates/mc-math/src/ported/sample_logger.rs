//! Faithful port of `net.minecraft.util.debugchart.SampleLogger`.
//!
//! Source: `net/minecraft/util/debugchart/SampleLogger.java` (Minecraft Java Edition 1.21.11).

/// A logger that receives debug-chart timing/performance samples.
///
/// This is the Rust equivalent of the Java `SampleLogger` interface.
/// Implementors define how full, single, and partial samples are recorded.
pub trait SampleLogger {
    // JAVA: net.minecraft.util.debugchart.SampleLogger#logFullSample
    fn log_full_sample(&mut self, values: &[i64]);

    // JAVA: net.minecraft.util.debugchart.SampleLogger#logSample
    fn log_sample(&mut self, value: i64);

    // JAVA: net.minecraft.util.debugchart.SampleLogger#logPartialSample
    fn log_partial_sample(&mut self, value: i64, count: i32);
}

#[cfg(test)]
mod tests {
    use super::SampleLogger;

    /// Simple recording implementation used to verify trait dispatch and value fidelity.
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct MockLogger {
        full_samples: Vec<Vec<i64>>,
        samples: Vec<i64>,
        partial_samples: Vec<(i64, i32)>,
    }

    impl SampleLogger for MockLogger {
        fn log_full_sample(&mut self, values: &[i64]) {
            self.full_samples.push(values.to_vec());
        }

        fn log_sample(&mut self, value: i64) {
            self.samples.push(value);
        }

        fn log_partial_sample(&mut self, value: i64, count: i32) {
            self.partial_samples.push((value, count));
        }
    }

    /// Helper that exercises the trait through generic dispatch.
    fn record_via_trait(logger: &mut impl SampleLogger, value: i64, count: i32) {
        logger.log_sample(value);
        logger.log_partial_sample(value, count);
        logger.log_full_sample(&[value, i64::from(count)]);
    }

    #[test]
    fn log_sample_records_exact_values() {
        let mut logger = MockLogger::default();
        logger.log_sample(0);
        logger.log_sample(1);
        logger.log_sample(-1);

        assert_eq!(logger.samples, vec![0i64, 1, -1]);
    }

    #[test]
    fn log_sample_edge_i64_values() {
        let mut logger = MockLogger::default();
        let cases = [
            i64::MIN,
            i64::MIN + 1,
            -2,
            -1,
            0,
            1,
            2,
            i64::MAX - 1,
            i64::MAX,
        ];

        for &value in &cases {
            logger.log_sample(value);
        }

        assert_eq!(logger.samples.len(), cases.len());
        for (idx, &expected) in cases.iter().enumerate() {
            assert_eq!(logger.samples[idx], expected);
        }
    }

    #[test]
    fn log_sample_preserves_wrapping_overflow_results() {
        let mut logger = MockLogger::default();

        // Java long arithmetic wraps in two's complement; verify such values are preserved.
        let wrapped_add = i64::MAX.wrapping_add(1);
        let wrapped_sub = i64::MIN.wrapping_sub(1);
        let wrapped_mul = (i64::MAX / 2).wrapping_mul(3);

        logger.log_sample(wrapped_add);
        logger.log_sample(wrapped_sub);
        logger.log_sample(wrapped_mul);

        assert_eq!(logger.samples[0], i64::MIN);
        assert_eq!(logger.samples[1], i64::MAX);
        assert_eq!(logger.samples[2], (i64::MAX / 2).wrapping_mul(3));
    }

    #[test]
    fn log_full_sample_empty_array() {
        let mut logger = MockLogger::default();
        logger.log_full_sample(&[]);

        assert_eq!(logger.full_samples.len(), 1);
        assert!(logger.full_samples[0].is_empty());
    }

    #[test]
    fn log_full_sample_records_all_elements_in_order() {
        let mut logger = MockLogger::default();
        let values = [10i64, -20, 30, -40, 50];
        logger.log_full_sample(&values);

        assert_eq!(logger.full_samples.len(), 1);
        assert_eq!(logger.full_samples[0], values);
    }

    #[test]
    fn log_full_sample_edge_and_negative_values() {
        let mut logger = MockLogger::default();
        let values = [
            i64::MIN,
            -123_456_789_012_345i64,
            -1,
            0,
            1,
            123_456_789_012_345i64,
            i64::MAX,
        ];
        logger.log_full_sample(&values);

        assert_eq!(logger.full_samples[0], values);
    }

    #[test]
    fn log_full_sample_multiple_calls_are_independent() {
        let mut logger = MockLogger::default();
        logger.log_full_sample(&[1, 2, 3]);
        logger.log_full_sample(&[]);
        logger.log_full_sample(&[-1]);

        assert_eq!(logger.full_samples.len(), 3);
        assert_eq!(logger.full_samples[0], vec![1, 2, 3]);
        assert_eq!(logger.full_samples[1], Vec::<i64>::new());
        assert_eq!(logger.full_samples[2], vec![-1]);
    }

    #[test]
    fn log_partial_sample_records_value_and_count() {
        let mut logger = MockLogger::default();
        logger.log_partial_sample(42, 7);
        logger.log_partial_sample(-99, -3);

        assert_eq!(logger.partial_samples, vec![(42i64, 7i32), (-99i64, -3i32)]);
    }

    #[test]
    fn log_partial_sample_edge_integer_values() {
        let mut logger = MockLogger::default();
        let cases = [
            (i64::MIN, i32::MIN),
            (i64::MIN, -1),
            (i64::MIN, 0),
            (i64::MIN, 1),
            (i64::MIN, i32::MAX),
            (-1, i32::MIN),
            (0, 0),
            (1, i32::MAX),
            (i64::MAX, i32::MIN),
            (i64::MAX, -1),
            (i64::MAX, 0),
            (i64::MAX, 1),
            (i64::MAX, i32::MAX),
        ];

        for &(value, count) in &cases {
            logger.log_partial_sample(value, count);
        }

        assert_eq!(logger.partial_samples.len(), cases.len());
        for (idx, &expected) in cases.iter().enumerate() {
            assert_eq!(logger.partial_samples[idx], expected);
        }
    }

    #[test]
    fn log_partial_sample_preserves_wrapping_count_values() {
        let mut logger = MockLogger::default();

        // Java int arithmetic wraps in two's complement.
        let wrapped_count_add = i32::MAX.wrapping_add(1);
        let wrapped_count_sub = i32::MIN.wrapping_sub(1);
        let wrapped_count_mul = (i32::MAX / 2).wrapping_mul(3);

        logger.log_partial_sample(1, wrapped_count_add);
        logger.log_partial_sample(2, wrapped_count_sub);
        logger.log_partial_sample(3, wrapped_count_mul);

        assert_eq!(logger.partial_samples[0], (1i64, i32::MIN));
        assert_eq!(logger.partial_samples[1], (2i64, i32::MAX));
        assert_eq!(
            logger.partial_samples[2],
            (3i64, (i32::MAX / 2).wrapping_mul(3))
        );
    }

    #[test]
    fn float64_bit_patterns_preserved_through_i64_sample() {
        let mut logger = MockLogger::default();
        let patterns: [u64; 11] = [
            0.0f64.to_bits(),
            (-0.0f64).to_bits(),
            1.0f64.to_bits(),
            (-1.0f64).to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            f64::NAN.to_bits(),
            f64::MIN_POSITIVE.to_bits(),
            f64::MAX.to_bits(),
            f64::MIN.to_bits(),
            u64::MAX, // all bits set, also a NaN payload pattern
        ];

        for &bits in &patterns {
            logger.log_sample(bits as i64);
        }

        assert_eq!(logger.samples.len(), patterns.len());
        for (idx, &bits) in patterns.iter().enumerate() {
            let recorded_bits = logger.samples[idx] as u64;
            assert_eq!(
                recorded_bits, bits,
                "f64 bit pattern mismatch at index {idx}"
            );
            // Round-trip through f64 bit representation to ensure exact bit fidelity.
            assert_eq!(f64::from_bits(recorded_bits).to_bits(), bits);
        }
    }

    #[test]
    fn float32_bit_patterns_preserved_through_partial_sample_count() {
        let mut logger = MockLogger::default();
        let patterns: [u32; 10] = [
            0.0f32.to_bits(),
            (-0.0f32).to_bits(),
            1.0f32.to_bits(),
            (-1.0f32).to_bits(),
            f32::INFINITY.to_bits(),
            f32::NEG_INFINITY.to_bits(),
            f32::NAN.to_bits(),
            f32::MIN_POSITIVE.to_bits(),
            f32::MAX.to_bits(),
            u32::MAX,
        ];

        for (i, &bits) in patterns.iter().enumerate() {
            logger.log_partial_sample(i as i64, bits as i32);
        }

        assert_eq!(logger.partial_samples.len(), patterns.len());
        for (idx, &bits) in patterns.iter().enumerate() {
            let (_, recorded_count) = logger.partial_samples[idx];
            let recorded_bits = recorded_count as u32;
            assert_eq!(
                recorded_bits, bits,
                "f32 bit pattern mismatch at index {idx}"
            );
            assert_eq!(f32::from_bits(recorded_bits).to_bits(), bits);
        }
    }

    #[test]
    fn float_bit_patterns_preserved_in_full_sample() {
        let mut logger = MockLogger::default();
        let values: Vec<i64> = vec![
            f64::NAN.to_bits() as i64,
            f64::INFINITY.to_bits() as i64,
            f64::NEG_INFINITY.to_bits() as i64,
            (-0.0f64).to_bits() as i64,
            f32::NAN.to_bits() as i64,
            f32::NEG_INFINITY.to_bits() as i64,
        ];

        logger.log_full_sample(&values);

        assert_eq!(logger.full_samples[0], values);
        for (idx, &expected) in values.iter().enumerate() {
            assert_eq!(
                logger.full_samples[0][idx] as u64,
                expected as u64,
                "bit pattern mismatch in full sample at index {idx}"
            );
        }
    }

    #[test]
    fn trait_object_dispatch_works() {
        let mut logger = MockLogger::default();
        {
            let dyn_logger: &mut dyn SampleLogger = &mut logger;
            dyn_logger.log_sample(123);
            dyn_logger.log_partial_sample(-456, 2);
            dyn_logger.log_full_sample(&[7, 8, 9]);
        }

        assert_eq!(logger.samples, vec![123i64]);
        assert_eq!(logger.partial_samples, vec![(-456i64, 2i32)]);
        assert_eq!(logger.full_samples, vec![vec![7i64, 8, 9]]);
    }

    #[test]
    fn generic_dispatch_works() {
        let mut logger = MockLogger::default();
        record_via_trait(&mut logger, i64::MIN, i32::MAX);

        assert_eq!(logger.samples, vec![i64::MIN]);
        assert_eq!(logger.partial_samples, vec![(i64::MIN, i32::MAX)]);
        assert_eq!(
            logger.full_samples,
            vec![vec![i64::MIN, i64::from(i32::MAX)]]
        );
    }

    #[test]
    fn negative_and_overflow_edge_matrix() {
        let mut logger = MockLogger::default();

        // Exercise negative inputs and wrapping boundaries across all methods.
        let sample_values = [
            -1i64,
            -2,
            i64::MIN,
            i64::MAX.wrapping_add(1),
            i64::MIN.wrapping_sub(1),
            (-1i64).wrapping_mul(i64::MIN),
        ];
        for &v in &sample_values {
            logger.log_sample(v);
        }

        logger.log_full_sample(&sample_values);

        let partial_cases = [
            (-1i64, -1i32),
            (i64::MIN, i32::MIN),
            (i64::MAX, i32::MAX),
            (i64::MAX.wrapping_add(2), i32::MAX.wrapping_add(2)),
            (i64::MIN.wrapping_sub(2), i32::MIN.wrapping_sub(2)),
        ];
        for &(v, c) in &partial_cases {
            logger.log_partial_sample(v, c);
        }

        assert_eq!(logger.samples, sample_values);
        assert_eq!(logger.full_samples.len(), 1);
        assert_eq!(logger.full_samples[0], sample_values);
        assert_eq!(logger.partial_samples, partial_cases.to_vec());
    }
}
