//! Faithful port skeleton of `net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix`.
//!
//! The original Java class depends on Mojang DataFixerUpper (DSL/Typed/TypeRewriteRule).
//! Those types are not present in `mc-math`, so this module provides a self-contained,
//! behavior-faithful representation of the fix's core decision logic:
//! - white banner: rename `minecraft:custom_name` -> `minecraft:item_name` when
//!   the extracted translation key is `block.minecraft.ominous_banner`
//! - filled map: rename when extracted translation key is one of the known map names
//! - other items: no change
//!
//! This keeps the module compilable and testable without external DFU classes.

use std::collections::BTreeMap;

const WHITE_BANNER_ID: &str = "minecraft:white_banner";
const FILLED_MAP_ID: &str = "minecraft:filled_map";
const CUSTOM_NAME_KEY: &str = "minecraft:custom_name";
const ITEM_NAME_KEY: &str = "minecraft:item_name";
const OMINOUS_BANNER_TRANSLATION: &str = "block.minecraft.ominous_banner";

const MAP_NAMES: &[&str] = &[
    "filled_map.buried_treasure",
    "filled_map.explorer_jungle",
    "filled_map.explorer_swamp",
    "filled_map.mansion",
    "filled_map.monument",
    "filled_map.trial_chambers",
    "filled_map.village_desert",
    "filled_map.village_plains",
    "filled_map.village_savanna",
    "filled_map.village_snowy",
    "filled_map.village_taiga",
];

/// Minimal self-contained representation of an item stack's relevant datafix fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackData {
    pub item_id: String,
    /// Component map; values are raw serialized component strings for this faithful subset.
    pub components: BTreeMap<String, String>,
}

impl ItemStackData {
    // JAVA: net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix#ItemStackCustomNameToOverrideComponentFix
    pub fn new(item_id: String, components: BTreeMap<String, String>) -> Self {
        Self {
            item_id,
            components,
        }
    }
}

/// Faithful port of `LegacyComponentDataFixUtils.extractTranslationString` for the subset
/// needed by this fix: accepts a plain translation key string or a JSON object containing
/// `{"translate":"..."}`.
fn extract_translation_string(component: &str) -> Option<String> {
    // Plain string form (already a translation key in legacy datafix paths).
    let trimmed = component.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with('{') {
        return Some(trimmed.to_string());
    }

    // Minimal JSON object parse for {"translate":"..."}.
    // This is intentionally small and self-contained; it matches the key behavior used here.
    let parsed: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let obj = parsed.as_object()?;
    let translate = obj.get("translate")?.as_str()?;
    Some(translate.to_string())
}

fn is_map_name(name: &str) -> bool {
    MAP_NAMES.iter().any(|&n| n == name)
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix#fixMap
fn fix_map(mut stack: ItemStackData) -> ItemStackData {
    stack = fix_custom_name(stack, is_map_name);
    stack
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix#fixBanner
fn fix_banner(mut stack: ItemStackData) -> ItemStackData {
    stack = fix_custom_name(stack, |name| name == OMINOUS_BANNER_TRANSLATION);
    stack
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix#fixCustomName
fn fix_custom_name(mut stack: ItemStackData, predicate: impl Fn(&str) -> bool) -> ItemStackData {
    let custom_name = match stack.components.get(CUSTOM_NAME_KEY) {
        Some(v) => v.clone(),
        None => return stack,
    };

    let should_rename = extract_translation_string(&custom_name)
        .map(|s| predicate(&s))
        .unwrap_or(false);

    if should_rename {
        if let Some(value) = stack.components.remove(CUSTOM_NAME_KEY) {
            stack.components.insert(ITEM_NAME_KEY.to_string(), value);
        }
    }

    stack
}

// JAVA: net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix#makeRule
/// Applies the fix logic to a self-contained item stack representation.
///
/// Equivalent decision flow:
/// - if item is `minecraft:white_banner`, apply banner fix
/// - else if item is `minecraft:filled_map`, apply map fix
/// - else return unchanged
pub fn apply_fix(stack: ItemStackData) -> ItemStackData {
    if stack.item_id == WHITE_BANNER_ID {
        return fix_banner(stack);
    }
    if stack.item_id == FILLED_MAP_ID {
        return fix_map(stack);
    }
    stack
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn stack(item_id: &str, custom_name: Option<&str>) -> ItemStackData {
        let mut components = BTreeMap::new();
        if let Some(name) = custom_name {
            components.insert(CUSTOM_NAME_KEY.to_string(), name.to_string());
        }
        ItemStackData::new(item_id.to_string(), components)
    }

    #[test]
    fn banner_ominous_renamed() {
        let s = stack(WHITE_BANNER_ID, Some("block.minecraft.ominous_banner"));
        let out = apply_fix(s);
        assert!(!out.components.contains_key(CUSTOM_NAME_KEY));
        assert_eq!(
            out.components.get(ITEM_NAME_KEY).map(String::as_str),
            Some("block.minecraft.ominous_banner")
        );
    }

    #[test]
    fn banner_other_name_unchanged() {
        let s = stack(WHITE_BANNER_ID, Some("block.minecraft.other_banner"));
        let out = apply_fix(s.clone());
        assert_eq!(out, s);
    }

    #[test]
    fn filled_map_known_name_renamed() {
        let s = stack(FILLED_MAP_ID, Some("filled_map.mansion"));
        let out = apply_fix(s);
        assert!(!out.components.contains_key(CUSTOM_NAME_KEY));
        assert_eq!(
            out.components.get(ITEM_NAME_KEY).map(String::as_str),
            Some("filled_map.mansion")
        );
    }

    #[test]
    fn filled_map_unknown_name_unchanged() {
        let s = stack(FILLED_MAP_ID, Some("filled_map.unknown"));
        let out = apply_fix(s.clone());
        assert_eq!(out, s);
    }

    #[test]
    fn other_item_unchanged() {
        let s = stack("minecraft:stone", Some("filled_map.mansion"));
        let out = apply_fix(s.clone());
        assert_eq!(out, s);
    }

    #[test]
    fn json_translate_component_renamed() {
        let s = stack(
            FILLED_MAP_ID,
            Some(r#"{"translate":"filled_map.buried_treasure"}"#),
        );
        let out = apply_fix(s);
        assert!(!out.components.contains_key(CUSTOM_NAME_KEY));
        assert_eq!(
            out.components.get(ITEM_NAME_KEY).map(String::as_str),
            Some(r#"{"translate":"filled_map.buried_treasure"}"#)
        );
    }
}
