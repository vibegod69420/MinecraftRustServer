use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

/// Minimal stand-in for `net.minecraft.world.level.levelgen.feature.ConfiguredFeature`.
pub trait ConfiguredFeatureInfo: Send + Sync {
    /// Corresponds to `configuredFeature.feature()` in Java log output.
    fn feature_type_debug(&self) -> String;
    /// Corresponds to `configuredFeature.toString()` in Java log output.
    fn config_debug(&self) -> String;
}

/// Minimal stand-in for `net.minecraft.world.level.levelgen.placement.PlacedFeature`.
pub trait PlacedFeatureInfo: Send + Sync {}

/// Minimal stand-in for `net.minecraft.server.level.ServerLevel`.
pub trait ServerLevel: Send + Sync {
    /// `level.dimension().identifier().toString()`
    fn dimension_identifier(&self) -> String;
    /// `level.getServer().isRunning()`
    fn is_server_running(&self) -> bool;
    /// Resolves `topFeature().flatMap(registry::getResourceKey).map(ResourceKey::identifier)`
    /// at log time, matching Java `logCounts`.
    fn placed_feature_resource_key(
        &self,
        feature: &Arc<dyn PlacedFeatureInfo>,
    ) -> Option<String>;
}

// JAVA: net.minecraft.world.level.levelgen.feature.FeatureCountTracker$FeatureData
#[derive(Clone)]
pub struct FeatureData {
    pub feature: Arc<dyn ConfiguredFeatureInfo>,
    pub top_feature: Option<Arc<dyn PlacedFeatureInfo>>,
}

impl PartialEq for FeatureData {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.feature, &other.feature)
            && match (&self.top_feature, &other.top_feature) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for FeatureData {}

impl Hash for FeatureData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.feature) as *const ()).hash(state);
        match &self.top_feature {
            Some(p) => {
                1u8.hash(state);
                (Arc::as_ptr(p) as *const ()).hash(state);
            }
            None => {
                0u8.hash(state);
            }
        }
    }
}

impl fmt::Debug for FeatureData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FeatureData")
            .field("feature_type", &self.feature.feature_type_debug())
            .field("feature_config", &self.feature.config_debug())
            .field("has_top_feature", &self.top_feature.is_some())
            .finish()
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.FeatureCountTracker$LevelData
pub struct LevelData {
    pub feature_data: Mutex<HashMap<FeatureData, i32>>,
    pub chunks_with_features: Mutex<i32>,
}

impl LevelData {
    fn new() -> Self {
        Self {
            feature_data: Mutex::new(HashMap::new()),
            chunks_with_features: Mutex::new(0i32),
        }
    }
}

const EXPIRE_AFTER_ACCESS: Duration = Duration::from_secs(5 * 60);

struct CacheEntry {
    level: Weak<dyn ServerLevel>,
    last_access: Instant,
    data: Arc<LevelData>,
}

struct LevelCache {
    entries: HashMap<usize, CacheEntry>,
}

fn arc_key(level: &Arc<dyn ServerLevel>) -> usize {
    Arc::as_ptr(level) as *const () as usize
}

impl LevelCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn cleanup(&mut self, now: Instant) {
        self.entries.retain(|_, e| {
            e.level.strong_count() > 0
                && now.saturating_duration_since(e.last_access) < EXPIRE_AFTER_ACCESS
        });
    }

    fn get(&mut self, level: &Arc<dyn ServerLevel>) -> Arc<LevelData> {
        let now = Instant::now();
        self.cleanup(now);
        let key = arc_key(level);
        let entry = self.entries.entry(key).or_insert_with(|| CacheEntry {
            level: Arc::downgrade(level),
            last_access: now,
            data: Arc::new(LevelData::new()),
        });
        entry.last_access = now;
        Arc::clone(&entry.data)
    }
}

static DATA: OnceLock<Mutex<LevelCache>> = OnceLock::new();

fn data_cache() -> &'static Mutex<LevelCache> {
    DATA.get_or_init(|| Mutex::new(LevelCache::new()))
}

fn format_optional(opt: &Option<String>) -> String {
    match opt {
        Some(v) => format!("Optional[{}]", v),
        None => "Optional.empty".to_string(),
    }
}

fn java_format_f10_6(v: f64) -> String {
    if v.is_nan() {
        return format!("{:>10}", "NaN");
    }
    if v.is_infinite() {
        if v.is_sign_negative() {
            return format!("{:>10}", "-Infinity");
        }
        return format!("{:>10}", "Infinity");
    }
    format!("{:>10.6}", v)
}

