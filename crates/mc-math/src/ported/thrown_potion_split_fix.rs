/// Self-contained port of `net.minecraft.util.datafix.fixes.ThrownPotionSplitFix`.
///
/// DFU Typed/OpticFinder machinery is represented by direct item-id input;
/// the core split logic and namespaced-id normalization are preserved 1:1.

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
pub fn ensure_namespaced(value: &str) -> String {
    if value.contains(':') {
        value.to_string()
    } else {
        format!("minecraft:{value}")
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ThrownPotionSplitFix$ItemIdFinder
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemIdFinder;

impl ItemIdFinder {
    // JAVA: net.minecraft.util.datafix.fixes.ThrownPotionSplitFix$ItemIdFinder#getItemId
    pub fn get_item_id(&self, item_id: Option<&str>) -> String {
        item_id.map(ensure_namespaced).unwrap_or_default()
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ThrownPotionSplitFix
#[derive(Debug, Clone)]
pub struct ThrownPotionSplitFix {
    name: String,
    changes_type: bool,
    item_id_finder: ItemIdFinder,
}

impl Default for ThrownPotionSplitFix {
    fn default() -> Self {
        Self::new()
    }
}

impl ThrownPotionSplitFix {
    // JAVA: net.minecraft.util.datafix.fixes.ThrownPotionSplitFix#<init>
    pub fn new() -> Self {
        Self {
            name: "ThrownPotionSplitFix".to_string(),
            changes_type: true,
            item_id_finder: ItemIdFinder,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn changes_type(&self) -> bool {
        self.changes_type
    }

    // JAVA: net.minecraft.util.datafix.fixes.ThrownPotionSplitFix#fix
    pub fn fix(&self, entity_type: &str, item_id: Option<&str>) -> (String, Option<String>) {
        if entity_type != "minecraft:potion" {
            return (entity_type.to_string(), item_id.map(|s| s.to_string()));
        }

        let resolved_item_id = self.item_id_finder.get_item_id(item_id);
        let new_type = if resolved_item_id == "minecraft:lingering_potion" {
            "minecraft:lingering_potion".to_string()
        } else {
            "minecraft:splash_potion".to_string()
        };
        (new_type, item_id.map(|s| s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_potion_entity_unchanged() {
        let fix = ThrownPotionSplitFix::new();
        let (ty, item) = fix.fix("minecraft:snowball", Some("minecraft:snowball"));
        assert_eq!(ty, "minecraft:snowball");
        assert_eq!(item.as_deref(), Some("minecraft:snowball"));
        assert_eq!(fix.name(), "ThrownPotionSplitFix");
        assert!(fix.changes_type());
    }

    #[test]
    fn lingering_potion_splits_to_lingering_entity() {
        let fix = ThrownPotionSplitFix::new();
        let (ty, item) = fix.fix("minecraft:potion", Some("minecraft:lingering_potion"));
        assert_eq!(ty, "minecraft:lingering_potion");
        assert_eq!(item.as_deref(), Some("minecraft:lingering_potion"));
    }

    #[test]
    fn splash_potion_splits_to_splash_entity() {
        let fix = ThrownPotionSplitFix::new();
        let (ty, _) = fix.fix("minecraft:potion", Some("minecraft:splash_potion"));
        assert_eq!(ty, "minecraft:splash_potion");
    }

    #[test]
    fn missing_or_unknown_item_defaults_to_splash() {
        let fix = ThrownPotionSplitFix::new();
        let (ty_none, _) = fix.fix("minecraft:potion", None);
        assert_eq!(ty_none, "minecraft:splash_potion");
        let (ty_unknown, _) = fix.fix("minecraft:potion", Some("minecraft:water"));
        assert_eq!(ty_unknown, "minecraft:splash_potion");
    }

    #[test]
    fn ensure_namespaced_normalizes_ids() {
        assert_eq!(ensure_namespaced("lingering_potion"), "minecraft:lingering_potion");
        assert_eq!(ensure_namespaced("custom:potion"), "custom:potion");
        assert_eq!(ensure_namespaced(""), "minecraft:");
    }

    #[test]
    fn unnamespaced_lingering_item_splits_correctly() {
        let fix = ThrownPotionSplitFix::new();
        let finder = ItemIdFinder;
        assert_eq!(finder.get_item_id(Some("lingering_potion")), "minecraft:lingering_potion");
        let (ty, _) = fix.fix("minecraft:potion", Some("lingering_potion"));
        assert_eq!(ty, "minecraft:lingering_potion");
    }
}
