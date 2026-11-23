use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Int(i32),
    Long(i64),
    Short(i16),
    Byte(i8),
    Float(f32),
    Double(f64),
    String(String),
    Bool(bool),
    Map(BTreeMap<String, DynValue>),
    List(Vec<DynValue>),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    // JAVA: com.mojang.serialization.Dynamic#new
    pub fn new(value: DynValue) -> Self {
        Self { value }
    }

    // JAVA: com.mojang.serialization.Dynamic#get
    pub fn get(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => Dynamic::new(m.get(key).cloned().unwrap_or(DynValue::Null)),
            _ => Dynamic::new(DynValue::Null),
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asString
    pub fn as_string_result(&self) -> Option<String> {
        match &self.value {
            DynValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NbtFormatException {
    pub message: String,
}

impl std::fmt::Display for NbtFormatException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NbtFormatException {}

pub struct WorldGenSettingsDisallowOldCustomWorldsFix;

impl WorldGenSettingsDisallowOldCustomWorldsFix {
    // JAVA: net.minecraft.util.datafix.fixes.WorldGenSettingsDisallowOldCustomWorldsFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.WorldGenSettingsDisallowOldCustomWorldsFix#makeRule
    pub fn make_rule(&self) -> Box<dyn Fn(Dynamic) -> Result<Dynamic, NbtFormatException>> {
        Box::new(check_world_gen_settings)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.WorldGenSettingsDisallowOldCustomWorldsFix#makeRule (dimensions validation)
fn check_world_gen_settings(settings: Dynamic) -> Result<Dynamic, NbtFormatException> {
    let dimensions = settings.get("dimensions");
    if let DynValue::Map(entries) = &dimensions.value {
        for (_key, dim_value) in entries {
            let dim = Dynamic::new(dim_value.clone());
            let type_result = dim.get("type").as_string_result();
            if type_result.is_none() {
                return Err(NbtFormatException {
                    message: "Unable load old custom worlds.".to_string(),
                });
            }
        }
    }
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_gen_settings(dims: &[(&str, &str)]) -> Dynamic {
        let mut dim_map = BTreeMap::new();
        for (name, ty) in dims {
            let mut d = BTreeMap::new();
            d.insert("type".to_string(), DynValue::String((*ty).to_string()));
            dim_map.insert((*name).to_string(), DynValue::Map(d));
        }
        let mut root = BTreeMap::new();
        root.insert("dimensions".to_string(), DynValue::Map(dim_map));
        Dynamic::new(DynValue::Map(root))
    }

    fn world_gen_settings_raw(dims: &[(&str, DynValue)]) -> Dynamic {
        let mut dim_map = BTreeMap::new();
        for (name, val) in dims {
            dim_map.insert((*name).to_string(), val.clone());
        }
        let mut root = BTreeMap::new();
        root.insert("dimensions".to_string(), DynValue::Map(dim_map));
        Dynamic::new(DynValue::Map(root))
    }

    #[test]
    fn valid_dimensions_pass() {
        let fix = WorldGenSettingsDisallowOldCustomWorldsFix::new(());
        let settings = world_gen_settings(&[
            ("minecraft:overworld", "minecraft:overworld"),
            ("minecraft:the_nether", "minecraft:the_nether"),
        ]);
        let rule = fix.make_rule();
        let out = rule(settings.clone()).unwrap();
        assert_eq!(out, settings);
    }

    #[test]
    fn missing_type_throws() {
        let fix = WorldGenSettingsDisallowOldCustomWorldsFix::new(());
        let mut dim = BTreeMap::new();
        dim.insert("generator".to_string(), DynValue::String("noise".to_string()));
        let settings = world_gen_settings_raw(&[("minecraft:overworld", DynValue::Map(dim))]);
        let rule = fix.make_rule();
        let err = rule(settings).unwrap_err();
        assert_eq!(err.message, "Unable load old custom worlds.");
    }

    #[test]
    fn non_string_type_throws() {
        let fix = WorldGenSettingsDisallowOldCustomWorldsFix::new(());
        let mut dim = BTreeMap::new();
        dim.insert("type".to_string(), DynValue::Int(1));
        let settings = world_gen_settings_raw(&[("dim", DynValue::Map(dim))]);
        let rule = fix.make_rule();
        let err = rule(settings).unwrap_err();
        assert_eq!(err.message, "Unable load old custom worlds.");
    }

    #[test]
    fn empty_string_type_is_allowed() {
        let fix = WorldGenSettingsDisallowOldCustomWorldsFix::new(());
        let settings = world_gen_settings(&[("dim", "")]);
        let rule = fix.make_rule();
        assert!(rule(settings).is_ok());
    }

    #[test]
    fn missing_dimensions_field_passes() {
        let fix = WorldGenSettingsDisallowOldCustomWorldsFix::new(());
        let settings = Dynamic::new(DynValue::Map(BTreeMap::new()));
        let rule = fix.make_rule();
        assert!(rule(settings).is_ok());
    }
}
