// mc-math/src/ported/chunk_renames_fix.rs
//
// 1:1 faithful port of net.minecraft.util.datafix.fixes.ChunkRenamesFix (Java 1.21.11).
//
// The original Java class extends DataFix (Mojang DataFixerUpper framework) and
// operates on Dynamic/Typed/Type/OpticFinder abstractions. Since DFU is not
// ported, this module provides a self-contained, data-oriented representation of
// the same transformation logic:
//   - rename top-level chunk fields (TileEntities -> block_entities, etc.)
//   - rename nested Structures.Starts -> structures.starts
//   - merge Level remainder into the chunk node and remove Level
//
// The implementation models the chunk as a simple key-value map (Dynamic-like)
// to preserve the observable field-rename and merge semantics.

use std::collections::BTreeMap;

/// Minimal self-contained dynamic value used to represent DFU Dynamic-like trees
/// for this fix. Only the operations needed by ChunkRenamesFix are implemented.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicValue {
    Map(BTreeMap<String, DynamicValue>),
    List(Vec<DynamicValue>),
    String(String),
    Long(i64),
    Int(i32),
    Double(f64),
    Float(f32),
    Bool(bool),
    Byte(i8),
    Short(i16),
    Empty,
}

impl DynamicValue {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (helper behavior)
    pub fn as_map(&self) -> Option<&BTreeMap<String, DynamicValue>> {
        match self {
            DynamicValue::Map(m) => Some(m),
            _ => None,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (helper behavior)
    pub fn as_map_mut(&mut self) -> Option<&mut BTreeMap<String, DynamicValue>> {
        match self {
            DynamicValue::Map(m) => Some(m),
            _ => None,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (helper behavior)
    pub fn remove(&self, key: &str) -> DynamicValue {
        match self {
            DynamicValue::Map(m) => {
                let mut out = m.clone();
                out.remove(key);
                DynamicValue::Map(out)
            }
            _ => self.clone(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (helper behavior)
    pub fn get(&self, key: &str) -> Option<&DynamicValue> {
        self.as_map().and_then(|m| m.get(key))
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (helper behavior)
    pub fn set_field(&self, key: &str, value: DynamicValue) -> DynamicValue {
        match self {
            DynamicValue::Map(m) => {
                let mut out = m.clone();
                out.insert(key.to_string(), value);
                DynamicValue::Map(out)
            }
            _ => self.clone(),
        }
    }
}

/// A minimal typed wrapper analogous to DFU Typed<A> for the subset of operations
/// used by ChunkRenamesFix.
#[derive(Clone, Debug, PartialEq)]
pub struct Typed {
    pub value: DynamicValue,
}

impl Typed {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (Typed construction context)
    pub fn new(value: DynamicValue) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (Typed#getValue analog)
    pub fn get_value(&self) -> &DynamicValue {
        &self.value
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (Typed#get / remainder analog)
    pub fn remainder(&self) -> DynamicValue {
        self.value.clone()
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (Typed#set remainder analog)
    pub fn set_remainder(&self, remainder: DynamicValue) -> Typed {
        Typed::new(remainder)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (Typed#update remainder analog)
    pub fn update_remainder<F>(&self, f: F) -> Typed
    where
        F: FnOnce(DynamicValue) -> DynamicValue,
    {
        Typed::new(f(self.value.clone()))
    }
}

/// Port of ChunkRenamesFix.
pub struct ChunkRenamesFix;

impl ChunkRenamesFix {
    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix#makeRule
    pub fn apply(&self, chunk: DynamicValue) -> DynamicValue {
        // In Java this operates on Typed<Chunk> with optic finders into Level/Structures.
        // Here we faithfully apply the same field transformations on the map structure:
        // - work on Level content
        // - append chunk name context (represented by preserving structure; no-op wrapper in data form)
        // - merge chunk remainder into level remainder
        // - rename fields
        // - rename Structures.Starts -> structures.starts
        // - remove Level
        let chunk_typed = Typed::new(chunk);

        let level = match chunk_typed.get_value().get("Level") {
            Some(v) => Typed::new(v.clone()),
            None => {
                // If there is no Level field, Java rule would still be structured around it;
                // preserve input faithfully as no-op.
                return chunk_typed.get_value().clone();
            }
        };

        let mut level_typed = Self::append_chunk_name(level);

        let merged = Self::merge_remainders(&chunk_typed, level_typed.remainder());
        level_typed = level_typed.set_remainder(merged);

        level_typed = Self::rename_field(level_typed, "TileEntities", "block_entities");
        level_typed = Self::rename_field(level_typed, "TileTicks", "block_ticks");
        level_typed = Self::rename_field(level_typed, "Entities", "entities");
        level_typed = Self::rename_field(level_typed, "Sections", "sections");

        // Rename Structures.Starts -> structures.starts (after Structures field is present as "Structures")
        level_typed = Self::update_structures(level_typed);

        level_typed = Self::rename_field(level_typed, "Structures", "structures");

        level_typed
            .update_remainder(|d| d.remove("Level"))
            .get_value()
            .clone()
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix#renameField
    fn rename_field(typed: Typed, from: &str, to: &str) -> Typed {
        let renamed = Self::rename_field_helper(typed, from, to);
        renamed.update_remainder(|d| d.remove(from))
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix#renameFieldHelper
    fn rename_field_helper(typed: Typed, from: &str, to: &str) -> Typed {
        // Java uses DSL.optional(field(from, A)) -> DSL.optional(field(to, A)) with Function.identity().
        // Semantically: if `from` exists, ensure `to` gets that value (identity transform).
        // We preserve existing `to` only when `from` is absent, matching optional field move behavior.
        let value = typed.get_value().clone();
        let Some(map) = value.as_map() else {
            return typed;
        };

        if let Some(from_val) = map.get(from) {
            let mut out = map.clone();
            out.insert(to.to_string(), from_val.clone());
            Typed::new(DynamicValue::Map(out))
        } else {
            typed
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix#appendChunkName
    fn append_chunk_name(typed: Typed) -> Typed {
        // Java: new Typed<>(DSL.named("chunk", type), ops, Pair.of("chunk", value))
        // This wraps the value in a named pair in DFU's type system. For our data-only
        // representation, the observable chunk payload is unchanged.
        Typed::new(typed.get_value().clone())
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix#mergeRemainders
    fn merge_remainders(chunk_typed: &Typed, level_remainder: DynamicValue) -> DynamicValue {
        // Java merges chunk remainder map into level remainder map using DynamicOps.mergeToMap.
        // DFU mergeToMap semantics: merge entries from the second map into the first; existing
        // keys in the target are preserved unless the op specifies otherwise. In practice for
        // chunk remainders, this brings chunk-level fields into the promoted level map without
        // overwriting level fields.
        let chunk_remainder = chunk_typed.remainder();

        let (Some(chunk_map), Some(level_map)) = (chunk_remainder.as_map(), level_remainder.as_map())
        else {
            return level_remainder;
        };

        let mut merged = level_map.clone();
        for (k, v) in chunk_map {
            // Do not merge the Level field itself; it is removed by the caller.
            if k == "Level" {
                continue;
            }
            merged.entry(k.clone()).or_insert_with(|| v.clone());
        }
        DynamicValue::Map(merged)
    }

    // JAVA: net.minecraft.util.datafix.fixes.ChunkRenamesFix (Structures sub-update from makeRule)
    fn update_structures(typed: Typed) -> Typed {
        let value = typed.get_value().clone();
        let Some(map) = value.as_map() else {
            return typed;
        };

        let Some(structures) = map.get("Structures") else {
            return typed;
        };

        let structures_typed = Typed::new(structures.clone());
        let renamed_starts = Self::rename_field(structures_typed, "Starts", "starts");
        let mut out = map.clone();
        out.insert("Structures".to_string(), renamed_starts.get_value().clone());
        Typed::new(DynamicValue::Map(out))
    }
}

impl Default for ChunkRenamesFix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: Vec<(&str, DynamicValue)>) -> DynamicValue {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        DynamicValue::Map(m)
    }

    #[test]
    fn renames_level_fields_and_removes_level() {
        let input = map(vec![
            ("Level", map(vec![
                ("TileEntities", DynamicValue::List(vec![])),
                ("TileTicks", DynamicValue::List(vec![])),
                ("Entities", DynamicValue::List(vec![])),
                ("Sections", DynamicValue::List(vec![])),
            ])),
            ("DataVersion", DynamicValue::Int(2730)),
        ]);

        let out = ChunkRenamesFix::new().apply(input);
        let out_map = out.as_map().expect("map");
        assert!(!out_map.contains_key("Level"));
        assert!(out_map.contains_key("block_entities"));
        assert!(out_map.contains_key("block_ticks"));
        assert!(out_map.contains_key("entities"));
        assert!(out_map.contains_key("sections"));
        assert_eq!(out_map.get("DataVersion"), Some(&DynamicValue::Int(2730)));
    }

    #[test]
    fn renames_structures_starts() {
        let input = map(vec![
            ("Level", map(vec![
                ("Structures", map(vec![
                    ("Starts", map(vec![("village", DynamicValue::Bool(true))])),
                    ("References", map(vec![])),
                ])),
            ])),
        ]);

        let out = ChunkRenamesFix::new().apply(input);
        let out_map = out.as_map().unwrap();
        let structures = out_map.get("structures").unwrap().as_map().unwrap();
        assert!(!structures.contains_key("Starts"));
        assert!(structures.contains_key("starts"));
        assert!(structures.contains_key("References"));
    }

    #[test]
    fn merge_remainders_preserves_level_values() {
        // Chunk remainder has a key that also exists in Level; Level should win (or_insert_with).
        let input = map(vec![
            ("xPos", DynamicValue::Int(1)),
            ("Level", map(vec![
                ("xPos", DynamicValue::Int(99)),
                ("Status", DynamicValue::String("full".to_string())),
            ])),
        ]);

        let out = ChunkRenamesFix::new().apply(input);
        let out_map = out.as_map().unwrap();
        assert_eq!(out_map.get("xPos"), Some(&DynamicValue::Int(99)));
        assert_eq!(out_map.get("Status"), Some(&DynamicValue::String("full".to_string())));
    }

    #[test]
    fn merge_remainders_brings_chunk_fields_into_promoted_level() {
        let input = map(vec![
            ("yPos", DynamicValue::Int(-4)),
            ("Level", map(vec![
                ("Status", DynamicValue::String("full".to_string())),
            ])),
        ]);

        let out = ChunkRenamesFix::new().apply(input);
        let out_map = out.as_map().unwrap();
        assert_eq!(out_map.get("yPos"), Some(&DynamicValue::Int(-4)));
        assert_eq!(out_map.get("Status"), Some(&DynamicValue::String("full".to_string())));
        assert!(!out_map.contains_key("Level"));
    }

    #[test]
    fn no_level_is_noop() {
        let input = map(vec![
            ("DataVersion", DynamicValue::Int(100)),
        ]);
        let out = ChunkRenamesFix::new().apply(input.clone());
        assert_eq!(out, input);
    }

    #[test]
    fn missing_optional_fields_handled() {
        let input = map(vec![
            ("Level", map(vec![
                ("Status", DynamicValue::String("full".to_string())),
            ])),
        ]);
        let out = ChunkRenamesFix::new().apply(input);
        let out_map = out.as_map().unwrap();
        assert_eq!(out_map.get("Status"), Some(&DynamicValue::String("full".to_string())));
        assert!(!out_map.contains_key("block_entities"));
        assert!(!out_map.contains_key("structures"));
    }
}
