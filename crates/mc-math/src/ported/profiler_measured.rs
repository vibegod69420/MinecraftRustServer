//! Faithful port of `net.minecraft.util.profiling.metrics.ProfilerMeasured`.
//!
//! Source: `net.minecraft.util.profiling.metrics.ProfilerMeasured` (Java 1.21.11).
//!
//! This module is self-contained. Because `MetricSampler` is not yet ported, a
//! minimal placeholder type is provided to preserve the interface shape.

/// Minimal self-contained placeholder for
/// `net.minecraft.util.profiling.metrics.MetricSampler`.
///
/// The full profiler metrics sampler is outside the scope of this module. This
/// placeholder keeps [`ProfilerMeasured`] compilable and testable in isolation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricSampler {
    /// Human-readable sampler name.
    pub name: String,
}

impl MetricSampler {
    /// Creates a new metric sampler placeholder with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Faithful port of `net.minecraft.util.profiling.metrics.ProfilerMeasured`.
///
/// Implementors expose metric samplers consumed by the profiling/metrics
/// subsystem.
pub trait ProfilerMeasured {
    // JAVA: net.minecraft.util.profiling.metrics.ProfilerMeasured#profiledMetrics
    fn profiled_metrics(&self) -> Vec<MetricSampler>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[derive(Debug, Default)]
    struct NoMetrics;

    impl ProfilerMeasured for NoMetrics {
        // JAVA: net.minecraft.util.profiling.metrics.ProfilerMeasured#profiledMetrics
        fn profiled_metrics(&self) -> Vec<MetricSampler> {
            Vec::new()
        }
    }

    #[derive(Debug, Clone)]
    struct StaticMetrics {
        names: Vec<String>,
    }

    impl StaticMetrics {
        fn new(names: &[&str]) -> Self {
            Self {
                names: names.iter().map(|name| (*name).to_string()).collect(),
            }
        }
    }

    impl ProfilerMeasured for StaticMetrics {
        // JAVA: net.minecraft.util.profiling.metrics.ProfilerMeasured#profiledMetrics
        fn profiled_metrics(&self) -> Vec<MetricSampler> {
            self.names
                .iter()
                .map(|name| MetricSampler::new(name.clone()))
                .collect()
        }
    }

    #[derive(Debug)]
    struct CountingMetrics {
        base: i32,
        count: usize,
    }

    impl ProfilerMeasured for CountingMetrics {
        // JAVA: net.minecraft.util.profiling.metrics.ProfilerMeasured#profiledMetrics
        fn profiled_metrics(&self) -> Vec<MetricSampler> {
            let mut out = Vec::with_capacity(self.count);
            let mut i = 0usize;
            while i < self.count {
                let offset = i32::try_from(i).unwrap_or(i32::MAX);
                let value = self.base.wrapping_add(offset);
                out.push(MetricSampler::new(format!("metric_{value}")));
                i = i.wrapping_add(1);
            }
            out
        }
    }

    #[test]
    fn metric_sampler_new_stores_name() {
        let sampler = MetricSampler::new("tickTime");
        assert_eq!(sampler.name, "tickTime");
    }

    #[test]
    fn metric_sampler_new_accepts_owned_string() {
        let sampler = MetricSampler::new(String::from("heap"));
        assert_eq!(sampler.name, "heap");
    }

    #[test]
    fn metric_sampler_empty_name_edge_case() {
        let sampler = MetricSampler::new("");
        assert_eq!(sampler.name, "");
        assert!(sampler.name.is_empty());
    }

