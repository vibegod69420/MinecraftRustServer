use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeTemplate(pub String);

pub mod references {
    use super::TypeTemplate;

    pub fn entity_tree() -> TypeTemplate {
        TypeTemplate("entity_tree".to_string())
    }
}

pub mod dsl {
    use super::TypeTemplate;

    pub fn optional_fields(_fields: &[(&str, TypeTemplate)]) -> TypeTemplate {
        TypeTemplate("optional_fields".to_string())
    }

    pub fn list(_inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate("list".to_string())
    }
}

pub struct SchemaRegistry {
    pub entities: BTreeMap<String, TypeTemplate>,
    pub block_entities: BTreeMap<String, TypeTemplate>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
            block_entities: BTreeMap::new(),
        }
    }

    pub fn register_entity_simple(
        &mut self,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
    ) {
        let template = TypeTemplate("simple".to_string());
        self.entities.insert(name.to_string(), template.clone());
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }

    pub fn register_block_entity(
        &mut self,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
        template_factory: impl Fn(&SchemaRegistry) -> TypeTemplate + 'static,
    ) {
        let template = template_factory(self);
        self.block_entities.insert(name.to_string(), template.clone());
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema (minimal stub for V2100)
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(
        &self,
        _schema: &SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        BTreeMap::new()
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(
        &self,
        _schema: &SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        BTreeMap::new()
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V2100
pub struct V2100 {
    pub base: NamespacedSchema,
}

impl V2100 {
    // JAVA: net.minecraft.util.datafix.schemas.V2100#V2100
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent_version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2100#registerMob
    fn register_mob(
        schema: &mut SchemaRegistry,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
    ) {
        schema.register_entity_simple(map, name);
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2100#registerEntities
    pub fn register_entities(
        &self,
        schema: &mut SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut map = self.base.register_entities(schema);
        Self::register_mob(schema, &mut map, "minecraft:bee");
        Self::register_mob(schema, &mut map, "minecraft:bee_stinger");
        map
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2100#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &mut SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut map = self.base.register_block_entities(schema);
        schema.register_block_entity(&mut map, "minecraft:beehive", |_s| {
            dsl::optional_fields(&[(
                "Bees",
                dsl::list(dsl::optional_fields(&[(
                    "EntityData",
                    references::entity_tree(),
                )])),
            )])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2100_registers_bee_entities() {
        let v = V2100::new(2100, 1906);
        let mut schema = SchemaRegistry::new();
        let map = v.register_entities(&mut schema);
        assert!(map.contains_key("minecraft:bee"));
        assert!(map.contains_key("minecraft:bee_stinger"));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("minecraft:bee").unwrap()().0, "simple");
    }

    #[test]
    fn v2100_registers_beehive_block_entity() {
        let v = V2100::new(2100, 1906);
        let mut schema = SchemaRegistry::new();
        let map = v.register_block_entities(&mut schema);
        assert!(map.contains_key("minecraft:beehive"));
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("minecraft:beehive").unwrap()().0,
            "optional_fields"
        );
    }

    #[test]
    fn v2100_schema_registry_tracks_registrations() {
        let v = V2100::new(2100, 1906);
        let mut schema = SchemaRegistry::new();
        v.register_entities(&mut schema);
        v.register_block_entities(&mut schema);
        assert!(schema.entities.contains_key("minecraft:bee"));
        assert!(schema.entities.contains_key("minecraft:bee_stinger"));
        assert!(schema.block_entities.contains_key("minecraft:beehive"));
    }

    #[test]
    fn v2100_version_preserved() {
        let v = V2100::new(2100, 2000);
        assert_eq!(v.base.version, 2100);
        assert_eq!(v.base.parent_version, 2000);
    }
}
