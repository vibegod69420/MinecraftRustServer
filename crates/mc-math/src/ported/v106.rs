use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reference(pub &'static str);

impl Reference {
    pub fn in_schema(&self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Ref(self.0.to_string())
    }
}

pub struct References;

impl References {
    pub const UNTAGGED_SPAWNER: Reference = Reference("minecraft:untagged_spawner");
    pub const ENTITY_TREE: Reference = Reference("minecraft:entity_tree");
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Empty,
    Ref(String),
    List(Box<TypeTemplate>),
    Fields(Vec<(String, TypeTemplate)>),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        )
    }

    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }

    pub fn fields(pairs: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::Fields(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
    pub registered_types: BTreeMap<String, (bool, TypeTemplate)>,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self {
            version,
            registered_types: BTreeMap::new(),
        }
    }

    pub fn register_type(
        &mut self,
        recursive: bool,
        reference: &Reference,
        template: TypeTemplate,
    ) {
        self.registered_types
            .insert(reference.0.to_string(), (recursive, template));
    }
}

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct V106 {
    version: i32,
}

impl V106 {
    // JAVA: net.minecraft.util.datafix.schemas.V106#<init>(int, Schema)
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V106#registerTypes(Schema, Map, Map)
    pub fn register_types(
        &self,
        schema: &mut Schema,
        types: &mut BTreeMap<String, TypeTemplateSupplier>,
        entity_types: &mut BTreeMap<String, TypeTemplateSupplier>,
    ) {
        // super.registerTypes(schema, types, entity_types) - base Schema is no-op
        let _ = (types, entity_types);
        let entity_tree = References::ENTITY_TREE.in_schema(schema);
        let template = Dsl::optional_fields(vec![
            (
                "SpawnPotentials",
                Dsl::list(Dsl::fields(vec![("Entity", entity_tree.clone())])),
            ),
            ("SpawnData", entity_tree),
        ]);
        schema.register_type(true, &References::UNTAGGED_SPAWNER, template);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v106_new_stores_version() {
        let v = V106::new(106, Schema::new(0));
        assert_eq!(v.version, 106);
    }

    #[test]
    fn v106_registers_untagged_spawner() {
        let mut schema = Schema::new(106);
        let mut types = BTreeMap::new();
        let mut entity_types = BTreeMap::new();
        V106::new(106, Schema::new(0)).register_types(&mut schema, &mut types, &mut entity_types);

        assert!(types.is_empty());
        assert!(entity_types.is_empty());
        let (recursive, tpl) = schema
            .registered_types
            .get("minecraft:untagged_spawner")
            .expect("untagged_spawner registered");
        assert!(*recursive);
        match tpl {
            TypeTemplate::OptionalFields(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "SpawnPotentials");
                assert_eq!(fields[1].0, "SpawnData");
            }
            other => panic!("expected OptionalFields, got {:?}", other),
        }
    }

    #[test]
    fn v106_spawn_potentials_structure() {
        let mut schema = Schema::new(106);
        let mut types = BTreeMap::new();
        let mut entity_types = BTreeMap::new();
        V106::new(106, Schema::new(0)).register_types(&mut schema, &mut types, &mut entity_types);

        let (_, tpl) = &schema.registered_types["minecraft:untagged_spawner"];
        let TypeTemplate::OptionalFields(fields) = tpl else {
            panic!("expected OptionalFields");
        };
        match &fields[0].1 {
            TypeTemplate::List(inner) => match inner.as_ref() {
                TypeTemplate::Fields(f) => {
                    assert_eq!(f.len(), 1);
                    assert_eq!(f[0].0, "Entity");
                    assert_eq!(
                        f[0].1,
                        TypeTemplate::Ref("minecraft:entity_tree".to_string())
                    );
                }
                other => panic!("expected Fields, got {:?}", other),
            },
            other => panic!("expected List, got {:?}", other),
        }
    }

    #[test]
    fn v106_spawn_data_is_entity_tree_ref() {
        let mut schema = Schema::new(106);
        let mut types = BTreeMap::new();
        let mut entity_types = BTreeMap::new();
        V106::new(106, Schema::new(0)).register_types(&mut schema, &mut types, &mut entity_types);

        let (_, tpl) = &schema.registered_types["minecraft:untagged_spawner"];
        let TypeTemplate::OptionalFields(fields) = tpl else {
            panic!("expected OptionalFields");
        };
        assert_eq!(
            fields[1].1,
            TypeTemplate::Ref("minecraft:entity_tree".to_string())
        );
    }
}
