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
    pub fn item_stack_in(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference("References.ITEM_STACK".to_string())
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

pub struct V3204 {
    pub base: NamespacedSchema,
}

impl V3204 {
    // JAVA: net.minecraft.util.datafix.schemas.V3204#<init>
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3204#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        let s = *schema;
        schema.register(&mut map, "minecraft:chiseled_bookshelf", move || {
            Dsl::optional_fields(vec![(
                "Items".to_string(),
                Dsl::list(References::item_stack_in(&s)),
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
    fn registers_chiseled_bookshelf_with_items() {
        let schema = Schema;
        let v = V3204::new(3204, &schema);
        let map = v.register_block_entities(&schema);

        assert_eq!(map.len(), 1);
        let tpl = resolve(&map, "minecraft:chiseled_bookshelf");
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Items".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Reference(
                    "References.ITEM_STACK".to_string()
                ))),
            )])
        );
    }

    #[test]
    fn supplier_is_repeatable() {
        let schema = Schema;
        let v = V3204::new(3204, &schema);
        let map = v.register_block_entities(&schema);
        let first = resolve(&map, "minecraft:chiseled_bookshelf");
        let second = resolve(&map, "minecraft:chiseled_bookshelf");
        assert_eq!(first, second);
    }

    #[test]
    fn version_integer_edge_cases() {
        let schema = Schema;
        for version in [0_i32, -1, 3204, i32::MIN, i32::MAX] {
            let v = V3204::new(version, &schema);
            assert_eq!(v.base.version, version);
        }
    }

    #[test]
    fn does_not_register_unrelated_block_entities() {
        let schema = Schema;
        let v = V3204::new(3204, &schema);
        let map = v.register_block_entities(&schema);
        assert!(map.contains_key("minecraft:chiseled_bookshelf"));
        assert!(!map.contains_key("minecraft:trapped_chest"));
    }
}
