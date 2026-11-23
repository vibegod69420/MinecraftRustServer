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

// JAVA: net.minecraft.util.datafix.schemas.V143
pub struct V143 {
    version_key: i32,
    parent: Box<dyn Schema>,
}

impl V143 {
    // JAVA: net.minecraft.util.datafix.schemas.V143#<init>
    pub fn new(version_key: i32, parent: Box<dyn Schema>) -> Self {
        Self { version_key, parent }
    }
}

impl Schema for V143 {
    fn version_key(&self) -> i32 {
        self.version_key
    }

    // JAVA: net.minecraft.util.datafix.schemas.V143#registerEntities
    fn register_entities(&self, schema: &dyn Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.parent.register_entities(schema);
        map.remove("TippedArrow");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_key_stored() {
        let v143 = V143::new(143, Box::new(BaseSchema::new(0, vec![])));
        assert_eq!(v143.version_key(), 143);
    }

    #[test]
    fn removes_tipped_arrow_preserves_others() {
        let parent = BaseSchema::new(142, vec!["TippedArrow", "Arrow", "Skeleton"]);
        let v143 = V143::new(143, Box::new(parent));
        let map = v143.register_entities(&v143);
        assert!(!map.contains_key("TippedArrow"));
        assert!(map.contains_key("Arrow"));
        assert!(map.contains_key("Skeleton"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn no_tipped_arrow_leaves_map_unchanged() {
        let parent = BaseSchema::new(142, vec!["Arrow", "Zombie"]);
        let v143 = V143::new(143, Box::new(parent));
        let map = v143.register_entities(&v143);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("Arrow"));
        assert!(map.contains_key("Zombie"));
    }

    #[test]
    fn empty_parent_returns_empty() {
        let v143 = V143::new(143, Box::new(BaseSchema::new(0, vec![])));
        let map = v143.register_entities(&v143);
        assert!(map.is_empty());
    }

    #[test]
    fn supplier_returns_type_template() {
        let parent = BaseSchema::new(142, vec!["Arrow"]);
        let v143 = V143::new(143, Box::new(parent));
        let map = v143.register_entities(&v143);
        let supplier = map.get("Arrow").expect("Arrow should exist");
        assert_eq!(supplier(), TypeTemplate);
    }
}
