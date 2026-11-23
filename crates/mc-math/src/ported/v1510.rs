use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct Schema;

#[derive(Clone, Debug, Default)]
pub struct NamespacedSchema {
    pub version: i32,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version: i32, _parent: Schema) -> Self {
        Self { version }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        BTreeMap::new()
    }
}

pub struct V1510 {
    pub base: NamespacedSchema,
}

impl V1510 {
    // JAVA: net.minecraft.util.datafix.schemas.V1510#<init>
    pub fn new(version: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1510#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> BTreeMap<String, TemplateSupplier> {
        let base = self.base.register_entities(schema);
        self.register_entities_from_base(base)
    }

    pub fn register_entities_from_base(
        &self,
        mut entities: BTreeMap<String, TemplateSupplier>,
    ) -> BTreeMap<String, TemplateSupplier> {
        rename_entity(&mut entities, "minecraft:commandblock_minecart", "minecraft:command_block_minecart");
        rename_entity(&mut entities, "minecraft:ender_crystal", "minecraft:end_crystal");
        rename_entity(&mut entities, "minecraft:snowman", "minecraft:snow_golem");
        rename_entity(&mut entities, "minecraft:evocation_illager", "minecraft:evoker");
        rename_entity(&mut entities, "minecraft:evocation_fangs", "minecraft:evoker_fangs");
        rename_entity(&mut entities, "minecraft:illusion_illager", "minecraft:illusioner");
        rename_entity(&mut entities, "minecraft:vindication_illager", "minecraft:vindicator");
        rename_entity(&mut entities, "minecraft:villager_golem", "minecraft:iron_golem");
        rename_entity(&mut entities, "minecraft:xp_orb", "minecraft:experience_orb");
        rename_entity(&mut entities, "minecraft:xp_bottle", "minecraft:experience_bottle");
        rename_entity(&mut entities, "minecraft:eye_of_ender_signal", "minecraft:eye_of_ender");
        rename_entity(&mut entities, "minecraft:fireworks_rocket", "minecraft:firework_rocket");
        entities
    }
}

fn rename_entity(entities: &mut BTreeMap<String, TemplateSupplier>, old: &str, new_name: &str) {
    if let Some(v) = entities.remove(old) {
        entities.insert(new_name.to_string(), v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_supplier() -> TemplateSupplier {
        Box::new(|| TypeTemplate)
    }

    fn seeded_base() -> BTreeMap<String, TemplateSupplier> {
        let mut base = BTreeMap::new();
        for name in [
            "minecraft:commandblock_minecart",
            "minecraft:ender_crystal",
            "minecraft:snowman",
            "minecraft:evocation_illager",
            "minecraft:evocation_fangs",
            "minecraft:illusion_illager",
            "minecraft:vindication_illager",
            "minecraft:villager_golem",
            "minecraft:xp_orb",
            "minecraft:xp_bottle",
            "minecraft:eye_of_ender_signal",
            "minecraft:fireworks_rocket",
        ] {
            base.insert(name.to_string(), dummy_supplier());
        }
        base
    }

    #[test]
    fn renames_all_twelve_entities() {
        let v = V1510::new(1510, Schema);
        let out = v.register_entities_from_base(seeded_base());
        let expected = [
            "minecraft:command_block_minecart",
            "minecraft:end_crystal",
            "minecraft:snow_golem",
            "minecraft:evoker",
            "minecraft:evoker_fangs",
            "minecraft:illusioner",
            "minecraft:vindicator",
            "minecraft:iron_golem",
            "minecraft:experience_orb",
            "minecraft:experience_bottle",
            "minecraft:eye_of_ender",
            "minecraft:firework_rocket",
        ];
        for name in expected {
            assert!(out.contains_key(name), "missing {name}");
        }
        assert_eq!(out.len(), 12);
    }

    #[test]
    fn old_names_removed() {
        let v = V1510::new(1510, Schema);
        let out = v.register_entities_from_base(seeded_base());
        for old in [
            "minecraft:commandblock_minecart",
            "minecraft:ender_crystal",
            "minecraft:snowman",
            "minecraft:evocation_illager",
            "minecraft:evocation_fangs",
            "minecraft:illusion_illager",
            "minecraft:vindication_illager",
            "minecraft:villager_golem",
            "minecraft:xp_orb",
            "minecraft:xp_bottle",
            "minecraft:eye_of_ender_signal",
            "minecraft:fireworks_rocket",
        ] {
            assert!(!out.contains_key(old), "old key still present: {old}");
        }
    }

    #[test]
    fn missing_keys_are_ignored_edge_case() {
        let v = V1510::new(1510, Schema);
        let out = v.register_entities_from_base(BTreeMap::new());
        assert!(out.is_empty());
    }

    #[test]
    fn preserves_unrelated_entries() {
        let v = V1510::new(1510, Schema);
        let mut base = seeded_base();
        base.insert("minecraft:zombie".to_string(), dummy_supplier());
        let out = v.register_entities_from_base(base);
        assert!(out.contains_key("minecraft:zombie"));
        assert_eq!(out.len(), 13);
    }

    #[test]
    fn register_entities_uses_base_schema() {
        let v = V1510::new(1510, Schema);
        let out = v.register_entities(&Schema);
        assert!(out.is_empty());
    }
}
