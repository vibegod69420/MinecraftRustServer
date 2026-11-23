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

    pub const GAME_EVENT_NAME: Reference = Reference("GAME_EVENT_NAME");
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities(Schema)
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        HashMap::new()
    }
}

pub struct V2684 {
    pub base: NamespacedSchema,
}

impl V2684 {
    // JAVA: net.minecraft.util.datafix.schemas.V2684#<init>(int, Schema)
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2684#registerTypes(Schema, Map, Map)
    pub fn register_types(
        &self,
        _schema: &Schema,
        _entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        _block_entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
    ) {
        let _ = NamespacedSchema::namespaced_string();
        let _ = references::GAME_EVENT_NAME;
    }

    // Helper mirroring Java registerType(false, References.GAME_EVENT_NAME, ...).
    // JAVA: net.minecraft.util.datafix.schemas.V2684#registerTypes(Schema, Map, Map)
    pub fn register_types_with_registry(
        &self,
        _schema: &Schema,
        _entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        _block_entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        type_registry: &mut HashMap<String, (bool, Box<dyn Fn() -> TypeTemplate>)>,
    ) {
        let _ = NamespacedSchema::namespaced_string();
        let template = TypeTemplate;
        type_registry.insert(
            references::GAME_EVENT_NAME.0.to_string(),
            (false, Box::new(move || template.clone())),
        );
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2684#registerBlockEntities(Schema)
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut registry = self.base.register_block_entities(schema);
        let _ = references::GAME_EVENT_NAME;
        let template = TypeTemplate;
        self.base.register(
            &mut registry,
            "minecraft:sculk_sensor",
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
    fn constructor_preserves_version_and_parent() {
        let parent = make_schema(2683);
        let v = V2684::new(2684, parent);
        assert_eq!(v.base.version, 2684);
        assert_eq!(v.base.parent.as_ref().unwrap().version, 2683);
    }

    #[test]
    fn register_types_with_registry_adds_game_event_name() {
        let schema = make_schema(2684);
        let v = V2684::new(2684, schema.clone());
        let mut entity_types = HashMap::new();
        let mut block_entity_types = HashMap::new();
        let mut type_registry: HashMap<String, (bool, Box<dyn Fn() -> TypeTemplate>)> =
            HashMap::new();
        v.register_types_with_registry(
            &schema,
            &mut entity_types,
            &mut block_entity_types,
            &mut type_registry,
        );
        assert!(type_registry.contains_key("GAME_EVENT_NAME"));
        let (recursive, factory) = type_registry.get("GAME_EVENT_NAME").unwrap();
        assert!(!(*recursive));
        assert_eq!(factory(), TypeTemplate);
    }

    #[test]
    fn register_block_entities_contains_sculk_sensor() {
        let schema = make_schema(2684);
        let v = V2684::new(2684, schema.clone());
        let registry = v.register_block_entities(&schema);
        assert!(registry.contains_key("minecraft:sculk_sensor"));
        assert_eq!(registry.len(), 1);
        let factory = registry.get("minecraft:sculk_sensor").unwrap();
        assert_eq!(factory(), TypeTemplate);
    }

    #[test]
    fn namespaced_string_constant() {
        assert_eq!(NamespacedSchema::namespaced_string(), "namespaced_string");
    }
}
