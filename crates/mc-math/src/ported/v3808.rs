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

pub struct V3808 {
    pub base: NamespacedSchema,
}

impl V3808 {
    // JAVA: net.minecraft.util.datafix.schemas.V3808#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3808#registerEntities
    pub fn register_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, Box<dyn Fn(&Schema) -> TypeTemplate>> {
        let mut map = self.base.register_entities(schema);
        let s = schema.clone();
        schema.register(&mut map, "minecraft:horse", move |_entity_schema| {
            Dsl::optional_fields(vec![("SaddleItem", References::item_stack(&s))])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_stores_version_and_parent() {
        let parent = Schema { version: 3807 };
        let schema = V3808::new(3808, parent.clone());
        assert_eq!(schema.base.version, 3808);
        assert_eq!(schema.base.parent, parent);
    }

    #[test]
    fn constructor_handles_integer_edge_cases() {
        let min = V3808::new(i32::MIN, Schema { version: i32::MAX });
        assert_eq!(min.base.version, i32::MIN);
        assert_eq!(min.base.parent.version, i32::MAX);

        let neg = V3808::new(-1, Schema { version: -100 });
        assert_eq!(neg.base.version, -1);
        assert_eq!(neg.base.parent.version, -100);
    }

    #[test]
    fn register_entities_adds_horse_with_saddle_item() {
        let schema = V3808::new(3808, Schema { version: 3807 });
        let ds = Schema { version: 3808 };
        let map = schema.register_entities(&ds);
        assert_eq!(map.len(), 1);
        let tpl = map.get("minecraft:horse").expect("horse entry")(&ds);
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "SaddleItem".to_string(),
                TypeTemplate::ItemStack
            )])
        );
    }

    #[test]
    fn register_entities_closure_is_stable_across_input_schemas() {
        let schema = V3808::new(3808, Schema { version: 3807 });
        let ds = Schema { version: 3808 };
        let other = Schema { version: 0 };
        let map = schema.register_entities(&ds);
        let f = map.get("minecraft:horse").expect("horse entry");
        assert_eq!(f(&ds), f(&other));
    }
}
