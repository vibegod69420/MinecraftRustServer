use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTemplate {
    OptionalFields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
    Ref {
        reference: &'static str,
        schema_version: i32,
    },
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeReference(pub &'static str);

impl TypeReference {
    // JAVA: com.mojang.datafixers.DSL$TypeReference#in
    pub fn in_schema(self, schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref {
            reference: self.0,
            schema_version: schema.version,
        }
    }
}

pub struct References;

impl References {
    pub const TEXT_COMPONENT: TypeReference = TypeReference("TEXT_COMPONENT");
    pub const SAVED_DATA_MAP_DATA: TypeReference = TypeReference("SAVED_DATA_MAP_DATA");
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredType {
    pub recursive: bool,
    pub reference: &'static str,
    pub template: TypeTemplate,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub version: i32,
    pub parent_version: i32,
    pub registered_types: RefCell<Vec<RegisteredType>>,
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.parent_version == other.parent_version
            && *self.registered_types.borrow() == *other.registered_types.borrow()
    }
}

impl Eq for Schema {}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#Schema
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
            registered_types: RefCell::new(Vec::new()),
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerType
    pub fn register_type(&self, recursive: bool, reference: TypeReference, template: TypeTemplate) {
        self.registered_types.borrow_mut().push(RegisteredType {
            recursive,
            reference: reference.0,
            template,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    pub fn register_types(
        &self,
        _schema: &Schema,
        _entity_types: &mut BTreeMap<String, TypeTemplate>,
        _block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V3813 {
    pub base: NamespacedSchema,
}

impl V3813 {
    // JAVA: net.minecraft.util.datafix.schemas.V3813#V3813
    pub fn new(version: i32, parent: &Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3813#registerTypes
    pub fn register_types(
        &self,
        schema: &Schema,
        entity_types: &mut BTreeMap<String, TypeTemplate>,
        block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.base
            .register_types(schema, entity_types, block_entity_types);
        schema.register_type(
            false,
            References::SAVED_DATA_MAP_DATA,
            Dsl::optional_fields(vec![(
                "data",
                Dsl::optional_fields(vec![(
                    "banners",
                    Dsl::list(Dsl::optional_fields(vec![(
                        "name",
                        References::TEXT_COMPONENT.in_schema(schema),
                    )])),
                )]),
            )]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_saved_data_map_data_type() {
        let parent = Schema::new(3812, 0);
        let schema_inst = V3813::new(3813, &parent);
        let target = Schema::new(3813, 3812);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();

        schema_inst.register_types(&target, &mut entities, &mut block_entities);

        let types = target.registered_types.borrow();
        assert_eq!(types.len(), 1);
        let reg = &types[0];
        assert!(!reg.recursive);
        assert_eq!(reg.reference, "SAVED_DATA_MAP_DATA");

        let expected = TypeTemplate::OptionalFields(vec![(
            "data".to_string(),
            TypeTemplate::OptionalFields(vec![(
                "banners".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::OptionalFields(vec![(
                    "name".to_string(),
                    TypeTemplate::Ref {
                        reference: "TEXT_COMPONENT",
                        schema_version: 3813,
                    },
                )]))),
            )]),
        )]);
        assert_eq!(reg.template, expected);
    }

    #[test]
    fn negative_version_edge_case() {
        let parent = Schema::new(-1, i32::MIN);
        let schema_inst = V3813::new(-3813, &parent);
        let target = Schema::new(-3813, -1);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();

        schema_inst.register_types(&target, &mut entities, &mut block_entities);
        let types = target.registered_types.borrow();
        assert_eq!(types.len(), 1);
        if let TypeTemplate::OptionalFields(data) = &types[0].template {
            if let TypeTemplate::OptionalFields(banners) = &data[0].1 {
                if let TypeTemplate::List(inner) = &banners[0].1 {
                    if let TypeTemplate::OptionalFields(name) = inner.as_ref() {
                        if let TypeTemplate::Ref { schema_version, .. } = &name[0].1 {
                            assert_eq!(*schema_version, -3813);
                            return;
                        }
                    }
                }
            }
        }
        panic!("unexpected structure");
    }

    #[test]
    fn overflow_version_edge_case() {
        let parent = Schema::new(i32::MAX, i32::MIN);
        let schema_inst = V3813::new(i32::MAX, &parent);
        let target = Schema::new(i32::MAX, i32::MIN);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();

        schema_inst.register_types(&target, &mut entities, &mut block_entities);
        let types = target.registered_types.borrow();
        assert_eq!(types[0].reference, "SAVED_DATA_MAP_DATA");
        assert_eq!(target.version.wrapping_add(0), i32::MAX);
    }

    #[test]
    fn version_float_bits_roundtrip() {
        let v = f32::from_bits(0xFF80_0000u32).to_bits() as i32; // -inf bits
        let parent = Schema::new(0, 0);
        let schema_inst = V3813::new(v, &parent);
        assert_eq!(schema_inst.base.version, v);
        assert_eq!(f32::from_bits(schema_inst.base.version as u32).to_bits() as i32, v);
    }

    #[test]
    fn register_types_does_not_modify_entity_maps() {
        let parent = Schema::new(3812, 0);
        let schema_inst = V3813::new(3813, &parent);
        let target = Schema::new(3813, 3812);
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        entities.insert("pre".to_string(), TypeTemplate::OptionalFields(vec![]));

        schema_inst.register_types(&target, &mut entities, &mut block_entities);

        assert_eq!(entities.len(), 1);
        assert!(entities.contains_key("pre"));
        assert!(block_entities.is_empty());
    }
}
