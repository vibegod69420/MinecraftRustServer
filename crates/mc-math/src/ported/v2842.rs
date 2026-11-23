use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Reference {
    Chunk,
    EntityTree,
    BlockEntity,
    BlockName,
    Biome,
    BlockState,
    StructureFeature,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeTemplate {
    Remainder,
    Reference(Reference),
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    Fields(Vec<(String, TypeTemplate)>),
    OptionalFields(Vec<(String, TypeTemplate)>),
    CompoundList(Box<TypeTemplate>),
}

impl Reference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference(self.clone())
    }
}

pub mod dsl {
    use super::TypeTemplate;

    pub fn remainder() -> TypeTemplate {
        TypeTemplate::Remainder
    }

    pub fn list(t: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(t))
    }

    pub fn or(a: TypeTemplate, b: TypeTemplate) -> TypeTemplate {
        TypeTemplate::Or(Box::new(a), Box::new(b))
    }

    pub fn fields(entries: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::Fields(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn optional_fields(entries: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn compound_list(t: TypeTemplate) -> TypeTemplate {
        TypeTemplate::CompoundList(Box::new(t))
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

pub struct V2842 {
    pub base: NamespacedSchema,
}

impl V2842 {
    // JAVA: net.minecraft.util.datafix.schemas.V2842#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2842#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TemplateSupplier>,
        block_entity_types: &mut BTreeMap<String, TemplateSupplier>,
    ) {
        self.base
            .register_types(schema, entity_types, block_entity_types);
        let s: &Schema = &*schema;
        let chunk_template = dsl::optional_fields(vec![
            ("entities", dsl::list(Reference::EntityTree.in_schema(s))),
            (
                "block_entities",
                dsl::list(dsl::or(
                    Reference::BlockEntity.in_schema(s),
                    dsl::remainder(),
                )),
            ),
            (
                "block_ticks",
                dsl::list(dsl::fields(vec![(
                    "i",
                    Reference::BlockName.in_schema(s),
                )])),
            ),
            (
                "sections",
                dsl::list(dsl::optional_fields(vec![
                    (
                        "biomes",
                        dsl::optional_fields(vec![(
                            "palette",
                            dsl::list(Reference::Biome.in_schema(s)),
                        )]),
                    ),
                    (
                        "block_states",
                        dsl::optional_fields(vec![(
                            "palette",
                            dsl::list(Reference::BlockState.in_schema(s)),
                        )]),
                    ),
                ])),
            ),
            (
                "structures",
                dsl::optional_fields(vec![(
                    "starts",
                    dsl::compound_list(Reference::StructureFeature.in_schema(s)),
                )]),
            ),
        ]);
        schema.register_type(false, Reference::Chunk, chunk_template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (V2842, Schema, BTreeMap<String, TemplateSupplier>, BTreeMap<String, TemplateSupplier>) {
        let v = V2842::new(2842, Schema::default());
        (v, Schema::default(), BTreeMap::new(), BTreeMap::new())
    }

    #[test]
    fn registers_chunk_type_non_recursive() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        assert_eq!(schema.registrations.len(), 1);
        let reg = &schema.registrations[0];
        assert!(!reg.recursive);
        assert_eq!(reg.reference, Reference::Chunk);
    }

    #[test]
    fn chunk_template_has_expected_top_level_fields() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::OptionalFields(fields) = &schema.registrations[0].template else {
            panic!("expected OptionalFields");
        };
        let names: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, vec!["entities", "block_entities", "block_ticks", "sections", "structures"]);
    }

    #[test]
    fn block_entities_uses_or_with_remainder() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::OptionalFields(fields) = &schema.registrations[0].template else {
            panic!("expected OptionalFields");
        };
        let be = &fields[1].1;
        let TypeTemplate::List(inner) = be else { panic!("expected List"); };
        let TypeTemplate::Or(left, right) = &**inner else { panic!("expected Or"); };
        assert_eq!(**left, TypeTemplate::Reference(Reference::BlockEntity));
        assert_eq!(**right, TypeTemplate::Remainder);
    }

    #[test]
    fn sections_nested_biomes_and_block_states_palettes() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::OptionalFields(fields) = &schema.registrations[0].template else {
            panic!("expected OptionalFields");
        };
        let TypeTemplate::List(sec_inner) = &fields[3].1 else { panic!("expected List"); };
        let TypeTemplate::OptionalFields(sec_fields) = &**sec_inner else { panic!("expected OptionalFields"); };
        assert_eq!(sec_fields[0].0, "biomes");
        assert_eq!(sec_fields[1].0, "block_states");
        let TypeTemplate::OptionalFields(biomes) = &sec_fields[0].1 else { panic!("expected OptionalFields"); };
        assert_eq!(biomes[0].0, "palette");
        assert_eq!(biomes[0].1, TypeTemplate::List(Box::new(TypeTemplate::Reference(Reference::Biome))));
    }

    #[test]
    fn structures_starts_is_compound_list() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::OptionalFields(fields) = &schema.registrations[0].template else {
            panic!("expected OptionalFields");
        };
        let TypeTemplate::OptionalFields(struct_fields) = &fields[4].1 else { panic!("expected OptionalFields"); };
        assert_eq!(struct_fields[0].0, "starts");
        assert_eq!(
            struct_fields[0].1,
            TypeTemplate::CompoundList(Box::new(TypeTemplate::Reference(Reference::StructureFeature)))
        );
    }
}
