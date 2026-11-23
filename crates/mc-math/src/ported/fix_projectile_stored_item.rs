//! Faithful port skeleton of `net.minecraft.util.datafix.fixes.FixProjectileStoredItem`.
//!
//! The original Java class depends on Mojang DataFixerUpper (Typed/TypeRewriteRule/OpticFinder).
//! Those types are not present in `mc-math`, so this module provides a self-contained,
//! behavior-faithful representation of the fix's core entity transformation logic:
//! - `minecraft:arrow`: sets `item` to `{id: "minecraft:arrow", Count: 1}` when Potion is empty,
//!   otherwise `{id: "minecraft:tipped_arrow", Count: 1}`
//! - `minecraft:spectral_arrow`: sets `item` to `{id: "minecraft:spectral_arrow", Count: 1}`
//! - `minecraft:trident`: unchanged (cast/identity in DFU terms)
//!
//! Dynamic data is represented with `serde_json::Value` for a self-contained module.

use serde_json::{json, Value};

const EMPTY_POTION: &str = "minecraft:empty";
const ARROW_ID: &str = "minecraft:arrow";
const TIPPED_ARROW_ID: &str = "minecraft:tipped_arrow";
const SPECTRAL_ARROW_ID: &str = "minecraft:spectral_arrow";
const TRIDENT_ID: &str = "minecraft:trident";

/// Minimal self-contained entity representation for this fix.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityData {
    pub entity_id: String,
    /// Entity fields as a JSON object map.
    pub data: Value,
}

impl EntityData {
    // JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#FixProjectileStoredItem
    pub fn new(entity_id: String, data: Value) -> Self {
        Self { entity_id, data }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#createItemStack
fn create_item_stack(_entity: &Value, item_id: &str) -> Value {
    json!({
        "id": item_id,
        "Count": 1
    })
}

// JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#getArrowType
fn get_arrow_type(entity: &Value) -> &'static str {
    let potion = entity
        .get("Potion")
        .and_then(|v| v.as_str())
        .unwrap_or(EMPTY_POTION);
    if potion == EMPTY_POTION {
        ARROW_ID
    } else {
        TIPPED_ARROW_ID
    }
}

// JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#fixArrow
fn fix_arrow(mut entity: EntityData) -> EntityData {
    let item_id = get_arrow_type(&entity.data);
    let item = create_item_stack(&entity.data, item_id);
    if let Some(obj) = entity.data.as_object_mut() {
        obj.insert("item".to_string(), item);
    }
    entity
}

// JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#fixSpectralArrow
fn fix_spectral_arrow(mut entity: EntityData) -> EntityData {
    let item = create_item_stack(&entity.data, SPECTRAL_ARROW_ID);
    if let Some(obj) = entity.data.as_object_mut() {
        obj.insert("item".to_string(), item);
    }
    entity
}

// JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#castUnchecked
fn cast_unchecked(entity: EntityData) -> EntityData {
    entity
}

// JAVA: net.minecraft.util.datafix.fixes.FixProjectileStoredItem#makeRule
/// Applies the fix logic to a self-contained entity representation.
///
/// Equivalent choice dispatch:
/// - `minecraft:trident` -> identity/cast
/// - `minecraft:arrow` -> set stored item based on Potion
/// - `minecraft:spectral_arrow` -> set spectral arrow item
/// - others -> unchanged
pub fn apply_fix(entity: EntityData) -> EntityData {
    match entity.entity_id.as_str() {
        "minecraft:trident" => cast_unchecked(entity),
        "minecraft:arrow" => fix_arrow(entity),
        "minecraft:spectral_arrow" => fix_spectral_arrow(entity),
        _ => entity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entity(id: &str, data: Value) -> EntityData {
        EntityData::new(id.to_string(), data)
    }

    #[test]
    fn arrow_empty_potion_becomes_arrow() {
        let e = entity("minecraft:arrow", json!({ "Potion": "minecraft:empty" }));
        let out = apply_fix(e);
        assert_eq!(
            out.data.get("item"),
            Some(&json!({"id": "minecraft:arrow", "Count": 1}))
        );
    }

    #[test]
    fn arrow_missing_potion_defaults_to_arrow() {
        let e = entity("minecraft:arrow", json!({}));
        let out = apply_fix(e);
        assert_eq!(
            out.data.get("item"),
            Some(&json!({"id": "minecraft:arrow", "Count": 1}))
        );
    }

    #[test]
    fn arrow_non_empty_potion_becomes_tipped_arrow() {
        let e = entity("minecraft:arrow", json!({ "Potion": "minecraft:poison" }));
        let out = apply_fix(e);
        assert_eq!(
            out.data.get("item"),
            Some(&json!({"id": "minecraft:tipped_arrow", "Count": 1}))
        );
    }

    #[test]
    fn spectral_arrow_gets_item() {
        let e = entity("minecraft:spectral_arrow", json!({ "damage": 2.0 }));
        let out = apply_fix(e);
        assert_eq!(
            out.data.get("item"),
            Some(&json!({"id": "minecraft:spectral_arrow", "Count": 1}))
        );
        assert_eq!(out.data.get("damage"), Some(&json!(2.0)));
    }

    #[test]
    fn trident_unchanged() {
        let e = entity("minecraft:trident", json!({ "Trident": { "tag": {} } }));
        let out = apply_fix(e.clone());
        assert_eq!(out, e);
    }

    #[test]
    fn other_entity_unchanged() {
        let e = entity("minecraft:zombie", json!({ "Health": 20 }));
        let out = apply_fix(e.clone());
        assert_eq!(out, e);
    }
}