    #[test]
    fn metric_sampler_clone_eq_and_hash() {
        let a = MetricSampler::new("cpu");
        let b = a.clone();
        assert_eq!(a, b);

        let c = MetricSampler::new("gpu");
        assert_ne!(a, c);

        let mut set = HashSet::new();
        set.insert(a.clone());
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&MetricSampler::new("cpu")));
    }

    #[test]
    fn metric_sampler_unicode_and_special_names() {
        let names = [
            "métrica-Δ",
            "指标",
            "line\nbreak",
            "tab\tchar",
            "spaces  inside",
            "🧪profiler",
        ];
        for name in names {
            let sampler = MetricSampler::new(name);
            assert_eq!(sampler.name, name);
        }
    }

    #[test]
    fn metric_sampler_name_from_negative_and_boundary_integers() {
        let cases = [
            (i32::MIN, "metric_-2147483648"),
            (-1_i32, "metric_-1"),
            (0_i32, "metric_0"),
            (1_i32, "metric_1"),
            (i32::MAX, "metric_2147483647"),
        ];
        for (value, expected) in cases {
            let sampler = MetricSampler::new(format!("metric_{value}"));
            assert_eq!(sampler.name, expected);
        }
    }

    #[test]
    fn metric_sampler_name_from_float_bit_patterns() {
        let values = [
            f32::NEG_INFINITY,
            -0.0_f32,
            0.0_f32,
            f32::MIN_POSITIVE,
            f32::MAX,
            f32::INFINITY,
            f32::NAN,
        ];
        for value in values {
            let bits = value.to_bits();
            let sampler = MetricSampler::new(format!("f32_{bits:08x}"));
            assert_eq!(sampler.name, format!("f32_{bits:08x}"));

            let roundtrip = f32::from_bits(bits);
            assert_eq!(roundtrip.to_bits(), bits);
        }

        let dvalues = [
            f64::NEG_INFINITY,
            -0.0_f64,
            0.0_f64,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::INFINITY,
            f64::NAN,
        ];
        for value in dvalues {
            let bits = value.to_bits();
            let sampler = MetricSampler::new(format!("f64_{bits:016x}"));
            assert_eq!(sampler.name, format!("f64_{bits:016x}"));

            let roundtrip = f64::from_bits(bits);
            assert_eq!(roundtrip.to_bits(), bits);
        }
    }

    #[test]
    fn profiled_metrics_empty() {
        let measured = NoMetrics;
        let metrics = measured.profiled_metrics();
        assert!(metrics.is_empty());
    }

    #[test]
    fn profiled_metrics_returns_configured_samplers() {
        let measured = StaticMetrics::new(&["a", "b", "c"]);
        let metrics = measured.profiled_metrics();
        assert_eq!(metrics.len(), 3);
        assert_eq!(metrics[0], MetricSampler::new("a"));
        assert_eq!(metrics[1], MetricSampler::new("b"));
        assert_eq!(metrics[2], MetricSampler::new("c"));
    }

    #[test]
    fn profiled_metrics_single_element() {
        let measured = StaticMetrics::new(&["only"]);
        let metrics = measured.profiled_metrics();
        assert_eq!(metrics, vec![MetricSampler::new("only")]);
    }

    #[test]
    fn profiled_metrics_preserves_order_for_many_elements() {
        let names: Vec<String> = (0_i32..64_i32).map(|i| format!("metric_{i}")).collect();
        let measured = StaticMetrics {
            names: names.clone(),
        };
        let metrics = measured.profiled_metrics();
        let got: Vec<String> = metrics.into_iter().map(|m| m.name).collect();
        assert_eq!(got, names);
    }

    #[test]
    fn profiled_metrics_trait_object_boxed() {
        let measured: Box<dyn ProfilerMeasured> = Box::new(StaticMetrics::new(&["x", "y"]));
        let metrics = measured.profiled_metrics();
        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].name, "x");
        assert_eq!(metrics[1].name, "y");
    }

    #[test]
    fn profiled_metrics_trait_object_ref() {
        let measured = NoMetrics;
        let as_trait: &dyn ProfilerMeasured = &measured;
        assert!(as_trait.profiled_metrics().is_empty());
    }

    #[test]
    fn profiled_metrics_can_be_called_multiple_times_consistently() {
        let measured = StaticMetrics::new(&["repeat", "again"]);
        let first = measured.profiled_metrics();
        let second = measured.profiled_metrics();
        let third = measured.profiled_metrics();
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn profiled_metrics_wrapping_integer_generation_edge_cases() {
        let measured = CountingMetrics {
            base: i32::MAX - 1,
            count: 4,
        };
        let metrics = measured.profiled_metrics();
        let names: Vec<String> = metrics.into_iter().map(|m| m.name).collect();
        assert_eq!(
            names,
            vec![
                format!("metric_{}", i32::MAX - 1),
                format!("metric_{}", i32::MAX),
                format!("metric_{}", i32::MIN),
                format!("metric_{}", i32::MIN + 1),
            ]
        );

        let measured_neg = CountingMetrics {
            base: i32::MIN,
            count: 3,
        };
        let names_neg: Vec<String> = measured_neg
            .profiled_metrics()
            .into_iter()
            .map(|m| m.name)
            .collect();
        assert_eq!(
            names_neg,
            vec![
                format!("metric_{}", i32::MIN),
                format!("metric_{}", i32::MIN + 1),
                format!("metric_{}", i32::MIN + 2),
            ]
        );
    }

    #[test]
    fn metric_sampler_debug_contains_name() {
        let sampler = MetricSampler::new("debug_me");
        let dbg = format!("{sampler:?}");
        assert!(dbg.contains("MetricSampler"));
        assert!(dbg.contains("debug_me"));
    }
}
