use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTemplate {
    Empty,
    Ref(String),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Fields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;
pub type TemplateMap = BTreeMap<String, TemplateSupplier>;

#[derive(Debug, Clone)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn register(
        &mut self,
        map: &mut TemplateMap,
        name: &str,
        factory: Box<dyn Fn(&str) -> TypeTemplate>,
    ) {
        let n = name.to_string();
        let f = factory;
        map.insert(name.to_string(), Box::new(move || f(&n)));
    }

    pub fn register_simple(&mut self, map: &mut TemplateMap, name: &str) {
        map.insert(name.to_string(), Box::new(|| TypeTemplate::Empty));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeReference(pub &'static str);

impl TypeReference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref(self.0.to_string())
    }
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }
}

pub struct References;

impl References {
    pub const ITEM_STACK: TypeReference = TypeReference("item_stack");
    pub const BLOCK_STATE: TypeReference = TypeReference("block_state");
    pub const TEXT_COMPONENT: TypeReference = TypeReference("text_component");
}

pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &mut Schema) -> TemplateMap {
        BTreeMap::new()
    }
}

pub struct V3325 {
    pub base: NamespacedSchema,
}

impl V3325 {
    // JAVA: net.minecraft.util.datafix.schemas.V3325#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3325#registerEntities
    pub fn register_entities(&self, schema: &mut Schema) -> TemplateMap {
        let mut map = self.base.register_entities(schema);
        let s = schema.clone();

        let s1 = s.clone();
        schema.register(
            &mut map,
            "minecraft:item_display",
            Box::new(move |_name| {
                Dsl::optional_fields(vec![("item", References::ITEM_STACK.in_schema(&s1))])
            }),
        );

        let s2 = s.clone();
        schema.register(
            &mut map,
            "minecraft:block_display",
            Box::new(move |_name| {
                Dsl::optional_fields(vec![(
                    "block_state",
                    References::BLOCK_STATE.in_schema(&s2),
                )])
            }),
        );

        let s3 = s.clone();
        schema.register(
            &mut map,
            "minecraft:text_display",
            Box::new(move |_name| {
                Dsl::optional_fields(vec![("text", References::TEXT_COMPONENT.in_schema(&s3))])
            }),
        );

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_and_parent() {
        let v = V3325::new(3325, Schema::new(3320));
        assert_eq!(v.base.version, 3325);
        assert_eq!(v.base.parent.version, 3320);
    }

    #[test]
    fn register_entities_contains_expected_keys() {
        let mut schema = Schema::new(3325);
        let v = V3325::new(3325, Schema::new(3320));
        let map = v.register_entities(&mut schema);
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("minecraft:item_display"));
        assert!(map.contains_key("minecraft:block_display"));
        assert!(map.contains_key("minecraft:text_display"));
    }

    #[test]
    fn item_display_template() {
        let mut schema = Schema::new(3325);
        let v = V3325::new(3325, Schema::new(3320));
        let map = v.register_entities(&mut schema);
        let tpl = map.get("minecraft:item_display").unwrap()();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "item".to_string(),
                TypeTemplate::Ref("item_stack".to_string())
            )])
        );
    }

    #[test]
    fn block_and_text_display_templates() {
        let mut schema = Schema::new(3325);
        let v = V3325::new(3325, Schema::new(3320));
        let map = v.register_entities(&mut schema);
        let block_tpl = map.get("minecraft:block_display").unwrap()();
        let text_tpl = map.get("minecraft:text_display").unwrap()();
        assert_eq!(
            block_tpl,
            TypeTemplate::OptionalFields(vec![(
                "block_state".to_string(),
                TypeTemplate::Ref("block_state".to_string())
            )])
        );
        assert_eq!(
            text_tpl,
            TypeTemplate::OptionalFields(vec![(
                "text".to_string(),
                TypeTemplate::Ref("text_component".to_string())
            )])
        );
    }
}
