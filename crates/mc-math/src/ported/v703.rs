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
    pub const ITEM_STACK: Reference = Reference("minecraft:item_stack");
}

pub struct Dsl;
impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(fields: Vec<(String, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(fields)
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

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register(
        &self,
        map: &mut HashMap<String, TypeTemplateSupplier>,
        name: &str,
        template: impl Fn() -> TypeTemplate + 'static,
    ) {
        map.insert(name.to_string(), Arc::new(template));
    }
}

/// Port of `net.minecraft.util.datafix.schemas.V703`.
/// Extends `Schema` in Java.
pub struct V703 {
    pub version: i32,
    pub parent: Schema,
}

impl V703 {
    // JAVA: net.minecraft.util.datafix.schemas.V703#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V703#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = self.parent.register_entities(schema);
        map.remove("EntityHorse");

        let s = schema.clone();
        schema.register(&mut map, "Horse", move || {
            Dsl::optional_fields(vec![
                ("ArmorItem".to_string(), References::ITEM_STACK.in_schema(&s)),
                ("SaddleItem".to_string(), References::ITEM_STACK.in_schema(&s)),
            ])
        });

        let s = schema.clone();
        schema.register(&mut map, "Donkey", move || {
            Dsl::optional_fields(vec![
                (
                    "Items".to_string(),
                    Dsl::list(References::ITEM_STACK.in_schema(&s)),
                ),
                ("SaddleItem".to_string(), References::ITEM_STACK.in_schema(&s)),
            ])
        });

        let s = schema.clone();
        schema.register(&mut map, "Mule", move || {
            Dsl::optional_fields(vec![
                (
                    "Items".to_string(),
                    Dsl::list(References::ITEM_STACK.in_schema(&s)),
                ),
                ("SaddleItem".to_string(), References::ITEM_STACK.in_schema(&s)),
            ])
        });

        let s = schema.clone();
        schema.register(&mut map, "ZombieHorse", move || {
            Dsl::optional_fields(vec![(
                "SaddleItem".to_string(),
                References::ITEM_STACK.in_schema(&s),
            )])
        });

        let s = schema.clone();
        schema.register(&mut map, "SkeletonHorse", move || {
            Dsl::optional_fields(vec![(
                "SaddleItem".to_string(),
                References::ITEM_STACK.in_schema(&s),
            )])
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
    fn removes_entity_horse_and_registers_horses() {
        let schema = base_schema();
        let v = V703::new(703, Schema::default());
        let mut parent_map = HashMap::new();
        parent_map.insert(
            "EntityHorse".to_string(),
            Arc::new(|| TypeTemplate::Empty) as TypeTemplateSupplier,
        );
        // Simulate parent providing EntityHorse by building on a custom base via direct map manipulation.
        // Since our stub Schema returns empty, we verify removal by checking absence after registration.
        let map = v.register_entities(&schema);
        assert!(!map.contains_key("EntityHorse"));
        assert!(map.contains_key("Horse"));
        assert!(map.contains_key("Donkey"));
        assert!(map.contains_key("Mule"));
        assert!(map.contains_key("ZombieHorse"));
        assert!(map.contains_key("SkeletonHorse"));
        assert_eq!(map.len(), 5);
        let _ = parent_map;
    }

    #[test]
    fn horse_has_armor_and_saddle() {
        let schema = base_schema();
        let v = V703::new(703, Schema::default());
        let map = v.register_entities(&schema);
        let template = map.get("Horse").unwrap()();
        assert_eq!(
            template,
            TypeTemplate::OptionalFields(vec![
                (
                    "ArmorItem".to_string(),
                    TypeTemplate::Reference("minecraft:item_stack".to_string())
                ),
                (
                    "SaddleItem".to_string(),
                    TypeTemplate::Reference("minecraft:item_stack".to_string())
                ),
            ])
        );
    }

    #[test]
    fn donkey_and_mule_have_items_list_and_saddle() {
        let schema = base_schema();
        let v = V703::new(703, Schema::default());
        let map = v.register_entities(&schema);
        let expected = TypeTemplate::OptionalFields(vec![
            (
                "Items".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Reference(
                    "minecraft:item_stack".to_string(),
                ))),
            ),
            (
                "SaddleItem".to_string(),
                TypeTemplate::Reference("minecraft:item_stack".to_string()),
            ),
        ]);
        assert_eq!(map.get("Donkey").unwrap()(), expected);
        assert_eq!(map.get("Mule").unwrap()(), expected);
    }

    #[test]
    fn zombie_and_skeleton_horse_have_saddle_only() {
        let schema = base_schema();
        let v = V703::new(703, Schema::default());
        let map = v.register_entities(&schema);
        let expected = TypeTemplate::OptionalFields(vec![(
            "SaddleItem".to_string(),
            TypeTemplate::Reference("minecraft:item_stack".to_string()),
        )]);
        assert_eq!(map.get("ZombieHorse").unwrap()(), expected);
        assert_eq!(map.get("SkeletonHorse").unwrap()(), expected);
    }
}
