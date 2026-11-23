use std::collections::HashMap;

/// Minimal placeholder for a DataFixerUpper `TypeTemplate`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeTemplate;

/// Minimal schema interface used by this datafix schema port.
pub trait Schema {
    fn register_simple(&self, map: &mut HashMap<String, fn() -> TypeTemplate>, name: &str);
}

/// Minimal base for `net.minecraft.util.datafix.schemas.NamespacedSchema`.
#[derive(Debug, Clone)]
pub struct NamespacedSchema {
    version_key: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version_key: i32, _parent: &dyn Schema) -> Self {
        Self { version_key }
    }

    pub fn version_key(&self) -> i32 {
        self.version_key
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &dyn Schema) -> HashMap<String, fn() -> TypeTemplate> {
        HashMap::new()
    }
}

/// Port of `net.minecraft.util.datafix.schemas.V2522`.
#[derive(Debug, Clone)]
pub struct V2522 {
    base: NamespacedSchema,
}

impl V2522 {
    // JAVA: net.minecraft.util.datafix.schemas.V2522#<init>
    pub fn new(version_key: i32, parent: &dyn Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version_key, parent),
        }
    }

    pub fn version_key(&self) -> i32 {
        self.base.version_key()
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2522#registerEntities
    pub fn register_entities(
        &self,
        schema: &dyn Schema,
    ) -> HashMap<String, fn() -> TypeTemplate> {
        let mut map = self.base.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:zoglin");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn default_template() -> TypeTemplate {
        TypeTemplate
    }

    struct MockSchema {
        registered: RefCell<Vec<String>>,
    }

    impl MockSchema {
        fn new() -> Self {
            Self {
                registered: RefCell::new(Vec::new()),
            }
        }

        fn registered_names(&self) -> Vec<String> {
            self.registered.borrow().clone()
        }
    }

    impl Schema for MockSchema {
        fn register_simple(
            &self,
            map: &mut HashMap<String, fn() -> TypeTemplate>,
            name: &str,
        ) {
            self.registered.borrow_mut().push(name.to_string());
            map.insert(name.to_string(), default_template);
        }
    }

    #[test]
    fn constructor_stores_version_key() {
        let schema = MockSchema::new();
        let v = V2522::new(2522, &schema);
        assert_eq!(v.version_key(), 2522);
    }

    #[test]
    fn constructor_preserves_negative_version_key() {
        let schema = MockSchema::new();
        let v = V2522::new(-1, &schema);
        assert_eq!(v.version_key(), -1);
    }

    #[test]
    fn register_entities_registers_zoglin() {
        let schema = MockSchema::new();
        let v = V2522::new(2522, &schema);
        let _ = v.register_entities(&schema);
        assert_eq!(schema.registered_names(), vec!["minecraft:zoglin"]);
    }

    #[test]
    fn register_entities_returns_map_with_zoglin() {
        let schema = MockSchema::new();
        let v = V2522::new(2522, &schema);
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:zoglin"));
        let supplier = map.get("minecraft:zoglin").expect("zoglin supplier missing");
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn register_entities_can_be_called_multiple_times() {
        let schema = MockSchema::new();
        let v = V2522::new(2522, &schema);
        let m1 = v.register_entities(&schema);
        let m2 = v.register_entities(&schema);
        assert_eq!(m1.len(), 1);
        assert_eq!(m2.len(), 1);
        assert_eq!(
            schema.registered_names(),
            vec!["minecraft:zoglin", "minecraft:zoglin"]
        );
    }
}
