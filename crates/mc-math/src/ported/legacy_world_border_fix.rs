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
    pub fn make_map() -> Self {
        Self {
            value: Some(DynamicValue::Map(BTreeMap::new())),
        }
    }

    pub fn int_val(v: i32) -> Self {
        Self {
            value: Some(DynamicValue::Int(v)),
        }
    }

    pub fn long_val(v: i64) -> Self {
        Self {
            value: Some(DynamicValue::Long(v)),
        }
    }

    pub fn double_val(v: f64) -> Self {
        Self {
            value: Some(DynamicValue::Double(v)),
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

    pub fn as_double(&self, default: f64) -> f64 {
        match &self.value {
            Some(DynamicValue::Double(v)) => *v,
            Some(DynamicValue::Int(v)) => *v as f64,
            Some(DynamicValue::Long(v)) => *v as f64,
            _ => default,
        }
    }

    pub fn as_long(&self, default: i64) -> i64 {
        match &self.value {
            Some(DynamicValue::Long(v)) => *v,
            Some(DynamicValue::Int(v)) => *v as i64,
            _ => default,
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            Some(DynamicValue::Int(v)) => *v,
            Some(DynamicValue::Long(v)) => *v as i32,
            _ => default,
        }
    }

    pub fn create_double(&self, v: f64) -> Dynamic {
        Self::double_val(v)
    }

    pub fn create_long(&self, v: i64) -> Dynamic {
        Self::long_val(v)
    }

    pub fn create_int(&self, v: i32) -> Dynamic {
        Self::int_val(v)
    }

    pub fn empty_map(&self) -> Dynamic {
        Self::make_map()
    }
}

pub struct LegacyWorldBorderFix {
    _schema: Schema,
}

impl LegacyWorldBorderFix {
    // JAVA: net.minecraft.util.datafix.fixes.LegacyWorldBorderFix#<init>
    pub fn new(schema: Schema) -> Self {
        Self { _schema: schema }
    }

    // JAVA: net.minecraft.util.datafix.fixes.LegacyWorldBorderFix#makeRule
    pub fn fix_level(&self, data: Dynamic) -> Dynamic {
        let border = data
            .empty_map()
            .set(
                "center_x",
                data.create_double(data.get("BorderCenterX").as_double(0.0)),
            )
            .set(
                "center_z",
                data.create_double(data.get("BorderCenterZ").as_double(0.0)),
            )
            .set(
                "size",
                data.create_double(
                    data.get("BorderSize")
                        .as_double(5.999997e7_f32 as f64),
                ),
            )
            .set(
                "lerp_time",
                data.create_long(data.get("BorderSizeLerpTime").as_long(0_i64)),
            )
            .set(
                "lerp_target",
                data.create_double(data.get("BorderSizeLerpTarget").as_double(0.0)),
            )
            .set(
                "safe_zone",
                data.create_double(data.get("BorderSafeZone").as_double(5.0)),
            )
            .set(
                "damage_per_block",
                data.create_double(data.get("BorderDamagePerBlock").as_double(0.2)),
            )
            .set(
                "warning_blocks",
                data.create_int(data.get("BorderWarningBlocks").as_int(5)),
            )
            .set(
                "warning_time",
                data.create_int(data.get("BorderWarningTime").as_int(15)),
            );

        let data = data
            .remove("BorderCenterX")
            .remove("BorderCenterZ")
            .remove("BorderSize")
            .remove("BorderSizeLerpTime")
            .remove("BorderSizeLerpTarget")
            .remove("BorderSafeZone")
            .remove("BorderDamagePerBlock")
            .remove("BorderWarningBlocks")
            .remove("BorderWarningTime");

        data.set("world_border", border)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_missing() {
        let fix = LegacyWorldBorderFix::new(Schema);
        let out = fix.fix_level(Dynamic::make_map());
        let wb = out.get("world_border").result().unwrap();
        assert_eq!(wb.get("center_x").as_double(-1.0), 0.0);
        assert_eq!(wb.get("center_z").as_double(-1.0), 0.0);
        assert_eq!(wb.get("size").as_double(0.0), 5.999997e7_f32 as f64);
        assert_eq!(wb.get("lerp_time").as_long(-1), 0_i64);
        assert_eq!(wb.get("lerp_target").as_double(-1.0), 0.0);
        assert_eq!(wb.get("safe_zone").as_double(-1.0), 5.0);
        assert_eq!(wb.get("damage_per_block").as_double(-1.0), 0.2);
        assert_eq!(wb.get("warning_blocks").as_int(-1), 5);
        assert_eq!(wb.get("warning_time").as_int(-1), 15);
    }

    #[test]
    fn reads_existing_and_removes_old_keys() {
        let fix = LegacyWorldBorderFix::new(Schema);
        let input = Dynamic::make_map()
            .set("BorderCenterX", Dynamic::double_val(1.5))
            .set("BorderCenterZ", Dynamic::double_val(-2.5))
            .set("BorderSize", Dynamic::double_val(100.0))
            .set("BorderSizeLerpTime", Dynamic::long_val(42))
            .set("BorderSizeLerpTarget", Dynamic::double_val(200.0))
            .set("BorderSafeZone", Dynamic::double_val(3.0))
            .set("BorderDamagePerBlock", Dynamic::double_val(0.5))
            .set("BorderWarningBlocks", Dynamic::int_val(10))
            .set("BorderWarningTime", Dynamic::int_val(20))
            .set("keep", Dynamic::int_val(7));
        let out = fix.fix_level(input);
        let wb = out.get("world_border").result().unwrap();
        assert_eq!(wb.get("center_x").as_double(0.0), 1.5);
        assert_eq!(wb.get("center_z").as_double(0.0), -2.5);
        assert_eq!(wb.get("size").as_double(0.0), 100.0);
        assert_eq!(wb.get("lerp_time").as_long(0), 42);
        assert_eq!(wb.get("warning_blocks").as_int(0), 10);
        assert_eq!(wb.get("warning_time").as_int(0), 20);
        assert!(out.get("BorderCenterX").result().is_none());
        assert!(out.get("BorderSize").result().is_none());
        assert_eq!(out.get("keep").as_int(0), 7);
    }

    #[test]
    fn preserves_negative_and_extreme_integers() {
        let fix = LegacyWorldBorderFix::new(Schema);
        let input = Dynamic::make_map()
            .set("BorderSizeLerpTime", Dynamic::long_val(i64::MIN))
            .set("BorderWarningBlocks", Dynamic::int_val(i32::MIN))
            .set("BorderWarningTime", Dynamic::int_val(i32::MAX));
        let out = fix.fix_level(input);
        let wb = out.get("world_border").result().unwrap();
        assert_eq!(wb.get("lerp_time").as_long(0), i64::MIN);
        assert_eq!(wb.get("warning_blocks").as_int(0), i32::MIN);
        assert_eq!(wb.get("warning_time").as_int(0), i32::MAX);
    }

    #[test]
    fn preserves_double_bits() {
        let fix = LegacyWorldBorderFix::new(Schema);
        let neg_inf_bits = f64::NEG_INFINITY.to_bits();
        let nan_bits = f64::NAN.to_bits();
        let input = Dynamic::make_map()
            .set(
                "BorderCenterX",
                Dynamic::double_val(f64::from_bits(neg_inf_bits)),
            )
            .set(
                "BorderDamagePerBlock",
                Dynamic::double_val(f64::from_bits(nan_bits)),
            );
        let out = fix.fix_level(input);
        let wb = out.get("world_border").result().unwrap();
        assert_eq!(wb.get("center_x").as_double(0.0).to_bits(), neg_inf_bits);
        assert_eq!(
            wb.get("damage_per_block").as_double(0.0).to_bits(),
            nan_bits
        );
    }
}
