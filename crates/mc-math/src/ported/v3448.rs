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

    // JAVA: com.mojang.datafixers.schemas.Schema#registerBlockEntities
    pub fn register_block_entities(&self, _schema: &NamespacedSchema) -> BTreeMap<String, Template> {
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
pub struct V3448 {
    pub version: i32,
    pub parent_version: i32,
}

impl V3448 {
    // JAVA: net.minecraft.util.datafix.schemas.V3448#<init>
    pub fn new(version: i32, parent: NamespacedSchema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3448#registerBlockEntities
    pub fn register_block_entities(&self, schema: &NamespacedSchema) -> BTreeMap<String, Template> {
        let mut map = schema.register_block_entities(schema);
        schema.register(&mut map, "minecraft:decorated_pot", |_| {
            Dsl::optional_fields(vec![
                ("sherds", Dsl::list(References::ITEM_NAME.r#in(schema))),
                ("item", References::ITEM_STACK.r#in(schema)),
            ])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_block_entities_adds_decorated_pot() {
        let schema = NamespacedSchema::default();
        let v = V3448::new(3448, NamespacedSchema::default());
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        let expected = Template::OptionalFields(vec![
            (
                "sherds",
                Template::List(Box::new(Template::Ref("References.ITEM_NAME"))),
            ),
            ("item", Template::Ref("References.ITEM_STACK")),
        ]);
        assert_eq!(map.get("minecraft:decorated_pot"), Some(&expected));
    }

    #[test]
    fn decorated_pot_uses_item_name_and_item_stack_refs() {
        let schema = NamespacedSchema::default();
        let v = V3448::new(3448, NamespacedSchema::default());
        let map = v.register_block_entities(&schema);
        let tpl = map.get("minecraft:decorated_pot").unwrap();
        match tpl {
            Template::OptionalFields(fields) => {
                assert_eq!(fields[0].0, "sherds");
                assert_eq!(fields[1].0, "item");
            }
            _ => panic!("expected optional fields"),
        }
    }

    #[test]
    fn constructor_negative_and_overflow_versions() {
        let v = V3448::new(-1, NamespacedSchema { version: i32::MIN, parent_version: 0 });
        assert_eq!(v.version, -1);
        assert_eq!(v.parent_version, i32::MIN);
        let ov = i32::MAX.wrapping_mul(2);
        assert_eq!(ov, -2); // overflow edge
    }

    #[test]
    fn register_simple_and_empty_map_behavior() {
        let schema = NamespacedSchema::default();
        let mut map = BTreeMap::new();
        schema.register_simple(&mut map, "test");
        assert_eq!(map.get("test"), Some(&Template::Empty));
        assert!(schema.register_block_entities(&schema).is_empty());
    }

    #[test]
    fn float_bits_and_negative_inputs() {
        let neg_zero = (-0.0f32).to_bits();
        assert_eq!(neg_zero, 0x8000_0000);
        let pos_inf = f32::INFINITY.to_bits();
        assert_eq!(pos_inf, 0x7F80_0000);
        let neg: i32 = -128;
        assert_eq!(neg.wrapping_shr(1), (-64i32) as u32 as i32); // negative input
    }
}
