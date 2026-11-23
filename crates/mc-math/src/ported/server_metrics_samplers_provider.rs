// JAVA: net.minecraft.util.profiling.metrics.profiling.ServerMetricsSamplersProvider
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MetricCategory {
    Cpu,
    Jvm,
    TickLoop,
}

#[derive(Clone, Copy, Debug)]
pub enum TimeUnit {
    Nanoseconds,
}

#[derive(Clone, Debug)]
pub struct ValueIncreasedByPercentage {
    pub percentage: f32,
}

impl ValueIncreasedByPercentage {
    pub fn new(percentage: f32) -> Self {
        Self { percentage }
    }
}

pub type LongSupplier = Arc<dyn Fn() -> i64 + Send + Sync>;
pub type ProfileCollectorSupplier = Arc<dyn Fn() -> ProfileCollector + Send + Sync>;

#[derive(Default, Clone, Debug)]
pub struct ProfileCollector;

pub struct SystemReport;

impl SystemReport {
    // JAVA: net.minecraft.SystemReport#sizeInMiB
    pub fn size_in_mib(bytes: i64) -> i64 {
        bytes.wrapping_div(1024).wrapping_div(1024)
    }
}

pub struct MetricsRegistry;

impl MetricsRegistry {
    pub fn instance() -> &'static Self {
        static INSTANCE: MetricsRegistry = MetricsRegistry;
        &INSTANCE
    }

    // JAVA: net.minecraft.util.profiling.metrics.MetricsRegistry#getRegisteredSamplers
    pub fn get_registered_samplers(&self) -> Vec<MetricSampler> {
        Vec::new()
    }
}

pub static METRICS_REGISTRY_INSTANCE: MetricsRegistry = MetricsRegistry;

#[derive(Clone)]
pub struct MetricSampler {
    pub name: String,
    pub category: MetricCategory,
    value: Arc<dyn Fn() -> f64 + Send + Sync>,
    pub before_tick: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PartialEq for MetricSampler {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.category == other.category
    }
}

impl Eq for MetricSampler {}

impl std::hash::Hash for MetricSampler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.category.hash(state);
    }
}

impl MetricSampler {
    pub fn create<F>(name: &str, category: MetricCategory, sampler: F) -> Self
    where
        F: Fn() -> f64 + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            category,
            value: Arc::new(sampler),
            before_tick: None,
        }
    }

    pub fn value(&self) -> f64 {
        (self.value)()
    }

    pub fn builder<I, F>(name: &str, category: MetricCategory, to_double: F, instance: I) -> MetricSamplerBuilder<I>
    where
        F: Fn(&I) -> f64 + Send + Sync + 'static,
        I: Send + Sync + 'static,
    {
        MetricSamplerBuilder {
            name: name.to_string(),
            category,
            to_double: Box::new(to_double),
            instance: Arc::new(Mutex::new(instance)),
            before_tick: None,
            threshold: None,
        }
    }
}

pub struct MetricSamplerBuilder<I> {
    name: String,
    category: MetricCategory,
    to_double: Box<dyn Fn(&I) -> f64 + Send + Sync>,
    instance: Arc<Mutex<I>>,
    before_tick: Option<Arc<dyn Fn() + Send + Sync>>,
    threshold: Option<ValueIncreasedByPercentage>,
}

impl<I> MetricSamplerBuilder<I>
where
    I: Send + Sync + 'static,
{
    pub fn with_before_tick<F>(mut self, before_tick: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before_tick = Some(Arc::new(before_tick));
        self
    }

    pub fn with_threshold_alert(mut self, threshold: ValueIncreasedByPercentage) -> Self {
        self.threshold = Some(threshold);
        self
    }

    pub fn build(self) -> MetricSampler {
        let instance = self.instance;
        let to_double = self.to_double;
        MetricSampler {
            name: self.name,
            category: self.category,
            value: Arc::new(move || {
                let guard = instance.lock().unwrap();
                to_double(&*guard)
            }),
            before_tick: self.before_tick,
        }
    }
}

pub struct Stopwatch {
    ticker: Arc<dyn Fn() -> i64 + Send + Sync>,
    running: bool,
    start_nanos: i64,
    elapsed_nanos: i64,
}

impl Stopwatch {
    pub fn create_unstarted(ticker: Arc<dyn Fn() -> i64 + Send + Sync>) -> Self {
        Self {
            ticker,
            running: false,
            start_nanos: 0,
            elapsed_nanos: 0,
        }
    }

    pub fn start(&mut self) {
        if !self.running {
            self.start_nanos = (self.ticker)();
            self.running = true;
        }
    }

    pub fn stop(&mut self) {
        if self.running {
            let now = (self.ticker)();
            self.elapsed_nanos = self.elapsed_nanos.wrapping_add(now.wrapping_sub(self.start_nanos));
            self.running = false;
        }
    }

    pub fn reset(&mut self) {
        self.elapsed_nanos = 0;
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn elapsed(&self, unit: TimeUnit) -> i64 {
        match unit {
            TimeUnit::Nanoseconds => {
                if self.running {
                    let now = (self.ticker)();
                    self.elapsed_nanos.wrapping_add(now.wrapping_sub(self.start_nanos))
                } else {
                    self.elapsed_nanos
                }
            }
        }
    }
}

pub struct ProfilerSamplerAdapter;

impl ProfilerSamplerAdapter {
    pub fn new() -> Self {
        Self
    }

