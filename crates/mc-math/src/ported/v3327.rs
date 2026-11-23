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

    pub fn item_name(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref("References.ITEM_NAME".to_owned())
    }

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
pub struct V3327 {
    pub base: NamespacedSchema,
}

impl V3327 {
    // JAVA: net.minecraft.util.datafix.schemas.V3327#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema {
                version,
                parent_version,
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3327#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);

        let s = schema.clone();
        schema.register(&mut map, "minecraft:decorated_pot", move || {
            dsl::optional_fields(vec![
                ("shards", dsl::list(references::item_name(&s))),
                ("item", references::item_stack(&s)),
            ])
        });

        let s = schema.clone();
        schema.register(&mut map, "minecraft:suspicious_sand", move || {
            dsl::optional_fields(vec![("item", references::item_stack(&s))])
        });

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_expected_block_entities() {
        let v = V3327::new(3327, 3326);
        let schema = Schema { version: 3327 };
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("minecraft:decorated_pot"));
        assert!(map.contains_key("minecraft:suspicious_sand"));
    }

    #[test]
    fn decorated_pot_template_structure() {
        let v = V3327::new(3327, 3326);
        let schema = Schema { version: 3327 };
        let map = v.register_block_entities(&schema);
        let tpl = map.get("minecraft:decorated_pot").unwrap()();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![
                (
                    "shards".to_owned(),
                    TypeTemplate::List(Box::new(TypeTemplate::Ref(
                        "References.ITEM_NAME".to_owned()
                    )))
                ),
                (
                    "item".to_owned(),
                    TypeTemplate::Ref("References.ITEM_STACK".to_owned())
                ),
            ])
        );
    }

    #[test]
    fn suspicious_sand_template_structure() {
        let v = V3327::new(3327, 3326);
        let schema = Schema { version: 3327 };
        let map = v.register_block_entities(&schema);
        let tpl = map.get("minecraft:suspicious_sand").unwrap()();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "item".to_owned(),
                TypeTemplate::Ref("References.ITEM_STACK".to_owned())
            )])
        );
    }

    #[test]
    fn constructor_preserves_versions() {
        let v = V3327::new(3327, -3327);
        assert_eq!(v.base.version, 3327);
        assert_eq!(v.base.parent_version, -3327);
    }
}
