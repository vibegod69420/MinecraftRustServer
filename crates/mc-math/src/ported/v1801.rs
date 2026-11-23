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
pub struct V1801 {
    pub base: NamespacedSchema,
}

impl V1801 {
    // JAVA: net.minecraft.util.datafix.schemas.V1801#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1801#registerEntities
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:illager_beast");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_illager_beast() {
        let schema = Schema;
        let v = V1801::new(1801, Schema);
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:illager_beast"));
        let template = map["minecraft:illager_beast"]();
        assert_eq!(template, TypeTemplate);
    }

    #[test]
    fn version_edge_cases() {
        for ver in [0_i32, -1, i32::MIN, i32::MAX, 1801] {
            let v = V1801::new(ver, Schema);
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
        schema.register_simple(&mut map, "minecraft:illager_beast");
        schema.register_simple(&mut map, "minecraft:illager_beast");
        assert_eq!(map.len(), 1);
        assert_eq!(map["minecraft:illager_beast"](), TypeTemplate);
    }
}
