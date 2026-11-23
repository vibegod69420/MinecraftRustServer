use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub ops: (),
    pub remainder: BTreeMap<String, i64>,
}

impl Dynamic {
    pub fn new(remainder: BTreeMap<String, i64>) -> Self {
        Self { ops: (), remainder }
    }

    pub fn get(&self, key: &str) -> Self {
        let mut remainder = BTreeMap::new();
        if let Some(value) = self.remainder.get(key) {
            remainder.insert(key.to_string(), *value);
        }
        Self { ops: (), remainder }
    }

    pub fn as_long(&self, default_value: i64) -> i64 {
        self.remainder.values().next().copied().unwrap_or(default_value)
    }
}

pub struct MissingDimensionFix {
    _input_schema: (),
    _erase: bool,
}

impl MissingDimensionFix {
    pub fn new(_input_schema: (), erase: bool) -> Self {
        Self { _input_schema: (), _erase: erase }
    }

    // JAVA: net.minecraft.util.datafix.fixes.MissingDimensionFix#recreateSettings
    pub fn recreate_settings(&self, dynamic: Dynamic) -> Dynamic {
        recreate_settings(dynamic)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.MissingDimensionFix#recreateSettings
pub fn recreate_settings(dynamic: Dynamic) -> Dynamic {
    let seed = dynamic.get("seed").as_long(0);
    let overworld = WorldGenSettingsFix::default_overworld(&dynamic, seed);
    let value = WorldGenSettingsFix::vanilla_levels(&dynamic, seed, overworld, false);
    Dynamic::new(value)
}

mod WorldGenSettingsFix {
    use super::Dynamic;
    use std::collections::BTreeMap;

    // JAVA: net.minecraft.util.datafix.fixes.WorldGenSettingsFix#defaultOverworld
    pub fn default_overworld(_dynamic: &Dynamic, _seed: i64) -> () {
    }

    // JAVA: net.minecraft.util.datafix.fixes.WorldGenSettingsFix#vanillaLevels
    pub fn vanilla_levels(_dynamic: &Dynamic, seed: i64, _overworld: (), _amplified: bool) -> BTreeMap<String, i64> {
        let mut remainder = BTreeMap::new();
        remainder.insert("seed".to_string(), seed);
        remainder.insert("dimensions".to_string(), 1);
        remainder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recreate_settings_preserves_zero_seed() {
        let dynamic = Dynamic::new(BTreeMap::new());
        let result = recreate_settings(dynamic);
        assert_eq!(result.get("seed").as_long(-1), 0);
    }

    #[test]
    fn recreate_settings_preserves_positive_seed() {
        let mut remainder = BTreeMap::new();
        remainder.insert("seed".to_string(), 12345);
        let dynamic = Dynamic::new(remainder);
        let result = recreate_settings(dynamic);
        assert_eq!(result.get("seed").as_long(0), 12345);
    }

    #[test]
    fn recreate_settings_preserves_negative_seed() {
        let mut remainder = BTreeMap::new();
        remainder.insert("seed".to_string(), -987654321);
        let dynamic = Dynamic::new(remainder);
        let result = recreate_settings(dynamic);
        assert_eq!(result.get("seed").as_long(0), -987654321);
    }

    #[test]
    fn recreate_settings_preserves_i64_min_seed() {
        let mut remainder = BTreeMap::new();
        remainder.insert("seed".to_string(), i64::MIN);
        let dynamic = Dynamic::new(remainder);
        let result = recreate_settings(dynamic);
        assert_eq!(result.get("seed").as_long(0), i64::MIN);
    }

    #[test]
    fn recreate_settings_creates_dimensions() {
        let dynamic = Dynamic::new(BTreeMap::new());
        let result = recreate_settings(dynamic);
        assert_eq!(result.get("dimensions").as_long(0), 1);
    }

    #[test]
    fn missing_dimension_fix_method_matches_function() {
        let fix = MissingDimensionFix::new((), true);
        let mut remainder = BTreeMap::new();
        remainder.insert("seed".to_string(), 42);
        let dynamic = Dynamic::new(remainder);
        assert_eq!(fix.recreate_settings(dynamic.clone()), recreate_settings(dynamic));
    }
}
