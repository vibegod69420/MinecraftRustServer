use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reference {
    Chunk,
    EntityTree,
    BlockEntity,
    BlockName,
    BlockState,
    StructureFeature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Remainder,
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    Fields(String, Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    CompoundList(Box<TypeTemplate>),
    RefInSchema {
        reference: Reference,
        schema_version: i32,
    },
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#remainder
    pub fn remainder() -> TypeTemplate {
        TypeTemplate::Remainder
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
    }

    // JAVA: com.mojang.datafixers.DSL#or
    pub fn or(a: TypeTemplate, b: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or(Box::new(a), Box::new(b))
    }

    // JAVA: com.mojang.datafixers.DSL#fields
    pub fn fields(name: impl Into<String>, child: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Fields(name.into(), Box::new(child))
    }

    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(entries: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(entries)
    }

    // JAVA: com.mojang.datafixers.DSL#compoundList
    pub fn compound_list(value: TypeTemplate) -> TypeTemplate {
        TypeTemplate::CompoundList(Box::new(value))
    }
}

// JAVA: com.mojang.datafixers.types.templates.TypeTemplate#in
pub fn ref_in_schema(reference: Reference, schema: &Schema) -> TypeTemplate {
    TypeTemplate::RefInSchema {
        reference,
        schema_version: schema.version,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
    pub types: BTreeMap<Reference, (bool, TypeTemplate)>,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#registerType
    pub fn register_type(&mut self, recursive: bool, reference: Reference, template: TypeTemplate) {
        self.types.insert(reference, (recursive, template));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1466 {
    pub version: i32,
    pub parent_version: i32,
}

impl V1466 {
    // JAVA: net.minecraft.util.datafix.schemas.V1466#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    fn super_register_types(
        &self,
        _schema: &mut Schema,
        _entity_types: &mut BTreeMap<String, TypeTemplate>,
        _block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        // Base NamespacedSchema registrations are not modeled in this self-contained port.
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1466#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TypeTemplate>,
        block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.super_register_types(schema, entity_types, block_entity_types);
        schema.register_type(
            false,
            Reference::Chunk,
            Dsl::fields(
                "Level",
                Dsl::optional_fields(vec![
                    (
                        "Entities".to_string(),
                        Dsl::list(ref_in_schema(Reference::EntityTree, schema)),
                    ),
                    (
                        "TileEntities".to_string(),
                        Dsl::list(Dsl::or(
                            ref_in_schema(Reference::BlockEntity, schema),
                            Dsl::remainder(),
                        )),
                    ),
                    (
                        "TileTicks".to_string(),
                        Dsl::list(Dsl::fields(
                            "i",
                            ref_in_schema(Reference::BlockName, schema),
                        )),
                    ),
                    (
                        "Sections".to_string(),
                        Dsl::list(Dsl::optional_fields(vec![(
                            "Palette".to_string(),
                            Dsl::list(ref_in_schema(Reference::BlockState, schema)),
                        )])),
                    ),
                    (
                        "Structures".to_string(),
                        Dsl::optional_fields(vec![(
                            "Starts".to_string(),
                            Dsl::compound_list(ref_in_schema(Reference::StructureFeature, schema)),
                        )]),
                    ),
                ]),
            ),
        );
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    fn super_register_block_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        // Base NamespacedSchema registrations are not modeled in this self-contained port.
        BTreeMap::new()
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1466#registerBlockEntities
    pub fn register_block_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.super_register_block_entities(schema);
        map.insert("DUMMY".to_string(), Dsl::remainder());
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_versions() {
        let v = V1466::new(1466, 1451);
        assert_eq!(v.version, 1466);
        assert_eq!(v.parent_version, 1451);
    }

    #[test]
    fn register_block_entities_contains_dummy_remainder() {
        let v = V1466::new(1466, 1451);
        let schema = Schema {
            version: 1466,
            ..Default::default()
        };
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("DUMMY"), Some(&TypeTemplate::Remainder));
    }

    #[test]
    fn register_types_registers_chunk_non_recursive() {
        let v = V1466::new(1466, 1451);
        let mut schema = Schema {
            version: 1466,
            ..Default::default()
        };
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut schema, &mut entities, &mut block_entities);
        let (recursive, template) = schema.types.get(&Reference::Chunk).expect("chunk registered");
        assert!(!*recursive);
        assert!(matches!(template, TypeTemplate::Fields(name, _) if name == "Level"));
    }

    #[test]
    fn register_types_propagates_schema_version_into_refs() {
        let v = V1466::new(1466, 1451);
        let mut schema = Schema {
            version: 1466,
            ..Default::default()
        };
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut schema, &mut entities, &mut block_entities);
        let (_, template) = schema.types.get(&Reference::Chunk).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains("RefInSchema { reference: EntityTree, schema_version: 1466 }"));
        assert!(dbg.contains("RefInSchema { reference: BlockState, schema_version: 1466 }"));
    }

    #[test]
    fn register_types_does_not_modify_entity_maps() {
        let v = V1466::new(1466, 1451);
        let mut schema = Schema {
            version: 1466,
            ..Default::default()
        };
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut schema, &mut entities, &mut block_entities);
        assert!(entities.is_empty());
        assert!(block_entities.is_empty());
    }

    #[test]
    fn negative_schema_version_is_preserved_in_refs() {
        let v = V1466::new(-1, i32::MIN);
        let mut schema = Schema {
            version: -1,
            ..Default::default()
        };
        let mut entities = BTreeMap::new();
        let mut block_entities = BTreeMap::new();
        v.register_types(&mut schema, &mut entities, &mut block_entities);
        let (_, template) = schema.types.get(&Reference::Chunk).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains("schema_version: -1"));
    }
}
