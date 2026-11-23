use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeTemplate(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
    pub parent: Option<i32>,
}

pub mod references {
    use super::TypeTemplate;

    pub fn item_stack() -> TypeTemplate {
        TypeTemplate("item_stack".to_string())
    }
}

pub mod dsl {
    use super::TypeTemplate;

    pub fn optional_fields(_fields: &[(&str, TypeTemplate)]) -> TypeTemplate {
        TypeTemplate("optional_fields".to_string())
    }

    pub fn remainder() -> TypeTemplate {
        TypeTemplate("remainder".to_string())
    }
}

pub struct SchemaRegistry {
    pub block_entities: BTreeMap<String, TypeTemplate>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            block_entities: BTreeMap::new(),
        }
    }

    pub fn register(
        &mut self,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
        template_factory: impl Fn(&SchemaRegistry) -> TypeTemplate + 'static,
    ) {
        let template = template_factory(self);
        self.block_entities.insert(name.to_string(), template.clone());
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }

    pub fn register_simple(
        &mut self,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
    ) {
        let template = TypeTemplate("simple".to_string());
        self.block_entities.insert(name.to_string(), template.clone());
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema (minimal stub for V1906)
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(
        &self,
        _schema: &SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        BTreeMap::new()
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1458 (minimal stub for V1906)
pub struct V1458;

impl V1458 {
    // JAVA: net.minecraft.util.datafix.schemas.V1458#nameableInventory
    pub fn nameable_inventory(_schema: &SchemaRegistry) -> TypeTemplate {
        TypeTemplate("nameable_inventory".to_string())
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1906
pub struct V1906 {
    pub base: NamespacedSchema,
}

impl V1906 {
    // JAVA: net.minecraft.util.datafix.schemas.V1906#V1906
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent_version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1906#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &mut SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut map = self.base.register_block_entities(schema);
        Self::register_inventory(schema, &mut map, "minecraft:barrel");
        Self::register_inventory(schema, &mut map, "minecraft:smoker");
        Self::register_inventory(schema, &mut map, "minecraft:blast_furnace");
        schema.register(&mut map, "minecraft:lectern", |_s| {
            dsl::optional_fields(&[("Book", references::item_stack())])
        });
        schema.register_simple(&mut map, "minecraft:bell");
        map
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1906#registerInventory
    fn register_inventory(
        schema: &mut SchemaRegistry,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
    ) {
        schema.register(map, name, |s| V1458::nameable_inventory(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1906_registers_expected_block_entities() {
        let v1906 = V1906::new(1906, 1904);
        let mut schema = SchemaRegistry::new();
        let map = v1906.register_block_entities(&mut schema);

        assert!(map.contains_key("minecraft:barrel"));
        assert!(map.contains_key("minecraft:smoker"));
        assert!(map.contains_key("minecraft:blast_furnace"));
        assert!(map.contains_key("minecraft:lectern"));
        assert!(map.contains_key("minecraft:bell"));
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn v1906_inventory_factories_produce_nameable_inventory() {
        let v1906 = V1906::new(1906, 1904);
        let mut schema = SchemaRegistry::new();
        let map = v1906.register_block_entities(&mut schema);

        for name in [
            "minecraft:barrel",
            "minecraft:smoker",
            "minecraft:blast_furnace",
        ] {
            let template = map.get(name).unwrap()();
            assert_eq!(template, TypeTemplate("nameable_inventory".to_string()));
        }
    }

    #[test]
    fn v1906_lectern_has_optional_book_field() {
        let v1906 = V1906::new(1906, 1904);
        let mut schema = SchemaRegistry::new();
        let map = v1906.register_block_entities(&mut schema);
        let template = map.get("minecraft:lectern").unwrap()();
        assert_eq!(template, TypeTemplate("optional_fields".to_string()));
    }

    #[test]
    fn v1906_bell_is_simple() {
        let v1906 = V1906::new(1906, 1904);
        let mut schema = SchemaRegistry::new();
        let map = v1906.register_block_entities(&mut schema);
        let template = map.get("minecraft:bell").unwrap()();
        assert_eq!(template, TypeTemplate("simple".to_string()));
    }

    #[test]
    fn v1906_schema_registry_tracks_registered_entities() {
        let v1906 = V1906::new(1906, 1904);
        let mut schema = SchemaRegistry::new();
        v1906.register_block_entities(&mut schema);
        assert!(schema.block_entities.contains_key("minecraft:barrel"));
        assert_eq!(schema.block_entities.len(), 5);
    }
}
