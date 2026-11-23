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

    pub const BLOCK_STATE: Reference = Reference("BLOCK_STATE");
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#namespacedString()
    pub fn namespaced_string() -> String {
        "namespaced_string".to_string()
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities(Schema)
    pub fn register_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        HashMap::new()
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities(Schema)
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        HashMap::new()
    }
}

pub struct V3685 {
    pub base: NamespacedSchema,
}

impl V3685 {
    // JAVA: net.minecraft.util.datafix.schemas.V3685#<init>(int, Schema)
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3685#abstractArrow(Schema)
    pub fn abstract_arrow(_schema: &Schema) -> TypeTemplate {
        let _ = references::BLOCK_STATE;
        let _ = references::ITEM_STACK;
        TypeTemplate
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3685#registerEntities(Schema)
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut registry = self.base.register_entities(schema);
        let arrow = Self::abstract_arrow(schema);
        let arrow_trident = arrow.clone();
        let arrow_spectral = arrow.clone();
        let arrow_arrow = arrow;
        self.base.register(
            &mut registry,
            "minecraft:trident",
            Box::new(move || arrow_trident.clone()),
        );
        self.base.register(
            &mut registry,
            "minecraft:spectral_arrow",
            Box::new(move || arrow_spectral.clone()),
        );
        self.base.register(
            &mut registry,
            "minecraft:arrow",
            Box::new(move || arrow_arrow.clone()),
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
    fn abstract_arrow_returns_type_template() {
        let schema = make_schema(3685);
        let template = V3685::abstract_arrow(&schema);
        assert_eq!(template, TypeTemplate);
    }

    #[test]
    fn register_entities_contains_arrow_types() {
        let schema = make_schema(3685);
        let v = V3685::new(3685, schema.clone());
        let registry = v.register_entities(&schema);
        assert!(registry.contains_key("minecraft:trident"));
        assert!(registry.contains_key("minecraft:spectral_arrow"));
        assert!(registry.contains_key("minecraft:arrow"));
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn registered_factories_produce_templates() {
        let schema = make_schema(3685);
        let v = V3685::new(3685, schema.clone());
        let registry = v.register_entities(&schema);
        for key in [
            "minecraft:trident",
            "minecraft:spectral_arrow",
            "minecraft:arrow",
        ] {
            let factory = registry.get(key).expect("missing entity");
            assert_eq!(factory(), TypeTemplate);
        }
    }

    #[test]
    fn constructor_preserves_version() {
        let schema = make_schema(3684);
        let v = V3685::new(3685, schema);
        assert_eq!(v.base.version, 3685);
        assert_eq!(v.base.parent.as_ref().unwrap().version, 3684);
    }
}
