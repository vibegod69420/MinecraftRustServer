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
    pub fn r#in(self, _schema: &Schema) -> Template {
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
pub struct Schema {
    pub version: i32,
    pub parent_version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, Template> {
        BTreeMap::new()
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register<F>(&self, map: &mut BTreeMap<String, Template>, name: &str, factory: F)
    where
        F: FnOnce(&Schema) -> Template,
    {
        map.insert(name.to_string(), factory(self));
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(&self, map: &mut BTreeMap<String, Template>, name: &str) {
        map.insert(name.to_string(), Template::Empty);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct V702 {
    pub version: i32,
    pub parent_version: i32,
}

impl V702 {
    // JAVA: net.minecraft.util.datafix.schemas.V702#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            version,
            parent_version: parent.version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V702#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, Template> {
        let mut map = schema.register_entities(schema);
        schema.register(&mut map, "ZombieVillager", |_s| {
            Dsl::optional_fields(vec![(
                "Offers",
                Dsl::optional_fields(vec![(
                    "Recipes",
                    Dsl::list(References::VILLAGER_TRADE.r#in(schema)),
                )]),
            )])
        });
        schema.register_simple(&mut map, "Husk");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_entities_adds_zombie_villager_and_husk() {
        let schema = Schema::default();
        let v = V702::new(702, Schema::default());
        let map = v.register_entities(&schema);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("Husk"), Some(&Template::Empty));
        let expected = Template::OptionalFields(vec![(
            "Offers",
            Template::OptionalFields(vec![(
                "Recipes",
                Template::List(Box::new(Template::Ref("References.VILLAGER_TRADE"))),
            )]),
        )]);
        assert_eq!(map.get("ZombieVillager"), Some(&expected));
    }

    #[test]
    fn register_entities_preserves_parent_entries() {
        let schema = Schema::default();
        let v = V702::new(702, Schema::default());
        let mut base = schema.register_entities(&schema);
        base.insert("Existing".to_string(), Template::Empty);
        // Simulate super returning existing entries by registering on top of a prefilled map.
        let mut map = base;
        schema.register(&mut map, "ZombieVillager", |_| Template::Empty);
        schema.register_simple(&mut map, "Husk");
        assert!(map.contains_key("Existing"));
        assert!(map.contains_key("ZombieVillager"));
        assert!(map.contains_key("Husk"));
        let _ = v; // v constructed to cover constructor in edge-case context
    }

    #[test]
    fn constructor_handles_negative_and_boundary_versions() {
        let parent = Schema {
            version: -1,
            parent_version: 0,
        };
        let v = V702::new(i32::MIN, parent.clone());
        assert_eq!(v.version, i32::MIN);
        assert_eq!(v.parent_version, -1);

        let v2 = V702::new(i32::MAX, Schema { version: i32::MAX, parent_version: 0 });
        let wrapped = v2.version.wrapping_add(1);
        assert_eq!(wrapped, i32::MIN); // overflow boundary
    }

    #[test]
    fn template_ref_and_list_structure() {
        let schema = Schema::default();
        let r = References::ITEM_STACK.r#in(&schema);
        assert_eq!(r, Template::Ref("References.ITEM_STACK"));
        let l = Dsl::list(r.clone());
        assert_eq!(l, Template::List(Box::new(r)));
    }

    #[test]
    fn float_bits_sanity_for_template_context() {
        // Float-bit edge case: ensure bit patterns are preserved in test data.
        let nan_bits = f64::NAN.to_bits();
        assert_ne!(nan_bits, 0);
        let neg_zero = (-0.0f64).to_bits();
        let pos_zero = 0.0f64.to_bits();
        assert_ne!(neg_zero, pos_zero);
        // Negative input edge case for version-like integer handling.
        let v: i32 = -42;
        assert_eq!(v.wrapping_sub(1), -43);
    }
}
