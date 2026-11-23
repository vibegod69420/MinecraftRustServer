use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Empty,
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

impl Schema {
    pub fn register_simple(
        &self,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
    ) {
        map.insert(name.to_string(), Box::new(|| TypeTemplate::Empty));
    }
}

pub struct NamespacedSchema {
    pub version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, _parent: &Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

pub struct V1928 {
    pub base: NamespacedSchema,
}

impl V1928 {
    // JAVA: net.minecraft.util.datafix.schemas.V1928#<init>
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1928#registerMob
    pub fn register_mob(
        schema: &Schema,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
    ) {
        schema.register_simple(map, name);
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1928#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        map.remove("minecraft:illager_beast");
        Self::register_mob(schema, &mut map, "minecraft:ravager");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(map: &BTreeMap<String, TemplateSupplier>, key: &str) -> TypeTemplate {
        map.get(key).expect("missing key")()
    }

    #[test]
    fn adds_ravager_and_removes_illager_beast() {
        let schema = Schema;
        let v = V1928::new(1928, &schema);
        let map = v.register_entities(&schema);

        assert!(!map.contains_key("minecraft:illager_beast"));
        assert!(map.contains_key("minecraft:ravager"));
        assert_eq!(resolve(&map, "minecraft:ravager"), TypeTemplate::Empty);
    }

    #[test]
    fn register_mob_registers_simple_entity() {
        let schema = Schema;
        let mut map: BTreeMap<String, TemplateSupplier> = BTreeMap::new();
        V1928::register_mob(&schema, &mut map, "minecraft:test_mob");

        assert_eq!(map.len(), 1);
        assert_eq!(resolve(&map, "minecraft:test_mob"), TypeTemplate::Empty);
    }

    #[test]
    fn remove_then_add_behaves_correctly_when_base_has_illager_beast() {
        let schema = Schema;
        let mut map: BTreeMap<String, TemplateSupplier> = BTreeMap::new();
        map.insert(
            "minecraft:illager_beast".to_string(),
            Box::new(|| TypeTemplate::Empty),
        );
        map.insert(
            "minecraft:other".to_string(),
            Box::new(|| TypeTemplate::Empty),
        );

        map.remove("minecraft:illager_beast");
        V1928::register_mob(&schema, &mut map, "minecraft:ravager");

        assert!(!map.contains_key("minecraft:illager_beast"));
        assert!(map.contains_key("minecraft:ravager"));
        assert!(map.contains_key("minecraft:other"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn version_integer_edge_cases() {
        let schema = Schema;
        for version in [0_i32, -1, 1928, i32::MIN, i32::MAX] {
            let v = V1928::new(version, &schema);
            assert_eq!(v.base.version, version);
        }
    }
}
