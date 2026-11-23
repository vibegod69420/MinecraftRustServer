use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    Empty,
    Ref(&'static str),
    List(Box<Template>),
    OptionalFields(Vec<(&'static str, Template)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceType(pub &'static str);

impl ReferenceType {
    pub fn r#in(self, _schema: &NamespacedSchema) -> Template {
        Template::Ref(self.0)
    }
}

pub struct References;

impl References {
    pub const VILLAGER_TRADE: ReferenceType = ReferenceType("References.VILLAGER_TRADE");
    pub const ITEM_STACK: ReferenceType = ReferenceType("References.ITEM_STACK");
    pub const ITEM_NAME: ReferenceType = ReferenceType("References.ITEM_NAME");
}

pub struct Dsl;

impl Dsl {
    pub fn list(template: Template) -> Template {
        Template::List(Box::new(template))
    }

    pub fn optional_fields(fields: Vec<(&'static str, Template)>) -> Template {
        Template::OptionalFields(fields)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespacedSchema {
    pub version: i32,
    pub parent_version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, parent: NamespacedSchema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerEntities
    pub fn register_entities(&self, _schema: &NamespacedSchema) -> BTreeMap<String, Template> {
        BTreeMap::new()
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register<F>(&self, map: &mut BTreeMap<String, Template>, name: &str, factory: F)
    where
        F: FnOnce(&NamespacedSchema) -> Template,
    {
        map.insert(name.to_string(), factory(self));
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(&self, map: &mut BTreeMap<String, Template>, name: &str) {
        map.insert(name.to_string(), Template::Empty);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct V3808_1 {
    pub version: i32,
    pub parent_version: i32,
}

impl V3808_1 {
    // JAVA: net.minecraft.util.datafix.schemas.V3808_1#<init>
    pub fn new(version: i32, parent: NamespacedSchema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3808_1#registerEntities
    pub fn register_entities(&self, schema: &NamespacedSchema) -> BTreeMap<String, Template> {
        let mut map = schema.register_entities(schema);
        schema.register(&mut map, "minecraft:llama", |_s| {
            Dsl::optional_fields(vec![
                ("Items", Dsl::list(References::ITEM_STACK.r#in(schema))),
                ("SaddleItem", References::ITEM_STACK.r#in(schema)),
            ])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_entities_adds_llama() {
        let schema = NamespacedSchema::default();
        let v = V3808_1::new(3808, NamespacedSchema::default());
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 1);
        let expected = Template::OptionalFields(vec![
            (
                "Items",
                Template::List(Box::new(Template::Ref("References.ITEM_STACK"))),
            ),
            ("SaddleItem", Template::Ref("References.ITEM_STACK")),
        ]);
        assert_eq!(map.get("minecraft:llama"), Some(&expected));
    }

    #[test]
    fn register_entities_returns_empty_base_plus_llama() {
        let schema = NamespacedSchema::default();
        let v = V3808_1::new(0, NamespacedSchema::default());
        let map = v.register_entities(&schema);
        assert!(map.contains_key("minecraft:llama"));
        assert!(!map.contains_key("minecraft:horse"));
    }

    #[test]
    fn constructor_negative_and_overflow_versions() {
        let v = V3808_1::new(-3808, NamespacedSchema { version: -1, parent_version: 0 });
        assert_eq!(v.version, -3808);
        assert_eq!(v.parent_version, -1);
        let max = i32::MAX;
        assert_eq!(max.wrapping_add(1), i32::MIN);
    }

    #[test]
    fn llama_fields_reference_item_stack() {
        let schema = NamespacedSchema::default();
        let v = V3808_1::new(3808, NamespacedSchema::default());
        let map = v.register_entities(&schema);
        let tpl = map.get("minecraft:llama").unwrap();
        if let Template::OptionalFields(fields) = tpl {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "Items");
            assert_eq!(fields[1].0, "SaddleItem");
        } else {
            panic!("expected optional fields");
        }
    }

    #[test]
    fn float_bits_and_negative_integer_edge_cases() {
        let bits = f32::NEG_INFINITY.to_bits();
        assert_eq!(bits, 0xFF80_0000);
        let n: i64 = -1;
        assert_eq!(n.wrapping_mul(i64::MIN), i64::MIN); // overflow edge
    }
}
