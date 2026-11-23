use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Port of `net.minecraft.util.TimeUtil#NANOSECONDS_PER_MILLISECOND`.
pub const NANOSECONDS_PER_MILLISECOND: f64 = 1_000_000.0;

/// Minimal stub for `net.minecraft.util.profiling.metrics.MetricCategory`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricCategory(pub String);

/// Minimal stub for `net.minecraft.util.profiling.ActiveProfiler.PathEntry`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathEntry {
    max_duration: i64,
}

impl PathEntry {
    pub fn new(max_duration: i64) -> Self {
        Self { max_duration }
    }

    // JAVA: net.minecraft.util.profiling.ActiveProfiler$PathEntry#getMaxDuration
    pub fn get_max_duration(&self) -> i64 {
        self.max_duration
    }
}

/// Minimal stub for `net.minecraft.util.profiling.ProfileCollector`.
pub trait ProfileCollector {
    fn get_charted_paths(&self) -> Vec<(String, MetricCategory)>;
    fn get_entry(&self, path: &str) -> Option<PathEntry>;
}

/// Minimal stub for `net.minecraft.util.profiling.metrics.MetricSampler`.
pub struct MetricSampler {
    name: String,
    category: MetricCategory,
    sampler: Box<dyn Fn() -> f64>,
}

