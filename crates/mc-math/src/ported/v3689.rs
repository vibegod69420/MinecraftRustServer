use std::collections::HashMap;
use std::sync::Arc;

pub type TypeTemplateSupplier = Arc<dyn Fn() -> TypeTemplate>;

#[derive(Clone)]
pub enum TypeTemplate {
    Empty,
    Reference(String),
    OptionalFields(Vec<(String, TypeTemplate)>),
    Fields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
    Or(Box<TypeTemplate>, Box<TypeTemplate>),
    OptionalFieldsLazy(Vec<(String, TypeTemplateSupplier)>),
}

impl std::fmt::Debug for TypeTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Reference(r) => f.debug_tuple("Reference").field(r).finish(),
            Self::OptionalFields(v) => f.debug_tuple("OptionalFields").field(v).finish(),
            Self::Fields(v) => f.debug_tuple("Fields").field(v).finish(),
            Self::List(t) => f.debug_tuple("List").field(t).finish(),
            Self::Or(a, b) => f.debug_tuple("Or").field(a).field(b).finish(),
            Self::OptionalFieldsLazy(v) => {
                let resolved: Vec<_> = v.iter().map(|(k, s)| (k.clone(), s())).collect();
                f.debug_tuple("OptionalFieldsLazy").field(&resolved).finish()
            }
        }
    }
}

impl PartialEq for TypeTemplate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Reference(a), Self::Reference(b)) => a == b,
            (Self::OptionalFields(a), Self::OptionalFields(b)) => a == b,
            (Self::Fields(a), Self::Fields(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Or(a, b), Self::Or(c, d)) => a == c && b == d,
            (Self::OptionalFieldsLazy(a), Self::OptionalFieldsLazy(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|((ka, va), (kb, vb))| ka == kb && va() == vb())
            }
            _ => false,
        }
    }
}

impl Eq for TypeTemplate {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reference(pub &'static str);

impl Reference {
    // JAVA: com.mojang.datafixers.DSL.TypeReference#in
    pub fn in_schema(self, _schema: &Schema) -> TypeTemplate {
        TypeTemplate::Reference(self.0.to_string())
    }
}

pub struct References;
impl References {
    pub const ENTITY_TREE: Reference = Reference("minecraft:entity_tree");
}

pub struct Dsl;
impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(fields: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(fields)
    }

    // JAVA: com.mojang.datafixers.DSL#fields
    pub fn fields(fields: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::Fields(fields)
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(template: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(template))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#<init>
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> HashMap<String, TypeTemplateSupplier> {
        HashMap::new()
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerBlockEntities
    pub fn register_block_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, TypeTemplateSupplier> {
        HashMap::new()
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register(
        &self,
        map: &mut HashMap<String, TypeTemplateSupplier>,
        name: &str,
        template: impl Fn() -> TypeTemplate + 'static,
    ) {
        map.insert(name.to_string(), Arc::new(template));
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(&self, map: &mut HashMap<String, TypeTemplateSupplier>, name: &str) {
        map.insert(name.to_string(), Arc::new(|| TypeTemplate::Empty));
    }
}

/// Port of `net.minecraft.util.datafix.schemas.V3689`.
/// Extends `NamespacedSchema` in Java.
pub struct V3689 {
    pub version: i32,
    pub parent: Schema,
}

impl V3689 {
    // JAVA: net.minecraft.util.datafix.schemas.V3689#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3689#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = self.parent.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:breeze");
        schema.register_simple(&mut map, "minecraft:wind_charge");
        schema.register_simple(&mut map, "minecraft:breeze_wind_charge");
        map
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3689#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = self.parent.register_block_entities(schema);
        let schema_clone = schema.clone();
        schema.register(&mut map, "minecraft:trial_spawner", move || {
            Dsl::optional_fields(vec![
                (
                    "spawn_potentials".to_string(),
                    Dsl::list(Dsl::fields(vec![(
                        "data".to_string(),
                        Dsl::fields(vec![(
                            "entity".to_string(),
                            References::ENTITY_TREE.in_schema(&schema_clone),
                        )]),
                    )])),
                ),
                (
                    "spawn_data".to_string(),
                    Dsl::fields(vec![(
                        "entity".to_string(),
                        References::ENTITY_TREE.in_schema(&schema_clone),
                    )]),
                ),
            ])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_schema() -> Schema {
        Schema::new(0, Schema::default())
    }

    #[test]
    fn register_entities_adds_breeze_entities() {
        let schema = base_schema();
        let v = V3689::new(3689, Schema::default());
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 3);
        for name in [
            "minecraft:breeze",
            "minecraft:wind_charge",
            "minecraft:breeze_wind_charge",
        ] {
            assert_eq!(map.get(name).unwrap()(), TypeTemplate::Empty, "{name}");
        }
    }

    #[test]
    fn register_block_entities_adds_trial_spawner() {
        let schema = base_schema();
        let v = V3689::new(3689, Schema::default());
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        let template = map.get("minecraft:trial_spawner").unwrap()();
        let entity_ref = TypeTemplate::Reference("minecraft:entity_tree".to_string());
        let expected = Dsl::optional_fields(vec![
            (
                "spawn_potentials".to_string(),
                Dsl::list(Dsl::fields(vec![(
                    "data".to_string(),
                    Dsl::fields(vec![("entity".to_string(), entity_ref.clone())]),
                )])),
            ),
            (
                "spawn_data".to_string(),
                Dsl::fields(vec![("entity".to_string(), entity_ref)]),
            ),
        ]);
        assert_eq!(template, expected);
    }

    #[test]
    fn constructor_sets_version_and_parent() {
        let parent = Schema::new(99, Schema::default());
        let v = V3689::new(3689, parent);
        assert_eq!(v.version, 3689);
        assert_eq!(v.parent.version, 99);
    }

    #[test]
    fn trial_spawner_spawn_data_has_entity_field() {
        let schema = base_schema();
        let v = V3689::new(3689, Schema::default());
        let map = v.register_block_entities(&schema);
        let template = map.get("minecraft:trial_spawner").unwrap()();
        if let TypeTemplate::OptionalFields(fields) = template {
            let spawn_data = fields.iter().find(|(k, _)| k == "spawn_data").unwrap();
            assert_eq!(
                spawn_data.1,
                TypeTemplate::Fields(vec![(
                    "entity".to_string(),
                    TypeTemplate::Reference("minecraft:entity_tree".to_string())
                )])
            );
        } else {
            panic!("expected OptionalFields");
        }
    }
}
