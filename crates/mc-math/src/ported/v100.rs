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

    pub const ENTITY_EQUIPMENT: Reference = Reference("ENTITY_EQUIPMENT");
    pub const ITEM_STACK: Reference = Reference("ITEM_STACK");
}

pub struct V100 {
    pub version: i32,
    pub parent: Option<Box<Schema>>,
}

impl V100 {
    // JAVA: net.minecraft.util.datafix.schemas.V100#<init>(int, Schema)
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent: Some(Box::new(parent)),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V100#registerTypes(Schema, Map, Map)
    pub fn register_types(
        &self,
        _schema: &Schema,
        entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        block_entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
    ) {
        let _ = entity_types;
        let _ = block_entity_types;
        let _ = references::ENTITY_EQUIPMENT;
        let _ = references::ITEM_STACK;
    }

    // Helper mirroring the Java behavior: register the ENTITY_EQUIPMENT type.
    // JAVA: net.minecraft.util.datafix.schemas.V100#registerTypes(Schema, Map, Map)
    pub fn register_types_with_registry(
        &self,
        _schema: &Schema,
        _entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        _block_entity_types: &mut HashMap<String, Box<dyn Fn() -> TypeTemplate>>,
        type_registry: &mut HashMap<String, (bool, Box<dyn Fn() -> TypeTemplate>)>,
    ) {
        let template = TypeTemplate;
        type_registry.insert(
            references::ENTITY_EQUIPMENT.0.to_string(),
            (true, Box::new(move || template.clone())),
        );
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
        let parent = make_schema(99);
        let v = V100::new(100, parent);
        assert_eq!(v.version, 100);
        assert_eq!(v.parent.as_ref().unwrap().version, 99);
    }

    #[test]
    fn register_types_does_not_panic() {
        let schema = make_schema(100);
        let v = V100::new(100, schema.clone());
        let mut entity_types: HashMap<String, Box<dyn Fn() -> TypeTemplate>> = HashMap::new();
        let mut block_entity_types: HashMap<String, Box<dyn Fn() -> TypeTemplate>> = HashMap::new();
        v.register_types(&schema, &mut entity_types, &mut block_entity_types);
        assert!(entity_types.is_empty());
        assert!(block_entity_types.is_empty());
    }

    #[test]
    fn register_types_with_registry_adds_entity_equipment() {
        let schema = make_schema(100);
        let v = V100::new(100, schema.clone());
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
        assert!(type_registry.contains_key("ENTITY_EQUIPMENT"));
        let (recursive, factory) = type_registry.get("ENTITY_EQUIPMENT").unwrap();
        assert!(*recursive);
        assert_eq!(factory(), TypeTemplate);
    }

    #[test]
    fn entity_equipment_reference_value() {
        assert_eq!(references::ENTITY_EQUIPMENT.0, "ENTITY_EQUIPMENT");
        assert_eq!(references::ITEM_STACK.0, "ITEM_STACK");
    }
}
