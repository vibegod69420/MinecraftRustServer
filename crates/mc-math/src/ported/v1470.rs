use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    BlockState,
    ItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeTemplate {
    Empty,
    Ref(Reference),
    OptionalFields(Vec<(String, TypeTemplate)>),
}

#[derive(Debug, Clone, Default)]
pub struct Schema {
    version: i32,
}

impl Schema {
    // JAVA: com.mojang.datafixers.schemas.Schema#Schema
    pub fn new(version: i32) -> Self {
        Self { version }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#getVersionKey
    pub fn version(&self) -> i32 {
        self.version
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
    ) {
        map.insert(name.to_string(), TypeTemplate::Empty);
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#register
    pub fn register<F>(
        &self,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
        template_factory: F,
    ) where
        F: FnOnce(&Schema) -> TypeTemplate,
    {
        map.insert(name.to_string(), template_factory(self));
    }
}

#[derive(Debug, Clone, Default)]
pub struct NamespacedSchema {
    schema: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#NamespacedSchema
    pub fn new(version: i32, _parent: Option<&Schema>) -> Self {
        Self {
            schema: Schema::new(version),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        BTreeMap::new()
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

pub struct V1470 {
    base: NamespacedSchema,
}

impl V1470 {
    // JAVA: net.minecraft.util.datafix.schemas.V1470#<init>(int,Schema)
    pub fn new(version: i32, parent: Option<&Schema>) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1470#registerMob(Schema,Map,String)
    pub fn register_mob(
        schema: &Schema,
        map: &mut BTreeMap<String, TypeTemplate>,
        name: &str,
    ) {
        schema.register_simple(map, name);
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1470#registerEntities(Schema)
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TypeTemplate> {
        let mut map = self.base.register_entities(schema);
        Self::register_mob(schema, &mut map, "minecraft:turtle");
        Self::register_mob(schema, &mut map, "minecraft:cod_mob");
        Self::register_mob(schema, &mut map, "minecraft:tropical_fish");
        Self::register_mob(schema, &mut map, "minecraft:salmon_mob");
        Self::register_mob(schema, &mut map, "minecraft:puffer_fish");
        Self::register_mob(schema, &mut map, "minecraft:phantom");
        Self::register_mob(schema, &mut map, "minecraft:dolphin");
        Self::register_mob(schema, &mut map, "minecraft:drowned");
        schema.register(&mut map, "minecraft:trident", |_s| {
            TypeTemplate::OptionalFields(vec![
                ("inBlockState".to_string(), TypeTemplate::Ref(Reference::BlockState)),
                ("Trident".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
            ])
        });
        map
    }

    pub fn schema(&self) -> &Schema {
        self.base.schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_mob_adds_empty_template() {
        let schema = Schema::new(1470);
        let mut map = BTreeMap::new();
        V1470::register_mob(&schema, &mut map, "minecraft:turtle");
        assert_eq!(map.get("minecraft:turtle"), Some(&TypeTemplate::Empty));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_register_entities_contains_all_mobs() {
        let v1470 = V1470::new(1470, None);
        let schema = Schema::new(1470);
        let map = v1470.register_entities(&schema);
        let expected_mobs = [
            "minecraft:turtle",
            "minecraft:cod_mob",
            "minecraft:tropical_fish",
            "minecraft:salmon_mob",
            "minecraft:puffer_fish",
            "minecraft:phantom",
            "minecraft:dolphin",
            "minecraft:drowned",
        ];
        for mob in expected_mobs {
            assert_eq!(map.get(mob), Some(&TypeTemplate::Empty), "missing mob {mob}");
        }
    }

    #[test]
    fn test_trident_has_optional_fields() {
        let v1470 = V1470::new(1470, None);
        let schema = Schema::new(1470);
        let map = v1470.register_entities(&schema);
        let trident = map.get("minecraft:trident").expect("trident missing");
        let expected = TypeTemplate::OptionalFields(vec![
            ("inBlockState".to_string(), TypeTemplate::Ref(Reference::BlockState)),
            ("Trident".to_string(), TypeTemplate::Ref(Reference::ItemStack)),
        ]);
        assert_eq!(trident, &expected);
    }

    #[test]
    fn test_register_entities_count_and_version() {
        let v1470 = V1470::new(1470, None);
        assert_eq!(v1470.schema().version(), 1470);
        let schema = Schema::new(1470);
        let map = v1470.register_entities(&schema);
        assert_eq!(map.len(), 9);
    }

    #[test]
    fn test_register_simple_overwrites() {
        let schema = Schema::new(0);
        let mut map = BTreeMap::new();
        schema.register(&mut map, "minecraft:turtle", |_| {
            TypeTemplate::Ref(Reference::ItemStack)
        });
        schema.register_simple(&mut map, "minecraft:turtle");
        assert_eq!(map.get("minecraft:turtle"), Some(&TypeTemplate::Empty));
    }
}
