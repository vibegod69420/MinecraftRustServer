use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Clone, Debug)]
pub struct References;

impl References {
    pub const ENTITY: &'static str = "minecraft:entity";
}

#[derive(Clone, Debug)]
pub struct NamedEntityWriteReadFix {
    schema: Schema,
    bypass_protection: bool,
    name: String,
    type_reference: &'static str,
    entity_name: String,
}

impl NamedEntityWriteReadFix {
    // JAVA: net.minecraft.util.datafix.fixes.NamedEntityWriteReadFix#<init>
    pub fn new(
        schema: Schema,
        bypass_protection: bool,
        name: String,
        type_reference: &'static str,
        entity_name: String,
    ) -> Self {
        Self {
            schema,
            bypass_protection,
            name,
            type_reference,
            entity_name,
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn bypass_protection(&self) -> bool {
        self.bypass_protection
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_reference(&self) -> &'static str {
        self.type_reference
    }

    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }
}

pub trait DynamicOps: Clone {
    type Value: Clone + PartialEq + fmt::Debug;

    fn empty(&self) -> Self::Value;
    fn create_string(&self, value: &str) -> Self::Value;
    fn create_map(&self, entries: Vec<(Self::Value, Self::Value)>) -> Self::Value;
    fn get_map_value(&self, value: &Self::Value, key: &str) -> Option<Self::Value>;
    fn set_map_value(&self, value: &Self::Value, key: &str, new_value: Self::Value) -> Self::Value;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic<T: DynamicOps> {
    ops: T,
    value: T::Value,
}

impl<T: DynamicOps> Dynamic<T> {
    pub fn new(ops: T, value: T::Value) -> Self {
        Self { ops, value }
    }

    pub fn ops(&self) -> &T {
        &self.ops
    }

    pub fn value(&self) -> &T::Value {
        &self.value
    }

    pub fn into_value(self) -> T::Value {
        self.value
    }

    pub fn get(&self, key: &str) -> OptionalDynamic<T> {
        let result = self
            .ops
            .get_map_value(&self.value, key)
            .map(|v| Dynamic::new(self.ops.clone(), v));
        OptionalDynamic { result }
    }

    pub fn set(&self, key: &str, value: Dynamic<T>) -> Dynamic<T> {
        let new_value = self.ops.set_map_value(&self.value, key, value.value);
        Dynamic::new(self.ops.clone(), new_value)
    }

    pub fn create_string(&self, value: &str) -> Dynamic<T> {
        Dynamic::new(self.ops.clone(), self.ops.create_string(value))
    }

    pub fn create_map(&self, entries: Vec<(Dynamic<T>, Dynamic<T>)>) -> Dynamic<T> {
        let raw_entries = entries
            .into_iter()
            .map(|(k, v)| (k.value, v.value))
            .collect();
        Dynamic::new(self.ops.clone(), self.ops.create_map(raw_entries))
    }
}

#[derive(Clone, Debug)]
pub struct OptionalDynamic<T: DynamicOps> {
    result: Option<Dynamic<T>>,
}

impl<T: DynamicOps> OptionalDynamic<T> {
    pub fn get(self) -> Self {
        self
    }

    pub fn result(self) -> Option<Dynamic<T>> {
        self.result
    }
}

#[derive(Clone, Debug)]
pub struct PrimedTntBlockStateFixer {
    base: NamedEntityWriteReadFix,
}

impl PrimedTntBlockStateFixer {
    // JAVA: net.minecraft.util.datafix.fixes.PrimedTntBlockStateFixer#<init>
    pub fn new(schema: Schema) -> Self {
        Self {
            base: NamedEntityWriteReadFix::new(
                schema,
                true,
                "PrimedTnt BlockState fixer".to_string(),
                References::ENTITY,
                "minecraft:tnt".to_string(),
            ),
        }
    }

    pub fn base(&self) -> &NamedEntityWriteReadFix {
        &self.base
    }

