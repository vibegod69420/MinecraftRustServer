//! Port of `net.minecraft.util.datafix.fixes.EmptyItemInHotbarFix`
// JAVA: net.minecraft.util.datafix.fixes.EmptyItemInHotbarFix#makeRule
#![forbid(unsafe_code)]

/// Represents a minimal item stack, as referenced by the fix logic.
#[derive(Debug, Clone)]
pub struct ItemStack {
    /// The resource location name of the item (e.g., "minecraft:stone" or "" for air).
    pub name: String,
    /// The item count. Java uses `int`, here `i32`.
    pub count: i32,
}

/// Implements the core condition check from `EmptyItemInHotbarFix#makeRule`.
// JAVA: net.minecraft.util.datafix.fixes.EmptyItemInHotbarFix#makeRule (condition check)
pub fn should_clear_item(stack: &ItemStack) -> bool {
    // Java: `boolean $$3 = $$1x.isEmpty() || $$1x.get().equals("minecraft:air");`
    let name_empty_or_air = stack.name.is_empty() || stack.name == "minecraft:air";
    // Java: `boolean $$4 = $$2.get("Count").asInt(0) <= 0;`
    let count_nonpositive = stack.count <= 0;
    name_empty_or_air || count_nonpositive
}

/// Applies the EmptyItemInHotbarFix transformation logic.
// JAVA: net.minecraft.util.datafix.fixes.EmptyItemInHotbarFix#makeRule (transformation)
pub fn apply_fix(stack: ItemStack) -> (ItemStack, bool) {
    if should_clear_item(&stack) {
        (stack, true)
    } else {
        (stack, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_air_item_should_clear() {
        let stack = ItemStack { name: "minecraft:air".to_string(), count: 64 };
        assert!(should_clear_item(&stack));
    }

    #[test]
    fn test_empty_name_should_clear() {
        let stack = ItemStack { name: "".to_string(), count: 64 };
        assert!(should_clear_item(&stack));
    }

    #[test]
    fn test_normal_item_should_keep() {
        let stack = ItemStack { name: "minecraft:stone".to_string(), count: 64 };
        assert!(!should_clear_item(&stack));
    }

    #[test]
    fn test_count_zero_should_clear() {
        let stack = ItemStack { name: "minecraft:stone".to_string(), count: 0 };
        assert!(should_clear_item(&stack));
    }

    #[test]
    fn test_negative_count_should_clear() {
        let stack = ItemStack { name: "minecraft:stone".to_string(), count: -1 };
        assert!(should_clear_item(&stack));
    }

    #[test]
    fn test_full_apply_fix() {
        let (stack, cleared) = apply_fix(ItemStack { name: "minecraft:air".to_string(), count: 64 });
        assert!(cleared);
        let (stack2, cleared2) = apply_fix(ItemStack { name: "minecraft:stone".to_string(), count: 64 });
        assert!(!cleared2);
    }
}
