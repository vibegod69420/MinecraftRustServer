/// Faithful port of `net.minecraft.util.debugchart.SampleStorage`
/// (Minecraft Java Edition 1.21.11).
///
/// A fixed-capacity storage of debug chart samples represented as `i64`
/// (Java `long`).
///
/// This module defines the [`SampleStorage`] trait, which mirrors the Java
/// interface contract. Concrete implementations are expected to provide the
/// actual storage and indexing semantics.
pub trait SampleStorage {
    // JAVA: net.minecraft.util.debugchart.SampleStorage#capacity
    fn capacity(&self) -> i32;

    // JAVA: net.minecraft.util.debugchart.SampleStorage#size
    fn size(&self) -> i32;

    // JAVA: net.minecraft.util.debugchart.SampleStorage#get
    fn get(&self, index: i32) -> i64;

    // JAVA: net.minecraft.util.debugchart.SampleStorage#get
    fn get_with_offset(&self, index: i32, offset: i32) -> i64;

    // JAVA: net.minecraft.util.debugchart.SampleStorage#reset
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple ring-buffer mock implementation used only for testing the
    /// [`SampleStorage`] trait contract. This is not a port of a Minecraft
    /// class; it exists to exercise the trait API.
    #[derive(Debug, Clone)]
    struct MockSampleStorage {
        samples: Vec<i64>,
        len: i32,
    }

    impl MockSampleStorage {
        fn new(capacity: i32) -> Self {
            assert!(capacity >= 0, "capacity must be non-negative");
            Self {
                samples: vec![0i64; capacity as usize],
                len: 0,
            }
        }

        fn push(&mut self, value: i64) {
            let cap = self.capacity();
            if cap <= 0 {
                return;
            }

            // Match Java int arithmetic semantics for index progression.
            let idx_i32 = self.len % cap;
            let idx = idx_i32 as usize;
            self.samples[idx] = value;

            if self.len < cap {
                self.len = self.len.wrapping_add(1);
            } else {
                self.len = cap;
            }
        }
    }

    impl SampleStorage for MockSampleStorage {
        // JAVA: net.minecraft.util.debugchart.SampleStorage#capacity
        fn capacity(&self) -> i32 {
            self.samples.len() as i32
        }

        // JAVA: net.minecraft.util.debugchart.SampleStorage#size
        fn size(&self) -> i32 {
            self.len
        }

        // JAVA: net.minecraft.util.debugchart.SampleStorage#get
        fn get(&self, index: i32) -> i64 {
            let cap = self.capacity();
            if cap <= 0 {
                return 0i64;
            }

            // Use Euclidean modulo for safe indexing with negative inputs in tests.
            let wrapped = index.rem_euclid(cap) as usize;
            self.samples[wrapped]
        }

        // JAVA: net.minecraft.util.debugchart.SampleStorage#get
        fn get_with_offset(&self, index: i32, offset: i32) -> i64 {
            // Match Java int addition semantics.
            self.get(index.wrapping_add(offset))
        }

        // JAVA: net.minecraft.util.debugchart.SampleStorage#reset
        fn reset(&mut self) {
            for sample in &mut self.samples {
                *sample = 0i64;
            }
            self.len = 0;
        }
    }

    #[test]
    fn test_capacity_and_size_basic() {
        let mut storage = MockSampleStorage::new(4);
        assert_eq!(storage.capacity(), 4);
        assert_eq!(storage.size(), 0);

        storage.push(10);
        assert_eq!(storage.size(), 1);
        storage.push(20);
        storage.push(30);
        storage.push(40);
        assert_eq!(storage.size(), 4);

        // Pushing beyond capacity keeps size at capacity in this mock.
        storage.push(50);
        assert_eq!(storage.size(), 4);
        assert_eq!(storage.capacity(), 4);
    }

    #[test]
    fn test_reset_clears_samples_and_size() {
        let mut storage = MockSampleStorage::new(3);
        storage.push(1);
        storage.push(2);
        assert_eq!(storage.size(), 2);
        assert_eq!(storage.get(0), 1);
        assert_eq!(storage.get(1), 2);

        storage.reset();

        assert_eq!(storage.capacity(), 3);
        assert_eq!(storage.size(), 0);
        assert_eq!(storage.get(0), 0);
        assert_eq!(storage.get(1), 0);
        assert_eq!(storage.get(2), 0);
    }