    // JAVA: net.minecraft.util.datafix.fixes.PrimedTntBlockStateFixer#renameFuse
    fn rename_fuse<T: DynamicOps>(dynamic: Dynamic<T>) -> Dynamic<T> {
        let fuse: Option<Dynamic<T>> = dynamic.get("Fuse").get().result();
        match fuse {
            Some(value) => dynamic.set("fuse", value),
            None => dynamic,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.PrimedTntBlockStateFixer#insertBlockState
    fn insert_block_state<T: DynamicOps>(dynamic: Dynamic<T>) -> Dynamic<T> {
        let block_state = dynamic.create_map(vec![(
            dynamic.create_string("Name"),
            dynamic.create_string("minecraft:tnt"),
        )]);
        dynamic.set("block_state", block_state)
    }

    // JAVA: net.minecraft.util.datafix.fixes.PrimedTntBlockStateFixer#fix
    pub fn fix<T: DynamicOps>(&self, dynamic: Dynamic<T>) -> Dynamic<T> {
        Self::rename_fuse(Self::insert_block_state(dynamic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct TestOps;

    #[derive(Clone, Debug, PartialEq)]
    enum TestValue {
        Null,
        String(String),
        Map(Vec<(TestValue, TestValue)>),
    }

    impl DynamicOps for TestOps {
        type Value = TestValue;

        fn empty(&self) -> Self::Value {
            TestValue::Null
        }

        fn create_string(&self, value: &str) -> Self::Value {
            TestValue::String(value.to_string())
        }

        fn create_map(&self, entries: Vec<(Self::Value, Self::Value)>) -> Self::Value {
            TestValue::Map(entries)
        }

        fn get_map_value(&self, value: &Self::Value, key: &str) -> Option<Self::Value> {
            match value {
                TestValue::Map(entries) => entries.iter().find_map(|(k, v)| match k {
                    TestValue::String(s) if s == key => Some(v.clone()),
                    _ => None,
                }),
                _ => None,
            }
        }

        fn set_map_value(
            &self,
            value: &Self::Value,
            key: &str,
            new_value: Self::Value,
        ) -> Self::Value {
            let mut entries = match value {
                TestValue::Map(entries) => entries.clone(),
                _ => Vec::new(),
            };
            let key_val = TestValue::String(key.to_string());
            if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| *k == key_val) {
                *existing = new_value;
            } else {
                entries.push((key_val, new_value));
            }
            TestValue::Map(entries)
        }
    }

    fn dyn_map(entries: Vec<(&str, TestValue)>) -> Dynamic<TestOps> {
        Dynamic::new(
            TestOps,
            TestValue::Map(
                entries
                    .into_iter()
                    .map(|(k, v)| (TestValue::String(k.to_string()), v))
                    .collect(),
            ),
        )
    }

    fn get_str(dynamic: &Dynamic<TestOps>, key: &str) -> Option<String> {
        match dynamic.get(key).result()?.value() {
            TestValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn get_map_name(dynamic: &Dynamic<TestOps>, key: &str) -> Option<String> {
        let map_dyn = dynamic.get(key).result()?;
        get_str(&map_dyn, "Name")
    }

    #[test]
    fn fix_renames_fuse_and_inserts_block_state() {
        let fixer = PrimedTntBlockStateFixer::new(Schema);
        let input = dyn_map(vec![("Fuse", TestValue::String("80".to_string()))]);
        let out = fixer.fix(input);

        assert_eq!(get_str(&out, "fuse"), Some("80".to_string()));
        assert_eq!(get_map_name(&out, "block_state"), Some("minecraft:tnt".to_string()));
        // Original Fuse key is preserved, matching Java set() behavior.
        assert_eq!(get_str(&out, "Fuse"), Some("80".to_string()));
    }

    #[test]
    fn fix_inserts_block_state_when_fuse_missing() {
        let fixer = PrimedTntBlockStateFixer::new(Schema);
        let input = dyn_map(vec![("id", TestValue::String("minecraft:tnt".to_string()))]);
        let out = fixer.fix(input);

        assert!(out.get("fuse").result().is_none());
        assert_eq!(get_map_name(&out, "block_state"), Some("minecraft:tnt".to_string()));
        assert_eq!(get_str(&out, "id"), Some("minecraft:tnt".to_string()));
    }

    #[test]
    fn fix_overwrites_existing_block_state() {
        let fixer = PrimedTntBlockStateFixer::new(Schema);
        let old_block_state = TestValue::Map(vec![(
            TestValue::String("Name".to_string()),
            TestValue::String("minecraft:sand".to_string()),
        )]);
        let input = dyn_map(vec![
            ("Fuse", TestValue::String("10".to_string())),
            ("block_state", old_block_state),
        ]);
        let out = fixer.fix(input);

        assert_eq!(get_map_name(&out, "block_state"), Some("minecraft:tnt".to_string()));
        assert_eq!(get_str(&out, "fuse"), Some("10".to_string()));
    }

    #[test]
    fn fix_handles_empty_map_and_non_map_input() {
        let fixer = PrimedTntBlockStateFixer::new(Schema);

        let empty = dyn_map(vec![]);
        let out_empty = fixer.fix(empty);
        assert_eq!(
            get_map_name(&out_empty, "block_state"),
            Some("minecraft:tnt".to_string())
        );
        assert!(out_empty.get("fuse").result().is_none());

        let non_map = Dynamic::new(TestOps, TestValue::Null);
        let out_non_map = fixer.fix(non_map);
        assert_eq!(
            get_map_name(&out_non_map, "block_state"),
            Some("minecraft:tnt".to_string())
        );
    }

    #[test]
    fn constructor_and_base_metadata_match_java() {
        let fixer = PrimedTntBlockStateFixer::new(Schema);
        let base = fixer.base();
        assert!(base.bypass_protection());
        assert_eq!(base.name(), "PrimedTnt BlockState fixer");
        assert_eq!(base.type_reference(), References::ENTITY);
        assert_eq!(base.entity_name(), "minecraft:tnt");
        assert_eq!(base.schema(), &Schema);
    }
}
