use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    ItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    Ref(Reference),
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

#[derive(Debug, Clone, Default)]
pub struct Schema {
    version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#Schema
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
    ) {
        map.insert(name.to_string(), TypeTemplate::Empty);
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register<F>(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
        template_factory: F,
    ) where
        F: FnOnce(&Schema) -> TypeTemplate,
    {
        map.insert(name.to_string(), template_factory(self));
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespacedSchema {
    schema: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, _parent: Option<&Schema>) -> Self {
        Self {
            schema: Schema::new(version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        BTreeMap::new()
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

pub struct V4300 {
    base: NamespacedSchema,
}

impl V4300 {
    // JAVA: net.minecraft.util.datafix.schemas.V4300#<init>(int,Schema)
    pub fn new(version: i32, parent: Option<&Schema>) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4300#entityWithInventory(Schema)
    pub fn entity_with_inventory(schema: &Schema) -> TypeTemplate {
        let _ = schema;
        TypeTemplate::OptionalFields(vec![(
            "Items".to_string(),
            TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack))),
        )])
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4300#registerEntities(Schema)
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_entities(schema);
        schema.register(&mut map, "minecraft:llama", |s| Self::entity_with_inventory(s));
        schema.register(&mut map, "minecraft:trader_llama", |s| Self::entity_with_inventory(s));
        schema.register(&mut map, "minecraft:donkey", |s| Self::entity_with_inventory(s));
        schema.register(&mut map, "minecraft:mule", |s| Self::entity_with_inventory(s));
        schema.register_simple(&mut map, "minecraft:horse");
        schema.register_simple(&mut map, "minecraft:skeleton_horse");
        schema.register_simple(&mut map, "minecraft:zombie_horse");
        map
    }

    pub fn schema(&self) -> &Schema {
        self.base.schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_inventory_template() -> TypeTemplate {
        TypeTemplate::OptionalFields(vec![(
            "Items".to_string(),
            TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack))),
        )])
    }

    #[test]
    fn test_entity_with_inventory_structure() {
        let schema = Schema::new(4300);
        let template = V4300::entity_with_inventory(&schema);
        assert_eq!(template, expected_inventory_template());
    }

    #[test]
    fn test_register_entities_inventory_mobs() {
        let v4300 = V4300::new(4300, None);
        let schema = Schema::new(4300);
        let map = v4300.register_entities(&schema);
        for name in [
            "minecraft:llama",
            "minecraft:trader_llama",
            "minecraft:donkey",
            "minecraft:mule",
        ] {
            assert_eq!(map.get(name), Some(&expected_inventory_template()), "{name}");
        }
    }

    #[test]
    fn test_register_entities_simple_horses() {
        let v4300 = V4300::new(4300, None);
        let schema = Schema::new(4300);
        let map = v4300.register_entities(&schema);
        for name in [
            "minecraft:horse",
            "minecraft:skeleton_horse",
            "minecraft:zombie_horse",
        ] {
            assert_eq!(map.get(name), Some(&TypeTemplate::Empty), "{name}");
        }
    }

    #[test]
    fn test_register_entities_count_and_version() {
        let v4300 = V4300::new(4300, None);
        assert_eq!(v4300.schema().version(), 4300);
        let schema = Schema::new(4300);
        let map = v4300.register_entities(&schema);
        assert_eq!(map.len(), 7);
    }
}
