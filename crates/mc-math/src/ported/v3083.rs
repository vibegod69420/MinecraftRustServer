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

    pub fn game_event_name(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref("References.GAME_EVENT_NAME".to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

#[derive(Debug, Clone)]
pub struct V3083 {
    pub base: NamespacedSchema,
}

impl V3083 {
    // JAVA: net.minecraft.util.datafix.schemas.V3083#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            base: NamespacedSchema {
                version,
                parent_version,
            },
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3083#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        let s = schema.clone();
        schema.register(&mut map, "minecraft:allay", move || {
            dsl::optional_fields(vec![
                ("Inventory", dsl::list(references::item_stack(&s))),
                (
                    "listener",
                    dsl::optional_fields(vec![(
                        "event",
                        dsl::optional_fields(vec![(
                            "game_event",
                            references::game_event_name(&s),
                        )]),
                    )]),
                ),
            ])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_allay_entity() {
        let v = V3083::new(3083, 3082);
        let schema = Schema { version: 3083 };
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:allay"));
    }

    #[test]
    fn allay_template_structure() {
        let v = V3083::new(3083, 3082);
        let schema = Schema { version: 3083 };
        let map = v.register_entities(&schema);
        let tpl = map.get("minecraft:allay").unwrap()();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![
                (
                    "Inventory".to_owned(),
                    TypeTemplate::List(Box::new(TypeTemplate::Ref(
                        "References.ITEM_STACK".to_owned()
                    )))
                ),
                (
                    "listener".to_owned(),
                    TypeTemplate::OptionalFields(vec![(
                        "event".to_owned(),
                        TypeTemplate::OptionalFields(vec![(
                            "game_event".to_owned(),
                            TypeTemplate::Ref("References.GAME_EVENT_NAME".to_owned())
                        )])
                    )])
                ),
            ])
        );
    }

    #[test]
    fn constructor_preserves_versions() {
        let v = V3083::new(3083, 0);
        assert_eq!(v.base.version, 3083);
        assert_eq!(v.base.parent_version, 0);
    }

    #[test]
    fn template_supplier_is_repeatable() {
        let v = V3083::new(3083, 3082);
        let schema = Schema { version: 3083 };
        let map = v.register_entities(&schema);
        let supplier = map.get("minecraft:allay").unwrap();
        assert_eq!(supplier(), supplier());
    }
}
