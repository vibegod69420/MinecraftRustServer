use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TemplateSupplier = fn() -> TypeTemplate;

fn empty_template() -> TypeTemplate {
    TypeTemplate
}

#[derive(Clone, Debug)]
pub struct Schema {
    version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn register_simple(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str) {
        map.insert(name.to_string(), empty_template);
    }
}

#[derive(Clone, Debug)]
pub struct NamespacedSchema {
    version: i32,
    parent: Schema,
}

impl NamespacedSchema {
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn parent(&self) -> &Schema {
        &self.parent
    }

    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }

    pub fn register_simple(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str) {
        map.insert(name.to_string(), empty_template);
    }
}

#[derive(Clone, Debug)]
pub struct V2519 {
    base: NamespacedSchema,
}

impl V2519 {
    // JAVA: net.minecraft.util.datafix.schemas.V2519#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    pub fn version(&self) -> i32 {
        self.base.version()
    }

    pub fn parent(&self) -> &Schema {
        self.base.parent()
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2519#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:strider");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_strider() {
        let schema = V2519::new(2519, Schema::new(0));
        let map = schema.register_entities(&Schema::new(2519));
        assert!(map.contains_key("minecraft:strider"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn constructor_stores_version_and_parent() {
        let schema = V2519::new(2519, Schema::new(2500));
        assert_eq!(schema.version(), 2519);
        assert_eq!(schema.parent().version(), 2500);
    }

    #[test]
    fn negative_and_min_version_edge_cases() {
        let schema = V2519::new(-1, Schema::new(i32::MIN));
        assert_eq!(schema.version(), -1);
        assert_eq!(schema.parent().version(), i32::MIN);
        let map = schema.register_entities(&Schema::new(i32::MAX));
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:strider"));
    }

    #[test]
    fn supplier_returns_template() {
        let schema = V2519::new(2519, Schema::new(0));
        let map = schema.register_entities(&Schema::new(0));
        let supplier = map.get("minecraft:strider").expect("missing strider");
        assert_eq!(supplier(), TypeTemplate);
    }
}
