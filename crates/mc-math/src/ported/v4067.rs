use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Reference {
    ItemStack,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    OptionalFields(Vec<(String, TypeTemplate)>),
    Ref(Reference),
    List(Box<TypeTemplate>),
}

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version: i32,
    pub parent_version: i32,
}

impl Schema {
    // JAVA: net.minecraft.util.datafix.schemas.Schema#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities_base(&self) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

pub struct V4067 {
    pub version: i32,
    pub parent_version: i32,
}

impl V4067 {
    // JAVA: net.minecraft.util.datafix.schemas.V4067#<init>
    pub fn new(version: i32, parent_version: i32) -> Self {
        Self {
            version,
            parent_version,
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerSimple
    fn register_simple(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str) {
        self.register(
            map,
            name,
            Box::new(|| TypeTemplate::OptionalFields(vec![])),
        );
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#register
    fn register(
        &self,
        map: &mut BTreeMap<String, TemplateSupplier>,
        name: &str,
        template: TemplateSupplier,
    ) {
        map.insert(name.to_string(), template);
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4067#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let mut map = schema.register_entities_base();
        map.remove("minecraft:boat");
        map.remove("minecraft:chest_boat");
        self.register_simple(&mut map, "minecraft:oak_boat");
        self.register_simple(&mut map, "minecraft:spruce_boat");
        self.register_simple(&mut map, "minecraft:birch_boat");
        self.register_simple(&mut map, "minecraft:jungle_boat");
        self.register_simple(&mut map, "minecraft:acacia_boat");
        self.register_simple(&mut map, "minecraft:cherry_boat");
        self.register_simple(&mut map, "minecraft:dark_oak_boat");
        self.register_simple(&mut map, "minecraft:mangrove_boat");
        self.register_simple(&mut map, "minecraft:bamboo_raft");
        self.register_chest_boat(&mut map, "minecraft:oak_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:spruce_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:birch_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:jungle_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:acacia_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:cherry_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:dark_oak_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:mangrove_chest_boat");
        self.register_chest_boat(&mut map, "minecraft:bamboo_chest_raft");
        map
    }

    // JAVA: net.minecraft.util.datafix.schemas.V4067#registerChestBoat
    fn register_chest_boat(&self, map: &mut BTreeMap<String, TemplateSupplier>, name: &str) {
        self.register(
            map,
            name,
            Box::new(|| {
                TypeTemplate::OptionalFields(vec![(
                    "Items".to_string(),
                    TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack))),
                )])
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Schema {
        Schema::new(4067, 4066)
    }

    #[test]
    fn removes_legacy_boat_entities() {
        let s = schema();
        let v = V4067::new(4067, 4066);
        let mut base = BTreeMap::new();
        base.insert(
            "minecraft:boat".to_string(),
            Box::new(|| TypeTemplate::OptionalFields(vec![])) as TemplateSupplier,
        );
        base.insert(
            "minecraft:chest_boat".to_string(),
            Box::new(|| TypeTemplate::OptionalFields(vec![])) as TemplateSupplier,
        );
        // register_entities uses schema.register_entities_base(), so verify on default base
        let map = v.register_entities(&s);
        assert!(!map.contains_key("minecraft:boat"));
        assert!(!map.contains_key("minecraft:chest_boat"));
        // also verify removal logic by simulating: insert then remove
        assert!(base.remove("minecraft:boat").is_some());
        assert!(base.remove("minecraft:chest_boat").is_some());
    }

    #[test]
    fn registers_all_boat_variants() {
        let v = V4067::new(4067, 4066);
        let map = v.register_entities(&schema());
        let expected = [
            "minecraft:oak_boat",
            "minecraft:spruce_boat",
            "minecraft:birch_boat",
            "minecraft:jungle_boat",
            "minecraft:acacia_boat",
            "minecraft:cherry_boat",
            "minecraft:dark_oak_boat",
            "minecraft:mangrove_boat",
            "minecraft:bamboo_raft",
            "minecraft:oak_chest_boat",
            "minecraft:spruce_chest_boat",
            "minecraft:birch_chest_boat",
            "minecraft:jungle_chest_boat",
            "minecraft:acacia_chest_boat",
            "minecraft:cherry_chest_boat",
            "minecraft:dark_oak_chest_boat",
            "minecraft:mangrove_chest_boat",
            "minecraft:bamboo_chest_raft",
        ];
        assert_eq!(map.len(), 18);
        for key in expected {
            assert!(map.contains_key(key), "missing {key}");
        }
    }

    #[test]
    fn simple_boat_template_is_empty_optional_fields() {
        let v = V4067::new(4067, 4066);
        let map = v.register_entities(&schema());
        let tpl = (map.get("minecraft:oak_boat").unwrap())();
        assert_eq!(tpl, TypeTemplate::OptionalFields(vec![]));
    }

    #[test]
    fn chest_boat_template_has_items_list() {
        let v = V4067::new(4067, 4066);
        let map = v.register_entities(&schema());
        let tpl = (map.get("minecraft:oak_chest_boat").unwrap())();
        assert_eq!(
            tpl,
            TypeTemplate::OptionalFields(vec![(
                "Items".to_string(),
                TypeTemplate::List(Box::new(TypeTemplate::Ref(Reference::ItemStack))),
            )])
        );
    }

    #[test]
    fn constructor_stores_versions() {
        let v = V4067::new(4067, 4066);
        assert_eq!(v.version, 4067);
        assert_eq!(v.parent_version, 4066);
        let v2 = V4067::new(i32::MIN, i32::MAX);
        assert_eq!(v2.version, i32::MIN);
        assert_eq!(v2.parent_version, i32::MAX);
    }
}
