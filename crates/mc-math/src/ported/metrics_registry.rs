use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(i32::from(c));
    }
    h
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetricCategory {
    name: String,
}

impl MetricCategory {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub trait ProfilerMeasured: Send + Sync {
    fn profiled_metrics(&self) -> Vec<MetricSampler>;
}

type SamplerFn = Arc<dyn Fn() -> f64 + Send + Sync>;
type OnStartTickFn = Arc<dyn Fn() + Send + Sync>;
type ThresholdTestFn = Arc<dyn Fn(f64) -> bool + Send + Sync>;

struct MetricSamplerInner {
    name: String,
    category: MetricCategory,
    sampler: SamplerFn,
    on_start_tick: OnStartTickFn,
    threshold_test: Option<ThresholdTestFn>,
    delegates: Option<Vec<MetricSampler>>,
}

#[derive(Clone)]
pub struct MetricSampler {
    inner: Arc<MetricSamplerInner>,
}

impl MetricSampler {
    pub fn new(
        name: impl Into<String>,
        category: MetricCategory,
        sampler: impl Fn() -> f64 + Send + Sync + 'static,
        on_start_tick: impl Fn() + Send + Sync + 'static,
        threshold_test: Option<ThresholdTestFn>,
    ) -> Self {
        Self {
            inner: Arc::new(MetricSamplerInner {
                name: name.into(),
                category,
                sampler: Arc::new(sampler),
                on_start_tick: Arc::new(on_start_tick),
                threshold_test,
                delegates: None,
            }),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.inner.name
    }

    pub fn get_category(&self) -> &MetricCategory {
        &self.inner.category
    }

    pub fn sample(&self) -> f64 {
        (self.inner.sampler)()
    }

    pub fn on_start_tick(&self) {
        (self.inner.on_start_tick)();
    }

    pub fn is_aggregated(&self) -> bool {
        self.inner.delegates.is_some()
    }

    fn base_equals(&self, other: &Self) -> bool {
        self.inner.name == other.inner.name && self.inner.category == other.inner.category
    }

    fn base_hash_code(&self) -> i32 {
        let mut h: i32 = 1;
        h = h
            .wrapping_mul(31)
            .wrapping_add(java_string_hash(&self.inner.name));
        h = h
            .wrapping_mul(31)
            .wrapping_add(java_string_hash(&self.inner.category.name));
        h
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry$AggregatedMetricSampler#<init>
    fn aggregated(name: String, delegates: Vec<MetricSampler>) -> Self {
        let category = delegates[0].get_category().clone();
        let sampler_delegates = delegates.clone();
        let tick_delegates = delegates.clone();
        let threshold_test = Self::threshold_test(&delegates);
        Self {
            inner: Arc::new(MetricSamplerInner {
                name,
                category,
                sampler: Arc::new(move || Self::average_value_from_delegates(&sampler_delegates)),
                on_start_tick: Arc::new(move || Self::before_tick(&tick_delegates)),
                threshold_test: Some(threshold_test),
                delegates: Some(delegates),
            }),
        }
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry$AggregatedMetricSampler#thresholdTest
    fn threshold_test(delegates: &[MetricSampler]) -> ThresholdTestFn {
        let delegates = delegates.to_vec();
        Arc::new(move |value: f64| {
            delegates
                .iter()
                .any(|d| d.inner.threshold_test.as_ref().is_some_and(|t| t(value)))
        })
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry$AggregatedMetricSampler#beforeTick
    fn before_tick(delegates: &[MetricSampler]) {
        for sampler in delegates {
            sampler.on_start_tick();
        }
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry$AggregatedMetricSampler#averageValueFromDelegates
    fn average_value_from_delegates(delegates: &[MetricSampler]) -> f64 {
        let mut sum: f64 = 0.0;
        for sampler in delegates {
            sum += (sampler.inner.sampler)();
        }
        sum / (delegates.len() as f64)
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry$AggregatedMetricSampler#equals
    pub fn equals(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.inner, &other.inner) {
            return true;
        }
        let (Some(self_d), Some(other_d)) = (&self.inner.delegates, &other.inner.delegates) else {
            return false;
        };
        if !self.base_equals(other) {
            return false;
        }
        self_d.len() == other_d.len()
            && self_d.iter().zip(other_d.iter()).all(|(a, b)| a.equals(b))
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry$AggregatedMetricSampler#hashCode
    pub fn hash_code(&self) -> i32 {
        let super_hash = self.base_hash_code();
        match &self.inner.delegates {
            Some(delegates) => {
                let mut delegates_hash: i32 = 1;
                for d in delegates {
                    delegates_hash = delegates_hash.wrapping_mul(31).wrapping_add(d.hash_code());
                }
                let mut h: i32 = 1;
                h = h.wrapping_mul(31).wrapping_add(super_hash);
                h = h.wrapping_mul(31).wrapping_add(delegates_hash);
                h
            }
            None => super_hash,
        }
    }
}

impl PartialEq for MetricSampler {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for MetricSampler {}

impl std::hash::Hash for MetricSampler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.inner) as *const ()).hash(state);
    }
}

pub struct MetricsRegistry {
    measured_instances: Mutex<Vec<Weak<dyn ProfilerMeasured>>>,
}

// JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry#INSTANCE
pub static INSTANCE: MetricsRegistry = MetricsRegistry::new();

impl MetricsRegistry {
    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry#<init>
    const fn new() -> Self {
        Self {
            measured_instances: Mutex::new(Vec::new()),
        }
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry#add
    pub fn add(&self, measured: Arc<dyn ProfilerMeasured>) {
        let mut instances = self
            .measured_instances
            .lock()
            .expect("metrics registry lock poisoned");
        instances.retain(|w| w.strong_count() > 0);
        let ptr = Arc::as_ptr(&measured) as *const ();
        let exists = instances.iter().any(|w| {
            w.upgrade()
                .is_some_and(|upgraded| Arc::as_ptr(&upgraded) as *const () == ptr)
        });
        if !exists {
            instances.push(Arc::downgrade(&measured));
        }
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry#getRegisteredSamplers
    pub fn get_registered_samplers(&self) -> Vec<MetricSampler> {
        let measured: Vec<Arc<dyn ProfilerMeasured>> = {
            let mut instances = self
                .measured_instances
                .lock()
                .expect("metrics registry lock poisoned");
            instances.retain(|w| w.strong_count() > 0);
            instances.iter().filter_map(|w| w.upgrade()).collect()
        };
        let mut grouped: HashMap<String, Vec<MetricSampler>> = HashMap::new();
        for m in &measured {
            for sampler in m.profiled_metrics() {
                grouped
                    .entry(sampler.get_name().to_string())
                    .or_default()
                    .push(sampler);
            }
        }
        Self::aggregate_duplicates(grouped)
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry#aggregateDuplicates
    fn aggregate_duplicates(groups: HashMap<String, Vec<MetricSampler>>) -> Vec<MetricSampler> {
        groups
            .into_iter()
            .map(|(name, samplers)| {
                if samplers.len() > 1 {
                    MetricSampler::aggregated(name, samplers)
                } else {
                    samplers.into_iter().next().unwrap()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    struct TestMeasured(Vec<MetricSampler>);

    impl ProfilerMeasured for TestMeasured {
        fn profiled_metrics(&self) -> Vec<MetricSampler> {
            self.0.clone()
        }
    }

    fn sampler(name: &str, value: f64) -> MetricSampler {
        MetricSampler::new(
            name,
            MetricCategory::new("test"),
            move || value,
            || {},
            None,
        )
    }

    #[test]
    fn aggregate_duplicates_average_order_and_negatives() {
        let a = sampler("x", 1e16);
        let b = sampler("x", 1.0);
        let c = sampler("x", -2.0);
        let agg = MetricSampler::aggregated("x".to_string(), vec![a, b, c]);
        let expected = ((1e16_f64 + 1.0_f64) + (-2.0_f64)) / 3.0_f64;
        assert_eq!(agg.sample().to_bits(), expected.to_bits());
        assert!(agg.is_aggregated());

        let single = sampler("y", 42.0);
        let mut groups = HashMap::new();
        groups.insert("y".to_string(), vec![single]);
        let out = MetricsRegistry::aggregate_duplicates(groups);
        assert_eq!(out.len(), 1);
        assert!(!out[0].is_aggregated());
        assert_eq!(out[0].sample().to_bits(), 42.0_f64.to_bits());
    }

    #[test]
    fn threshold_test_any_match_and_nulls() {
        let s1 = MetricSampler::new("x", MetricCategory::new("c"), || 0.0, || {}, None);
        let s2 = MetricSampler::new(
            "x",
            MetricCategory::new("c"),
            || 0.0,
            || {},
            Some(Arc::new(|v: f64| v > 10.0)),
        );
        let agg = MetricSampler::aggregated("x".to_string(), vec![s1, s2]);
        let tt = agg.inner.threshold_test.as_ref().expect("threshold");
        assert!(!tt(5.0));
        assert!(tt(11.0));

        let no_t = MetricSampler::new("n", MetricCategory::new("c"), || 0.0, || {}, None);
        let agg2 = MetricSampler::aggregated("n".to_string(), vec![no_t.clone(), no_t]);
        let tt2 = agg2.inner.threshold_test.as_ref().expect("threshold");
        assert!(!tt2(f64::INFINITY));
    }

    #[test]
    fn before_tick_calls_all_delegates() {
        let c1 = Arc::new(AtomicI32::new(0));
        let c2 = Arc::new(AtomicI32::new(0));
        let c1c = c1.clone();
        let c2c = c2.clone();
        let s1 = MetricSampler::new(
            "x",
            MetricCategory::new("c"),
            || 0.0,
            move || {
                c1c.fetch_add(1, Ordering::SeqCst);
            },
            None,
        );
        let s2 = MetricSampler::new(
            "x",
            MetricCategory::new("c"),
            || 0.0,
            move || {
                c2c.fetch_add(2, Ordering::SeqCst);
            },
            None,
        );
        let agg = MetricSampler::aggregated("x".to_string(), vec![s1, s2]);
        agg.on_start_tick();
        assert_eq!(c1.load(Ordering::SeqCst), 1);
        assert_eq!(c2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn hash_code_wraps_and_equals_structural() {
        assert_eq!(java_string_hash(""), 0);
        assert_eq!(java_string_hash("Aa"), 2112i32);
        assert_eq!(java_string_hash("BB"), 2112i32);
        let expected = 0i32
            .wrapping_mul(31)
            .wrapping_add(0x8000)
            .wrapping_mul(31)
            .wrapping_add(0x8000);
        assert_eq!(java_string_hash("\u{8000}\u{8000}"), expected);

        let s1 = sampler("x", 1.0);
        let s2 = sampler("x", 2.0);
        let a1 = MetricSampler::aggregated("x".to_string(), vec![s1.clone(), s2.clone()]);
        let a2 = MetricSampler::aggregated("x".to_string(), vec![s1.clone(), s2.clone()]);
        assert!(a1.equals(&a2));
        assert_eq!(a1.hash_code(), a2.hash_code());

        let s3 = sampler("x", 3.0);
        let a3 = MetricSampler::aggregated("x".to_string(), vec![s1, s3]);
        assert!(!a1.equals(&a3));
    }

    #[test]
    fn registry_add_get_and_weak_cleanup() {
        let registry = MetricsRegistry::new();
        let m1: Arc<dyn ProfilerMeasured> = Arc::new(TestMeasured(vec![
            sampler("cpu", 10.0),
            sampler("mem", 5.0),
        ]));
        let m2: Arc<dyn ProfilerMeasured> = Arc::new(TestMeasured(vec![sampler("cpu", 30.0)]));
        registry.add(m1.clone());
        registry.add(m2.clone());

        let samplers = registry.get_registered_samplers();
        assert_eq!(samplers.len(), 2);
        let cpu = samplers.iter().find(|s| s.get_name() == "cpu").unwrap();
        let mem = samplers.iter().find(|s| s.get_name() == "mem").unwrap();
        assert_eq!(cpu.sample().to_bits(), 20.0_f64.to_bits());
        assert_eq!(mem.sample().to_bits(), 5.0_f64.to_bits());

        drop(m2);
        let after = registry.get_registered_samplers();
        let cpu_after = after.iter().find(|s| s.get_name() == "cpu").unwrap();
        assert_eq!(cpu_after.sample().to_bits(), 10.0_f64.to_bits());
        assert!(!cpu_after.is_aggregated());
    }
}
