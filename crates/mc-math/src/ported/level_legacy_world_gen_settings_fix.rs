use std::collections::BTreeMap;

const WORLD_GEN_SETTINGS: &str = "WorldGenSettings";
const OLD_SETTINGS_KEYS: &[&str] = &[
    "RandomSeed",
    "generatorName",
    "generatorOptions",
    "generatorVersion",
    "legacy_custom_options",
    "MapFeatures",
    "BonusChest",
];

#[derive(Clone, Debug, PartialEq)]
pub enum Dynamic {
    Null,
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Map(BTreeMap<String, Dynamic>),
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Dynamic::Map(BTreeMap::new())
    }

    pub fn string(s: impl Into<String>) -> Self {
        Dynamic::String(s.into())
    }

    pub fn get(&self, key: &str) -> DynamicResult {
        match self {
            Dynamic::Map(m) => DynamicResult(m.get(key).cloned()),
            _ => DynamicResult(None),
        }
    }

    pub fn remove(self, key: &str) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.remove(key);
                Dynamic::Map(m)
            }
            other => other,
        }
    }

    pub fn set(self, key: &str, value: Dynamic) -> Self {
        match self {
            Dynamic::Map(mut m) => {
                m.insert(key.to_string(), value);
                Dynamic::Map(m)
            }
            other => other,
        }
    }
}

pub struct DynamicResult(Option<Dynamic>);

impl DynamicResult {
    pub fn or_else_empty_map(self) -> Dynamic {
        self.0.unwrap_or_else(Dynamic::empty_map)
    }

    pub fn result(self) -> Option<Dynamic> {
        self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

pub struct LevelLegacyWorldGenSettingsFix {
    pub schema: Schema,
}

impl LevelLegacyWorldGenSettingsFix {
    // JAVA: net.minecraft.util.datafix.fixes.LevelLegacyWorldGenSettingsFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.LevelLegacyWorldGenSettingsFix#makeRule
    pub fn make_rule(&self) -> impl Fn(Dynamic) -> Dynamic + '_ {
        move |data| {
            let mut level = data;
            let mut world_gen_settings = level.get(WORLD_GEN_SETTINGS).or_else_empty_map();
            for &key in OLD_SETTINGS_KEYS {
                if let Some(val) = level.get(key).result() {
                    level = level.remove(key);
                    world_gen_settings = world_gen_settings.set(key, val);
                }
            }
            level.set(WORLD_GEN_SETTINGS, world_gen_settings)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        Dynamic::Map(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    fn get_map<'a>(d: &'a Dynamic, key: &str) -> Option<&'a BTreeMap<String, Dynamic>> {
        match d {
            Dynamic::Map(m) => match m.get(key) {
                Some(Dynamic::Map(inner)) => Some(inner),
                _ => None,
            },
            _ => None,
        }
    }

    #[test]
    fn moves_old_keys_into_world_gen_settings() {
        let fix = LevelLegacyWorldGenSettingsFix::new(Schema);
        let input = map(vec![
            ("RandomSeed", Dynamic::Long(42)),
            ("generatorName", Dynamic::string("flat")),
            ("other", Dynamic::Int(7)),
        ]);
        let out = fix.make_rule()(input);
        let wgs = get_map(&out, WORLD_GEN_SETTINGS).unwrap();
        assert_eq!(wgs.get("RandomSeed"), Some(&Dynamic::Long(42)));
        assert_eq!(wgs.get("generatorName"), Some(&Dynamic::string("flat")));
        if let Dynamic::Map(root) = &out {
            assert!(!root.contains_key("RandomSeed"));
            assert!(!root.contains_key("generatorName"));
            assert_eq!(root.get("other"), Some(&Dynamic::Int(7)));
        } else {
            panic!("not a map");
        }
    }

    #[test]
    fn preserves_existing_world_gen_settings() {
        let fix = LevelLegacyWorldGenSettingsFix::new(Schema);
        let input = map(vec![
            (
                "WorldGenSettings",
                map(vec![("existing", Dynamic::Bool(true))]),
            ),
            ("BonusChest", Dynamic::Bool(false)),
        ]);
        let out = fix.make_rule()(input);
        let wgs = get_map(&out, WORLD_GEN_SETTINGS).unwrap();
        assert_eq!(wgs.get("existing"), Some(&Dynamic::Bool(true)));
        assert_eq!(wgs.get("BonusChest"), Some(&Dynamic::Bool(false)));
    }

    #[test]
    fn creates_world_gen_settings_when_missing() {
        let fix = LevelLegacyWorldGenSettingsFix::new(Schema);
        let input = map(vec![("level", Dynamic::Int(1))]);
        let out = fix.make_rule()(input);
        let wgs = get_map(&out, WORLD_GEN_SETTINGS).unwrap();
        assert!(wgs.is_empty());
    }

    #[test]
    fn moves_all_old_keys() {
        let fix = LevelLegacyWorldGenSettingsFix::new(Schema);
        let mut entries = Vec::new();
        for (i, k) in OLD_SETTINGS_KEYS.iter().enumerate() {
            entries.push((k.to_string(), Dynamic::Int(i as i32)));
        }
        let input = Dynamic::Map(entries.into_iter().collect());
        let out = fix.make_rule()(input);
        let wgs = get_map(&out, WORLD_GEN_SETTINGS).unwrap();
        assert_eq!(wgs.len(), OLD_SETTINGS_KEYS.len());
        if let Dynamic::Map(root) = &out {
            assert_eq!(root.len(), 1);
            assert!(root.contains_key(WORLD_GEN_SETTINGS));
        } else {
            panic!("not a map");
        }
    }

    #[test]
    fn root_key_overwrites_existing_wgs_key() {
        let fix = LevelLegacyWorldGenSettingsFix::new(Schema);
        let input = map(vec![
            (
                "WorldGenSettings",
                map(vec![("RandomSeed", Dynamic::Long(1))]),
            ),
            ("RandomSeed", Dynamic::Long(2)),
        ]);
        let out = fix.make_rule()(input);
        let wgs = get_map(&out, WORLD_GEN_SETTINGS).unwrap();
        assert_eq!(wgs.get("RandomSeed"), Some(&Dynamic::Long(2)));
    }
}
