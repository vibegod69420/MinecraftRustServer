use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Template {
    Ref {
        reference: &'static str,
        schema_version: i32,
    },
    List(Box<Template>),
    OptionalFields(Vec<Field>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Field {
    Named(String, Template),
    Include(Template),
}

pub struct TemplateSupplier(Box<dyn Fn() -> Template>);

impl TemplateSupplier {
    pub fn new<F: Fn() -> Template + 'static>(f: F) -> Self {
        Self(Box::new(f))
    }

    pub fn get(&self) -> Template {
        (self.0)()
    }
}

impl fmt::Debug for TemplateSupplier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TemplateSupplier").field(&self.get()).finish()
    }
}

pub struct TypeRegistration {
    pub recursive: bool,
    pub reference: &'static str,
    pub template: TemplateSupplier,
}

impl fmt::Debug for TypeRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeRegistration")
            .field("recursive", &self.recursive)
            .field("reference", &self.reference)
            .field("template", &self.template)
            .finish()
    }
}

pub type TypeMap = BTreeMap<String, TemplateSupplier>;

#[derive(Debug)]
pub struct Schema {
    pub version_key: i32,
    pub parent_version: Option<i32>,
    pub type_registrations: Vec<TypeRegistration>,
}

impl Schema {
    pub fn new(version_key: i32, parent: &Schema) -> Self {
        Self {
            version_key,
            parent_version: Some(parent.version_key),
            type_registrations: Vec::new(),
        }
    }

    pub fn root(version_key: i32) -> Self {
        Self {
            version_key,
            parent_version: None,
            type_registrations: Vec::new(),
        }
    }

    pub fn register_type(
        &mut self,
        recursive: bool,
        reference: &'static str,
        template: TemplateSupplier,
    ) {
        self.type_registrations.push(TypeRegistration {
            recursive,
            reference,
            template,
        });
    }
}

fn ref_in(reference: &'static str, schema_version: i32) -> Template {
    Template::Ref {
        reference,
        schema_version,
    }
}

pub struct References;

impl References {
    pub const PLAYER: &'static str = "player";
    pub const ENTITY_TREE: &'static str = "entity_tree";
    pub const ENTITY: &'static str = "entity";
    pub const ITEM_STACK: &'static str = "item_stack";
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(fields: Vec<Field>) -> Template {
        Template::OptionalFields(fields)
    }

    pub fn list(inner: Template) -> Template {
        Template::List(Box::new(inner))
    }
}

pub struct V135 {
    pub schema: Schema,
}

impl V135 {
    // JAVA: net.minecraft.util.datafix.schemas.V135#<init>
    pub fn new(version_key: i32, parent: &Schema) -> Self {
        Self {
            schema: Schema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V135#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut TypeMap,
        block_entity_types: &mut TypeMap,
    ) {
        let _ = (entity_types, block_entity_types);
        // super.registerTypes(schema, entityTypes, blockEntityTypes) - base no-op in self-contained stub.

        let sv = schema.version_key;
        schema.register_type(
            false,
            References::PLAYER,
            TemplateSupplier::new(move || {
                Dsl::optional_fields(vec![
                    Field::Named(
                        "RootVehicle".to_string(),
                        Dsl::optional_fields(vec![Field::Named(
                            "Entity".to_string(),
                            ref_in(References::ENTITY_TREE, sv),
                        )]),
                    ),
                    Field::Named(
                        "ender_pearls".to_string(),
                        Dsl::list(ref_in(References::ENTITY_TREE, sv)),
                    ),
                    Field::Named(
                        "Inventory".to_string(),
                        Dsl::list(ref_in(References::ITEM_STACK, sv)),
                    ),
                    Field::Named(
                        "EnderItems".to_string(),
                        Dsl::list(ref_in(References::ITEM_STACK, sv)),
                    ),
                ])
            }),
        );

        let sv = schema.version_key;
        schema.register_type(
            true,
            References::ENTITY_TREE,
            TemplateSupplier::new(move || {
                Dsl::optional_fields(vec![
                    Field::Named(
                        "Passengers".to_string(),
                        Dsl::list(ref_in(References::ENTITY_TREE, sv)),
                    ),
                    Field::Include(ref_in(References::ENTITY, sv)),
                ])
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_version_and_parent() {
        let parent = Schema::root(134);
        let v = V135::new(135, &parent);
        assert_eq!(v.schema.version_key, 135);
        assert_eq!(v.schema.parent_version, Some(134));
    }

    #[test]
    fn register_types_player_structure() {
        let parent = Schema::root(100);
        let v = V135::new(135, &parent);
        let mut schema = Schema::root(135);
        let mut entity_types = BTreeMap::new();
        let mut block_entity_types = BTreeMap::new();
        v.register_types(&mut schema, &mut entity_types, &mut block_entity_types);

        assert_eq!(schema.type_registrations.len(), 2);
        let player = &schema.type_registrations[0];
        assert!(!player.recursive);
        assert_eq!(player.reference, "player");

        let tpl = player.template.get();
        match tpl {
            Template::OptionalFields(fields) => {
                assert_eq!(fields.len(), 4);
                match &fields[0] {
                    Field::Named(name, Template::OptionalFields(inner)) => {
                        assert_eq!(name, "RootVehicle");
                        assert_eq!(inner.len(), 1);
                    }
                    _ => panic!("expected RootVehicle optional fields"),
                }
            }
            _ => panic!("expected optional fields"),
        }
    }

    #[test]
    fn register_types_entity_tree_recursive_with_include() {
        let parent = Schema::root(0);
        let v = V135::new(135, &parent);
        let mut schema = Schema::root(135);
        v.register_types(&mut schema, &mut BTreeMap::new(), &mut BTreeMap::new());

        let et = &schema.type_registrations[1];
        assert!(et.recursive);
        assert_eq!(et.reference, "entity_tree");
        let tpl = et.template.get();
        match tpl {
            Template::OptionalFields(fields) => {
                assert_eq!(fields.len(), 2);
                assert!(matches!(&fields[0], Field::Named(n, _) if n == "Passengers"));
                assert!(matches!(
                    &fields[1],
                    Field::Include(Template::Ref {
                        reference: "entity",
                        ..
                    })
                ));
            }
            _ => panic!("expected optional fields"),
        }
    }

    #[test]
    fn negative_overflow_and_float_bits_version_edge_cases() {
        let bits = f32::NEG_INFINITY.to_bits() as i32;
        let parent = Schema::root(f64::NAN.to_bits() as i32);
        let v = V135::new(bits, &parent);
        assert_eq!(v.schema.version_key, bits);
        assert_eq!(v.schema.parent_version, Some(f64::NAN.to_bits() as i32));

        let parent2 = Schema::root(-1);
        let v2 = V135::new(i32::MIN, &parent2);
        let mut schema = Schema::root(i32::MAX);
        v2.register_types(&mut schema, &mut BTreeMap::new(), &mut BTreeMap::new());
        let tpl = schema.type_registrations[0].template.get();
        if let Template::OptionalFields(fields) = tpl {
            if let Field::Named(_, Template::List(inner)) = &fields[1] {
                if let Template::Ref { schema_version, .. } = inner.as_ref() {
                    assert_eq!(*schema_version, i32::MAX);
                } else {
                    panic!("expected ref");
                }
            } else {
                panic!("expected ender_pearls list");
            }
        } else {
            panic!("expected optional fields");
        }
    }
}
