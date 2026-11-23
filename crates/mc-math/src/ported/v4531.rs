use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TemplateSupplier = fn() -> TypeTemplate;

fn template_supplier() -> TypeTemplate {
    TypeTemplate
}

#[derive(Clone, Debug, Default)]
pub struct Schema;

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(
        &self,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
    ) {
        map.insert(name.to_string(), template_supplier);
    }
}

#[derive(Clone, Debug)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Option<Box<Schema>>,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent: Some(Box::new(parent)),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(
        &self,
        _schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

#[derive(Clone, Debug)]
pub struct V4531 {
    pub base: NamespacedSchema,
}

impl V4531 {
    // JAVA: net.minecraft.util.datafix.schemas.V4531#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4531#registerEntities
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:copper_golem");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_copper_golem() {
        let schema = Schema;
        let v = V4531::new(4531, Schema);
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:copper_golem"));
        let template = map["minecraft:copper_golem"]();
        assert_eq!(template, TypeTemplate);
    }

    #[test]
    fn version_edge_cases() {
        for ver in [0_i32, -1, i32::MIN, i32::MAX, 4531] {
            let v = V4531::new(ver, Schema);
            assert_eq!(v.base.version, ver);
        }
    }

    #[test]
    fn base_register_entities_is_empty() {
        let base = NamespacedSchema::new(0, Schema);
        let map = base.register_entities(&Schema);
        assert!(map.is_empty());
    }

    #[test]
    fn register_simple_overwrites_same_key() {
        let schema = Schema;
        let mut map = BTreeMap::new();
        schema.register_simple(&mut map, "minecraft:copper_golem");
        schema.register_simple(&mut map, "minecraft:copper_golem");
        assert_eq!(map.len(), 1);
        assert_eq!(map["minecraft:copper_golem"](), TypeTemplate);
    }
}
