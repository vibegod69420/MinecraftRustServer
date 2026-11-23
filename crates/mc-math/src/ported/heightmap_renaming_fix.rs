use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct TypeRewriteRule;

#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Double(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    value: Option<DynamicValue>,
}

impl Dynamic {
    pub fn empty_map() -> Self {
        Self {
            value: Some(DynamicValue::Map(BTreeMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Self {
        match &self.value {
            Some(DynamicValue::Map(m)) => m.get(key).cloned().unwrap_or(Self { value: None }),
            _ => Self { value: None },
        }
    }

    pub fn result(&self) -> Option<Self> {
        if self.value.is_some() {
            Some(self.clone())
        } else {
            None
        }
    }

    pub fn set(self, key: &str, val: Dynamic) -> Self {
        match self.value {
            Some(DynamicValue::Map(mut m)) => {
                m.insert(key.to_string(), val);
                Self {
                    value: Some(DynamicValue::Map(m)),
                }
            }
            other => Self { value: other },
        }
    }

    pub fn remove(self, key: &str) -> Self {
        match self.value {
            Some(DynamicValue::Map(mut m)) => {
                m.remove(key);
                Self {
                    value: Some(DynamicValue::Map(m)),
                }
            }
            other => Self { value: other },
        }
    }
}

pub struct HeightmapRenamingFix {
    _schema: Schema,
    _changes_type: bool,
}

impl HeightmapRenamingFix {
    // JAVA: net.minecraft.util.datafix.fixes.HeightmapRenamingFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.HeightmapRenamingFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // DFU type-rewrite wiring stub; core logic is in `fix`.
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.HeightmapRenamingFix#fix
    fn fix(&self, data: Dynamic) -> Dynamic {
        let Some(mut heightmaps) = data.get("Heightmaps").result() else {
            return data;
        };

        if let Some(liquid) = heightmaps.get("LIQUID").result() {
            heightmaps = heightmaps.remove("LIQUID");
            heightmaps = heightmaps.set("WORLD_SURFACE_WG", liquid);
        }

        if let Some(solid) = heightmaps.get("SOLID").result() {
            heightmaps = heightmaps.remove("SOLID");
            heightmaps = heightmaps.set("OCEAN_FLOOR_WG", solid.clone());
            heightmaps = heightmaps.set("OCEAN_FLOOR", solid);
        }

        if let Some(light) = heightmaps.get("LIGHT").result() {
            heightmaps = heightmaps.remove("LIGHT");
            heightmaps = heightmaps.set("LIGHT_BLOCKING", light);
        }

        if let Some(rain) = heightmaps.get("RAIN").result() {
            heightmaps = heightmaps.remove("RAIN");
            heightmaps = heightmaps.set("MOTION_BLOCKING", rain.clone());
            heightmaps = heightmaps.set("MOTION_BLOCKING_NO_LEAVES", rain);
        }

        data.set("Heightmaps", heightmaps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn map(entries: &[(&str, Dynamic)]) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Dynamic {
            value: Some(DynamicValue::Map(m)),
        }
    }

    fn s(v: &str) -> Dynamic {
        Dynamic {
            value: Some(DynamicValue::String(v.to_string())),
        }
    }

    fn i(v: i32) -> Dynamic {
        Dynamic {
            value: Some(DynamicValue::Int(v)),
        }
    }

    fn d(v: f64) -> Dynamic {
        Dynamic {
            value: Some(DynamicValue::Double(v)),
        }
    }

    #[test]
    fn missing_heightmaps_unchanged() {
        let fix = HeightmapRenamingFix::new(Schema, true);
        let input = map(&[("x", s("y"))]);
        assert_eq!(fix.fix(input.clone()), input);
    }

    #[test]
    fn renames_all_legacy_keys() {
        let fix = HeightmapRenamingFix::new(Schema, true);
        let input = map(&[(
            "Heightmaps",
            map(&[
                ("LIQUID", s("l")),
                ("SOLID", s("s")),
                ("LIGHT", s("b")),
                ("RAIN", s("r")),
            ]),
        )]);
        let out = fix.fix(input);
        let hm = out.get("Heightmaps").result().unwrap();
        assert!(hm.get("LIQUID").result().is_none());
        assert!(hm.get("SOLID").result().is_none());
        assert!(hm.get("LIGHT").result().is_none());
        assert!(hm.get("RAIN").result().is_none());
        assert_eq!(hm.get("WORLD_SURFACE_WG").result().unwrap(), s("l"));
        assert_eq!(hm.get("OCEAN_FLOOR_WG").result().unwrap(), s("s"));
        assert_eq!(hm.get("OCEAN_FLOOR").result().unwrap(), s("s"));
        assert_eq!(hm.get("LIGHT_BLOCKING").result().unwrap(), s("b"));
        assert_eq!(hm.get("MOTION_BLOCKING").result().unwrap(), s("r"));
        assert_eq!(
            hm.get("MOTION_BLOCKING_NO_LEAVES").result().unwrap(),
            s("r")
        );
    }

    #[test]
    fn partial_rename_preserves_custom_keys() {
        let fix = HeightmapRenamingFix::new(Schema, false);
        let input = map(&[(
            "Heightmaps",
            map(&[("LIQUID", s("l")), ("CUSTOM", s("c"))]),
        )]);
        let hm = fix.fix(input).get("Heightmaps").result().unwrap();
        assert_eq!(hm.get("WORLD_SURFACE_WG").result().unwrap(), s("l"));
        assert_eq!(hm.get("CUSTOM").result().unwrap(), s("c"));
        assert!(hm.get("OCEAN_FLOOR").result().is_none());
    }

    #[test]
    fn preserves_unrelated_fields_and_extreme_values() {
        let fix = HeightmapRenamingFix::new(Schema, true);
        let bits = f64::NAN.to_bits();
        let input = map(&[
            ("Heightmaps", map(&[("LIGHT", d(f64::from_bits(bits)))])),
            ("xPos", i(i32::MIN)),
            ("negLong", Dynamic { value: Some(DynamicValue::Long(i64::MIN)) }),
        ]);
        let out = fix.fix(input);
        assert_eq!(out.get("xPos").result().unwrap(), i(i32::MIN));
        assert_eq!(
            out.get("negLong").result().unwrap(),
            Dynamic { value: Some(DynamicValue::Long(i64::MIN)) }
        );
        let hm = out.get("Heightmaps").result().unwrap();
        let v = hm.get("LIGHT_BLOCKING").result().unwrap();
        match &v.value {
            Some(DynamicValue::Double(x)) => assert_eq!(x.to_bits(), bits),
            other => panic!("expected double, got {:?}", other),
        }
    }
}
