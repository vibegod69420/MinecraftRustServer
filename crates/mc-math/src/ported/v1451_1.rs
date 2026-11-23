use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeTemplate(pub String);

pub mod references {
    use super::TypeTemplate;

    pub fn entity_tree() -> TypeTemplate {
        TypeTemplate("entity_tree".to_string())
    }

    pub fn block_entity() -> TypeTemplate {
        TypeTemplate("block_entity".to_string())
    }

    pub fn block_name() -> TypeTemplate {
        TypeTemplate("block_name".to_string())
    }

    pub fn block_state() -> TypeTemplate {
        TypeTemplate("block_state".to_string())
    }

    pub fn chunk() -> &'static str {
        "chunk"
    }
}

pub mod dsl {
    use super::TypeTemplate;

    pub fn fields(_name: &str, _body: TypeTemplate) -> TypeTemplate {
        TypeTemplate("fields".to_string())
    }

    pub fn optional_fields(_fields: &[(&str, TypeTemplate)]) -> TypeTemplate {
        TypeTemplate("optional_fields".to_string())
    }

    pub fn list(_inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate("list".to_string())
    }

    pub fn or(_a: TypeTemplate, _b: TypeTemplate) -> TypeTemplate {
        TypeTemplate("or".to_string())
    }

    pub fn remainder() -> TypeTemplate {
        TypeTemplate("remainder".to_string())
    }
}

pub struct SchemaRegistry {
    pub types: BTreeMap<String, TypeTemplate>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            types: BTreeMap::new(),
        }
    }

    pub fn register_type(
        &mut self,
        _recursive: bool,
        name: &str,
        template_factory: impl Fn() -> TypeTemplate,
    ) {
        self.types.insert(name.to_string(), template_factory());
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema (minimal stub for V1451_1)
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    pub fn register_types(
        &self,
        _schema: &mut SchemaRegistry,
        _entity_types: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        _block_entity_types: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
    ) {
    }
}

// JAVA: net.minecraft.util.datafix.schemas.V1451_1
pub struct V1451_1 {
    pub base: NamespacedSchema,
}

impl V1451_1 {
    // JAVA: net.minecraft.util.datafix.schemas.V1451_1#V1451_1
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent_version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1451_1#registerTypes
    pub fn register_types(
        &self,
        schema: &mut SchemaRegistry,
        entity_types: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
        block_entity_types: &mut BTreeMap<String, Box<dyn Fn() -> TypeTemplate>>,
    ) {
        self.base
            .register_types(schema, entity_types, block_entity_types);
        schema.register_type(false, references::chunk(), || {
            dsl::fields(
                "Level",
                dsl::optional_fields(&[
                    ("Entities", dsl::list(references::entity_tree())),
                    (
                        "TileEntities",
                        dsl::list(dsl::or(references::block_entity(), dsl::remainder())),
                    ),
                    (
                        "TileTicks",
                        dsl::list(dsl::fields("i", references::block_name())),
                    ),
                    (
                        "Sections",
                        dsl::list(dsl::optional_fields(&[(
                            "Palette",
                            dsl::list(references::block_state()),
                        )])),
                    ),
                ]),
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1451_1_registers_chunk_type() {
        let v = V1451_1::new(1451, 1451);
        let mut schema = SchemaRegistry::new();
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut schema, &mut entities, &mut block_entities);

        assert!(schema.types.contains_key("chunk"));
        assert_eq!(schema.types.get("chunk").unwrap().0, "fields");
    }

    #[test]
    fn v1451_1_preserves_version() {
        let v = V1451_1::new(1451, 1450);
        assert_eq!(v.base.version, 1451);
        assert_eq!(v.base.parent_version, 1450);
    }

    #[test]
    fn v1451_1_dsl_builders_return_expected_tags() {
        assert_eq!(dsl::list(references::entity_tree()).0, "list");
        assert_eq!(dsl::or(references::block_entity(), dsl::remainder()).0, "or");
        assert_eq!(dsl::fields("i", references::block_name()).0, "fields");
        assert_eq!(dsl::remainder().0, "remainder");
    }

    #[test]
    fn v1451_1_references_return_expected_tags() {
        assert_eq!(references::entity_tree().0, "entity_tree");
        assert_eq!(references::block_entity().0, "block_entity");
        assert_eq!(references::block_name().0, "block_name");
        assert_eq!(references::block_state().0, "block_state");
        assert_eq!(references::chunk(), "chunk");
    }

    #[test]
    fn v1451_1_register_types_does_not_touch_entity_maps() {
        let v = V1451_1::new(1451, 1451);
        let mut schema = SchemaRegistry::new();
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut schema, &mut entities, &mut block_entities);
        assert!(entities.is_empty());
        assert!(block_entities.is_empty());
    }
}
