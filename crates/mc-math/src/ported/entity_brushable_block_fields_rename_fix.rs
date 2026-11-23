use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Long(i64),
    Int(i32),
    Float(f32),
    Double(f64),
    Bool(bool),
    List(Vec<Dynamic>),
    Null,
}

impl Dynamic {
    fn rename_field(self, old_name: &str, new_name: &str) -> Self {
        match self {
            Dynamic::Map(mut map) => {
                if let Some(value) = map.remove(old_name) {
                    map.insert(new_name.to_string(), value);
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: Dynamic,
}

impl Typed {
    fn update<F>(self, f: F) -> Self
    where
        F: FnOnce(Dynamic) -> Dynamic,
    {
        Typed {
            value: f(self.value),
        }
    }
}

pub struct EntityBrushableBlockFieldsRenameFix;

impl EntityBrushableBlockFieldsRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityBrushableBlockFieldsRenameFix#<init>
    pub fn new(_schema: ()) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityBrushableBlockFieldsRenameFix#fixTag
    pub fn fix_tag(&self, tag: Dynamic) -> Dynamic {
        tag.rename_field("loot_table", "LootTable")
            .rename_field("loot_table_seed", "LootTableSeed")
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityBrushableBlockFieldsRenameFix#fix
    pub fn fix(&self, typed: Typed) -> Typed {
        typed.update(|d| self.fix_tag(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: Vec<(&str, Dynamic)>) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in entries {
            map.insert(k.to_string(), v);
        }
        Dynamic::Map(map)
    }

    #[test]
    fn renames_both_fields() {
        let fix = EntityBrushableBlockFieldsRenameFix::new(());
        let input = compound(vec![
            (
                "loot_table",
                Dynamic::String("minecraft:chests/simple".into()),
            ),
            ("loot_table_seed", Dynamic::Long(42)),
        ]);
        let out = fix.fix_tag(input);
        let expected = compound(vec![
            (
                "LootTable",
                Dynamic::String("minecraft:chests/simple".into()),
            ),
            ("LootTableSeed", Dynamic::Long(42)),
        ]);
        assert_eq!(out, expected);
    }

    #[test]
    fn preserves_negative_and_overflow_values() {
        let fix = EntityBrushableBlockFieldsRenameFix::new(());
        let input = compound(vec![
            ("loot_table", Dynamic::Int(i32::MIN)),
            ("loot_table_seed", Dynamic::Long(i64::MIN)),
            ("other", Dynamic::Int(-1)),
        ]);
        let out = fix.fix_tag(input);
        match out {
            Dynamic::Map(map) => {
                assert_eq!(map.get("LootTable"), Some(&Dynamic::Int(i32::MIN)));
                assert_eq!(map.get("LootTableSeed"), Some(&Dynamic::Long(i64::MIN)));
                assert_eq!(map.get("other"), Some(&Dynamic::Int(-1)));
                assert!(!map.contains_key("loot_table"));
                assert!(!map.contains_key("loot_table_seed"));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn preserves_float_bits() {
        let fix = EntityBrushableBlockFieldsRenameFix::new(());
        let nan_bits = f64::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let input = compound(vec![
            (
                "loot_table_seed",
                Dynamic::Double(f64::from_bits(nan_bits)),
            ),
            ("keep_f32", Dynamic::Float(f32::from_bits(neg_zero_bits))),
        ]);
        let out = fix.fix_tag(input);
        match out {
            Dynamic::Map(map) => match map.get("LootTableSeed") {
                Some(Dynamic::Double(v)) => assert_eq!(v.to_bits(), nan_bits),
                other => panic!("unexpected LootTableSeed: {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
        match fix.fix_tag(compound(vec![(
            "keep_f32",
            Dynamic::Float(f32::from_bits(neg_zero_bits)),
        )])) {
            Dynamic::Map(map) => match map.get("keep_f32") {
                Some(Dynamic::Float(v)) => assert_eq!(v.to_bits(), neg_zero_bits),
                other => panic!("unexpected keep_f32: {:?}", other),
            },
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn missing_fields_noop() {
        let fix = EntityBrushableBlockFieldsRenameFix::new(());
        let input = compound(vec![("id", Dynamic::String("x".into()))]);
        let out = fix.fix_tag(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn fix_updates_typed_value() {
        let fix = EntityBrushableBlockFieldsRenameFix::new(());
        let typed = Typed {
            value: compound(vec![("loot_table", Dynamic::String("a".into()))]),
        };
        let out = fix.fix(typed);
        match out.value {
            Dynamic::Map(map) => {
                assert_eq!(map.get("LootTable"), Some(&Dynamic::String("a".into())));
                assert!(!map.contains_key("loot_table"));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }
}
