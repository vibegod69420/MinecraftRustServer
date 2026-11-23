use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

pub trait Schema {
    fn register_simple(&mut self, map: &mut HashMap<String, TypeTemplateSupplier>, name: &str);
}

#[derive(Debug, Default)]
pub struct SimpleSchema;

impl Schema for SimpleSchema {
    fn register_simple(&mut self, map: &mut HashMap<String, TypeTemplateSupplier>, name: &str) {
        map.insert(name.to_string(), Box::new(|| TypeTemplate));
    }
}

fn namespaced_schema_register_entities(
    _schema: &mut dyn Schema,
) -> HashMap<String, TypeTemplateSupplier> {
    HashMap::new()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct V1904 {
    version_key: i32,
}

impl V1904 {
    // JAVA: net.minecraft.util.datafix.schemas.V1904#<init>
    pub fn new(version_key: i32, _parent: &dyn Schema) -> Self {
        Self { version_key }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1904#registerEntities
    pub fn register_entities(
        &self,
        schema: &mut dyn Schema,
    ) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = namespaced_schema_register_entities(schema);
        schema.register_simple(&mut map, "minecraft:cat");
        map
    }

    pub fn version_key(&self) -> i32 {
        self.version_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_version_key() {
        let parent = SimpleSchema;
        let schema = V1904::new(1904, &parent);
        assert_eq!(schema.version_key(), 1904);
    }

    #[test]
    fn register_entities_adds_cat() {
        let parent = SimpleSchema;
        let v = V1904::new(1904, &parent);
        let mut schema = SimpleSchema;
        let map = v.register_entities(&mut schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:cat"));
        let template = map.get("minecraft:cat").unwrap()();
        assert_eq!(template, TypeTemplate);
    }

    #[test]
    fn version_key_edge_cases() {
        let parent = SimpleSchema;
        assert_eq!(V1904::new(0, &parent).version_key(), 0);
        assert_eq!(V1904::new(-1, &parent).version_key(), -1);
        assert_eq!(V1904::new(i32::MIN, &parent).version_key(), i32::MIN);
        assert_eq!(V1904::new(i32::MAX, &parent).version_key(), i32::MAX);
    }

    #[test]
    fn register_entities_returns_independent_maps() {
        let parent = SimpleSchema;
        let v = V1904::new(1904, &parent);
        let mut schema = SimpleSchema;
        let map1 = v.register_entities(&mut schema);
        let map2 = v.register_entities(&mut schema);
        assert_eq!(map1.len(), 1);
        assert_eq!(map2.len(), 1);
        assert!(map1.contains_key("minecraft:cat"));
        assert!(map2.contains_key("minecraft:cat"));
    }
}
