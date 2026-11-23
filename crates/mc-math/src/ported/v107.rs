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

// JAVA: net.minecraft.util.datafix.schemas.V107
pub struct V107 {
    version_key: i32,
    parent: Box<dyn Schema>,
}

impl V107 {
    // JAVA: net.minecraft.util.datafix.schemas.V107#<init>
    pub fn new(version_key: i32, parent: Box<dyn Schema>) -> Self {
        Self { version_key, parent }
    }
}

impl Schema for V107 {
    fn version_key(&self) -> i32 {
        self.version_key
    }

    // JAVA: net.minecraft.util.datafix.schemas.V107#registerEntities
    fn register_entities(&self, schema: &dyn Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.parent.register_entities(schema);
        map.remove("Minecart");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_key_stored() {
        let v107 = V107::new(107, Box::new(BaseSchema::new(0, vec![])));
        assert_eq!(v107.version_key(), 107);
    }

    #[test]
    fn removes_minecart_preserves_others() {
        let parent = BaseSchema::new(106, vec!["Minecart", "Pig", "Zombie"]);
        let v107 = V107::new(107, Box::new(parent));
        let map = v107.register_entities(&v107);
        assert!(!map.contains_key("Minecart"));
        assert!(map.contains_key("Pig"));
        assert!(map.contains_key("Zombie"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn no_minecart_leaves_map_unchanged() {
        let parent = BaseSchema::new(106, vec!["Pig", "Cow"]);
        let v107 = V107::new(107, Box::new(parent));
        let map = v107.register_entities(&v107);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("Pig"));
        assert!(map.contains_key("Cow"));
    }

    #[test]
    fn empty_parent_returns_empty() {
        let v107 = V107::new(107, Box::new(BaseSchema::new(0, vec![])));
        let map = v107.register_entities(&v107);
        assert!(map.is_empty());
    }

    #[test]
    fn supplier_returns_type_template() {
        let parent = BaseSchema::new(106, vec!["Pig"]);
        let v107 = V107::new(107, Box::new(parent));
        let map = v107.register_entities(&v107);
        let supplier = map.get("Pig").expect("Pig should exist");
        assert_eq!(supplier(), TypeTemplate);
    }
}
