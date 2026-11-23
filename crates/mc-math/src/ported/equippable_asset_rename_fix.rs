use std::collections::BTreeMap;

/// Minimal self-contained dynamic value for datafix porting.
#[derive(Debug, Clone, PartialEq)]
pub enum Dynamic {
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
    Long(i64),
    Bool(bool),
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

pub struct EquippableAssetRenameFix;

impl EquippableAssetRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.EquippableAssetRenameFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EquippableAssetRenameFix#makeRule
    pub fn fix_data_components(&self, components: Dynamic) -> Dynamic {
        match components {
            Dynamic::Map(mut map) => {
                if let Some(equippable) = map.remove("minecraft:equippable") {
                    let fixed = equippable.rename_field("model", "asset_id");
                    map.insert("minecraft:equippable".to_string(), fixed);
                }
                Dynamic::Map(map)
            }
            other => other,
        }
    }
}

impl Default for EquippableAssetRenameFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(pairs: &[(&str, Dynamic)]) -> Dynamic {
        let mut map = BTreeMap::new();
        for (k, v) in pairs {
            map.insert((*k).to_string(), v.clone());
        }
        Dynamic::Map(map)
    }

    #[test]
    fn renames_model_to_asset_id() {
        let fix = EquippableAssetRenameFix::new();
        let input = obj(&[(
            "minecraft:equippable",
            obj(&[("model", Dynamic::String("minecraft:iron".to_string()))]),
        )]);
        let out = fix.fix_data_components(input);
        let Dynamic::Map(root) = out else { panic!("expected map") };
        let Some(Dynamic::Map(eq)) = root.get("minecraft:equippable") else {
            panic!("missing equippable")
        };
        assert!(!eq.contains_key("model"));
        assert_eq!(
            eq.get("asset_id"),
            Some(&Dynamic::String("minecraft:iron".to_string()))
        );
    }

    #[test]
    fn no_equippable_unchanged() {
        let fix = EquippableAssetRenameFix::new();
        let input = obj(&[("minecraft:damage", Dynamic::Int(5))]);
        let out = fix.fix_data_components(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn equippable_without_model_unchanged() {
        let fix = EquippableAssetRenameFix::new();
        let input = obj(&[(
            "minecraft:equippable",
            obj(&[("slot", Dynamic::String("head".to_string()))]),
        )]);
        let out = fix.fix_data_components(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn preserves_other_fields_and_components() {
        let fix = EquippableAssetRenameFix::new();
        let input = obj(&[
            (
                "minecraft:equippable",
                obj(&[
                    ("model", Dynamic::String("a".to_string())),
                    ("slot", Dynamic::String("chest".to_string())),
                ]),
            ),
            ("minecraft:damage", Dynamic::Int(3)),
        ]);
        let out = fix.fix_data_components(input);
        let Dynamic::Map(root) = out else { panic!("expected map") };
        let Some(Dynamic::Map(eq)) = root.get("minecraft:equippable") else {
            panic!("missing equippable")
        };
        assert_eq!(eq.get("asset_id"), Some(&Dynamic::String("a".to_string())));
        assert_eq!(
            eq.get("slot"),
            Some(&Dynamic::String("chest".to_string()))
        );
        assert_eq!(root.get("minecraft:damage"), Some(&Dynamic::Int(3)));
    }

    #[test]
    fn non_map_input_passthrough() {
        let fix = EquippableAssetRenameFix::new();
        let input = Dynamic::String("not-a-map".to_string());
        let out = fix.fix_data_components(input.clone());
        assert_eq!(out, input);
    }
}
