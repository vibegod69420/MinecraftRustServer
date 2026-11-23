use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTemplate {
    Unit,
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Ref(String),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Debug, Clone)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn register<F>(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str, supplier: F)
    where
        F: Fn() -> TypeTemplate + 'static,
    {
        let _ = self;
        map.insert(name.to_owned(), Box::new(supplier));
    }
}

pub mod dsl {
    use super::TypeTemplate;

    pub fn optional_fields(fields: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            fields.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        )
    }

    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }
}

pub mod references {
    use super::{Schema, TypeTemplate};

    pub fn item_stack(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref("References.ITEM_STACK".to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    pub fn register_block_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

#[derive(Debug, Clone)]
pub struct V808 {
    pub base: NamespacedSchema,
}

impl V808 {
    // JAVA: net.minecraft.util.datafix.schemas.V808#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema {
                version,
                parent_version,
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V808#registerInventory
    pub fn register_inventory(
        schema: &Schema,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
    ) {
        let s = schema.clone();
        schema.register(map, name, move || {
            dsl::optional_fields(vec![("Items", dsl::list(references::item_stack(&s)))])
        });
    }

    // JAVA: net.minecraft.util.datafix.schemas.V808#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        Self::register_inventory(schema, &mut map, "minecraft:shulker_box");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_shulker_box_block_entity() {
        let v808 = V808::new(808, 807);
        let schema = Schema { version: 808 };
        let map = v808.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:shulker_box"));
    }

    #[test]
    fn shulker_box_template_structure() {
        let v808 = V808::new(808, 807);
        let schema = Schema { version: 808 };
        let map = v808.register_block_entities(&schema);
        let tpl = map.get("minecraft:shulker_box").unwrap()();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Items".to_owned(),
                TypeTemplate::List(Box::new(TypeTemplate::Ref(
                    "References.ITEM_STACK".to_owned()
                )))
            )])
        );
    }

    #[test]
    fn register_inventory_supports_custom_name() {
        let schema = Schema { version: 808 };
        let mut map = BTreeMap::new();
        V808::register_inventory(&schema, &mut map, "minecraft:custom_inventory");
        assert_eq!(map.len(), 1);
        let tpl = map.get("minecraft:custom_inventory").unwrap()();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Items".to_owned(),
                TypeTemplate::List(Box::new(TypeTemplate::Ref(
                    "References.ITEM_STACK".to_owned()
                )))
            )])
        );
    }

    #[test]
    fn constructor_preserves_versions_including_negative_parent() {
        let v808 = V808::new(808, -1);
        assert_eq!(v808.base.version, 808);
        assert_eq!(v808.base.parent_version, -1);
    }
}
