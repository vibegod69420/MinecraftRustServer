use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Reference {
    Player,
    EntityTree,
    EntityEquipment,
    ItemStack,
    Recipe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeTemplate {
    Reference(Reference),
    List(Box<TypeTemplate>),
    And(Box<TypeTemplate>, Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

impl Reference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference(self.clone())
    }
}

pub mod dsl {
    use super::TypeTemplate;

    pub fn list(t: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(t))
    }

    pub fn and(a: TypeTemplate, b: TypeTemplate) -> TypeTemplate {
        TypeTemplate::And(Box::new(a), Box::new(b))
    }

    pub fn optional_fields(entries: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
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

pub struct V4312 {
    pub base: NamespacedSchema,
}

impl V4312 {
    // JAVA: net.minecraft.util.datafix.schemas.V4312#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4312#registerTypes
    pub fn register_types(
        &self,
        schema: &mut Schema,
        entity_types: &mut BTreeMap<String, TemplateSupplier>,
        block_entity_types: &mut BTreeMap<String, TemplateSupplier>,
    ) {
        self.base
            .register_types(schema, entity_types, block_entity_types);
        let s: &Schema = &*schema;
        let player_template = dsl::and(
            Reference::EntityEquipment.in_schema(s),
            dsl::optional_fields(vec![
                (
                    "RootVehicle",
                    dsl::optional_fields(vec![("Entity", Reference::EntityTree.in_schema(s))]),
                ),
                ("ender_pearls", dsl::list(Reference::EntityTree.in_schema(s))),
                ("Inventory", dsl::list(Reference::ItemStack.in_schema(s))),
                ("EnderItems", dsl::list(Reference::ItemStack.in_schema(s))),
                ("ShoulderEntityLeft", Reference::EntityTree.in_schema(s)),
                ("ShoulderEntityRight", Reference::EntityTree.in_schema(s)),
                (
                    "recipeBook",
                    dsl::optional_fields(vec![
                        ("recipes", dsl::list(Reference::Recipe.in_schema(s))),
                        ("toBeDisplayed", dsl::list(Reference::Recipe.in_schema(s))),
                    ]),
                ),
            ]),
        );
        schema.register_type(false, Reference::Player, player_template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (V4312, Schema, BTreeMap<String, TemplateSupplier>, BTreeMap<String, TemplateSupplier>) {
        let v = V4312::new(4312, Schema::default());
        (v, Schema::default(), BTreeMap::new(), BTreeMap::new())
    }

    #[test]
    fn registers_player_non_recursive() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        assert_eq!(schema.registrations.len(), 1);
        assert!(!schema.registrations[0].recursive);
        assert_eq!(schema.registrations[0].reference, Reference::Player);
    }

    #[test]
    fn player_template_is_and_with_entity_equipment() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::And(left, _right) = &schema.registrations[0].template else {
            panic!("expected And");
        };
        assert_eq!(**left, TypeTemplate::Reference(Reference::EntityEquipment));
    }

    #[test]
    fn player_optional_fields_contain_expected_keys() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::And(_, right) = &schema.registrations[0].template else {
            panic!("expected And");
        };
        let TypeTemplate::OptionalFields(fields) = &**right else {
            panic!("expected OptionalFields");
        };
        let names: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
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

    #[test]
    fn recipe_book_has_recipes_and_to_be_displayed() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::And(_, right) = &schema.registrations[0].template else {
            panic!("expected And");
        };
        let TypeTemplate::OptionalFields(fields) = &**right else {
            panic!("expected OptionalFields");
        };
        let recipe_book = &fields[6].1;
        let TypeTemplate::OptionalFields(rb_fields) = recipe_book else {
            panic!("expected OptionalFields");
        };
        assert_eq!(rb_fields[0].0, "recipes");
        assert_eq!(rb_fields[1].0, "toBeDisplayed");
        assert_eq!(rb_fields[0].1, TypeTemplate::List(Box::new(TypeTemplate::Reference(Reference::Recipe))));
        assert_eq!(rb_fields[1].1, TypeTemplate::List(Box::new(TypeTemplate::Reference(Reference::Recipe))));
    }

    #[test]
    fn root_vehicle_contains_entity_tree() {
        let (v, mut schema, mut et, mut bet) = setup();
        v.register_types(&mut schema, &mut et, &mut bet);
        let TypeTemplate::And(_, right) = &schema.registrations[0].template else {
            panic!("expected And");
        };
        let TypeTemplate::OptionalFields(fields) = &**right else {
            panic!("expected OptionalFields");
        };
        let TypeTemplate::OptionalFields(rv) = &fields[0].1 else {
            panic!("expected OptionalFields");
        };
        assert_eq!(rv[0].0, "Entity");
        assert_eq!(rv[0].1, TypeTemplate::Reference(Reference::EntityTree));
    }
}
