#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema;

pub struct NamespacedSchema;

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
    pub fn ensure_namespaced(s: &str) -> String {
        if s.contains(':') {
            s.to_string()
        } else {
            format!("minecraft:{s}")
        }
    }
}

pub struct AttributeIdPrefixFix {
    pub schema: Schema,
    pub name: String,
}

const PREFIXES: [&str; 4] = ["generic.", "horse.", "player.", "zombie."];

impl AttributeIdPrefixFix {
    // JAVA: net.minecraft.util.datafix.fixes.AttributeIdPrefixFix#AttributeIdPrefixFix
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            name: "AttributeIdPrefixFix".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.AttributeIdPrefixFix#replaceId
    fn replace_id(id: &str) -> String {
        let namespaced = NamespacedSchema::ensure_namespaced(id);
        for prefix in PREFIXES {
            let namespaced_prefix = NamespacedSchema::ensure_namespaced(prefix);
            if namespaced.starts_with(&namespaced_prefix) {
                return format!("minecraft:{}", &namespaced[namespaced_prefix.len()..]);
            }
        }
        id.to_string()
    }

    pub fn apply(&self, id: &str) -> String {
        Self::replace_id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_unqualified_generic_prefix() {
        assert_eq!(
            AttributeIdPrefixFix::replace_id("generic.attack_damage"),
            "minecraft:attack_damage"
        );
    }

    #[test]
    fn replaces_namespaced_generic_prefix() {
        assert_eq!(
            AttributeIdPrefixFix::replace_id("minecraft:generic.attack_damage"),
            "minecraft:attack_damage"
        );
    }

    #[test]
    fn replaces_all_supported_prefixes() {
        assert_eq!(
            AttributeIdPrefixFix::replace_id("horse.jump_strength"),
            "minecraft:jump_strength"
        );
        assert_eq!(
            AttributeIdPrefixFix::replace_id("player.block_interaction_range"),
            "minecraft:block_interaction_range"
        );
        assert_eq!(
            AttributeIdPrefixFix::replace_id("zombie.spawn_reinforcements"),
            "minecraft:spawn_reinforcements"
        );
    }

    #[test]
    fn preserves_unmatched_original_string() {
        assert_eq!(
            AttributeIdPrefixFix::replace_id("custom.attribute"),
            "custom.attribute"
        );
        assert_eq!(
            AttributeIdPrefixFix::replace_id("minecraft:movement_speed"),
            "minecraft:movement_speed"
        );
    }

    #[test]
    fn handles_empty_and_boundary_strings() {
        assert_eq!(AttributeIdPrefixFix::replace_id(""), "");
        assert_eq!(AttributeIdPrefixFix::replace_id("generic."), "minecraft:");
    }

    #[test]
    fn handles_negative_and_float_like_strings() {
        assert_eq!(AttributeIdPrefixFix::replace_id("-1"), "-1");
        assert_eq!(AttributeIdPrefixFix::replace_id("3.141592653589793"), "3.141592653589793");
        assert_eq!(AttributeIdPrefixFix::replace_id("-0.0e+10"), "-0.0e+10");
    }
}
