use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TemplateSupplier = fn() -> TypeTemplate;

pub fn default_template() -> TypeTemplate {
    TypeTemplate
}

/// Minimal self-contained Schema trait for DFU schema porting.
// JAVA: com.mojang.datafixers.schemas.Schema
pub trait Schema {
    fn version_key(&self) -> i32;

    // JAVA: com.mojang.datafixers.schemas.Schema#registerEntities
    fn register_entities(&self, schema: &dyn Schema) -> HashMap<String, TemplateSupplier>;

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    fn register_simple(&self, map: &mut HashMap<String, TemplateSupplier>, name: &str) {
        let _ = self;
        map.insert(name.to_string(), default_template as TemplateSupplier);
    }
}

/// Simple base schema for composition and testing.
pub struct BaseSchema {
    version_key: i32,
    entities: Vec<String>,
}

impl BaseSchema {
    pub fn new(version_key: i32, entities: Vec<&str>) -> Self {
        Self {
            version_key,
            entities: entities.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Schema for BaseSchema {
    fn version_key(&self) -> i32 {
        self.version_key
    }

    fn register_entities(&self, _schema: &dyn Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = HashMap::new();
        for name in &self.entities {
            map.insert(name.clone(), default_template as TemplateSupplier);
        }
        map
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V501
pub struct V501 {
    version_key: i32,
    parent: Box<dyn Schema>,
}

impl V501 {
    // JAVA: net.minecraft.util.datafix.schemas.V501#<init>
    pub fn new(version_key: i32, parent: Box<dyn Schema>) -> Self {
        Self { version_key, parent }
    }
}

impl Schema for V501 {
    fn version_key(&self) -> i32 {
        self.version_key
    }

    // JAVA: net.minecraft.util.datafix.schemas.V501#registerEntities
    fn register_entities(&self, schema: &dyn Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.parent.register_entities(schema);
        schema.register_simple(&mut map, "PolarBear");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_key_stored() {
        let v501 = V501::new(501, Box::new(BaseSchema::new(0, vec![])));
        assert_eq!(v501.version_key(), 501);
    }

    #[test]
    fn registers_polar_bear() {
        let parent = BaseSchema::new(500, vec!["Pig", "Cow"]);
        let v501 = V501::new(501, Box::new(parent));
        let map = v501.register_entities(&v501);
        assert!(map.contains_key("PolarBear"));
        assert!(map.contains_key("Pig"));
        assert!(map.contains_key("Cow"));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn empty_parent_adds_polar_bear() {
        let v501 = V501::new(501, Box::new(BaseSchema::new(0, vec![])));
        let map = v501.register_entities(&v501);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("PolarBear"));
    }

    #[test]
    fn supplier_returns_type_template() {
        let v501 = V501::new(501, Box::new(BaseSchema::new(0, vec![])));
        let map = v501.register_entities(&v501);
        let supplier = map.get("PolarBear").expect("PolarBear should exist");
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn overwrites_existing_polar_bear_entry() {
        let parent = BaseSchema::new(500, vec!["PolarBear", "Pig"]);
        let v501 = V501::new(501, Box::new(parent));
        let map = v501.register_entities(&v501);
        assert!(map.contains_key("PolarBear"));
        assert!(map.contains_key("Pig"));
        assert_eq!(map.len(), 2);
    }
}
