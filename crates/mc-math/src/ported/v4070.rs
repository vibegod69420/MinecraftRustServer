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
pub struct V4070 {
    pub version: i32,
    pub parent_version: i32,
}

impl V4070 {
    // JAVA: net.minecraft.util.datafix.schemas.V4070#<init>
    pub fn new(version: i32, parent: NamespacedSchema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4070#registerEntities
    pub fn register_entities(&self, schema: &NamespacedSchema) -> BTreeMap<String, Template> {
        let mut map = schema.register_entities(schema);
        schema.register_simple(&mut map, "minecraft:pale_oak_boat");
        schema.register(&mut map, "minecraft:pale_oak_chest_boat", |_s| {
            Dsl::optional_fields(vec![(
                "Items",
                Dsl::list(References::ITEM_STACK.r#in(schema)),
            )])
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_entities_adds_pale_oak_boats() {
        let schema = NamespacedSchema::default();
        let v = V4070::new(4070, NamespacedSchema::default());
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("minecraft:pale_oak_boat"), Some(&Template::Empty));
        let expected = Template::OptionalFields(vec![(
            "Items",
            Template::List(Box::new(Template::Ref("References.ITEM_STACK"))),
        )]);
        assert_eq!(map.get("minecraft:pale_oak_chest_boat"), Some(&expected));
    }

    #[test]
    fn chest_boat_has_items_list() {
        let schema = NamespacedSchema::default();
        let v = V4070::new(4070, NamespacedSchema::default());
        let map = v.register_entities(&schema);
        match map.get("minecraft:pale_oak_chest_boat").unwrap() {
            Template::OptionalFields(fields) => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "Items");
            }
            _ => panic!("expected optional fields"),
        }
    }

    #[test]
    fn constructor_negative_and_overflow_versions() {
        let v = V4070::new(i32::MIN, NamespacedSchema { version: -100, parent_version: 0 });
        assert_eq!(v.version, i32::MIN);
        assert_eq!(v.parent_version, -100);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX); // overflow edge
    }

    #[test]
    fn register_simple_overwrites_with_empty() {
        let schema = NamespacedSchema::default();
        let mut map = BTreeMap::new();
        map.insert(
            "minecraft:pale_oak_boat".to_string(),
            Template::Ref("old"),
        );
        schema.register_simple(&mut map, "minecraft:pale_oak_boat");
        assert_eq!(map.get("minecraft:pale_oak_boat"), Some(&Template::Empty));
    }

    #[test]
    fn float_bits_and_negative_inputs() {
        let nan = f64::NAN.to_bits();
        assert_eq!(nan & (0x7FF0_0000_0000_0000), 0x7FF0_0000_0000_0000);
        let neg: i32 = -7;
        assert_eq!(neg.wrapping_add(-1), -8);
        let negf = (-1.5f32).to_bits();
        assert_ne!(negf, 1.5f32.to_bits());
    }
}
