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

#[derive(Clone, Debug)]
pub struct StringDataResult(Option<String>);

impl StringDataResult {
    pub fn result(&self) -> Option<String> {
        self.0.clone()
    }
}

impl Dynamic {
    pub fn make_map(entries: &[(&str, Dynamic)]) -> Self {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        Self {
            value: Some(DynamicValue::Map(m)),
        }
    }

    pub fn string_val(s: &str) -> Self {
        Self {
            value: Some(DynamicValue::String(s.to_string())),
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

    pub fn update<F: FnOnce(Dynamic) -> Dynamic>(self, key: &str, f: F) -> Self {
        let current = self.get(key);
        let updated = f(current);
        self.set(key, updated)
    }

    pub fn as_string(&self) -> StringDataResult {
        match &self.value {
            Some(DynamicValue::String(s)) => StringDataResult(Some(s.clone())),
            _ => StringDataResult(None),
        }
    }

    pub fn create_string(&self, s: &str) -> Self {
        Self::string_val(s)
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.value {
            Some(DynamicValue::Int(v)) => *v,
            Some(DynamicValue::Long(v)) => *v as i32,
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

    pub fn as_double(&self, default: f64) -> f64 {
        match &self.value {
            Some(DynamicValue::Double(v)) => *v,
            Some(DynamicValue::Int(v)) => *v as f64,
            Some(DynamicValue::Long(v)) => *v as f64,
            _ => default,
        }
    }
}

const WALL_BLOCKS: &[&str] = &[
    "minecraft:andesite_wall",
    "minecraft:brick_wall",
    "minecraft:cobblestone_wall",
    "minecraft:diorite_wall",
    "minecraft:end_stone_brick_wall",
    "minecraft:granite_wall",
    "minecraft:mossy_cobblestone_wall",
    "minecraft:mossy_stone_brick_wall",
    "minecraft:nether_brick_wall",
    "minecraft:prismarine_wall",
    "minecraft:red_nether_brick_wall",
    "minecraft:red_sandstone_wall",
    "minecraft:sandstone_wall",
    "minecraft:stone_brick_wall",
];

pub struct WallPropertyFix {
    _schema: Schema,
    _changes_type: bool,
}

impl WallPropertyFix {
    // JAVA: net.minecraft.util.datafix.fixes.WallPropertyFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            _schema: schema,
            _changes_type: changes_type,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.WallPropertyFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        // DFU type-rewrite wiring stub; core logic is in `upgrade_block_state_tag`.
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.WallPropertyFix#mapProperty
    fn map_property(value: &str) -> String {
        if "true" == value {
            "low".to_string()
        } else {
            "none".to_string()
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.WallPropertyFix#fixWallProperty
    fn fix_wall_property(properties: Dynamic, key: &str) -> Dynamic {
        properties.update(key, |v| {
            let mapped = v.as_string().result().map(|s| Self::map_property(&s));
            match mapped {
                Some(s) => v.create_string(&s),
                None => v,
            }
        })
    }

    // JAVA: net.minecraft.util.datafix.fixes.WallPropertyFix#upgradeBlockStateTag
    fn upgrade_block_state_tag(data: Dynamic) -> Dynamic {
        let is_wall = data
            .get("Name")
            .as_string()
            .result()
            .is_some_and(|n| WALL_BLOCKS.contains(&n.as_str()));
        if !is_wall {
            return data;
        }
        data.update("Properties", |props| {
            let props = Self::fix_wall_property(props, "east");
            let props = Self::fix_wall_property(props, "west");
            let props = Self::fix_wall_property(props, "north");
            Self::fix_wall_property(props, "south")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_property_values() {
        assert_eq!(WallPropertyFix::map_property("true"), "low");
        assert_eq!(WallPropertyFix::map_property("false"), "none");
        assert_eq!(WallPropertyFix::map_property("low"), "none");
    }

    #[test]
    fn non_wall_unchanged() {
        let input = Dynamic::make_map(&[
            ("Name", Dynamic::string_val("minecraft:stone")),
            (
                "Properties",
                Dynamic::make_map(&[("east", Dynamic::string_val("true"))]),
            ),
        ]);
        let out = WallPropertyFix::upgrade_block_state_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn wall_upgrades_directions() {
        let input = Dynamic::make_map(&[
            ("Name", Dynamic::string_val("minecraft:cobblestone_wall")),
            (
                "Properties",
                Dynamic::make_map(&[
                    ("east", Dynamic::string_val("true")),
                    ("west", Dynamic::string_val("false")),
                    ("north", Dynamic::string_val("low")),
                    ("south", Dynamic::string_val("true")),
                ]),
            ),
        ]);
        let out = WallPropertyFix::upgrade_block_state_tag(input);
        let props = out.get("Properties").result().unwrap();
        assert_eq!(
            props.get("east").as_string().result().as_deref(),
            Some("low")
        );
        assert_eq!(
            props.get("west").as_string().result().as_deref(),
            Some("none")
        );
        assert_eq!(
            props.get("north").as_string().result().as_deref(),
            Some("none")
        );
        assert_eq!(
            props.get("south").as_string().result().as_deref(),
            Some("low")
        );
    }

    #[test]
    fn preserves_non_string_and_extreme_values() {
        let bits = f64::NAN.to_bits();
        let input = Dynamic::make_map(&[
            ("Name", Dynamic::string_val("minecraft:brick_wall")),
            (
                "Properties",
                Dynamic::make_map(&[
                    ("east", Dynamic::int_val(i32::MIN)),
                    ("west", Dynamic::string_val("true")),
                    ("north", Dynamic::string_val("false")),
                    ("south", Dynamic::string_val("false")),
                ]),
            ),
            ("neg", Dynamic::long_val(i64::MIN)),
            ("nan_bits", Dynamic::double_val(f64::from_bits(bits))),
        ]);
        let out = WallPropertyFix::upgrade_block_state_tag(input);
        let props = out.get("Properties").result().unwrap();
        assert_eq!(props.get("east").as_int(0), i32::MIN);
        assert_eq!(
            props.get("west").as_string().result().as_deref(),
            Some("low")
        );
        assert_eq!(out.get("neg").as_long(0), i64::MIN);
        assert_eq!(out.get("nan_bits").as_double(0.0).to_bits(), bits);
    }
}
