use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeTemplate;

pub type TemplateSupplier = Box<dyn Fn() -> TypeTemplate>;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub version_key: i32,
}

impl Schema {
    // JAVA: net.minecraft.util.datafix.schemas.Schema#<init>
    pub fn new(version_key: i32, _parent: Schema) -> Self {
        Self { version_key }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NamespacedSchema {
    pub schema: Schema,
}

impl NamespacedSchema {
    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#<init>
    pub fn new(version_key: i32, parent: Schema) -> Self {
        Self {
            schema: Schema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#registerBlockEntities
    pub fn register_block_entities(&self, _schema: &Schema) -> HashMap<String, TemplateSupplier> {
        let mut map: HashMap<String, TemplateSupplier> = HashMap::new();
        map.insert(
            "minecraft:flower_pot".to_string(),
            Box::new(|| TypeTemplate),
        );
        map.insert(
            "minecraft:noteblock".to_string(),
            Box::new(|| TypeTemplate),
        );
        map.insert("minecraft:chest".to_string(), Box::new(|| TypeTemplate));
        map
    }
}

pub struct V1451_5 {
    pub base: NamespacedSchema,
}

impl V1451_5 {
    // JAVA: net.minecraft.util.datafix.schemas.V1451_5#<init>
    pub fn new(version_key: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V1451_5#registerBlockEntities
    pub fn register_block_entities(&self, schema: &Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        map.remove("minecraft:flower_pot");
        map.remove("minecraft:noteblock");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_flower_pot_and_noteblock() {
        let schema = Schema::default();
        let v = V1451_5::new(1451, Schema::default());
        let map = v.register_block_entities(&schema);
        assert!(!map.contains_key("minecraft:flower_pot"));
        assert!(!map.contains_key("minecraft:noteblock"));
        assert!(map.contains_key("minecraft:chest"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_missing_keys_is_harmless() {
        let mut map: HashMap<String, TemplateSupplier> = HashMap::new();
        assert!(map.remove("minecraft:flower_pot").is_none());
        assert!(map.remove("minecraft:noteblock").is_none());
        assert!(map.is_empty());
    }

    #[test]
    fn negative_and_min_version_key() {
        let v_neg = V1451_5::new(-1451, Schema::default());
        assert_eq!(v_neg.base.schema.version_key, -1451);
        let v_min = V1451_5::new(i32::MIN, Schema::default());
        assert_eq!(v_min.base.schema.version_key, i32::MIN);
    }

    #[test]
    fn overflow_and_float_bits_version_key() {
        let vk_float_bits = f32::NEG_INFINITY.to_bits() as i32;
        let v = V1451_5::new(vk_float_bits, Schema::default());
        assert_eq!(v.base.schema.version_key, vk_float_bits);

        let vk_overflow = i32::MAX.wrapping_add(1);
        assert_eq!(vk_overflow, i32::MIN);
        let v2 = V1451_5::new(vk_overflow, Schema::default());
        assert_eq!(v2.base.schema.version_key, vk_overflow);
    }
}
