use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Empty,
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Reference(String),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

impl Schema {
    pub fn register(
        &self,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
        supplier: impl Fn() -> TypeTemplate + 'static,
    ) {
        map.insert(name.to_string(), Box::new(supplier));
    }
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(fields: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(fields)
    }

    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }
}

pub struct References;

impl References {
    pub fn entity_tree_in(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference("References.ENTITY_TREE".to_string())
    }
}

pub struct NamespacedSchema {
    pub version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, _parent: &Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

pub struct V3818 {
    pub base: NamespacedSchema,
}

impl V3818 {
    // JAVA: net.minecraft.util.datafix.schemas.V3818#<init>
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3818#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        let s = *schema;
        schema.register(&mut map, "minecraft:beehive", move || {
            Dsl::optional_fields(vec![(
                "bees".to_string(),
                Dsl::list(Dsl::optional_fields(vec![(
                    "entity_data".to_string(),
                    References::entity_tree_in(&s),
                )])),
            )])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(map: &BTreeMap<String, TemplateSupplier>, key: &str) -> TypeTemplate {
        map.get(key).expect("missing key")()
    }

    #[test]
    fn registers_beehive_with_nested_bees_entity_data() {
        let schema = Schema;
        let v = V3818::new(3818, &schema);
        let map = v.register_block_entities(&schema);

        assert_eq!(map.len(), 1);
        let tpl = resolve(&map, "minecraft:beehive");
        let expected = TypeTemplate::OptionalFields(vec![(
            "bees".to_string(),
            TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                "entity_data".to_string(),
                TypeTemplate::Reference("References.ENTITY_TREE".to_string()),
            )]))),
        )]);
        assert_eq!(tpl, expected);
    }

    #[test]
    fn supplier_is_repeatable() {
        let schema = Schema;
        let v = V3818::new(3818, &schema);
        let map = v.register_block_entities(&schema);
        let first = resolve(&map, "minecraft:beehive");
        let second = resolve(&map, "minecraft:beehive");
        assert_eq!(first, second);
    }

    #[test]
    fn version_integer_edge_cases() {
        let schema = Schema;
        for version in [0_i32, -1, 3818, i32::MIN, i32::MAX] {
            let v = V3818::new(version, &schema);
            assert_eq!(v.base.version, version);
        }
    }

    #[test]
    fn does_not_register_unrelated_block_entities() {
        let schema = Schema;
        let v = V3818::new(3818, &schema);
        let map = v.register_block_entities(&schema);
        assert!(map.contains_key("minecraft:beehive"));
        assert!(!map.contains_key("minecraft:bee_nest"));
    }
}
