use std::collections::HashMap;

/// Faithful port of `net.minecraft.util.datafix.fixes.RenameEnchantmentsFix`.
///
/// This module provides a self-contained, DataFixerUpper-free representation of
/// the rename logic. Dynamic/tag structures are modeled as simple Rust enums so
/// the module compiles independently in `mc-math`.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    String(String),
    List(Vec<DynamicValue>),
    Compound(Vec<(String, DynamicValue)>),
    Other,
}

impl DynamicValue {
    // JAVA: com.mojang.serialization.Dynamic#asString
    fn as_string(&self) -> Option<&str> {
        match self {
            DynamicValue::String(s) => Some(s),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#asStreamOpt
    fn as_stream_opt(&self) -> Option<&[DynamicValue]> {
        match self {
            DynamicValue::List(items) => Some(items),
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#createString
    fn create_string(value: String) -> DynamicValue {
        DynamicValue::String(value)
    }

    // JAVA: com.mojang.serialization.Dynamic#createList
    fn create_list(items: Vec<DynamicValue>) -> DynamicValue {
        DynamicValue::List(items)
    }

    // JAVA: com.mojang.serialization.Dynamic#get
    fn get(&self, key: &str) -> Option<&DynamicValue> {
        match self {
            DynamicValue::Compound(entries) => {
                entries.iter().find_map(|(k, v)| if k == key { Some(v) } else { None })
            }
            _ => None,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#update
    fn update<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(DynamicValue) -> DynamicValue,
    {
        match self {
            DynamicValue::Compound(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    let old = std::mem::replace(v, DynamicValue::Other);
                    *v = f(old);
                }
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }

    // JAVA: com.mojang.serialization.Dynamic#update (field-level helper for nested compound updates)
    fn update_field<F>(self, key: &str, f: F) -> Self
    where
        F: FnOnce(DynamicValue) -> DynamicValue,
    {
        match self {
            DynamicValue::Compound(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    let old = std::mem::replace(v, DynamicValue::Other);
                    *v = f(old);
                }
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
fn ensure_namespaced(value: &str) -> String {
    if value.contains(':') {
        value.to_string()
    } else {
        format!("minecraft:{value}")
    }
}

pub struct RenameEnchantmentsFix {
    pub name: String,
    pub renames: HashMap<String, String>,
}

impl RenameEnchantmentsFix {
    // JAVA: net.minecraft.util.datafix.fixes.RenameEnchantmentsFix#<init>
    pub fn new(name: String, renames: HashMap<String, String>) -> Self {
        Self { name, renames }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RenameEnchantmentsFix#fixTag
    fn fix_tag(&self, tag: DynamicValue) -> DynamicValue {
        let tag = self.fix_enchantment_list(tag, "Enchantments");
        self.fix_enchantment_list(tag, "StoredEnchantments")
    }

    // JAVA: net.minecraft.util.datafix.fixes.RenameEnchantmentsFix#fixEnchantmentList
    fn fix_enchantment_list(&self, tag: DynamicValue, key: &str) -> DynamicValue {
        let original = tag.clone();
        let Some(list_value) = tag.get(key).cloned() else {
            return original;
        };

        let Some(stream) = list_value.as_stream_opt() else {
            return original;
        };

        let mapped: Vec<DynamicValue> = stream
            .iter()
            .cloned()
            .map(|entry| {
                entry.update_field("id", |id_dyn| {
                    let Some(id_str) = id_dyn.as_string() else {
                        return id_dyn;
                    };
                    let namespaced = ensure_namespaced(id_str);
                    let renamed = self
                        .renames
                        .get(&namespaced)
                        .cloned()
                        .unwrap_or_else(|| id_str.to_string());
                    DynamicValue::create_string(renamed)
                })
            })
            .collect();

        let new_list = DynamicValue::create_list(mapped);
        original.update(key, |_| new_list)
    }

    /// Applies the fix to an item-stack-like dynamic value by updating its `tag` field.
    /// This mirrors the Java rule's `updateTyped(tag, fixTag)` behavior in a self-contained form.
    // JAVA: net.minecraft.util.datafix.fixes.RenameEnchantmentsFix#makeRule (behavioral equivalent)
    pub fn apply_to_item_stack(&self, stack: DynamicValue) -> DynamicValue {
        match stack {
            DynamicValue::Compound(mut entries) => {
                if let Some((_, tag_val)) = entries.iter_mut().find(|(k, _)| k == "tag") {
                    let old = std::mem::replace(tag_val, DynamicValue::Other);
                    *tag_val = self.fix_tag(old);
                }
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        DynamicValue::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn string(s: &str) -> DynamicValue {
        DynamicValue::String(s.to_string())
    }

    fn enchantment(id: &str) -> DynamicValue {
        compound(vec![("id", string(id))])
    }

    #[test]
    fn renames_enchantments_and_stored_enchantments() {
        let mut renames = HashMap::new();
        renames.insert("minecraft:sharpness".to_string(), "minecraft:power".to_string());
        renames.insert("minecraft:loot_bonus_mobs".to_string(), "minecraft:looting".to_string());
        let fix = RenameEnchantmentsFix::new("RenameEnchantmentsFix".to_string(), renames);

        let stack = compound(vec![(
            "tag",
            compound(vec![
                (
                    "Enchantments",
                    DynamicValue::List(vec![enchantment("minecraft:sharpness"), enchantment("minecraft:unbreaking")]),
                ),
                (
                    "StoredEnchantments",
                    DynamicValue::List(vec![enchantment("loot_bonus_mobs")]),
                ),
            ]),
        )]);

        let fixed = fix.apply_to_item_stack(stack);
        let tag = fixed.get("tag").unwrap();
        let ench = tag.get("Enchantments").unwrap().as_stream_opt().unwrap();
        let stored = tag.get("StoredEnchantments").unwrap().as_stream_opt().unwrap();

        assert_eq!(ench[0].get("id").unwrap().as_string(), Some("minecraft:power"));
        assert_eq!(ench[1].get("id").unwrap().as_string(), Some("minecraft:unbreaking"));
        assert_eq!(stored[0].get("id").unwrap().as_string(), Some("minecraft:looting"));
    }

    #[test]
    fn ensures_namespaced_before_lookup() {
        let mut renames = HashMap::new();
        renames.insert("minecraft:protection".to_string(), "minecraft:defense".to_string());
        let fix = RenameEnchantmentsFix::new("test".to_string(), renames);

        let tag = compound(vec![(
            "Enchantments",
            DynamicValue::List(vec![enchantment("protection")]),
        )]);
        let fixed = fix.fix_tag(tag);
        let ench = fixed.get("Enchantments").unwrap().as_stream_opt().unwrap();
        assert_eq!(ench[0].get("id").unwrap().as_string(), Some("minecraft:defense"));
    }

    #[test]
    fn missing_or_invalid_lists_are_preserved() {
        let fix = RenameEnchantmentsFix::new("test".to_string(), HashMap::new());

        let no_lists = compound(vec![("Damage", DynamicValue::Other)]);
        assert_eq!(fix.fix_tag(no_lists.clone()), no_lists);

        let bad_list = compound(vec![("Enchantments", string("not-a-list"))]);
        assert_eq!(fix.fix_tag(bad_list.clone()), bad_list);
    }

    #[test]
    fn non_string_id_is_left_unchanged() {
        let mut renames = HashMap::new();
        renames.insert("minecraft:sharpness".to_string(), "minecraft:power".to_string());
        let fix = RenameEnchantmentsFix::new("test".to_string(), renames);

        let tag = compound(vec![(
            "Enchantments",
            DynamicValue::List(vec![compound(vec![("id", DynamicValue::Other)])]),
        )]);
        let fixed = fix.fix_tag(tag.clone());
        assert_eq!(fixed, tag);
    }

    #[test]
    fn apply_to_item_stack_without_tag_is_noop() {
        let fix = RenameEnchantmentsFix::new("test".to_string(), HashMap::new());
        let stack = compound(vec![("id", string("minecraft:diamond_sword"))]);
        assert_eq!(fix.apply_to_item_stack(stack.clone()), stack);
    }
}