impl MetricSampler {
    // JAVA: net.minecraft.util.profiling.metrics.MetricSampler#create
    pub fn create(name: String, category: MetricCategory, sampler: Box<dyn Fn() -> f64>) -> Self {
        Self {
            name,
            category,
            sampler,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_category(&self) -> &MetricCategory {
        &self.category
    }

    pub fn sample(&self) -> f64 {
        (self.sampler)()
    }
}

impl PartialEq for MetricSampler {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for MetricSampler {}

impl Hash for MetricSampler {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl std::fmt::Debug for MetricSampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricSampler")
            .field("name", &self.name)
            .field("category", &self.category)
            .finish()
    }
}

/// Port of `net.minecraft.util.profiling.metrics.profiling.ProfilerSamplerAdapter`.
pub struct ProfilerSamplerAdapter {
    previously_found_sampler_names: HashSet<String>,
}

impl ProfilerSamplerAdapter {
    pub fn new() -> Self {
        Self {
            previously_found_sampler_names: HashSet::new(),
        }
    }

    // JAVA: net.minecraft.util.profiling.metrics.profiling.ProfilerSamplerAdapter#newSamplersFoundInProfiler
    pub fn new_samplers_found_in_profiler(
        &mut self,
        supplier: Rc<dyn Fn() -> Rc<dyn ProfileCollector>>,
    ) -> HashSet<MetricSampler> {
        let collector = supplier();
        let mut result: HashSet<MetricSampler> = HashSet::new();

        for (path, category) in collector.get_charted_paths() {
            if !self.previously_found_sampler_names.contains(&path) {
                let sampler =
                    Self::sampler_for_profiling_path(Rc::clone(&supplier), path, category);
                result.insert(sampler);
            }
        }

        for sampler in &result {
            self.previously_found_sampler_names
                .insert(sampler.get_name().to_string());
        }

        result
    }

    // JAVA: net.minecraft.util.profiling.metrics.profiling.ProfilerSamplerAdapter#samplerForProfilingPath
    fn sampler_for_profiling_path(
        supplier: Rc<dyn Fn() -> Rc<dyn ProfileCollector>>,
        path: String,
        category: MetricCategory,
    ) -> MetricSampler {
        let name = path.clone();
        MetricSampler::create(
            name,
            category,
            Box::new(move || {
                let collector = supplier();
                match collector.get_entry(&path) {
                    Some(entry) => {
                        entry.get_max_duration() as f64 / NANOSECONDS_PER_MILLISECOND
                    }
                    None => 0.0,
                }
            }),
        )
    }
}

impl Default for ProfilerSamplerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockCollector {
        paths: Vec<(String, MetricCategory)>,
        entries: HashMap<String, i64>,
    }

    impl ProfileCollector for MockCollector {
        fn get_charted_paths(&self) -> Vec<(String, MetricCategory)> {
            self.paths.clone()
        }

        fn get_entry(&self, path: &str) -> Option<PathEntry> {
            self.entries.get(path).map(|&d| PathEntry::new(d))
        }
    }

    fn make_supplier(
        paths: Vec<(String, MetricCategory)>,
        entries: HashMap<String, i64>,
    ) -> Rc<dyn Fn() -> Rc<dyn ProfileCollector>> {
        let collector: Rc<dyn ProfileCollector> = Rc::new(MockCollector { paths, entries });
        Rc::new(move || Rc::clone(&collector))
    }

    #[test]
    fn finds_new_samplers_and_records_names() {
        let supplier = make_supplier(
            vec![
                ("root/a".to_string(), MetricCategory("cat".to_string())),
                ("root/b".to_string(), MetricCategory("cat".to_string())),
            ],
            HashMap::new(),
        );
        let mut adapter = ProfilerSamplerAdapter::new();
        let samplers = adapter.new_samplers_found_in_profiler(Rc::clone(&supplier));
        assert_eq!(samplers.len(), 2);
        let names: HashSet<String> = samplers.iter().map(|s| s.get_name().to_string()).collect();
        assert!(names.contains("root/a"));
        assert!(names.contains("root/b"));
    }

    #[test]
    fn previously_found_names_are_excluded() {
        let supplier = make_supplier(
            vec![
                ("a".to_string(), MetricCategory("c".to_string())),
                ("b".to_string(), MetricCategory("c".to_string())),
            ],
            HashMap::new(),
        );
        let mut adapter = ProfilerSamplerAdapter::new();
        let first = adapter.new_samplers_found_in_profiler(Rc::clone(&supplier));
        assert_eq!(first.len(), 2);
        let second = adapter.new_samplers_found_in_profiler(supplier);
        assert_eq!(second.len(), 0);
    }

    #[test]
    fn sampler_value_uses_nanos_per_ms() {
        let mut entries = HashMap::new();
        entries.insert("root/tick".to_string(), 5_500_000_i64);
        let supplier = make_supplier(
            vec![("root/tick".to_string(), MetricCategory("t".to_string()))],
            entries,
        );
        let mut adapter = ProfilerSamplerAdapter::new();
        let samplers = adapter.new_samplers_found_in_profiler(supplier);
        assert_eq!(samplers.len(), 1);
        let sampler = samplers.iter().next().unwrap();
        assert!((sampler.sample() - 5.5).abs() < 1e-9);
    }

    #[test]
    fn missing_entry_returns_zero() {
        let supplier = make_supplier(
            vec![("missing".to_string(), MetricCategory("t".to_string()))],
            HashMap::new(),
        );
        let mut adapter = ProfilerSamplerAdapter::new();
        let samplers = adapter.new_samplers_found_in_profiler(supplier);
        let sampler = samplers.iter().next().unwrap();
        assert_eq!(sampler.sample(), 0.0);
    }

    #[test]
    fn new_paths_found_on_later_call() {
        let paths: Rc<RefCell<Vec<(String, MetricCategory)>>> =
            Rc::new(RefCell::new(vec![("a".to_string(), MetricCategory("c".to_string()))]));
        let paths_clone = Rc::clone(&paths);

        struct DynCollector {
            paths: Rc<RefCell<Vec<(String, MetricCategory)>>>,
        }
        impl ProfileCollector for DynCollector {
            fn get_charted_paths(&self) -> Vec<(String, MetricCategory)> {
                self.paths.borrow().clone()
            }
            fn get_entry(&self, _path: &str) -> Option<PathEntry> {
                None
            }
        }

        let supplier: Rc<dyn Fn() -> Rc<dyn ProfileCollector>> = Rc::new(move || {
            Rc::new(DynCollector {
                paths: Rc::clone(&paths_clone),
            }) as Rc<dyn ProfileCollector>
        });

        let mut adapter = ProfilerSamplerAdapter::new();
        assert_eq!(adapter.new_samplers_found_in_profiler(Rc::clone(&supplier)).len(), 1);

        paths.borrow_mut().push(("b".to_string(), MetricCategory("c".to_string())));
        let second = adapter.new_samplers_found_in_profiler(supplier);
        assert_eq!(second.len(), 1);
        assert_eq!(second.iter().next().unwrap().get_name(), "b");
    }

    #[test]
    fn negative_duration_and_zero_are_preserved() {
        let mut entries = HashMap::new();
        entries.insert("neg".to_string(), -2_000_000_i64);
        entries.insert("zero".to_string(), 0_i64);
        let supplier = make_supplier(
            vec![
                ("neg".to_string(), MetricCategory("c".to_string())),
                ("zero".to_string(), MetricCategory("c".to_string())),
            ],
            entries,
        );
        let mut adapter = ProfilerSamplerAdapter::new();
        let samplers = adapter.new_samplers_found_in_profiler(supplier);
        let mut by_name: HashMap<String, f64> = HashMap::new();
        for s in &samplers {
            by_name.insert(s.get_name().to_string(), s.sample());
        }
        assert!((by_name["neg"] - (-2.0)).abs() < 1e-9);
        assert_eq!(by_name["zero"], 0.0);
    }
}