// JAVA: net.minecraft.world.level.levelgen.feature.FeatureCountTracker#chunkDecorated
pub fn chunk_decorated(level: &Arc<dyn ServerLevel>) {
    let result = (|| {
        let level_data = {
            let mut cache = data_cache().lock().map_err(|e| format!("{}", e))?;
            cache.get(level)
        };
        let mut chunks = level_data
            .chunks_with_features
            .lock()
            .map_err(|e| format!("{}", e))?;
        *chunks = chunks.wrapping_add(1);
        Ok::<(), String>(())
    })();
    if let Err(e) = result {
        eprintln!("Failed to increment chunk count: {}", e);
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.FeatureCountTracker#featurePlaced
pub fn feature_placed(
    level: &Arc<dyn ServerLevel>,
    feature: Arc<dyn ConfiguredFeatureInfo>,
    top_feature: Option<Arc<dyn PlacedFeatureInfo>>,
) {
    let result = (|| {
        let level_data = {
            let mut cache = data_cache().lock().map_err(|e| format!("{}", e))?;
            cache.get(level)
        };
        let mut map = level_data
            .feature_data
            .lock()
            .map_err(|e| format!("{}", e))?;
        let key = FeatureData {
            feature,
            top_feature,
        };
        let count = map.entry(key).or_insert(0i32);
        *count = count.wrapping_add(1);
        Ok::<(), String>(())
    })();
    if let Err(e) = result {
        eprintln!("Failed to increment feature count: {}", e);
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.FeatureCountTracker#clearCounts
pub fn clear_counts() {
    let mut cache = data_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.entries.clear();
    eprintln!("Cleared feature counts");
}

// JAVA: net.minecraft.world.level.levelgen.feature.FeatureCountTracker#logCounts
pub fn log_counts() {
    eprintln!("Logging feature counts:");
    let entries: Vec<(Arc<dyn ServerLevel>, Arc<LevelData>)> = {
        let mut cache = data_cache().lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        cache.cleanup(now);
        cache
            .entries
            .values()
            .filter_map(|e| e.level.upgrade().map(|lvl| (lvl, Arc::clone(&e.data))))
            .collect()
    };

    for (level, level_data) in entries {
        let dim_id = level.dimension_identifier();
        let running = level.is_server_running();
        let status = if running { "running" } else { "dead" };
        let level_str = format!("{} {}", status, dim_id);

        let chunks: i32 = {
            let c = level_data
                .chunks_with_features
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            *c
        };

        eprintln!("{} total_chunks: {}", level_str, chunks);

        let feature_snapshot: Vec<(FeatureData, i32)> = {
            let map = level_data
                .feature_data
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            map.iter().map(|(k, v)| (k.clone(), *v)).collect()
        };

        for (fd, count) in feature_snapshot {
            let top_key = fd
                .top_feature
                .as_ref()
                .and_then(|pf| level.placed_feature_resource_key(pf));
            let ratio = f64::from(count) / f64::from(chunks);
            eprintln!(
                "{} {} {} {} {} {}",
                level_str,
                format!("{:>10}", count),
                java_format_f10_6(ratio),
                format_optional(&top_key),
                fd.feature.feature_type_debug(),
                fd.feature.config_debug(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    static TEST_LOCK: StdMutex<()> = StdMutex::new(());

    struct MockLevel {
        dim: String,
        running: bool,
        key: Option<String>,
    }

    impl ServerLevel for MockLevel {
        fn dimension_identifier(&self) -> String {
            self.dim.clone()
        }

        fn is_server_running(&self) -> bool {
            self.running
        }

        fn placed_feature_resource_key(
            &self,
            _feature: &Arc<dyn PlacedFeatureInfo>,
        ) -> Option<String> {
            self.key.clone()
        }
    }

    struct MockFeature {
        type_name: String,
        config: String,
    }

    impl ConfiguredFeatureInfo for MockFeature {
        fn feature_type_debug(&self) -> String {
            self.type_name.clone()
        }

        fn config_debug(&self) -> String {
            self.config.clone()
        }
    }

    struct MockPlaced;
    impl PlacedFeatureInfo for MockPlaced {}

    #[test]
    fn chunk_decorated_increments_and_wraps() {
        let _g = TEST_LOCK.lock().unwrap();
        clear_counts();

        let level: Arc<dyn ServerLevel> = Arc::new(MockLevel {
            dim: "minecraft:overworld".to_string(),
            running: true,
            key: None,
        });

        chunk_decorated(&level);
        chunk_decorated(&level);

        let level_data = {
            let mut cache = data_cache().lock().unwrap();
            cache.get(&level)
        };
        assert_eq!(*level_data.chunks_with_features.lock().unwrap(), 2i32);

        {
            let mut chunks = level_data.chunks_with_features.lock().unwrap();
            *chunks = i32::MAX;
        }
        chunk_decorated(&level);
        assert_eq!(
            *level_data.chunks_with_features.lock().unwrap(),
            i32::MIN
        );

        clear_counts();
    }

    #[test]
    fn feature_placed_counts_aggregates_and_wraps() {
        let _g = TEST_LOCK.lock().unwrap();
        clear_counts();

        let level: Arc<dyn ServerLevel> = Arc::new(MockLevel {
            dim: "minecraft:overworld".to_string(),
            running: true,
            key: None,
        });
        let feature: Arc<dyn ConfiguredFeatureInfo> = Arc::new(MockFeature {
            type_name: "minecraft:tree".to_string(),
            config: "cfg".to_string(),
        });
        let top: Arc<dyn PlacedFeatureInfo> = Arc::new(MockPlaced);

        feature_placed(&level, Arc::clone(&feature), Some(Arc::clone(&top)));
        feature_placed(&level, Arc::clone(&feature), Some(Arc::clone(&top)));
        feature_placed(&level, Arc::clone(&feature), None);

        let key_with_top = FeatureData {
            feature: Arc::clone(&feature),
            top_feature: Some(Arc::clone(&top)),
        };
        let key_without_top = FeatureData {
            feature: Arc::clone(&feature),
            top_feature: None,
        };

        {
            let level_data = {
                let mut cache = data_cache().lock().unwrap();
                cache.get(&level)
            };
            let map = level_data.feature_data.lock().unwrap();
            assert_eq!(map.get(&key_with_top), Some(&2i32));
            assert_eq!(map.get(&key_without_top), Some(&1i32));
        }

        // Negative input and wrapping edge cases
        {
            let level_data = {
                let mut cache = data_cache().lock().unwrap();
                cache.get(&level)
            };
            let mut map = level_data.feature_data.lock().unwrap();
            map.insert(key_without_top.clone(), -1i32);
        }
        feature_placed(&level, Arc::clone(&feature), None);
        {
            let level_data = {
                let mut cache = data_cache().lock().unwrap();
                cache.get(&level)
            };
            let map = level_data.feature_data.lock().unwrap();
            assert_eq!(map.get(&key_without_top), Some(&0i32));
        }

        {
            let level_data = {
                let mut cache = data_cache().lock().unwrap();
                cache.get(&level)
            };
            let mut map = level_data.feature_data.lock().unwrap();
            map.insert(key_without_top.clone(), i32::MAX);
        }
        feature_placed(&level, Arc::clone(&feature), None);
        {
            let level_data = {
                let mut cache = data_cache().lock().unwrap();
                cache.get(&level)
            };
            let map = level_data.feature_data.lock().unwrap();
            assert_eq!(map.get(&key_without_top), Some(&i32::MIN));
        }

        clear_counts();
    }

    #[test]
    fn clear_counts_empties_cache() {
        let _g = TEST_LOCK.lock().unwrap();
        clear_counts();

        let level: Arc<dyn ServerLevel> = Arc::new(MockLevel {
            dim: "d".to_string(),
            running: true,
            key: None,
        });
        chunk_decorated(&level);
        clear_counts();

        let cache = data_cache().lock().unwrap();
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn java_format_f10_6_handles_special_and_negative_floats() {
        let _g = TEST_LOCK.lock().unwrap();
        assert_eq!(java_format_f10_6(1.5_f64), "  1.500000");
        assert_eq!(java_format_f10_6(-0.25_f64), " -0.250000");
        assert_eq!(java_format_f10_6(f64::INFINITY), "  Infinity");
        assert_eq!(java_format_f10_6(f64::NEG_INFINITY), " -Infinity");
        assert_eq!(java_format_f10_6(f64::NAN), "       NaN");

        // Division-by-zero ratio and float bits edge cases
        let pos_inf = 1.0_f64 / 0.0_f64;
        let neg_inf = -1.0_f64 / 0.0_f64;
        let nan = 0.0_f64 / 0.0_f64;
        assert_eq!(java_format_f10_6(pos_inf), "  Infinity");
        assert_eq!(java_format_f10_6(neg_inf), " -Infinity");
        assert_eq!(java_format_f10_6(nan), "       NaN");
        assert!(pos_inf.to_bits() != neg_inf.to_bits());
    }

    #[test]
    fn log_counts_handles_zero_chunks_and_optional_formatting() {
        let _g = TEST_LOCK.lock().unwrap();
        clear_counts();

        let level: Arc<dyn ServerLevel> = Arc::new(MockLevel {
            dim: "minecraft:overworld".to_string(),
            running: false,
            key: Some("minecraft:oak".to_string()),
        });
        let feature: Arc<dyn ConfiguredFeatureInfo> = Arc::new(MockFeature {
            type_name: "minecraft:tree".to_string(),
            config: "cfg=A".to_string(),
        });
        let top: Arc<dyn PlacedFeatureInfo> = Arc::new(MockPlaced);

        // chunks_with_features stays 0 -> ratio is Infinity/NaN path; should not panic.
        feature_placed(&level, Arc::clone(&feature), Some(top));
        feature_placed(&level, Arc::clone(&feature), None);

        assert_eq!(
            format_optional(&Some("minecraft:oak".to_string())),
            "Optional[minecraft:oak]"
        );
        assert_eq!(format_optional(&None), "Optional.empty");

        log_counts();
        clear_counts();
    }
}
