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

    // JAVA: net.minecraft.util.datafix.schemas.Schema#register
    pub fn register(
        &self,
        map: &mut HashMap<String, TemplateSupplier>,
        name: &str,
        supplier: impl Fn() -> TypeTemplate + 'static,
    ) {
        map.insert(name.to_string(), Box::new(supplier));
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
        HashMap::new()
    }
}

// Stub for V1458 dependency; keeps module self-contained.
pub struct V1458;

impl V1458 {
    // JAVA: net.minecraft.util.datafix.schemas.V1458#nameableInventory
    pub fn nameable_inventory(_schema: &Schema) -> TypeTemplate {
        TypeTemplate
    }
}

pub struct V3682 {
    pub base: NamespacedSchema,
}

impl V3682 {
    // JAVA: net.minecraft.util.datafix.schemas.V3682#<init>
    pub fn new(version_key: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3682#registerBlockEntities
    pub fn register_block_entities(&self, schema: &Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        let schema_clone = schema.clone();
        schema.register(&mut map, "minecraft:crafter", move || {
            V1458::nameable_inventory(&schema_clone)
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_crafter() {
        let schema = Schema::default();
        let v = V3682::new(3682, Schema::default());
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:crafter"));
    }

    #[test]
    fn crafter_supplier_returns_template() {
        let schema = Schema::default();
        let v = V3682::new(3682, Schema::default());
        let map = v.register_block_entities(&schema);
        let supplier = map.get("minecraft:crafter").unwrap();
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn negative_and_min_version_key() {
        let v = V3682::new(-3682, Schema::default());
        assert_eq!(v.base.schema.version_key, -3682);
        let v_min = V3682::new(i32::MIN, Schema::default());
        assert_eq!(v_min.base.schema.version_key, i32::MIN);
    }

    #[test]
    fn overflow_and_float_bits_version_key() {
        let vk_float_bits = f32::INFINITY.to_bits() as i32;
        let v = V3682::new(vk_float_bits, Schema::default());
        assert_eq!(v.base.schema.version_key, vk_float_bits);

        let vk_overflow = i32::MAX.wrapping_add(1);
        let v2 = V3682::new(vk_overflow, Schema::default());
        assert_eq!(v2.base.schema.version_key, i32::MIN);
    }
}
