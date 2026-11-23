use std::collections::HashMap;

pub type Supplier<T> = Box<dyn Fn() -> T>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    OptionalFields(Vec<(String, TypeTemplate)>),
    List(Box<TypeTemplate>),
    Ref(String),
}

pub struct Dsl;

impl Dsl {
    // JAVA: com.mojang.datafixers.DSL#optionalFields
    pub fn optional_fields(fields: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            fields
                .into_iter()
                .map(|(name, template)| (name.to_string(), template))
                .collect(),
        )
    }

    // JAVA: com.mojang.datafixers.DSL#list
    pub fn list(element: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(element))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(
        &self,
        map: &mut HashMap<String, Supplier<TypeTemplate>>,
        name: &str,
    ) {
        let name_owned = name.to_string();
        map.insert(name_owned, Box::new(|| TypeTemplate::Empty));
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register<F>(
        &self,
        map: &mut HashMap<String, Supplier<TypeTemplate>>,
        name: &str,
        template_factory: F,
    ) where
        F: Fn(String) -> TypeTemplate + 'static,
    {
        let name_owned = name.to_string();
        let factory_name = name_owned.clone();
        map.insert(
            name_owned,
            Box::new(move || template_factory(factory_name.clone())),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum References {
    ItemStack,
}

impl References {
    pub const ITEM_STACK: References = References::ItemStack;

    // JAVA: com.mojang.datafixers.TypedOptic#in
    pub fn in_schema(self, _schema: &Schema) -> TypeTemplate {
        match self {
            References::ItemStack => TypeTemplate::Ref("References.ITEM_STACK".to_string()),
        }
    }
}

pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Option<Schema>,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent: Some(parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(
        &self,
        _schema: &Schema,
    ) -> HashMap<String, Supplier<TypeTemplate>> {
        HashMap::new()
    }
}

pub struct V1800 {
    pub base: NamespacedSchema,
}

impl V1800 {
    // JAVA: net.minecraft.util.datafix.schemas.V1800#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1800#registerEntities
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> HashMap<String, Supplier<TypeTemplate>> {
        let mut map = self.base.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:panda");
        let schema_copy = *schema;
        schema.register(&mut map, "minecraft:pillager", move |_name| {
            Dsl::optional_fields(vec![(
                "Inventory",
                Dsl::list(References::ITEM_STACK.in_schema(&schema_copy)),
            )])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_schema() -> (V1800, Schema) {
        let schema = Schema::new(1800);
        let v = V1800::new(1800, Schema::new(0));
        (v, schema)
    }

    #[test]
    fn test_register_entities_contains_panda_simple() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        let tpl = map.get("minecraft:panda").expect("missing panda")();
        assert_eq!(tpl, TypeTemplate::Empty);
    }

    #[test]
    fn test_register_entities_pillager_template() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        let tpl = map
            .get("minecraft:pillager")
            .expect("missing pillager")();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Inventory".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Ref(
                    "References.ITEM_STACK".to_string(),
                ))),
            )])
        );
    }

    #[test]
    fn test_register_entities_count() {
        let (v, schema) = make_schema();
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("minecraft:panda"));
        assert!(map.contains_key("minecraft:pillager"));
    }

    #[test]
    fn test_schema_is_copy_for_static_closure_capture() {
        let schema = Schema::new(99);
        let copied = schema;
        assert_eq!(schema.version, 99);
        let factory: Box<dyn Fn() -> TypeTemplate> = Box::new(move || {
            Dsl::list(References::ITEM_STACK.in_schema(&copied))
        });
        assert_eq!(
            factory(),
            TypeTemplate::List(Box::new(TypeTemplate::Ref(
                "References.ITEM_STACK".to_string(),
            )))
        );
    }

    #[test]
    fn test_constructor_preserves_version() {
        let parent = Schema::new(7);
        let v = V1800::new(1800, parent);
        assert_eq!(v.base.version, 1800);
        assert_eq!(v.base.parent, Some(Schema::new(7)));
    }
}
