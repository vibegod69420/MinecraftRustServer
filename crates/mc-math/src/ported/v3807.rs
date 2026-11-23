use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeTemplate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reference(pub &'static str);

pub mod references {
    use super::Reference;

    pub const ITEM_STACK: Reference = Reference("ITEM_STACK");
}

pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Option<Box<Schema>>,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>(int, Schema)
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent: Some(Box::new(parent)),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#register(Map, String, Supplier)
    pub fn register(
        &self,
        map: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
        template_factory: Box<dyn Fn() -> TypeTemplate>,
    ) {
        map.insert(name.to_string(), template_factory);
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities(Schema)
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        HashMap::new()
    }
}

pub struct V3807 {
    pub base: NamespacedSchema,
}

impl V3807 {
    // JAVA: net.minecraft.util.datafix.schemas.V3807#<init>(int, Schema)
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3807#registerBlockEntities(Schema)
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut registry = self.base.register_block_entities(schema);
        let _ = references::ITEM_STACK;
        let template = TypeTemplate;
        self.base.register(
            &mut registry,
            "minecraft:vault",
            Box::new(move || template.clone()),
        );
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_schema(version: i32) -> Schema {
        Schema { version }
    }

    #[test]
    fn register_block_entities_contains_vault() {
        let schema = make_schema(3807);
        let v = V3807::new(3807, schema.clone());
        let registry = v.register_block_entities(&schema);
        assert!(registry.contains_key("minecraft:vault"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn vault_factory_produces_template() {
        let schema = make_schema(3807);
        let v = V3807::new(3807, schema.clone());
        let registry = v.register_block_entities(&schema);
        let factory = registry.get("minecraft:vault").expect("missing vault");
        assert_eq!(factory(), TypeTemplate);
    }

    #[test]
    fn constructor_preserves_version_and_parent() {
        let parent = make_schema(3806);
        let v = V3807::new(3807, parent);
        assert_eq!(v.base.version, 3807);
        assert_eq!(v.base.parent.as_ref().unwrap().version, 3806);
    }

    #[test]
    fn base_register_block_entities_is_empty() {
        let schema = make_schema(1);
        let base = NamespacedSchema::new(2, schema.clone());
        let registry = base.register_block_entities(&schema);
        assert!(registry.is_empty());
    }
}