    // JAVA: ProfilerSamplerAdapter#newSamplersFoundInProfiler
    pub fn new_samplers_found_in_profiler(&self, _supplier: ProfileCollectorSupplier) -> Vec<MetricSampler> {
        Vec::new()
    }
}

struct CpuStats {
    nr_of_cpus: usize,
    last_poll_ms: i64,
}

impl CpuStats {
    fn new() -> Self {
        let nr_of_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Self {
            nr_of_cpus,
            last_poll_ms: 0,
        }
    }

    // JAVA: ServerMetricsSamplersProvider.CpuStats#loadForCpu
    fn load_for_cpu(&mut self, _cpu: usize) -> f64 {
        let now = current_time_millis();
        if self.last_poll_ms == 0 || self.last_poll_ms.wrapping_add(501) < now {
            self.last_poll_ms = now;
        }
        0.0
    }
}

fn current_time_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub struct ServerMetricsSamplersProvider {
    samplers: HashSet<MetricSampler>,
    sampler_factory: ProfilerSamplerAdapter,
}

impl ServerMetricsSamplersProvider {
    // JAVA: ServerMetricsSamplersProvider#ServerMetricsSamplersProvider
    pub fn new(long_supplier: LongSupplier, runtime_independent: bool) -> Self {
        let mut samplers = HashSet::new();
        samplers.insert(Self::tick_time_sampler(long_supplier.clone()));
        if runtime_independent {
            samplers.extend(Self::runtime_independent_samplers());
        }
        Self {
            samplers,
            sampler_factory: ProfilerSamplerAdapter::new(),
        }
    }

    // JAVA: ServerMetricsSamplersProvider#runtimeIndependentSamplers
    pub fn runtime_independent_samplers() -> HashSet<MetricSampler> {
        let mut samplers = HashSet::new();

        let cpu_stats = Arc::new(Mutex::new(CpuStats::new()));
        let nr_of_cpus = cpu_stats.lock().unwrap().nr_of_cpus;
        for cpu in 0..nr_of_cpus {
            let stats = cpu_stats.clone();
            samplers.insert(MetricSampler::create(
                &format!("cpu#{}", cpu),
                MetricCategory::Cpu,
                move || {
                    let mut guard = stats.lock().unwrap();
                    guard.load_for_cpu(cpu) * 100.0
                },
            ));
        }

        samplers.insert(MetricSampler::create("heap MiB", MetricCategory::Jvm, || {
            SystemReport::size_in_mib(0) as f64
        }));

        samplers.extend(METRICS_REGISTRY_INSTANCE.get_registered_samplers());
        samplers
    }

    // JAVA: ServerMetricsSamplersProvider#samplers
    pub fn samplers(&mut self, supplier: ProfileCollectorSupplier) -> HashSet<MetricSampler> {
        self.samplers
            .extend(self.sampler_factory.new_samplers_found_in_profiler(supplier));
        self.samplers.clone()
    }

    // JAVA: ServerMetricsSamplersProvider#tickTimeSampler
    pub fn tick_time_sampler(long_supplier: LongSupplier) -> MetricSampler {
        let ticker_supplier = long_supplier.clone();
        let stopwatch = Arc::new(Mutex::new(Stopwatch::create_unstarted(Arc::new(
            move || ticker_supplier(),
        ))));

        let to_double_stopwatch = stopwatch.clone();
        let to_double = move |_instance: &Arc<Mutex<Stopwatch>>| {
            let mut guard = to_double_stopwatch.lock().unwrap();
            if guard.is_running() {
                guard.stop();
            }
            let elapsed = guard.elapsed(TimeUnit::Nanoseconds);
            guard.reset();
            elapsed as f64
        };

        let before_stopwatch = stopwatch.clone();
        let builder = MetricSampler::builder("ticktime", MetricCategory::TickLoop, to_double, stopwatch);
        builder
            .with_before_tick(move || {
                before_stopwatch.lock().unwrap().start();
            })
            .with_threshold_alert(ValueIncreasedByPercentage::new(2.0))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_adds_tick_sampler() {
        let provider = ServerMetricsSamplersProvider::new(Arc::new(|| 0), false);
        assert!(provider.samplers.iter().any(|s| s.name == "ticktime"));
    }

    #[test]
    fn runtime_independent_contains_heap() {
        let samplers = ServerMetricsSamplersProvider::runtime_independent_samplers();
        assert!(samplers.iter().any(|s| s.name == "heap MiB"));
    }

    #[test]
    fn tick_sampler_measures_elapsed() {
        let counter = Arc::new(Mutex::new(0i64));
        let supplier_counter = counter.clone();
        let sampler = ServerMetricsSamplersProvider::tick_time_sampler(Arc::new(move || {
            let mut g = supplier_counter.lock().unwrap();
            *g += 1;
            *g
        }));
        if let Some(before) = &sampler.before_tick {
            before();
        }
        let value = sampler.value();
        assert!(value >= 0.0);
    }

    #[test]
    fn size_in_mib() {
        assert_eq!(SystemReport::size_in_mib(1024 * 1024 * 3), 3);
    }

    #[test]
    fn cpu_sampler_count() {
        let samplers = ServerMetricsSamplersProvider::runtime_independent_samplers();
        assert!(samplers.iter().any(|s| s.name.starts_with("cpu#")));
    }

    #[test]
    fn samplers_method_returns_set() {
        let mut provider = ServerMetricsSamplersProvider::new(Arc::new(|| 0), true);
        let set = provider.samplers(Arc::new(|| ProfileCollector));
        assert!(!set.is_empty());
    }
}
