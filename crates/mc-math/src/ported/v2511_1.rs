use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Unit,
    Reference {
        name: String,
        schema_version: i32,
    },
    OptionalFields(Vec<(String, TypeTemplate)>),
}

pub type TypeTemplateSupplier = Arc<dyn Fn() -> TypeTemplate + Send + Sync>;

pub struct References;

impl References {
    pub const ITEM_STACK: &'static str = "item_stack";

    pub fn in_schema(reference_name: &str, schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference {
            name: reference_name.to_string(),
            schema_version: schema.version_key,
        }
    }
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(fields: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(fields)
    }
}

#[derive(Debug, Default)]
pub struct Schema {
    pub version_key: i32,
    pub registered_entities: Vec<String>,
}

impl Schema {
    pub fn new(version_key: i32) -> Self {
        Self {
            version_key,
            registered_entities: Vec::new(),
        }
    }

    pub fn register<F>(
        &mut self,
        map: &mut HashMap<String, TypeTemplateSupplier>,
        name: &str,
        factory: F,
    ) where
        F: Fn(&Schema) -> TypeTemplate + Send + Sync + 'static,
    {
        let template = factory(self);
        map.insert(name.to_string(), Arc::new(move || template.clone()));
        self.registered_entities.push(name.to_string());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2511_1 {
    pub version_key: i32,
}

impl V2511_1 {
    // JAVA: net.minecraft.util.datafix.schemas.V2511_1#<init>
    pub fn new(version_key: i32, _parent: Option<&Schema>) -> Self {
        Self { version_key }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2511_1#registerEntities
    pub fn register_entities(
        &self,
        schema: &mut Schema,
        mut map: HashMap<String, TypeTemplateSupplier>,
    ) -> HashMap<String, TypeTemplateSupplier> {
        // super.registerEntities(schema) result passed as `map`
        let schema_version = schema.version_key;
        schema.register(&mut map, "minecraft:potion", move |_unused| {
            Dsl::optional_fields(vec![(
                "Item".to_string(),
                TypeTemplate::Reference {
                    name: References::ITEM_STACK.to_string(),
                    schema_version,
                },
            )])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_key_including_edges() {
        assert_eq!(V2511_1::new(2511, None).version_key, 2511);
        assert_eq!(V2511_1::new(-1, None).version_key, -1);
        assert_eq!(V2511_1::new(i32::MIN, None).version_key, i32::MIN);
    }

    #[test]
    fn register_entities_adds_potion_with_item_stack_reference() {
        let v = V2511_1::new(2511, None);
        let mut schema = Schema::new(2511);
        let base = HashMap::new();

        let result = v.register_entities(&mut schema, base);

        assert!(result.contains_key("minecraft:potion"));
        assert_eq!(
            schema.registered_entities,
            vec!["minecraft:potion".to_string()]
        );
        let template = (result.get("minecraft:potion").unwrap())();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![(
                "Item".to_string(),
                TypeTemplate::Reference {
                    name: "item_stack".to_string(),
                    schema_version: 2511,
                }
            )])
        );
    }

    #[test]
    fn register_entities_preserves_existing_entries() {
        let v = V2511_1::new(2511, None);
        let mut schema = Schema::new(2511);
        let mut base = HashMap::new();
        base.insert(
            "minecraft:arrow".to_string(),
            Arc::new(|| TypeTemplate::Unit) as TypeTemplateSupplier,
        );

        let result = v.register_entities(&mut schema, base);

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("minecraft:arrow"));
        assert!(result.contains_key("minecraft:potion"));
    }

    #[test]
    fn register_entities_uses_outer_schema_version_like_java_capture() {
        // Java lambda takes $$1x but uses $$0; we preserve the $$0 version capture.
        let v = V2511_1::new(2511, None);
        let mut schema = Schema::new(i32::MAX);
        let result = v.register_entities(&mut schema, HashMap::new());
        let template = (result.get("minecraft:potion").unwrap())();
        match template {
            TypeTemplate::OptionalFields(fields) => match &fields[0].1 {
                TypeTemplate::Reference { schema_version, .. } => {
                    assert_eq!(*schema_version, i32::MAX)
                }
                other => panic!("unexpected template: {:?}", other),
            },
            other => panic!("unexpected template: {:?}", other),
        }
    }
}