    #[test]
    fn test_get_and_get_with_offset() {
        let mut storage = MockSampleStorage::new(4);
        storage.push(100);
        storage.push(200);
        storage.push(300);

        assert_eq!(storage.get(0), 100);
        assert_eq!(storage.get(1), 200);
        assert_eq!(storage.get(2), 300);

        assert_eq!(storage.get_with_offset(0, 0), 100);
        assert_eq!(storage.get_with_offset(0, 1), 200);
        assert_eq!(storage.get_with_offset(1, 1), 300);
        assert_eq!(storage.get_with_offset(2, -1), 200);
    }

    #[test]
    fn test_negative_indices() {
        let mut storage = MockSampleStorage::new(4);
        storage.push(10);
        storage.push(20);
        storage.push(30);
        storage.push(40);

        // Mock uses rem_euclid for safe indexing; verify no panic and expected wrap.
        assert_eq!(storage.get(-1), 40); // -1 rem_euclid 4 == 3
        assert_eq!(storage.get(-2), 30); // -2 rem_euclid 4 == 2
        assert_eq!(storage.get(-4), 10); // -4 rem_euclid 4 == 0
        assert_eq!(storage.get(-5), 40); // -5 rem_euclid 4 == 3

        assert_eq!(storage.get_with_offset(-1, 0), 40);
        assert_eq!(storage.get_with_offset(0, -1), 40);
    }

    #[test]
    fn test_i32_edge_indices_no_panic() {
        let mut storage = MockSampleStorage::new(4);
        storage.push(11);
        storage.push(22);
        storage.push(33);
        storage.push(44);

        let _ = storage.get(i32::MIN);
        let _ = storage.get(i32::MAX);
        let _ = storage.get(0);
        let _ = storage.get(-1);

        let _ = storage.get_with_offset(i32::MAX, i32::MAX);
        let _ = storage.get_with_offset(i32::MIN, i32::MIN);
        let _ = storage.get_with_offset(i32::MIN, -1);
        let _ = storage.get_with_offset(i32::MAX, 1);
        let _ = storage.get_with_offset(-1, i32::MIN);
    }

    #[test]
    fn test_get_with_offset_wrapping_semantics() {
        let mut storage = MockSampleStorage::new(4);
        storage.push(10);
        storage.push(20);
        storage.push(30);
        storage.push(40);

        // The mock uses wrapping_add for the offset addition, matching Java int semantics.
        let wrapped_index = i32::MAX.wrapping_add(1);
        assert_eq!(wrapped_index, i32::MIN);
        assert_eq!(
            storage.get_with_offset(i32::MAX, 1),
            storage.get(wrapped_index)
        );

        let wrapped_neg = i32::MIN.wrapping_add(-1);
        assert_eq!(wrapped_neg, i32::MAX);
        assert_eq!(
            storage.get_with_offset(i32::MIN, -1),
            storage.get(wrapped_neg)
        );

        assert_eq!(storage.get_with_offset(1, -1), storage.get(0));
    }

    #[test]
    fn test_i64_edge_values_and_float_bits() {
        let mut storage = MockSampleStorage::new(8);
        let nan_f64_bits = f64::NAN.to_bits() as i64;
        let inf_f64_bits = f64::INFINITY.to_bits() as i64;
        let neg_inf_f64_bits = f64::NEG_INFINITY.to_bits() as i64;
        let nan_f32_bits = f32::NAN.to_bits() as i64;

        storage.push(i64::MIN);
        storage.push(i64::MAX);
        storage.push(0i64);
        storage.push(-1i64);
        storage.push(nan_f64_bits);
        storage.push(inf_f64_bits);
        storage.push(neg_inf_f64_bits);
        storage.push(nan_f32_bits);

        assert_eq!(storage.get(0), i64::MIN);
        assert_eq!(storage.get(1), i64::MAX);
        assert_eq!(storage.get(2), 0i64);
        assert_eq!(storage.get(3), -1i64);
        assert_eq!(storage.get(4), nan_f64_bits);
        assert_eq!(storage.get(5), inf_f64_bits);
        assert_eq!(storage.get(6), neg_inf_f64_bits);
        assert_eq!(storage.get(7), nan_f32_bits);
    }

    #[test]
    fn test_float_bit_patterns_preserved() {
        let mut storage = MockSampleStorage::new(4);
        let neg_zero = (-0.0f64).to_bits() as i64;
        let pos_zero = (0.0f64).to_bits() as i64;
        let nan_bits = f64::NAN.to_bits() as i64;
        let neg_inf = f64::NEG_INFINITY.to_bits() as i64;

        storage.push(neg_zero);
        storage.push(pos_zero);
        storage.push(nan_bits);
        storage.push(neg_inf);

        assert_eq!(storage.get(0), neg_zero);
        assert_eq!(storage.get(1), pos_zero);
        assert_eq!(storage.get(2), nan_bits);
        assert_eq!(storage.get(3), neg_inf);

        // -0.0 and +0.0 compare equal as floats but have distinct bit patterns.
        assert_ne!(storage.get(0), storage.get(1));
        assert_eq!((-0.0f64).to_bits(), neg_zero as u64);
        assert_eq!((0.0f64).to_bits(), pos_zero as u64);
    }

