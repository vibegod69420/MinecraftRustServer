use std::collections::BTreeMap;

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum References {
    ItemStack,
    Particle,
    BlockState,
    FlatBlockState,
}

impl References {
    pub fn in_schema(self, schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference {
            reference: self,
            version: schema.version,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Reference { reference: References, version: i32 },
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(fields: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            fields
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }

    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn register(
        &self,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
        supplier: TemplateSupplier,
    ) {
        map.insert(name.to_string(), supplier);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }

    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

pub struct V1909 {
    base: NamespacedSchema,
}

impl V1909 {
    // JAVA: net.minecraft.util.datafix.schemas.V1909#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1909#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        let s = schema.clone();
        schema.register(
            &mut map,
            "minecraft:jigsaw",
            Box::new(move || {
                Dsl::optional_fields(vec![(
                    "final_state",
                    References::FlatBlockState.in_schema(&s),
                )])
            }),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_jigsaw_block_entity() {
        let schema = Schema::new(1909);
        let v = V1909::new(1909, Schema::new(0));
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:jigsaw"));
    }

    #[test]
    fn jigsaw_template_structure() {
        let schema = Schema::new(1909);
        let v = V1909::new(1909, Schema::new(0));
        let map = v.register_block_entities(&schema);
        let template = map.get("minecraft:jigsaw").unwrap()();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![(
                "final_state".to_string(),
                TypeTemplate::Reference {
                    reference: References::FlatBlockState,
                    version: 1909,
                },
            )])
        );
    }

    #[test]
    fn schema_version_propagates_negative_and_bounds() {
        let schema = Schema::new(i32::MAX);
        let v = V1909::new(i32::MAX, Schema::new(i32::MIN));
        let map = v.register_block_entities(&schema);
        let template = map.get("minecraft:jigsaw").unwrap()();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![(
                "final_state".to_string(),
                TypeTemplate::Reference {
                    reference: References::FlatBlockState,
                    version: i32::MAX,
                },
            )])
        );
    }

    #[test]
    fn supplier_is_repeatable() {
        let schema = Schema::new(1909);
        let v = V1909::new(1909, Schema::new(0));
        let map = v.register_block_entities(&schema);
        let supplier = map.get("minecraft:jigsaw").unwrap();
        assert_eq!(supplier(), supplier());
    }
}
