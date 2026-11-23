/// Faithful port of `net.minecraft.util.datafix.fixes.ItemWaterPotionFix`.
///
/// Self-contained dynamic model for the potion tag fix. The Java version operates
/// on DataFixerUpper typed optics; this Rust version applies the same behavioral
/// transformation to a simple item-stack dynamic structure.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    String(String),
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

    // JAVA: com.mojang.serialization.Dynamic#createString
    fn create_string(value: String) -> DynamicValue {
        DynamicValue::String(value)
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

    // JAVA: com.mojang.serialization.Dynamic#set
    fn set(self, key: &str, value: DynamicValue) -> Self {
        match self {
            DynamicValue::Compound(mut entries) => {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key) {
                    *v = value;
                } else {
                    entries.push((key.to_string(), value));
                }
                DynamicValue::Compound(entries)
            }
            other => other,
        }
    }

    fn get_or_create_compound(&self, key: &str) -> DynamicValue {
        match self.get(key) {
            Some(DynamicValue::Compound(entries)) => DynamicValue::Compound(entries.clone()),
            _ => DynamicValue::Compound(Vec::new()),
        }
    }
}

pub struct ItemWaterPotionFix {
    pub changes_type: bool,
}

impl ItemWaterPotionFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemWaterPotionFix#<init>
    pub fn new(changes_type: bool) -> Self {
        Self { changes_type }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ItemWaterPotionFix#makeRule (behavioral equivalent)
    pub fn apply_to_item_stack(&self, stack: DynamicValue) -> DynamicValue {
        let DynamicValue::Compound(entries) = &stack else {
            return stack;
        };

        let Some(id_val) = entries.iter().find_map(|(k, v)| if k == "id" { Some(v) } else { None }) else {
            return stack;
        };
        let Some(id) = id_val.as_string() else {
            return stack;
        };

        if id != "minecraft:potion"
            && id != "minecraft:splash_potion"
            && id != "minecraft:lingering_potion"
            && id != "minecraft:tipped_arrow"
        {
            return stack;
        }

        let mut tag = stack.get_or_create_compound("tag");
        let has_potion = tag
            .get("Potion")
            .and_then(|v| v.as_string())
            .is_some();
        if !has_potion {
            tag = tag.set("Potion", DynamicValue::create_string("minecraft:water".to_string()));
        }

        stack.set("tag", tag)
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

    #[test]
    fn adds_water_potion_when_missing() {
        let fix = ItemWaterPotionFix::new(false);
        let stack = compound(vec![("id", string("minecraft:potion"))]);
        let fixed = fix.apply_to_item_stack(stack);
        let tag = fixed.get("tag").unwrap();
        assert_eq!(tag.get("Potion").unwrap().as_string(), Some("minecraft:water"));
    }

    #[test]
    fn preserves_existing_potion() {
        let fix = ItemWaterPotionFix::new(false);
        let stack = compound(vec![
            ("id", string("minecraft:splash_potion")),
            ("tag", compound(vec![("Potion", string("minecraft:healing"))])),
        ]);
        let fixed = fix.apply_to_item_stack(stack.clone());
        assert_eq!(fixed, stack);
    }

    #[test]
    fn ignores_non_potion_items() {
        let fix = ItemWaterPotionFix::new(true);
        let stack = compound(vec![("id", string("minecraft:apple"))]);
        assert_eq!(fix.apply_to_item_stack(stack.clone()), stack);
    }

    #[test]
    fn handles_all_supported_potion_ids() {
        let fix = ItemWaterPotionFix::new(false);
        for id in [
            "minecraft:potion",
            "minecraft:splash_potion",
            "minecraft:lingering_potion",
            "minecraft:tipped_arrow",
        ] {
            let stack = compound(vec![("id", string(id))]);
            let fixed = fix.apply_to_item_stack(stack);
            let tag = fixed.get("tag").unwrap();
            assert_eq!(tag.get("Potion").unwrap().as_string(), Some("minecraft:water"), "id={id}");
        }
    }

    #[test]
    fn missing_or_non_string_id_is_noop() {
        let fix = ItemWaterPotionFix::new(false);
        let no_id = compound(vec![("tag", compound(vec![]))]);
        assert_eq!(fix.apply_to_item_stack(no_id.clone()), no_id);

        let bad_id = compound(vec![("id", DynamicValue::Other)]);
        assert_eq!(fix.apply_to_item_stack(bad_id.clone()), bad_id);
    }
}