    #[test]
    fn test_zero_capacity_storage() {
        let mut storage = MockSampleStorage::new(0);
        assert_eq!(storage.capacity(), 0);
        assert_eq!(storage.size(), 0);
        assert_eq!(storage.get(0), 0i64);
        assert_eq!(storage.get(i32::MIN), 0i64);
        assert_eq!(storage.get(i32::MAX), 0i64);
        assert_eq!(storage.get_with_offset(0, 0), 0i64);
        assert_eq!(storage.get_with_offset(i32::MAX, 1), 0i64);

        storage.reset();
        assert_eq!(storage.capacity(), 0);
        assert_eq!(storage.size(), 0);
    }

    #[test]
    fn test_trait_object_safety() {
        let mut storage = MockSampleStorage::new(2);
        storage.push(42i64);
        storage.push(-7i64);

        {
            let obj: &dyn SampleStorage = &storage;
            assert_eq!(obj.capacity(), 2);
            assert_eq!(obj.size(), 2);
            assert_eq!(obj.get(0), 42i64);
            assert_eq!(obj.get(1), -7i64);
            assert_eq!(obj.get_with_offset(0, 1), -7i64);
        }

        {
            let obj_mut: &mut dyn SampleStorage = &mut storage;
            obj_mut.reset();
            assert_eq!(obj_mut.size(), 0);
            assert_eq!(obj_mut.capacity(), 2);
            assert_eq!(obj_mut.get(0), 0i64);
        }
    }

    fn accepts_storage<S: SampleStorage>(storage: &S) -> i64 {
        let cap = storage.capacity() as i64;
        let size = storage.size() as i64;
        cap.wrapping_add(size)
    }

    #[test]
    fn test_generic_bound() {
        let mut storage = MockSampleStorage::new(10);
        assert_eq!(accepts_storage(&storage), 10i64);
        storage.push(1);
        assert_eq!(accepts_storage(&storage), 11i64);
    }

    #[test]
    fn test_extreme_i32_and_i64_values_via_trait() {
        struct ExtremeMock;

        impl SampleStorage for ExtremeMock {
            // JAVA: net.minecraft.util.debugchart.SampleStorage#capacity
            fn capacity(&self) -> i32 {
                i32::MAX
            }

            // JAVA: net.minecraft.util.debugchart.SampleStorage#size
            fn size(&self) -> i32 {
                i32::MIN
            }

            // JAVA: net.minecraft.util.debugchart.SampleStorage#get
            fn get(&self, _index: i32) -> i64 {
                i64::MIN
            }

            // JAVA: net.minecraft.util.debugchart.SampleStorage#get
            fn get_with_offset(&self, _index: i32, _offset: i32) -> i64 {
                i64::MAX
            }

            // JAVA: net.minecraft.util.debugchart.SampleStorage#reset
            fn reset(&mut self) {}
        }

        let mut extreme = ExtremeMock;
        assert_eq!(extreme.capacity(), i32::MAX);
        assert_eq!(extreme.size(), i32::MIN);
        assert_eq!(extreme.get(0), i64::MIN);
        assert_eq!(extreme.get(i32::MIN), i64::MIN);
        assert_eq!(extreme.get(i32::MAX), i64::MIN);
        assert_eq!(extreme.get_with_offset(-1, 1), i64::MAX);
        assert_eq!(extreme.get_with_offset(i32::MAX, i32::MAX), i64::MAX);
        extreme.reset();
        // Values remain representable after reset call on this mock.
        assert_eq!(extreme.capacity(), i32::MAX);
    }

    #[test]
    fn test_reset_is_callable_multiple_times() {
        let mut storage = MockSampleStorage::new(3);
        storage.push(123);
        storage.reset();
        storage.reset();
        assert_eq!(storage.size(), 0);
        assert_eq!(storage.get(0), 0i64);

        storage.push(456);
        assert_eq!(storage.size(), 1);
        assert_eq!(storage.get(0), 456i64);
        storage.reset();
        assert_eq!(storage.size(), 0);
        assert_eq!(storage.get(0), 0i64);
    }
}
