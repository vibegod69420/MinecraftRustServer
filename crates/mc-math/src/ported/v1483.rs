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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> HashMap<String, TypeTemplateSupplier> {
        // Minimal base stub: include puffer_fish so V1483 rename behavior is verifiable.
        let mut map = HashMap::new();
        map.insert(
            "minecraft:puffer_fish".to_string(),
            Box::new(|| TypeTemplate) as TypeTemplateSupplier,
        );
        map
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct V1483 {
    pub namespaced_schema: NamespacedSchema,
}

impl V1483 {
    // JAVA: net.minecraft.util.datafix.schemas.V1483#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            namespaced_schema: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1483#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = self.namespaced_schema.register_entities(schema);
        if let Some(value) = map.remove("minecraft:puffer_fish") {
            map.insert("minecraft:pufferfish".to_string(), value);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let v = V1483::new(1483, Schema::new(0));
        assert_eq!(v.namespaced_schema.version, 1483);
        assert_eq!(v.namespaced_schema.parent.version, 0);
    }

    #[test]
    fn test_register_entities_renames_puffer_fish() {
        let v = V1483::new(1483, Schema::new(0));
        let schema = Schema::new(1483);
        let map = v.register_entities(&schema);
        assert!(!map.contains_key("minecraft:puffer_fish"));
        assert!(map.contains_key("minecraft:pufferfish"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_renamed_supplier_returns_template() {
        let v = V1483::new(1483, Schema::new(0));
        let schema = Schema::new(1483);
        let map = v.register_entities(&schema);
        let supplier = map.get("minecraft:pufferfish").expect("missing key");
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn test_rename_missing_key_leaves_map_unchanged() {
        // Directly exercise the rename logic on a map without the old key.
        let mut map: HashMap<String, TypeTemplateSupplier> = HashMap::new();
        map.insert("minecraft:other".to_string(), Box::new(|| TypeTemplate));
        if let Some(value) = map.remove("minecraft:puffer_fish") {
            map.insert("minecraft:pufferfish".to_string(), value);
        }
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:other"));
        assert!(!map.contains_key("minecraft:pufferfish"));
    }
}
