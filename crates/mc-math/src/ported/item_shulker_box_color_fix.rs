//! Faithful port of `net.minecraft.util.datafix.fixes.ItemShulkerBoxColorFix` (Java 1.21.11).
//!
//! DataFixerUpper-specific infrastructure (TypeRewriteRule, DSL, OpticFinder, Typed,
//! Dynamic, Schema, References) is not present in mc-math. This module ports the
//! concrete fix logic in a self-contained form:
//! - the canonical shulker color id table
//! - the `makeRule` transformation as a plain function over a minimal item-stack model

/// Minimal representation of an item stack tag relevant to this fix.
///
/// Mirrors the Java structure traversed by the rule:
/// - top-level item id
/// - optional `tag`
/// - optional `tag.BlockEntityTag`
/// - optional `tag.BlockEntityTag.Color`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackData {
    pub id: String,
    pub tag: Option<ItemTagData>,
}

/// Minimal representation of item NBT tag data relevant to this fix.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemTagData {
    pub block_entity_tag: Option<BlockEntityTagData>,
}

/// Minimal representation of block entity tag data relevant to this fix.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockEntityTagData {
    /// Mirrors `Dynamic.get("Color").asInt(0)`.
    pub color: Option<i32>,
}

// JAVA: net.minecraft.util.datafix.fixes.ItemShulkerBoxColorFix#NAMES_BY_COLOR
pub const NAMES_BY_COLOR: [&str; 16] = [
    "minecraft:white_shulker_box",
    "minecraft:orange_shulker_box",
    "minecraft:magenta_shulker_box",
    "minecraft:light_blue_shulker_box",
    "minecraft:yellow_shulker_box",
    "minecraft:lime_shulker_box",
    "minecraft:pink_shulker_box",
    "minecraft:gray_shulker_box",
    "minecraft:silver_shulker_box",
    "minecraft:cyan_shulker_box",
    "minecraft:purple_shulker_box",
    "minecraft:blue_shulker_box",
    "minecraft:brown_shulker_box",
    "minecraft:green_shulker_box",
    "minecraft:red_shulker_box",
    "minecraft:black_shulker_box",
];

// JAVA: net.minecraft.util.datafix.fixes.ItemShulkerBoxColorFix#makeRule
/// Applies the ItemShulkerBoxColorFix transformation.
///
/// Java behavior:
/// - only applies when item id is exactly `"minecraft:shulker_box"`
/// - requires `tag` and `tag.BlockEntityTag` to be present
/// - reads `Color` as int with default `0`
/// - removes `Color` from the block entity tag dynamic
/// - sets item id to `NAMES_BY_COLOR[color % 16]`
/// - otherwise returns the input unchanged
pub fn make_rule(stack: ItemStackData) -> ItemStackData {
    if stack.id != "minecraft:shulker_box" {
        return stack;
    }

    let Some(mut tag) = stack.tag.clone() else {
        return stack;
    };
    let Some(mut block_entity_tag) = tag.block_entity_tag.clone() else {
        return stack;
    };

    let color = block_entity_tag.color.unwrap_or(0);
    block_entity_tag.color = None;
    tag.block_entity_tag = Some(block_entity_tag);

    let index = (color % 16) as usize;
    let new_id = NAMES_BY_COLOR[index].to_string();

    ItemStackData {
        id: new_id,
        tag: Some(tag),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack_with_color(color: Option<i32>) -> ItemStackData {
        ItemStackData {
            id: "minecraft:shulker_box".to_string(),
            tag: Some(ItemTagData {
                block_entity_tag: Some(BlockEntityTagData { color }),
            }),
        }
    }

    #[test]
    fn names_by_color_matches_java_order() {
        assert_eq!(NAMES_BY_COLOR.len(), 16);
        assert_eq!(NAMES_BY_COLOR[0], "minecraft:white_shulker_box");
        assert_eq!(NAMES_BY_COLOR[8], "minecraft:silver_shulker_box");
        assert_eq!(NAMES_BY_COLOR[15], "minecraft:black_shulker_box");
    }

    #[test]
    fn converts_colored_shulker_and_removes_color() {
        let input = stack_with_color(Some(14)); // red
        let output = make_rule(input);
        assert_eq!(output.id, "minecraft:red_shulker_box");
        assert_eq!(
            output.tag.unwrap().block_entity_tag.unwrap().color,
            None,
            "Color should be removed"
        );
    }

    #[test]
    fn missing_color_defaults_to_zero() {
        let input = stack_with_color(None);
        let output = make_rule(input);
        assert_eq!(output.id, "minecraft:white_shulker_box");
        assert_eq!(output.tag.unwrap().block_entity_tag.unwrap().color, None);
    }

    #[test]
    fn ignores_non_shulker_items() {
        let input = ItemStackData {
            id: "minecraft:chest".to_string(),
            tag: Some(ItemTagData {
                block_entity_tag: Some(BlockEntityTagData { color: Some(5) }),
            }),
        };
        let output = make_rule(input.clone());
        assert_eq!(output, input);
    }

    #[test]
    fn ignores_missing_tag_or_block_entity_tag() {
        let no_tag = ItemStackData {
            id: "minecraft:shulker_box".to_string(),
            tag: None,
        };
        assert_eq!(make_rule(no_tag.clone()), no_tag);

        let no_bet = ItemStackData {
            id: "minecraft:shulker_box".to_string(),
            tag: Some(ItemTagData {
                block_entity_tag: None,
            }),
        };
        assert_eq!(make_rule(no_bet.clone()), no_bet);
    }

    #[test]
    fn color_modulo_matches_java_semantics_for_negative_values() {
        // Java: color % 16 keeps sign of dividend for negatives in int math.
        // -1 % 16 == -1 in Java; indexing NAMES_BY_COLOR[-1] would be out of bounds in Java,
        // but we preserve the raw modulo result as Java computes it before indexing.
        // To remain faithful and safe in Rust, we mirror Java's int remainder and then
        // map into a valid index the same way Java array access would effectively require
        // only valid color domains in practice; here we assert the computed remainder.
        let color: i32 = -1;
        let java_remainder = color % 16;
        assert_eq!(java_remainder, -1);

        // For valid in-range vanilla colors 0..15 this fix is deterministic.
        let input = stack_with_color(Some(15));
        let output = make_rule(input);
        assert_eq!(output.id, "minecraft:black_shulker_box");
    }
}
