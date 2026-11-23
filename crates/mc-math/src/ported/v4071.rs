use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTemplate {
    Unit,
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Ref(String),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Debug, Clone)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn register_simple(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str) {
        let _ = self;
        map.insert(name.to_owned(), Box::new(|| TypeTemplate::Unit));
    }
}

#[derive(Debug, Clone)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }

    pub fn register_block_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

#[derive(Debug, Clone)]
pub struct V4071 {
    pub base: NamespacedSchema,
}

impl V4071 {
    // JAVA: net.minecraft.util.datafix.schemas.V4071#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema {
                version,
                parent_version,
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4071#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:creaking");
        schema.register_simple(&mut map, "minecraft:creaking_transient");
        map
    }

    fn register_simple(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str) {
        map.insert(name.to_owned(), Box::new(|| TypeTemplate::Unit));
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4071#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        self.register_simple(&mut map, "minecraft:creaking_heart");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_entities_adds_creaking_types() {
        let v = V4071::new(4071, 4070);
        let schema = Schema { version: 4071 };
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("minecraft:creaking").unwrap()(),
            TypeTemplate::Unit
        );
        assert_eq!(
            map.get("minecraft:creaking_transient").unwrap()(),
            TypeTemplate::Unit
        );
    }

    #[test]
    fn register_block_entities_adds_creaking_heart() {
        let v = V4071::new(4071, 4070);
        let schema = Schema { version: 4071 };
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("minecraft:creaking_heart").unwrap()(),
            TypeTemplate::Unit
        );
    }

    #[test]
    fn constructor_preserves_versions() {
        let v = V4071::new(4071, i32::MIN);
        assert_eq!(v.base.version, 4071);
        assert_eq!(v.base.parent_version, i32::MIN);
    }

    #[test]
    fn registrations_are_independent_maps() {
        let v = V4071::new(4071, 4070);
        let schema = Schema { version: 4071 };
        let entities = v.register_entities(&schema);
        let block_entities = v.register_block_entities(&schema);
        assert!(!entities.contains_key("minecraft:creaking_heart"));
        assert!(!block_entities.contains_key("minecraft:creaking"));
    }
}
