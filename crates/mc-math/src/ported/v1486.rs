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
pub struct V1486 {
    pub version: i32,
    base_entities: EntityTemplates,
}

impl V1486 {
    // JAVA: net.minecraft.util.datafix.schemas.V1486#<init>
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

    // JAVA: net.minecraft.util.datafix.schemas.V1486#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> EntityTemplates {
        let mut map = self.base_entities.clone();
        let cod = map.remove("minecraft:cod_mob").unwrap_or(None);
        map.insert("minecraft:cod".to_string(), cod);
        let salmon = map.remove("minecraft:salmon_mob").unwrap_or(None);
        map.insert("minecraft:salmon".to_string(), salmon);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version() {
        let v = V1486::new(1486, Schema);
        assert_eq!(v.version, 1486);
    }

    #[test]
    fn renames_cod_mob_and_salmon_mob() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert("minecraft:cod_mob".to_string(), Some(default_template));
        base.insert("minecraft:salmon_mob".to_string(), Some(default_template));
        let v = V1486::new(1486, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert!(!map.contains_key("minecraft:cod_mob"));
        assert!(!map.contains_key("minecraft:salmon_mob"));
        assert_eq!(
            map.get("minecraft:cod"),
            Some(&Some(default_template as TemplateSupplier))
        );
        assert_eq!(
            map.get("minecraft:salmon"),
            Some(&Some(default_template as TemplateSupplier))
        );
    }

    #[test]
    fn missing_old_keys_insert_none_like_java_null_put() {
        let schema = Schema;
        let v = V1486::new(1486, Schema);
        let map = v.register_entities(&schema);
        assert_eq!(map.get("minecraft:cod"), Some(&None));
        assert_eq!(map.get("minecraft:salmon"), Some(&None));
    }

    #[test]
    fn preserves_unrelated_entries() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert("minecraft:pufferfish".to_string(), Some(default_template));
        base.insert("minecraft:cod_mob".to_string(), Some(default_template));
        let v = V1486::new(1486, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert!(map.contains_key("minecraft:pufferfish"));
        assert!(map.contains_key("minecraft:cod"));
        assert_eq!(map.get("minecraft:salmon"), Some(&None));
    }

    #[test]
    fn preserves_null_valued_supplier_on_rename() {
        let schema = Schema;
        let mut base = EntityTemplates::new();
        base.insert("minecraft:cod_mob".to_string(), None);
        base.insert("minecraft:salmon_mob".to_string(), Some(default_template));
        let v = V1486::new(1486, Schema).with_base_entities(base);
        let map = v.register_entities(&schema);
        assert_eq!(map.get("minecraft:cod"), Some(&None));
        assert_eq!(
            map.get("minecraft:salmon"),
            Some(&Some(default_template as TemplateSupplier))
        );
    }
}
