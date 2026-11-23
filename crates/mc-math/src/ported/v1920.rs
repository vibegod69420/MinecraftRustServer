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
    pub const ITEM_STACK: Reference = Reference("minecraft:item_stack");
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Empty,
    Ref(String),
    List(Box<TypeTemplate>),
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
}

pub type TypeTemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
}

impl Schema {
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    pub fn register(
        &self,
        map: &mut BTreeMap<String, TypeTemplateSupplier>,
        name: &str,
        template: TypeTemplate,
    ) {
        map.insert(name.to_string(), Box::new(move || template.clone()));
    }
}

pub struct NamespacedSchema;

impl NamespacedSchema {
    pub fn register_block_entities(_schema: &Schema) -> BTreeMap<String, TypeTemplateSupplier> {
        BTreeMap::new()
    }
}

#[derive(Clone, Debug, Default)]
pub struct V1920 {
    version: i32,
}

impl V1920 {
    // JAVA: net.minecraft.util.datafix.schemas.V1920#<init>(int, Schema)
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1920#registerInventory(Schema, Map, String)
    pub fn register_inventory(
        schema: &Schema,
        map: &mut BTreeMap<String, TypeTemplateSupplier>,
        name: &str,
    ) {
        let template = Dsl::optional_fields(vec![(
            "Items",
            Dsl::list(References::ITEM_STACK.in_schema(schema)),
        )]);
        schema.register(map, name, template);
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1920#registerBlockEntities(Schema)
    pub fn register_block_entities(
        &self,
        schema: &Schema,
    ) -> BTreeMap<String, TypeTemplateSupplier> {
        let mut map = NamespacedSchema::register_block_entities(schema);
        Self::register_inventory(schema, &mut map, "minecraft:campfire");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1920_new_stores_version() {
        let v = V1920::new(1920, Schema::new(0));
        assert_eq!(v.version, 1920);
    }

    #[test]
    fn v1920_registers_campfire_block_entity() {
        let schema = Schema::new(1920);
        let v = V1920::new(1920, Schema::new(0));
        let map = v.register_block_entities(&schema);
        assert!(map.contains_key("minecraft:campfire"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn v1920_campfire_has_items_list() {
        let schema = Schema::new(1920);
        let v = V1920::new(1920, Schema::new(0));
        let map = v.register_block_entities(&schema);
        let tpl = map["minecraft:campfire"]();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Items".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Ref(
                    "minecraft:item_stack".to_string()
                )))
            )])
        );
    }

    #[test]
    fn register_inventory_registers_arbitrary_name() {
        let schema = Schema::new(1920);
        let mut map = BTreeMap::new();
        V1920::register_inventory(&schema, &mut map, "minecraft:custom");
        assert!(map.contains_key("minecraft:custom"));
        let tpl = map["minecraft:custom"]();
        match tpl {
            TypeTemplate::OptionalFields(fields) => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "Items");
            }
            other => panic!("expected OptionalFields, got {:?}", other),
        }
    }
}
