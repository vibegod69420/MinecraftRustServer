use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reference {
    TextComponent,
    ItemStack,
    ItemName,
    EntityName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    StringType,
    Const(Box<TypeTemplate>),
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    TaggedChoice {
        field: String,
        key_type: Box<TypeTemplate>,
        choices: Vec<(String, TypeTemplate)>,
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

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
    }

    // JAVA: com.mojang.datafixers.DSL#or
    pub fn or(a: TypeTemplate, b: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or(Box::new(a), Box::new(b))
    }

    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(entries: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(entries)
    }

    // JAVA: com.mojang.datafixers.DSL#taggedChoice
    pub fn tagged_choice(
        field: impl Into<String>,
        key_type: TypeTemplate,
        choices: Vec<(String, TypeTemplate)>,
    ) -> TypeTemplate {
        TypeTemplate::TaggedChoice {
            field: field.into(),
            key_type: Box::new(key_type),
            choices,
        }
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
pub struct V4290 {
    pub version: i32,
    pub parent_version: i32,
}

impl V4290 {
    // JAVA: net.minecraft.util.datafix.schemas.V4290#<init>
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

    // JAVA: net.minecraft.util.datafix.schemas.V4290#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TypeTemplate>,
        block_entity_types: &mut BTreeMap<String, TypeTemplate>,
    ) {
        self.super_register_types(schema, entity_types, block_entity_types);
        schema.register_type(
            true,
            Reference::TextComponent,
            Dsl::or(
                Dsl::or(
                    Dsl::const_type(Dsl::string()),
                    Dsl::list(ref_in_schema(Reference::TextComponent, schema)),
                ),
                Dsl::optional_fields(vec![
                    (
                        "extra".to_string(),
                        Dsl::list(ref_in_schema(Reference::TextComponent, schema)),
                    ),
                    (
                        "separator".to_string(),
                        ref_in_schema(Reference::TextComponent, schema),
                    ),
                    (
                        "hoverEvent".to_string(),
                        Dsl::tagged_choice(
                            "action",
                            Dsl::string(),
                            vec![
                                (
                                    "show_text".to_string(),
                                    Dsl::optional_fields(vec![(
                                        "contents".to_string(),
                                        ref_in_schema(Reference::TextComponent, schema),
                                    )]),
                                ),
                                (
                                    "show_item".to_string(),
                                    Dsl::optional_fields(vec![(
                                        "contents".to_string(),
                                        Dsl::or(
                                            ref_in_schema(Reference::ItemStack, schema),
                                            ref_in_schema(Reference::ItemName, schema),
                                        ),
                                    )]),
                                ),
                                (
                                    "show_entity".to_string(),
                                    Dsl::optional_fields(vec![
                                        (
                                            "type".to_string(),
                                            ref_in_schema(Reference::EntityName, schema),
                                        ),
                                        (
                                            "name".to_string(),
                                            ref_in_schema(Reference::TextComponent, schema),
                                        ),
                                    ]),
                                ),
                            ],
                        ),
                    ),
                ]),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_versions() {
        let v = V4290::new(4290, 4287);
        assert_eq!(v.version, 4290);
        assert_eq!(v.parent_version, 4287);
    }

    #[test]
    fn text_component_is_registered_as_recursive() {
        let v = V4290::new(4290, 4287);
        let mut schema = Schema {
            version: 4290,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (recursive, _) = schema.types.get(&Reference::TextComponent).unwrap();
        assert!(*recursive);
    }

    #[test]
    fn hover_event_tagged_choice_has_three_actions() {
        let v = V4290::new(4290, 4287);
        let mut schema = Schema {
            version: 4290,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::TextComponent).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains("TaggedChoice { field: \"action\""));
        assert!(dbg.contains("\"show_text\""));
        assert!(dbg.contains("\"show_item\""));
        assert!(dbg.contains("\"show_entity\""));
    }

    #[test]
    fn show_item_contents_uses_or_item_stack_or_item_name() {
        let v = V4290::new(4290, 4287);
        let mut schema = Schema {
            version: 4290,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::TextComponent).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains(
            "Or(RefInSchema { reference: ItemStack, schema_version: 4290 }, RefInSchema { reference: ItemName, schema_version: 4290 })"
        ));
    }

    #[test]
    fn register_types_leaves_entity_maps_unchanged() {
        let v = V4290::new(4290, 4287);
        let mut schema = Schema {
            version: 4290,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        assert!(e.is_empty());
        assert!(b.is_empty());
        assert_eq!(schema.types.len(), 1);
    }

    #[test]
    fn negative_schema_version_is_preserved_in_refs() {
        let v = V4290::new(i32::MIN, -1);
        let mut schema = Schema {
            version: i32::MIN,
            ..Default::default()
        };
        let mut e = BTreeMap::new();
        let mut b = BTreeMap::new();
        v.register_types(&mut schema, &mut e, &mut b);
        let (_, template) = schema.types.get(&Reference::TextComponent).unwrap();
        let dbg = format!("{template:?}");
        assert!(dbg.contains(&format!("schema_version: {}", i32::MIN)));
    }
}
