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

    pub fn register_type(
        &mut self,
        recursive: bool,
        type_ref: &TypeReference,
        template: TemplateSupplier,
    ) -> RegisteredType {
        RegisteredType {
            recursive,
            type_ref: type_ref.0.to_string(),
            template: template(),
        }
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
        map.insert(
            name.to_string(),
            Box::new(|| TypeTemplate::Empty),
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredType {
    pub recursive: bool,
    pub type_ref: String,
    pub template: TypeTemplate,
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

    pub fn fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::Fields(
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
    pub const UNTAGGED_SPAWNER: TypeReference = TypeReference("untagged_spawner");
    pub const ENTITY_TREE: TypeReference = TypeReference("entity_tree");
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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    pub fn register_types(
        &self,
        _schema: &mut Schema,
        _entities: &mut TemplateMap,
        _block_entities: &mut TemplateMap,
    ) {
    }
}

pub struct V2831 {
    pub base: NamespacedSchema,
}

impl V2831 {
    // JAVA: net.minecraft.util.datafix.schemas.V2831#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2831#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entities: &mut TemplateMap,
        block_entities: &mut TemplateMap,
    ) -> RegisteredType {
        self.base.register_types(schema, entities, block_entities);
        let s = schema.clone();
        schema.register_type(
            true,
            &References::UNTAGGED_SPAWNER,
            Box::new(move || {
                Dsl::optional_fields(vec![
                    (
                        "SpawnPotentials",
                        Dsl::list(Dsl::fields(vec![(
                            "data",
                            Dsl::fields(vec![(
                                "entity",
                                References::ENTITY_TREE.in_schema(&s),
                            )]),
                        )])),
                    ),
                    (
                        "SpawnData",
                        Dsl::fields(vec![(
                            "entity",
                            References::ENTITY_TREE.in_schema(&s),
                        )]),
                    ),
                ])
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_schema() -> Schema {
        Schema::new(2831)
    }

    #[test]
    fn constructor_stores_version_and_parent() {
        let parent = Schema::new(2800);
        let v = V2831::new(2831, parent.clone());
        assert_eq!(v.base.version, 2831);
        assert_eq!(v.base.parent.version, parent.version);
    }

    #[test]
    fn register_types_registers_untagged_spawner() {
        let mut schema = mock_schema();
        let mut entities = TemplateMap::new();
        let mut block_entities = TemplateMap::new();
        let v = V2831::new(2831, Schema::new(2800));
        let registered = v.register_types(&mut schema, &mut entities, &mut block_entities);
        assert!(registered.recursive);
        assert_eq!(registered.type_ref, "untagged_spawner");
    }

    #[test]
    fn register_types_template_structure() {
        let mut schema = mock_schema();
        let mut entities = TemplateMap::new();
        let mut block_entities = TemplateMap::new();
        let v = V2831::new(2831, Schema::new(2800));
        let registered = v.register_types(&mut schema, &mut entities, &mut block_entities);
        let expected = TypeTemplate::OptionalFields(vec![
            (
                "SpawnPotentials".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Fields(vec![(
                    "data".to_string(),
                    TypeTemplate::Fields(vec![(
                        "entity".to_string(),
                        TypeTemplate::Ref("entity_tree".to_string()),
                    )]),
                )]))),
            ),
            (
                "SpawnData".to_string(),
                TypeTemplate::Fields(vec![(
                    "entity".to_string(),
                    TypeTemplate::Ref("entity_tree".to_string()),
                )]),
            ),
        ]);
        assert_eq!(registered.template, expected);
    }

    #[test]
    fn register_types_preserves_existing_maps() {
        let mut schema = mock_schema();
        let mut entities = TemplateMap::new();
        entities.insert("pre".to_string(), Box::new(|| TypeTemplate::Empty));
        let mut block_entities = TemplateMap::new();
        let v = V2831::new(2831, Schema::new(2800));
        let _ = v.register_types(&mut schema, &mut entities, &mut block_entities);
        assert!(entities.contains_key("pre"));
        assert_eq!(entities.len(), 1);
        assert!(block_entities.is_empty());
    }
}
