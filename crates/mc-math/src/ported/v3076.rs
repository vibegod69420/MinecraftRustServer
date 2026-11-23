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

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(&self, map: &mut HashMap<String, TypeTemplateSupplier>, name: &str) {
        map.insert(name.to_string(), Box::new(|| TypeTemplate));
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct V3076 {
    pub namespaced_schema: NamespacedSchema,
}

impl V3076 {
    // JAVA: net.minecraft.util.datafix.schemas.V3076#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            namespaced_schema: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3076#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = self.namespaced_schema.register_block_entities(schema);
        schema.register_simple(&mut map, "minecraft:sculk_catalyst");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let v = V3076::new(3076, Schema::new(0));
        assert_eq!(v.namespaced_schema.version, 3076);
        assert_eq!(v.namespaced_schema.parent.version, 0);
    }

    #[test]
    fn test_register_block_entities_adds_sculk_catalyst() {
        let v = V3076::new(3076, Schema::new(0));
        let schema = Schema::new(3076);
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:sculk_catalyst"));
    }

    #[test]
    fn test_registered_supplier_returns_default_template() {
        let v = V3076::new(3076, Schema::new(0));
        let schema = Schema::new(3076);
        let map = v.register_block_entities(&schema);
        let supplier = map.get("minecraft:sculk_catalyst").expect("missing key");
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn test_register_simple_overwrites_existing_key() {
        let schema = Schema::new(3076);
        let mut map: HashMap<String, TypeTemplateSupplier> = HashMap::new();
        map.insert("minecraft:sculk_catalyst".to_string(), Box::new(|| TypeTemplate));
        schema.register_simple(&mut map, "minecraft:sculk_catalyst");
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("minecraft:sculk_catalyst").unwrap()(), TypeTemplate);
    }
}
