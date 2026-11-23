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

    // JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#register
    pub fn register(
        &self,
        map: &mut HashMap<String, TemplateSupplier>,
        name: &str,
        supplier: impl Fn() -> TypeTemplate + 'static,
    ) {
        self.schema.register(map, name, supplier);
    }
}

// Stub for V3439 dependency; keeps module self-contained.
pub struct V3439;

impl V3439 {
    // JAVA: net.minecraft.util.datafix.schemas.V3439#sign
    pub fn sign(_schema: &Schema) -> TypeTemplate {
        TypeTemplate
    }
}

pub struct V3439_1 {
    pub base: NamespacedSchema,
}

impl V3439_1 {
    // JAVA: net.minecraft.util.datafix.schemas.V3439_1#<init>
    pub fn new(version_key: i32, parent: Schema) -> Self {
        Self {
            base: NamespacedSchema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3439_1#registerBlockEntities
    pub fn register_block_entities(&self, schema: &Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.base.register_block_entities(schema);
        let schema_clone = schema.clone();
        self.base.register(&mut map, "minecraft:hanging_sign", move || {
            V3439::sign(&schema_clone)
        });
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_hanging_sign() {
        let schema = Schema::default();
        let v = V3439_1::new(3439, Schema::default());
        let map = v.register_block_entities(&schema);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("minecraft:hanging_sign"));
    }

    #[test]
    fn hanging_sign_supplier_returns_template() {
        let schema = Schema::default();
        let v = V3439_1::new(3439, Schema::default());
        let map = v.register_block_entities(&schema);
        let supplier = map.get("minecraft:hanging_sign").unwrap();
        assert_eq!(supplier(), TypeTemplate);
    }

    #[test]
    fn negative_and_min_version_key() {
        let v = V3439_1::new(-3439, Schema::default());
        assert_eq!(v.base.schema.version_key, -3439);
        let v_min = V3439_1::new(i32::MIN, Schema::default());
        assert_eq!(v_min.base.schema.version_key, i32::MIN);
    }

    #[test]
    fn overflow_and_float_bits_version_key() {
        let vk_float_bits = f64::NAN.to_bits() as i32;
        let v = V3439_1::new(vk_float_bits, Schema::default());
        assert_eq!(v.base.schema.version_key, vk_float_bits);

        let vk_overflow = i32::MAX.wrapping_add(2);
        assert_eq!(vk_overflow, i32::MIN.wrapping_add(1));
        let v2 = V3439_1::new(vk_overflow, Schema::default());
        assert_eq!(v2.base.schema.version_key, vk_overflow);
    }
}
