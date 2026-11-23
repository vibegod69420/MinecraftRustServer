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
pub struct V4648 {
    pub version: i32,
    base_entities: EntityTemplates,
}

impl V4648 {
    // JAVA: net.minecraft.util.datafix.schemas.V4648#<init>
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

    // JAVA: net.minecraft.util.datafix.schemas.V4648#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> EntityTemplates {
        let mut map = self.base_entities.clone();
        schema.register_simple(&mut map, "minecraft:nautilus");
        schema.register_simple(&mut map, "minecraft:zombie_nautilus");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version() {
        let v = V4648::new(4648, Schema);
        assert_eq!(v.version, 4648);
    }

    #[test]
    fn registers_nautilus_and_zombie_nautilus() {
        let schema = Schema;
        let v = V4648::new(4648, Schema);
        let map = v.register_entities(&schema);
        assert_eq!(
            map.get("minecraft:nautilus"),
            Some(&Some(default_template as TemplateSupplier))
        );
        assert_eq!(
            map.get("minecraft:zombie_nautilus"),
            Some(&Some(default_template as TemplateSupplier))
        );
    }

    #[test]
    fn preserves_base_entities() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert("minecraft:squid".to_string(), Some(default_template));
        let v = V4648::new(4648, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert!(map.contains_key("minecraft:squid"));
        assert!(map.contains_key("minecraft:nautilus"));
        assert!(map.contains_key("minecraft:zombie_nautilus"));
    }

    #[test]
    fn registered_suppliers_return_default_template() {
        let schema = Schema;
        let v = V4648::new(4648, Schema);
        let map = v.register_entities(&schema);
        assert_eq!(map["minecraft:nautilus"].unwrap()(), TypeTemplate);
        assert_eq!(map["minecraft:zombie_nautilus"].unwrap()(), TypeTemplate);
    }

    #[test]
    fn does_not_remove_existing_entries() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert("minecraft:drowned".to_string(), Some(default_template));
        let v = V4648::new(4648, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert!(map.contains_key("minecraft:drowned"));
        assert_eq!(map.len(), 3);
    }
}
