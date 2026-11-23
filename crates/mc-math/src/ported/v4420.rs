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

pub struct V4420 {
    base: NamespacedSchema,
}

impl V4420 {
    // JAVA: net.minecraft.util.datafix.schemas.V4420#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4420#registerEntities
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TemplateSupplier> {
        let mut map = self.base.register_entities(schema);
        let s = schema.clone();
        schema.register(
            &mut map,
            "minecraft:area_effect_cloud",
            Box::new(move || {
                Dsl::optional_fields(vec![(
                    "custom_particle",
                    References::Particle.in_schema(&s),
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
    fn registers_area_effect_cloud_entity() {
        let schema = Schema::new(4420);
        let v = V4420::new(4420, Schema::new(0));
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:area_effect_cloud"));
    }

    #[test]
    fn area_effect_cloud_template_structure() {
        let schema = Schema::new(4420);
        let v = V4420::new(4420, Schema::new(0));
        let map = v.register_entities(&schema);
        let template = map.get("minecraft:area_effect_cloud").unwrap()();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![(
                "custom_particle".to_string(),
                TypeTemplate::Reference {
                    reference: References::Particle,
                    version: 4420,
                },
            )])
        );
    }

    #[test]
    fn schema_version_propagates_negative_and_bounds() {
        let schema = Schema::new(i32::MIN);
        let v = V4420::new(i32::MIN, Schema::new(-1));
        let map = v.register_entities(&schema);
        let template = map.get("minecraft:area_effect_cloud").unwrap()();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![(
                "custom_particle".to_string(),
                TypeTemplate::Reference {
                    reference: References::Particle,
                    version: i32::MIN,
                },
            )])
        );
    }

    #[test]
    fn supplier_is_repeatable() {
        let schema = Schema::new(4420);
        let v = V4420::new(4420, Schema::new(0));
        let map = v.register_entities(&schema);
        let supplier = map.get("minecraft:area_effect_cloud").unwrap();
        assert_eq!(supplier(), supplier());
    }
}
