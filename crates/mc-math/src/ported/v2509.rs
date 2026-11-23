use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypeTemplate;

pub type TemplateSupplier = fn() -> TypeTemplate;
pub type EntityTemplates = HashMap<String, Option<TemplateSupplier>>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Schema;

impl Schema {
    pub fn register_simple(&self, map: &mut EntityTemplates, name: &str) {
        map.insert(name.to_string(), Some(default_template));
    }
}

fn default_template() -> TypeTemplate {
    TypeTemplate
}

#[derive(Debug, Clone)]
pub struct V2509 {
    pub version: i32,
    base_entities: EntityTemplates,
}

impl V2509 {
    // JAVA: net.minecraft.util.datafix.schemas.V2509#<init>
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self {
            version,
            base_entities: HashMap::new(),
        }
    }

    pub fn with_base_entities(mut self, base_entities: EntityTemplates) -> Self {
        self.base_entities = base_entities;
        self
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2509#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> EntityTemplates {
        let mut map = self.base_entities.clone();
        map.remove("minecraft:zombie_pigman");
        schema.register_simple(&mut map, "minecraft:zombified_piglin");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version() {
        let v = V2509::new(2509, Schema);
        assert_eq!(v.version, 2509);
    }

    #[test]
    fn removes_zombie_pigman_and_adds_zombified_piglin() {
        let schema = Schema;
        let v = V2509::new(2509, Schema);
        let map = v.register_entities(&schema);
        assert!(!map.contains_key("minecraft:zombie_pigman"));
        assert_eq!(
            map.get("minecraft:zombified_piglin"),
            Some(&Some(default_template as TemplateSupplier))
        );
    }

    #[test]
    fn removes_existing_zombie_pigman_from_base() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert(
            "minecraft:zombie_pigman".to_string(),
            Some(default_template),
        );
        let v = V2509::new(2509, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert!(!map.contains_key("minecraft:zombie_pigman"));
        assert!(map.contains_key("minecraft:zombified_piglin"));
    }

    #[test]
    fn preserves_unrelated_base_entries() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert("minecraft:zombie".to_string(), Some(default_template));
        let v = V2509::new(2509, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert!(map.contains_key("minecraft:zombie"));
        assert!(map.contains_key("minecraft:zombified_piglin"));
    }

    #[test]
    fn registered_supplier_returns_default_template() {
        let schema = Schema;
        let v = V2509::new(2509, Schema);
        let map = v.register_entities(&schema);
        let supplier = map["minecraft:zombified_piglin"].expect("supplier present");
        assert_eq!(supplier(), TypeTemplate);
    }
}
