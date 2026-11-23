use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reference {
    Recipe,
    Player,
    Hotbar,
    EntityTree,
    ItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    NamespacedString,
    Const(Box<TypeTemplate>),
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    CompoundList(Box<TypeTemplate>),
    RefInSchema {
        reference: Reference,
        schema_version: i32,
    },
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#constType
    pub fn const_type(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Const(Box::new(template))
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V1022 {
    pub version: i32,
    pub parent_version: i32,
}

impl V1022 {
    // JAVA: net.minecraft.util.datafix.schemas.V1022#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerTypes
    fn super_register_types(
        &self,
        _schema: &mut Schema,
        _entity_types: &mut BTreeMap<String, TypeTemplate>,
        _block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        // Base Schema registrations are not modeled in this self-contained port.
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1022#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TypeTemplate>,
        block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.super_register_types(schema, entity_types, block_entity_types);

        schema.register_type(
            false,
            Reference::Recipe,
            Dsl::const_type(namespaced_string()),
        );

        schema.register_type(
            false,
            Reference::Player,
            Dsl::optional_fields(vec![
                (
                    "RootVehicle".to_string(),
                    Dsl::optional_fields(vec![(
                        "Entity".to_string(),
                        ref_in_schema(Reference::EntityTree, schema),
                    )]),
                ),
                (
                    "ender_pearls".to_string(),
                    Dsl::list(ref_in_schema(Reference::EntityTree, schema)),
                ),
                (
                    "Inventory".to_string(),
                    Dsl::list(ref_in_schema(Reference::ItemStack, schema)),
                ),
                (
                    "EnderItems".to_string(),
                    Dsl::list(ref_in_schema(Reference::ItemStack, schema)),
                ),
                (
                    "ShoulderEntityLeft".to_string(),
                    ref_in_schema(Reference::EntityTree, schema),
                ),
                (
                    "ShoulderEntityRight".to_string(),
                    ref_in_schema(Reference::EntityTree, schema),
                ),
                (
                    "recipeBook".to_string(),
                    Dsl::optional_fields(vec![
                        (
                            "recipes".to_string(),
                            Dsl::list(ref_in_schema(Reference::Recipe, schema)),
                        ),
                        (
                            "toBeDisplayed".to_string(),
                            Dsl::list(ref_in_schema(Reference::Recipe, schema)),
                        ),
                    ]),
                ),
            ]),
        );

        schema.register_type(
            false,
            Reference::Hotbar,
            Dsl::compound_list(Dsl::list(ref_in_schema(Reference::ItemStack, schema))),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_versions() {
        let v = V1022::new(1022, 1021);
        assert_eq!(v.version, 1022);
        assert_eq!(v.parent_version, 1021);
    }

    #[test]
    fn recipe_is_const_namespaced_string() {
        let v = V1022::new(1022, 1021);
        let mut schema = Schema {
            version: 1022,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (recursive, template) = schema.types.get(&Reference::Recipe).unwrap();
        assert!(!*recursive);
        assert_eq!(
            *template,
            TypeTemplate::Const(Box::new(TypeTemplate::NamespacedString))
        );
    }

    #[test]
    fn player_has_expected_top_level_fields() {
        let v = V1022::new(1022, 1021);
        let mut schema = Schema {
            version: 1022,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::Player).unwrap();
        match template {
            TypeTemplate::OptionalFields(fields) => {
                let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                assert_eq!(
                    names,
                    vec![
                        "RootVehicle",
                        "ender_pearls",
                        "Inventory",
                        "EnderItems",
                        "ShoulderEntityLeft",
                        "ShoulderEntityRight",
                        "recipeBook",
                    ]
                );
            }
            _ => panic!("expected OptionalFields"),
        }
    }

    #[test]
    fn hotbar_is_compound_list_of_item_stack_list() {
        let v = V1022::new(1022, 1021);
        let mut schema = Schema {
            version: 1022,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::Hotbar).unwrap();
        assert_eq!(
            *template,
            TypeTemplate::CompoundList(Box::new(TypeTemplate::List(Box::new(
                TypeTemplate::RefInSchema {
                    reference: Reference::ItemStack,
                    schema_version: 1022,
                }
            ))))
        );
    }

    #[test]
    fn register_types_leaves_entity_maps_unchanged() {
        let v = V1022::new(1022, 1021);
        let mut schema = Schema {
            version: 1022,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        assert!(e.is_empty());
        assert!(b.is_empty());
        assert_eq!(schema.types.len(), 3);
    }

    #[test]
    fn negative_schema_version_is_preserved_in_refs() {
        let v = V1022::new(-7, -8);
        let mut schema = Schema {
            version: -7,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::Hotbar).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains("schema_version: -7"));
    }
}
