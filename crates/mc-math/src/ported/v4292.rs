use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Reference {
    TextComponent,
    ItemStack,
    EntityName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeTemplate {
    StringType,
    Const(Box<TypeTemplate>),
    Reference(Reference),
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    TaggedChoice {
        tag: String,
        key_type: Box<TypeTemplate>,
        choices: BTreeMap<String, TypeTemplate>,
    },
}

impl Reference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference(self.clone())
    }
}

pub mod dsl {
    use super::TypeTemplate;
    use std::collections::BTreeMap;

    pub fn string() -> TypeTemplate {
        TypeTemplate::StringType
    }

    pub fn const_type(t: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Const(Box::new(t))
    }

    pub fn list(t: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(t))
    }

    pub fn or(a: TypeTemplate, b: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or(Box::new(a), Box::new(b))
    }

    pub fn optional_fields(entries: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn tagged_choice(
        tag: &str,
        key_type: TypeTemplate,
        choices: Vec<(&str, TypeTemplate)>,
    ) -> TypeTemplate {
        let mut map = BTreeMap::new();
        for (k, v) in choices {
            map.insert(k.to_string(), v);
        }
        TypeTemplate::TaggedChoice {
            tag: tag.to_string(),
            key_type: Box::new(key_type),
            choices: map,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeRegistration {
    pub recursive: bool,
    pub reference: Reference,
    pub template: TypeTemplate,
}

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub registrations: Vec<TypeRegistration>,
}

impl Schema {
    pub fn register_type(&mut self, recursive: bool, reference: Reference, template: TypeTemplate) {
        self.registrations.push(TypeRegistration {
            recursive,
            reference,
            template,
        });
    }
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

pub struct NamespacedSchema {
    pub version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerTypes
    pub fn register_types(
        &self,
        _schema: &mut Schema,
        _entity_types: &mut BTreeMap<String, TemplateSupplier>,
        _block_entity_types: &mut BTreeMap<String, TemplateSupplier>,
    ) {
    }
}

pub struct V4292 {
    pub base: NamespacedSchema,
}

impl V4292 {
    // JAVA: net.minecraft.util.datafix.schemas.V4292#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4292#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TemplateSupplier>,
        block_entity_types: &mut BTreeMap<String, TemplateSupplier>,
    ) {
        self.base
            .register_types(schema, entity_types, block_entity_types);
        let s: &Schema = &*schema;
        let text_component_template = dsl::or(
            dsl::or(
                dsl::const_type(dsl::string()),
                dsl::list(Reference::TextComponent.in_schema(s)),
            ),
            dsl::optional_fields(vec![
                ("extra", dsl::list(Reference::TextComponent.in_schema(s))),
                ("separator", Reference::TextComponent.in_schema(s)),
                (
                    "hover_event",
                    dsl::tagged_choice(
                        "action",
                        dsl::string(),
                        vec![
                            (
                                "show_text",
                                dsl::optional_fields(vec![(
                                    "value",
                                    Reference::TextComponent.in_schema(s),
                                )]),
                            ),
                            ("show_item", Reference::ItemStack.in_schema(s)),
                            (
                                "show_entity",
                                dsl::optional_fields(vec![
                                    ("id", Reference::EntityName.in_schema(s)),
                                    ("name", Reference::TextComponent.in_schema(s)),
                                ]),
                            ),
                        ],
                    ),
                ),
            ]),
        );
        schema.register_type(true, Reference::TextComponent, text_component_template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (V4292, Schema, BTreeMap<String, TemplateSupplier>, BTreeMap<String, TemplateSupplier>) {
        let v = V4292::new(4292, Schema::default());
        (v, Schema::default(), BTreeMap::new(), BTreeMap::new())
    }

    #[test]
    fn registers_text_component_recursive() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        assert_eq!(schema.registrations.len(), 1);
        let reg = &schema.registrations[0];
        assert!(reg.recursive);
        assert_eq!(reg.reference, Reference::TextComponent);
    }

    #[test]
    fn top_level_is_or_of_or_and_optional_fields() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::Or(left, right) = &schema.registrations[0].template else {
            panic!("expected Or");
        };
        assert!(matches!(**left, TypeTemplate::Or(_, _)));
        assert!(matches!(**right, TypeTemplate::OptionalFields(_)));
    }

    #[test]
    fn inner_or_contains_const_string_and_text_component_list() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::Or(left, _) = &schema.registrations[0].template else {
            panic!("expected Or");
        };
        let TypeTemplate::Or(a, b) = &**left else { panic!("expected inner Or"); };
        assert_eq!(**a, TypeTemplate::Const(Box::new(TypeTemplate::StringType)));
        assert_eq!(**b, TypeTemplate::List(Box::new(TypeTemplate::Reference(Reference::TextComponent))));
    }

    #[test]
    fn hover_event_tagged_choice_has_three_choices() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::Or(_, right) = &schema.registrations[0].template else {
            panic!("expected Or");
        };
        let TypeTemplate::OptionalFields(fields) = &**right else { panic!("expected OptionalFields"); };
        assert_eq!(fields[2].0, "hover_event");
        let TypeTemplate::TaggedChoice { tag, key_type, choices } = &fields[2].1 else {
            panic!("expected TaggedChoice");
        };
        assert_eq!(tag, "action");
        assert_eq!(**key_type, TypeTemplate::StringType);
        assert_eq!(choices.len(), 3);
        assert!(choices.contains_key("show_text"));
        assert!(choices.contains_key("show_item"));
        assert!(choices.contains_key("show_entity"));
    }

    #[test]
    fn show_entity_choice_contains_id_and_name() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::Or(_, right) = &schema.registrations[0].template else {
            panic!("expected Or");
        };
        let TypeTemplate::OptionalFields(fields) = &**right else { panic!("expected OptionalFields"); };
        let TypeTemplate::TaggedChoice { choices, .. } = &fields[2].1 else { panic!("expected TaggedChoice"); };
        let TypeTemplate::OptionalFields(entity_fields) = choices.get("show_entity").unwrap() else {
            panic!("expected OptionalFields");
        };
        assert_eq!(entity_fields[0], ("id".to_string(), TypeTemplate::Reference(Reference::EntityName)));
        assert_eq!(entity_fields[1], ("name".to_string(), TypeTemplate::Reference(Reference::TextComponent)));
    }
}
