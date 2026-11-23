use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeTemplate(pub String);

pub mod references {
    use super::TypeTemplate;

    pub fn item_stack() -> TypeTemplate {
        TypeTemplate("item_stack".to_string())
    }

    pub fn villager_trade() -> TypeTemplate {
        TypeTemplate("villager_trade".to_string())
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
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
        }
    }

    pub fn register_entity(
        &mut self,
        map: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        name: &str,
        template_factory: impl Fn(&SchemaRegistry) -> TypeTemplate + 'static,
    ) {
        let template = template_factory(self);
        self.entities.insert(name.to_string(), template.clone());
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema (minimal stub for V1929)
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
}

// JAVA: net.minecraft.util.datafix.schemas.V1929
pub struct V1929 {
    pub base: NamespacedSchema,
}

impl V1929 {
    // JAVA: net.minecraft.util.datafix.schemas.V1929#V1929
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent_version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1929#registerEntities
    pub fn register_entities(
        &self,
        schema: &mut SchemaRegistry,
    ) -> BTreeMap<String, Box<dyn Fn() -> TypeTemplate>> {
        let mut map = self.base.register_entities(schema);
        schema.register_entity(&mut map, "minecraft:wandering_trader", |_s| {
            dsl::optional_fields(&[
                ("Inventory", dsl::list(references::item_stack())),
                (
                    "Offers",
                    dsl::optional_fields(&[(
                        "Recipes",
                        dsl::list(references::villager_trade()),
                    )]),
                ),
            ])
        });
        schema.register_entity(&mut map, "minecraft:trader_llama", |_s| {
            dsl::optional_fields(&[
                ("Items", dsl::list(references::item_stack())),
                ("SaddleItem", references::item_stack()),
                ("DecorItem", references::item_stack()),
            ])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1929_registers_wandering_trader_and_llama() {
        let v = V1929::new(1929, 1928);
        let mut schema = SchemaRegistry::new();
        let map = v.register_entities(&mut schema);
        assert!(map.contains_key("minecraft:wandering_trader"));
        assert!(map.contains_key("minecraft:trader_llama"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn v1929_wandering_trader_template_is_optional_fields() {
        let v = V1929::new(1929, 1928);
        let mut schema = SchemaRegistry::new();
        let map = v.register_entities(&mut schema);
        let template = map.get("minecraft:wandering_trader").unwrap()();
        assert_eq!(template.0, "optional_fields");
    }

    #[test]
    fn v1929_trader_llama_template_is_optional_fields() {
        let v = V1929::new(1929, 1928);
        let mut schema = SchemaRegistry::new();
        let map = v.register_entities(&mut schema);
        let template = map.get("minecraft:trader_llama").unwrap()();
        assert_eq!(template.0, "optional_fields");
    }

    #[test]
    fn v1929_schema_registry_tracks_entities() {
        let v = V1929::new(1929, 1928);
        let mut schema = SchemaRegistry::new();
        v.register_entities(&mut schema);
        assert!(schema.entities.contains_key("minecraft:wandering_trader"));
        assert!(schema.entities.contains_key("minecraft:trader_llama"));
        assert_eq!(schema.entities.len(), 2);
    }

    #[test]
    fn v1929_version_preserved() {
        let v = V1929::new(1929, 1900);
        assert_eq!(v.base.version, 1929);
        assert_eq!(v.base.parent_version, 1900);
    }
}
