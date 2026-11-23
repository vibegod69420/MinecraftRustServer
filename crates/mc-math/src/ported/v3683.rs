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
    pub const BLOCK_STATE: &'static str = "block_state";

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
pub struct V3683 {
    pub version_key: i32,
}

impl V3683 {
    // JAVA: net.minecraft.util.datafix.schemas.V3683#<init>
    pub fn new(version_key: i32, _parent: Option<&Schema>) -> Self {
        Self { version_key }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3683#registerEntities
    pub fn register_entities(
        &self,
        schema: &mut Schema,
        mut map: HashMap<String, TypeTemplateSupplier>,
    ) -> HashMap<String, TypeTemplateSupplier> {
        // super.registerEntities(schema) result passed as `map`
        schema.register(&mut map, "minecraft:tnt", |s| {
            Dsl::optional_fields(vec![(
                "block_state".to_string(),
                References::in_schema(References::BLOCK_STATE, s),
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
        assert_eq!(V3683::new(3683, None).version_key, 3683);
        assert_eq!(V3683::new(-3683, None).version_key, -3683);
        assert_eq!(V3683::new(i32::MAX, None).version_key, i32::MAX);
    }

    #[test]
    fn register_entities_adds_tnt_with_block_state_reference() {
        let v = V3683::new(3683, None);
        let mut schema = Schema::new(3683);
        let base = HashMap::new();

        let result = v.register_entities(&mut schema, base);

        assert!(result.contains_key("minecraft:tnt"));
        assert_eq!(schema.registered_entities, vec!["minecraft:tnt".to_string()]);
        let template = (result.get("minecraft:tnt").unwrap())();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![(
                "block_state".to_string(),
                TypeTemplate::Reference {
                    name: "block_state".to_string(),
                    schema_version: 3683,
                }
            )])
        );
    }

    #[test]
    fn register_entities_preserves_existing_entries() {
        let v = V3683::new(3683, None);
        let mut schema = Schema::new(100);
        let mut base = HashMap::new();
        base.insert(
            "minecraft:creeper".to_string(),
            Arc::new(|| TypeTemplate::Unit) as TypeTemplateSupplier,
        );

        let result = v.register_entities(&mut schema, base);

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("minecraft:creeper"));
        assert!(result.contains_key("minecraft:tnt"));
    }

    #[test]
    fn register_entities_uses_schema_version_for_reference() {
        let v = V3683::new(3683, None);
        let mut schema = Schema::new(-7);
        let result = v.register_entities(&mut schema, HashMap::new());
        let template = (result.get("minecraft:tnt").unwrap())();
        match template {
            TypeTemplate::OptionalFields(fields) => match &fields[0].1 {
                TypeTemplate::Reference { schema_version, .. } => assert_eq!(*schema_version, -7),
                other => panic!("unexpected template: {:?}", other),
            },
            other => panic!("unexpected template: {:?}", other),
        }
    }
}
