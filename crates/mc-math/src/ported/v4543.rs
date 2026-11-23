use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeTemplate;

pub type TemplateSupplier = fn() -> TypeTemplate;

fn simple_template() -> TypeTemplate {
    TypeTemplate
}

#[derive(Debug, Clone, Default)]
pub struct Schema;

impl Schema {
    pub fn register_simple(
        &self,
        entities: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
    ) {
        entities.insert(name.to_string(), simple_template);
    }
}

#[derive(Debug, Clone)]
pub struct NamespacedSchema {
    pub version_key: i32,
}

impl NamespacedSchema {
    pub fn new(version_key: i32, _parent: &Schema) -> Self {
        Self { version_key }
    }

    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

#[derive(Debug, Clone)]
pub struct V4543 {
    pub base: NamespacedSchema,
}

impl V4543 {
    // JAVA: net.minecraft.util.datafix.schemas.V4543#<init>
    pub fn new(version_key: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4543#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let mut entities = self.base.register_entities(schema);
        schema.register_simple(&mut entities, "minecraft:mannequin");
        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_key() {
        let schema = Schema;
        let v = V4543::new(4543, &schema);
        assert_eq!(v.base.version_key, 4543);
    }

    #[test]
    fn constructor_handles_integer_edge_cases() {
        let schema = Schema;
        let v_min = V4543::new(i32::MIN, &schema);
        let v_max = V4543::new(i32::MAX, &schema);
        let v_neg = V4543::new(-1, &schema);
        assert_eq!(v_min.base.version_key, i32::MIN);
        assert_eq!(v_max.base.version_key, i32::MAX);
        assert_eq!(v_neg.base.version_key, -1);
    }

    #[test]
    fn register_entities_adds_mannequin() {
        let schema = Schema;
        let v = V4543::new(4543, &schema);
        let entities = v.register_entities(&schema);
        assert_eq!(entities.len(), 1);
        assert!(entities.contains_key("minecraft:mannequin"));
    }

    #[test]
    fn registered_supplier_returns_type_template() {
        let schema = Schema;
        let v = V4543::new(4543, &schema);
        let entities = v.register_entities(&schema);
        let supplier = entities
            .get("minecraft:mannequin")
            .expect("missing minecraft:mannequin");
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn register_entities_is_deterministic() {
        let schema = Schema;
        let v = V4543::new(4543, &schema);
        let first = v.register_entities(&schema);
        let second = v.register_entities(&schema);
        let first_keys: Vec<&String> = first.keys().collect();
        let second_keys: Vec<&String> = second.keys().collect();
        assert_eq!(first_keys, second_keys);
        assert_eq!(first_keys, vec!["minecraft:mannequin"]);
    }
}
