use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference;

pub struct References;

impl References {
    pub const BLOCK_ENTITY: Reference = Reference;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dynamic<T: Clone> {
    value: BTreeMap<String, T>,
}

impl<T: Clone> Dynamic<T> {
    pub fn new(value: BTreeMap<String, T>) -> Self {
        Self { value }
    }

    pub fn remove(mut self, key: &str) -> Self {
        self.value.remove(key);
        self
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.value.get(key)
    }

    pub fn into_map(self) -> BTreeMap<String, T> {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpticFinder;

pub struct DSL;

impl DSL {
    // JAVA: com.mojang.datafixers.DSL#remainderFinder
    pub fn remainder_finder() -> OpticFinder {
        OpticFinder
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typed<T: Clone> {
    value: Dynamic<T>,
}

impl<T: Clone> Typed<T> {
    pub fn new(value: Dynamic<T>) -> Self {
        Self { value }
    }

    pub fn update<F>(self, _finder: OpticFinder, f: F) -> Self
    where
        F: FnOnce(Dynamic<T>) -> Dynamic<T>,
    {
        Self {
            value: f(self.value),
        }
    }

    pub fn into_dynamic(self) -> Dynamic<T> {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityShulkerBoxColorFix {
    pub schema: Schema,
    pub changes_type: bool,
    pub name: String,
    pub type_reference: Reference,
    pub entity_name: String,
}

impl BlockEntityShulkerBoxColorFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityShulkerBoxColorFix#<init>
    pub fn new(schema: Schema, changes_type: bool) -> Self {
        Self {
            schema,
            changes_type,
            name: "BlockEntityShulkerBoxColorFix".to_string(),
            type_reference: References::BLOCK_ENTITY,
            entity_name: "minecraft:shulker_box".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityShulkerBoxColorFix#fix
    pub fn fix<T: Clone>(&self, typed: Typed<T>) -> Typed<T> {
        typed.update(DSL::remainder_finder(), |d| d.remove("Color"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn make_typed(pairs: &[(&str, &str)]) -> Typed<String> {
        let mut m = BTreeMap::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), (*v).to_string());
        }
        Typed::new(Dynamic::new(m))
    }

    #[test]
    fn removes_color_field() {
        let fix = BlockEntityShulkerBoxColorFix::new(Schema, true);
        let t = make_typed(&[
            ("id", "minecraft:shulker_box"),
            ("Color", "14"),
            ("x", "1"),
        ]);
        let out = fix.fix(t).into_dynamic().into_map();
        assert_eq!(out.len(), 2);
        assert!(!out.contains_key("Color"));
        assert_eq!(out.get("id").map(String::as_str), Some("minecraft:shulker_box"));
        assert_eq!(out.get("x").map(String::as_str), Some("1"));
    }

    #[test]
    fn handles_missing_color() {
        let fix = BlockEntityShulkerBoxColorFix::new(Schema, false);
        let t = make_typed(&[("id", "minecraft:shulker_box"), ("Items", "[]")]);
        let out = fix.fix(t).into_dynamic().into_map();
        assert_eq!(out.len(), 2);
        assert_eq!(out.get("id").map(String::as_str), Some("minecraft:shulker_box"));
        assert_eq!(out.get("Items").map(String::as_str), Some("[]"));
    }

    #[test]
    fn empty_input_and_negative_like_values_preserved() {
        let fix = BlockEntityShulkerBoxColorFix::new(Schema, true);
        let empty_out = fix.fix(make_typed(&[])).into_dynamic().into_map();
        assert!(empty_out.is_empty());

        let t = make_typed(&[("Color", "-1"), ("y", "-64")]);
        let out = fix.fix(t).into_dynamic().into_map();
        assert_eq!(out.len(), 1);
        assert_eq!(out.get("y").map(String::as_str), Some("-64"));
    }

    #[test]
    fn constructor_sets_expected_fields() {
        let fix = BlockEntityShulkerBoxColorFix::new(Schema, true);
        assert!(fix.changes_type);
        assert_eq!(fix.name, "BlockEntityShulkerBoxColorFix");
        assert_eq!(fix.entity_name, "minecraft:shulker_box");
        assert_eq!(fix.type_reference, References::BLOCK_ENTITY);
        assert_eq!(fix.schema, Schema);

        let fix2 = BlockEntityShulkerBoxColorFix::new(Schema, false);
        assert!(!fix2.changes_type);
    }
}
