//! Faithful port of `net.minecraft.util.profiling.metrics.MetricsSamplerProvider`.
//!
//! This module is self-contained: it defines minimal local stand-ins for
//! `ProfileCollector` and `MetricSampler` so the provider interface can compile
//! without the full Minecraft profiling/metrics stack.

use std::collections::HashSet;

/// Minimal self-contained stand-in for `net.minecraft.util.profiling.ProfileCollector`.
///
/// The real collector type is outside the scope of this ported interface module.
/// Implementations may carry profiling state; this trait only establishes the
/// type boundary required by [`MetricsSamplerProvider`].
pub trait ProfileCollector {}

/// Minimal self-contained stand-in for `net.minecraft.util.profiling.metrics.MetricSampler`.
///
/// The real sampler carries measurement metadata and sampling callbacks. This
/// placeholder preserves the typed set element required by [`MetricsSamplerProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricSampler {
    name: String,
}

impl MetricSampler {
    /// Creates a new placeholder metric sampler with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the sampler name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Port of `net.minecraft.util.profiling.metrics.MetricsSamplerProvider`.
///
/// A provider exposes metric samplers to a profiling/metrics pipeline. The supplier
/// argument mirrors Java's `Supplier<ProfileCollector>` and allows implementations to
/// obtain a profile collector lazily.
pub trait MetricsSamplerProvider {
    // JAVA: net.minecraft.util.profiling.metrics.MetricsSamplerProvider#samplers
    fn samplers(
        &self,
        profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
    ) -> HashSet<MetricSampler>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug)]
    struct MockCollector;
    impl ProfileCollector for MockCollector {}

    #[derive(Debug)]
    struct IdCollector {
        _id: i32,
    }
    impl ProfileCollector for IdCollector {}

    struct EmptyProvider;
    impl MetricsSamplerProvider for EmptyProvider {
        fn samplers(
            &self,
            _profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
        ) -> HashSet<MetricSampler> {
            HashSet::new()
        }
    }

    #[test]
    fn empty_provider_returns_empty_set() {
        let provider = EmptyProvider;
        let samplers = provider.samplers(&|| Box::new(MockCollector));
        assert!(samplers.is_empty());
    }

    #[test]
    fn provider_returns_expected_samplers() {
        struct FixedProvider;
        impl MetricsSamplerProvider for FixedProvider {
            fn samplers(
                &self,
                _profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let mut set = HashSet::new();
                set.insert(MetricSampler::new("cpu"));
                set.insert(MetricSampler::new("memory"));
                set
            }
        }

        let provider = FixedProvider;
        let samplers = provider.samplers(&|| Box::new(MockCollector));
        assert_eq!(samplers.len(), 2);
        assert!(samplers.contains(&MetricSampler::new("cpu")));
        assert!(samplers.contains(&MetricSampler::new("memory")));
    }

    #[test]
    fn supplier_is_invoked_and_can_be_counted() {
        struct CountingProvider;
        impl MetricsSamplerProvider for CountingProvider {
            fn samplers(
                &self,
                profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let _first: Box<dyn ProfileCollector> = profile_collector_supplier();
                let _second: Box<dyn ProfileCollector> = profile_collector_supplier();
                let mut set = HashSet::new();
                set.insert(MetricSampler::new("called"));
                set
            }
        }

        let calls = Cell::new(0i32);
        let provider = CountingProvider;
        let samplers = provider.samplers(&|| {
            calls.set(calls.get().wrapping_add(1));
            Box::new(MockCollector)
        });

        assert_eq!(calls.get(), 2);
        assert_eq!(samplers.len(), 1);
        assert!(samplers.contains(&MetricSampler::new("called")));
    }

    #[test]
    fn supplier_can_return_distinct_collectors_with_negative_and_overflow_ids() {
        struct CollectorAwareProvider;
        impl MetricsSamplerProvider for CollectorAwareProvider {
            fn samplers(
                &self,
                profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let _c1 = profile_collector_supplier();
                let _c2 = profile_collector_supplier();
                let _c3 = profile_collector_supplier();
                let mut set = HashSet::new();
                set.insert(MetricSampler::new("ok"));
                set
            }
        }

        let counter = Cell::new(i32::MAX);
        let provider = CollectorAwareProvider;
        let samplers = provider.samplers(&|| {
            let id = counter.get();
            counter.set(id.wrapping_add(1));
            Box::new(IdCollector { _id: id })
        });

        // i32::MAX -> wrapping_add(1) -> i32::MIN -> wrapping_add(1) -> i32::MIN + 1
        assert_eq!(counter.get(), i32::MAX.wrapping_add(3));
        assert_eq!(samplers.len(), 1);
        assert!(samplers.contains(&MetricSampler::new("ok")));
    }

    #[test]
    fn duplicate_samplers_are_deduplicated_by_set() {
        struct DupProvider;
        impl MetricsSamplerProvider for DupProvider {
            fn samplers(
                &self,
                _profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let mut set = HashSet::new();
                set.insert(MetricSampler::new("same"));
                set.insert(MetricSampler::new("same"));
                set.insert(MetricSampler::new("different"));
                set
            }
        }

        let provider = DupProvider;
        let samplers = provider.samplers(&|| Box::new(MockCollector));
        assert_eq!(samplers.len(), 2);
        assert!(samplers.contains(&MetricSampler::new("same")));
        assert!(samplers.contains(&MetricSampler::new("different")));
    }

    #[test]
    fn trait_is_object_safe() {
        struct DynProvider;
        impl MetricsSamplerProvider for DynProvider {
            fn samplers(
                &self,
                _profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let mut set = HashSet::new();
                set.insert(MetricSampler::new("dyn"));
                set
            }
        }

        let provider: Box<dyn MetricsSamplerProvider> = Box::new(DynProvider);
        let samplers = provider.samplers(&|| Box::new(MockCollector));
        assert_eq!(samplers.len(), 1);
        assert!(samplers.contains(&MetricSampler::new("dyn")));
    }

    #[test]
    fn wrapping_integer_semantics_edge_cases() {
        struct WrappingProvider;
        impl MetricsSamplerProvider for WrappingProvider {
            fn samplers(
                &self,
                _profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let mut set = HashSet::new();

                let max_i32: i32 = i32::MAX;
                let min_i32: i32 = i32::MIN;
                let neg_one: i32 = -1;

                let wrapped_add = max_i32.wrapping_add(1);
                let wrapped_sub = min_i32.wrapping_sub(1);
                let wrapped_mul = neg_one.wrapping_mul(min_i32);
                let negative_wrapped = (-12345i32).wrapping_sub(1);

                // Java int shift amount is masked by 31; long by 63.
                let shift32_amount: u32 = 32;
                let shift64_amount: u32 = 64;
                let masked_shift_i32 = 1u32.wrapping_shl(shift32_amount & 31);
                let masked_shift_i64 = 1u64.wrapping_shl(shift64_amount & 63);
                let negative_shift_amount: i32 = -1;
                let masked_negative_shift_i32 =
                    1u32.wrapping_shl((negative_shift_amount as u32) & 31);

                set.insert(MetricSampler::new(format!("add={wrapped_add}")));
                set.insert(MetricSampler::new(format!("sub={wrapped_sub}")));
                set.insert(MetricSampler::new(format!("mul={wrapped_mul}")));
                set.insert(MetricSampler::new(format!("neg={negative_wrapped}")));
                set.insert(MetricSampler::new(format!("shl32={masked_shift_i32}")));
                set.insert(MetricSampler::new(format!("shl64={masked_shift_i64}")));
                set.insert(MetricSampler::new(format!(
                    "shlneg={masked_negative_shift_i32}"
                )));
                set
            }
        }

        // Sanity checks for the Java-like integer semantics used in the test.
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!((-1i32).wrapping_mul(i32::MIN), i32::MIN);
        assert_eq!((-12345i32).wrapping_sub(1), -12346i32);
        assert_eq!(1u32.wrapping_shl(32u32 & 31), 1u32);
        assert_eq!(1u64.wrapping_shl(64u32 & 63), 1u64);
        assert_eq!(1u32.wrapping_shl((-1i32 as u32) & 31), 1u32 << 31);

        let provider = WrappingProvider;
        let samplers = provider.samplers(&|| Box::new(MockCollector));

        assert!(samplers.contains(&MetricSampler::new(format!("add={}", i32::MIN))));
        assert!(samplers.contains(&MetricSampler::new(format!("sub={}", i32::MAX))));
        assert!(samplers.contains(&MetricSampler::new(format!("mul={}", i32::MIN))));
        assert!(samplers.contains(&MetricSampler::new("neg=-12346".to_string())));
        assert!(samplers.contains(&MetricSampler::new("shl32=1".to_string())));
        assert!(samplers.contains(&MetricSampler::new("shl64=1".to_string())));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "shlneg={}",
            1u32 << 31
        ))));
        assert_eq!(samplers.len(), 7);
    }

    #[test]
    fn float_bit_patterns_are_preserved() {
        struct FloatProvider;
        impl MetricsSamplerProvider for FloatProvider {
            fn samplers(
                &self,
                _profile_collector_supplier: &dyn Fn() -> Box<dyn ProfileCollector>,
            ) -> HashSet<MetricSampler> {
                let values_f32: [f32; 7] = [
                    0.0,
                    -0.0,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NAN,
                    f32::MIN_POSITIVE,
                    -f32::MIN_POSITIVE,
                ];
                let values_f64: [f64; 7] = [
                    0.0,
                    -0.0,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NAN,
                    f64::MIN_POSITIVE,
                    -f64::MIN_POSITIVE,
                ];

                let mut set = HashSet::new();
                for v in values_f32 {
                    set.insert(MetricSampler::new(format!("f32:{:08x}", v.to_bits())));
                }
                for v in values_f64 {
                    set.insert(MetricSampler::new(format!("f64:{:016x}", v.to_bits())));
                }
                set
            }
        }

        let provider = FloatProvider;
        let samplers = provider.samplers(&|| Box::new(MockCollector));

        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            0.0f32.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            (-0.0f32).to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            f32::INFINITY.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            f32::NEG_INFINITY.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            f32::NAN.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            f32::MIN_POSITIVE.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f32:{:08x}",
            (-f32::MIN_POSITIVE).to_bits()
        ))));

        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            0.0f64.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            (-0.0f64).to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            f64::INFINITY.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            f64::NEG_INFINITY.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            f64::NAN.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            f64::MIN_POSITIVE.to_bits()
        ))));
        assert!(samplers.contains(&MetricSampler::new(format!(
            "f64:{:016x}",
            (-f64::MIN_POSITIVE).to_bits()
        ))));

        // +0.0 and -0.0 must remain distinguishable at bit level.
        assert_ne!(0.0f32.to_bits(), (-0.0f32).to_bits());
        assert_ne!(0.0f64.to_bits(), (-0.0f64).to_bits());

        assert_eq!(samplers.len(), 14);
    }

    #[test]
    fn metric_sampler_placeholder_behaves_as_value() {
        let a = MetricSampler::new("test");
        let b = MetricSampler::new("test");
        let c = MetricSampler::new("other");

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.name(), "test");
        assert_eq!(c.name(), "other");

        let mut set = HashSet::new();
        set.insert(a.clone());
        set.insert(b);
        set.insert(c.clone());
        assert_eq!(set.len(), 2);
        assert!(set.contains(&a));
        assert!(set.contains(&c));
    }

    #[test]
    fn supplier_not_called_when_provider_does_not_need_it() {
        let calls = Cell::new(0u32);
        let provider = EmptyProvider;
        let samplers = provider.samplers(&|| {
            calls.set(calls.get().wrapping_add(1));
            Box::new(MockCollector)
        });
        assert_eq!(calls.get(), 0);
        assert!(samplers.is_empty());
    }
}
