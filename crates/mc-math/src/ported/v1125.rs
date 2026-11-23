use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reference {
    Advancements,
    Biome,
    EntityName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    StringType,
    NamespacedString,
    Const(Box<TypeTemplate>),
    Remainder,
    OptionalFields(Vec<(String, TypeTemplate)>),
    CompoundListKeyed {
        key: Box<TypeTemplate>,
        value: Box<TypeTemplate>,
    },
    RefInSchema {
        reference: Reference,
        schema_version: i32,
    },
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#string
    pub fn string() -> TypeTemplate {
        TypeTemplate::StringType
    }

    // JAVA: com.mojang.datafixers.DSL#constType
    pub fn const_type(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Const(Box::new(template))
    }

    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(entries: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(entries)
    }

    // JAVA: com.mojang.datafixers.DSL#compoundList
    pub fn compound_list(key: TypeTemplate, value: TypeTemplate) -> TypeTemplate {
        TypeTemplate::CompoundListKeyed {
            key: Box::new(key),
            value: Box::new(value),
        }
    }

    // JAVA: com.mojang.datafixers.DSL#remainder
    pub fn remainder() -> TypeTemplate {
        TypeTemplate::Remainder
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#namespacedString
pub fn namespaced_string() -> TypeTemplate {
    TypeTemplate::NamespacedString
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

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(&self, block_entities: &mut BTreeMap<String, TypeTemplate>, name: &str) {
        block_entities.insert(name.to_string(), Dsl::remainder());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1125 {
    pub version: i32,
    pub parent_version: i32,
}

impl V1125 {
    // JAVA: net.minecraft.util.datafix.schemas.V1125#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    fn super_register_block_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        // Base NamespacedSchema registrations are not modeled in this self-contained port.
        BTreeMap::new()
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1125#registerBlockEntities
    pub fn register_block_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.super_register_block_entities(schema);
        schema.register_simple(&mut map, "minecraft:bed");
        map
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

    // JAVA: net.minecraft.util.datafix.schemas.V1125#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TypeTemplate>,
        block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.super_register_types(schema, entity_types, block_entity_types);
        schema.register_type(
            false,
            Reference::Advancements,
            Dsl::optional_fields(vec![
                (
                    "minecraft:adventure/adventuring_time".to_string(),
                    Dsl::optional_fields(vec![(
                        "criteria".to_string(),
                        Dsl::compound_list(
                            ref_in_schema(Reference::Biome, schema),
                            Dsl::const_type(Dsl::string()),
                        ),
                    )]),
                ),
                (
                    "minecraft:adventure/kill_a_mob".to_string(),
                    Dsl::optional_fields(vec![(
                        "criteria".to_string(),
                        Dsl::compound_list(
                            ref_in_schema(Reference::EntityName, schema),
                            Dsl::const_type(Dsl::string()),
                        ),
                    )]),
                ),
                (
                    "minecraft:adventure/kill_all_mobs".to_string(),
                    Dsl::optional_fields(vec![(
                        "criteria".to_string(),
                        Dsl::compound_list(
                            ref_in_schema(Reference::EntityName, schema),
                            Dsl::const_type(Dsl::string()),
                        ),
                    )]),
                ),
                (
                    "minecraft:husbandry/bred_all_animals".to_string(),
                    Dsl::optional_fields(vec![(
                        "criteria".to_string(),
                        Dsl::compound_list(
                            ref_in_schema(Reference::EntityName, schema),
                            Dsl::const_type(Dsl::string()),
                        ),
                    )]),
                ),
            ]),
        );
        schema.register_type(
            false,
            Reference::Biome,
            Dsl::const_type(namespaced_string()),
        );
        schema.register_type(
            false,
            Reference::EntityName,
            Dsl::const_type(namespaced_string()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_versions() {
        let v = V1125::new(1125, 1124);
        assert_eq!(v.version, 1125);
        assert_eq!(v.parent_version, 1124);
    }

    #[test]
    fn register_block_entities_adds_bed() {
        let v = V1125::new(1125, 1124);
        let schema = Schema {
            version: 1125,
            ..Default::default()
        };
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("minecraft:bed"), Some(&TypeTemplate::Remainder));
    }

    #[test]
    fn advancements_biome_entity_name_registered_non_recursive() {
        let v = V1125::new(1125, 1124);
        let mut schema = Schema {
            version: 1125,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        for r in [Reference::Advancements, Reference::Biome, Reference::EntityName] {
            let (recursive, _) = schema.types.get(&r).unwrap();
            assert!(!*recursive);
        }
    }

    #[test]
    fn biome_and_entity_name_are_const_namespaced_string() {
        let v = V1125::new(1125, 1124);
        let mut schema = Schema {
            version: 1125,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let expected = TypeTemplate::Const(Box::new(TypeTemplate::NamespacedString));
        assert_eq!(schema.types.get(&Reference::Biome).unwrap().1, expected);
        assert_eq!(schema.types.get(&Reference::EntityName).unwrap().1, expected);
    }

    #[test]
    fn advancements_contains_compound_list_criteria() {
        let v = V1125::new(1125, 1124);
        let mut schema = Schema {
            version: 1125,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::Advancements).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains("minecraft:adventure/adventuring_time"));
        assert!(dbg.contains("minecraft:husbandry/bred_all_animals"));
        assert!(dbg.contains("CompoundListKeyed"));
        assert!(dbg.contains("reference: Biome"));
        assert!(dbg.contains("reference: EntityName"));
    }

    #[test]
    fn negative_schema_version_is_preserved_in_refs() {
        let v = V1125::new(-100, i32::MIN);
        let mut schema = Schema {
            version: -100,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::Advancements).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains("schema_version: -100"));
    }
}
