use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reference(pub &'static str);

impl Reference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref(self.0.to_string())
    }
}

pub struct References;

impl References {
    pub const ITEM_STACK: Reference = Reference("minecraft:item_stack");
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Empty,
    Ref(String),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        )
    }
}

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn register(
        &self,
        map: &mut BTreeMap<String, TypeTemplateSupplier>,
        name: &str,
        template: TypeTemplate,
    ) {
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }
}

pub struct NamespacedSchema;

impl NamespacedSchema {
    // Minimal base stub: provides default entities for faithful override behavior.
    pub fn register_entities(_schema: &Schema) -> BTreeMap<String, TypeTemplateSupplier> {
        let mut map: BTreeMap<String, TypeTemplateSupplier> = BTreeMap::new();
        map.insert(
            "minecraft:potion".to_string(),
            Box::new(|| TypeTemplate::Empty),
        );
        map
    }
}

#[derive(Clone, Debug, Default)]
pub struct V4306 {
    version: i32,
}

impl V4306 {
    // JAVA: net.minecraft.util.datafix.schemas.V4306#<init>(int, Schema)
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4306#registerEntities(Schema)
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TypeTemplateSupplier> {
        let mut map = NamespacedSchema::register_entities(schema);
        map.remove("minecraft:potion");
        let item_stack = References::ITEM_STACK.in_schema(schema);
        schema.register(
            &mut map,
            "minecraft:splash_potion",
            Dsl::optional_fields(vec![("Item", item_stack.clone())]),
        );
        schema.register(
            &mut map,
            "minecraft:lingering_potion",
            Dsl::optional_fields(vec![("Item", item_stack)]),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v4306_new_stores_version() {
        let v = V4306::new(4306, Schema::new(0));
        assert_eq!(v.version, 4306);
    }

    #[test]
    fn v4306_removes_potion_and_adds_replacements() {
        let schema = Schema::new(4306);
        let v = V4306::new(4306, Schema::new(0));
        let map = v.register_entities(&schema);
        assert!(!map.contains_key("minecraft:potion"));
        assert!(map.contains_key("minecraft:splash_potion"));
        assert!(map.contains_key("minecraft:lingering_potion"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn v4306_splash_potion_has_item_field() {
        let schema = Schema::new(4306);
        let v = V4306::new(4306, Schema::new(0));
        let map = v.register_entities(&schema);
        let tpl = map["minecraft:splash_potion"]();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Item".to_string(),
                TypeTemplate::Ref("minecraft:item_stack".to_string())
            )])
        );
    }

    #[test]
    fn v4306_lingering_potion_has_item_field() {
        let schema = Schema::new(4306);
        let v = V4306::new(4306, Schema::new(0));
        let map = v.register_entities(&schema);
        let tpl = map["minecraft:lingering_potion"]();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Item".to_string(),
                TypeTemplate::Ref("minecraft:item_stack".to_string())
            )])
        );
    }

    #[test]
    fn v4306_remove_on_missing_potion_is_safe() {
        // Verifies Map.remove semantics: removing absent key is no-op.
        let mut map: BTreeMap<String, TypeTemplateSupplier> = BTreeMap::new();
        map.remove("minecraft:potion");
        assert!(map.is_empty());
    }
}
