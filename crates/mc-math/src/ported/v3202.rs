use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#<init>
    pub fn new(version: i32) -> Self {
        Self { version }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, TypeTemplateSupplier> {
        HashMap::new()
    }
}

// Minimal self-contained stub for the unported V99 dependency referenced by V3202.
mod v99 {
    use super::{Schema, TypeTemplate};

    // JAVA: net.minecraft.util.datafix.schemas.V99#sign
    pub fn sign(_schema: &Schema) -> TypeTemplate {
        TypeTemplate
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct V3202 {
    pub namespaced_schema: NamespacedSchema,
}

impl V3202 {
    // JAVA: net.minecraft.util.datafix.schemas.V3202#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            namespaced_schema: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3202#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = self.namespaced_schema.register_block_entities(schema);
        let schema_for_sign = schema.clone();
        map.insert(
            "minecraft:hanging_sign".to_string(),
            Box::new(move || v99::sign(&schema_for_sign)),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let v = V3202::new(3202, Schema::new(3076));
        assert_eq!(v.namespaced_schema.version, 3202);
        assert_eq!(v.namespaced_schema.parent.version, 3076);
    }

    #[test]
    fn test_register_block_entities_adds_hanging_sign() {
        let v = V3202::new(3202, Schema::new(0));
        let schema = Schema::new(3202);
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:hanging_sign"));
    }

    #[test]
    fn test_hanging_sign_supplier_returns_template() {
        let v = V3202::new(3202, Schema::new(0));
        let schema = Schema::new(3202);
        let map = v.register_block_entities(&schema);
        let supplier = map.get("minecraft:hanging_sign").expect("missing key");
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn test_v99_sign_stub_returns_template() {
        let schema = Schema::new(3202);
        assert_eq!(v99::sign(&schema), TypeTemplate);
    }
}
