use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn register(
        &self,
        map: &mut BTreeMap<String, Box<dyn Fn(&Schema) -> TypeTemplate>>,
        name: &str,
        template: impl Fn(&Schema) -> TypeTemplate + 'static,
    ) {
        map.insert(name.to_string(), Box::new(template));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeTemplate {
    ItemStack,
    List(Box<TypeTemplate>),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

pub struct References;

impl References {
    pub fn item_stack(_schema: &Schema) -> TypeTemplate {
        TypeTemplate::ItemStack
    }
}

pub struct Dsl;

impl Dsl {
    pub fn optional_fields(fields: Vec<(&str, TypeTemplate)>) -> TypeTemplate {
        TypeTemplate::OptionalFields(
            fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        )
    }

    pub fn list(inner: TypeTemplate) -> TypeTemplate {
        TypeTemplate::List(Box::new(inner))
    }
}

pub struct NamespacedSchema {
    pub version: i32,
    pub parent: Schema,
}

impl NamespacedSchema {
    pub fn new(version: i32, parent: Schema) -> Self {
        Self { version, parent }
    }

    pub fn register_entities(
        &self,
        _schema: &Schema,
    ) -> BTreeMap<String, Box<dyn Fn(&Schema) -> TypeTemplate>> {
        BTreeMap::new()
    }
}

pub struct V2505 {
    pub base: NamespacedSchema,
}

impl V2505 {
    // JAVA: net.minecraft.util.datafix.schemas.V2505#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V2505#registerEntities
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, Box<dyn Fn(&Schema) -> TypeTemplate>> {
        let mut map = self.base.register_entities(schema);
        let s = schema.clone();
        schema.register(&mut map, "minecraft:piglin", move |_entity_schema| {
            Dsl::optional_fields(vec![(
                "Inventory",
                Dsl::list(References::item_stack(&s)),
            )])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_and_parent() {
        let parent = Schema { version: 2504 };
        let schema = V2505::new(2505, parent.clone());
        assert_eq!(schema.base.version, 2505);
        assert_eq!(schema.base.parent, parent);
    }

    #[test]
    fn constructor_handles_integer_edge_cases() {
        let min = V2505::new(i32::MIN, Schema { version: i32::MIN });
        assert_eq!(min.base.version, i32::MIN);
        assert_eq!(min.base.parent.version, i32::MIN);
        let max = V2505::new(i32::MAX, Schema { version: i32::MAX });
        assert_eq!(max.base.version, i32::MAX);
        assert_eq!(max.base.parent.version, i32::MAX);
    }

    #[test]
    fn register_entities_adds_piglin_with_inventory_list() {
        let schema = V2505::new(2505, Schema { version: 2504 });
        let ds = Schema { version: 2505 };
        let map = schema.register_entities(&ds);
        assert_eq!(map.len(), 1);
        let tpl = map.get("minecraft:piglin").expect("piglin entry")(&ds);
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Inventory".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::ItemStack))
            )])
        );
    }

    #[test]
    fn register_entities_closure_is_stable_across_input_schemas() {
        let schema = V2505::new(2505, Schema { version: 2504 });
        let ds = Schema { version: 2505 };
        let other = Schema { version: -999 };
        let map = schema.register_entities(&ds);
        let f = map.get("minecraft:piglin").expect("piglin entry");
        assert_eq!(f(&ds), f(&other));
    }
}
