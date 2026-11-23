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

    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }
}

pub struct References;

impl References {
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(&self, _schema: &mut Schema) -> TemplateMap {
        BTreeMap::new()
    }
}

pub struct V4302 {
    pub base: NamespacedSchema,
}

impl V4302 {
    // JAVA: net.minecraft.util.datafix.schemas.V4302#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4302#registerBlockEntities
    pub fn register_block_entities(&self, schema: &mut Schema) -> TemplateMap {
        let mut map = self.base.register_block_entities(schema);
        schema.register_simple(&mut map, "minecraft:test_block");

        let s = schema.clone();
        schema.register(
            &mut map,
            "minecraft:test_instance_block",
            Box::new(move |_name| {
                Dsl::optional_fields(vec![
                    (
                        "data",
                        Dsl::optional_fields(vec![(
                            "error_message",
                            References::TEXT_COMPONENT.in_schema(&s),
                        )]),
                    ),
                    (
                        "errors",
                        Dsl::list(Dsl::optional_fields(vec![(
                            "text",
                            References::TEXT_COMPONENT.in_schema(&s),
                        )])),
                    ),
                ])
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
        let v = V4302::new(4302, Schema::new(4300));
        assert_eq!(v.base.version, 4302);
        assert_eq!(v.base.parent.version, 4300);
    }

    #[test]
    fn register_block_entities_contains_expected_keys() {
        let mut schema = Schema::new(4302);
        let v = V4302::new(4302, Schema::new(4300));
        let map = v.register_block_entities(&mut schema);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("minecraft:test_block"));
        assert!(map.contains_key("minecraft:test_instance_block"));
    }

    #[test]
    fn test_block_is_simple() {
        let mut schema = Schema::new(4302);
        let v = V4302::new(4302, Schema::new(4300));
        let map = v.register_block_entities(&mut schema);
        let tpl = map.get("minecraft:test_block").unwrap()();
        assert_eq!(tpl, TypeTemplate::Empty);
    }

    #[test]
    fn test_instance_block_template_structure() {
        let mut schema = Schema::new(4302);
        let v = V4302::new(4302, Schema::new(4300));
        let map = v.register_block_entities(&mut schema);
        let tpl = map.get("minecraft:test_instance_block").unwrap()();
        let expected = TypeTemplate::OptionalFields(vec![
            (
                "data".to_string(),
                TypeTemplate::OptionalFields(vec![(
                    "error_message".to_string(),
                    TypeTemplate::Ref("text_component".to_string()),
                )]),
            ),
            (
                "errors".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                    "text".to_string(),
                    TypeTemplate::Ref("text_component".to_string()),
                )]))),
            ),
        ]);
        assert_eq!(tpl, expected);
    }
}
