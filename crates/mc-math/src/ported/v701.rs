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

    // JAVA: net.minecraft.util.datafix.schemas.Schema#registerEntities
    pub fn register_entities(&self, _schema: &Schema) -> HashMap<String, TemplateSupplier> {
        HashMap::new()
    }

    // JAVA: net.minecraft.util.datafix.schemas.Schema#registerSimple
    pub fn register_simple(&self, map: &mut HashMap<String, TemplateSupplier>, name: &str) {
        map.insert(name.to_string(), Box::new(|| TypeTemplate));
    }
}

#[derive(Clone, Debug, Default)]
pub struct V701 {
    pub schema: Schema,
}

impl V701 {
    // JAVA: net.minecraft.util.datafix.schemas.V701#<init>
    pub fn new(version_key: i32, parent: Schema) -> Self {
        Self {
            schema: Schema::new(version_key, parent),
        }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V701#registerEntities
    pub fn register_entities(&self, schema: &Schema) -> HashMap<String, TemplateSupplier> {
        let mut map = self.schema.register_entities(schema);
        schema.register_simple(&mut map, "WitherSkeleton");
        schema.register_simple(&mut map, "Stray");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_wither_skeleton_and_stray() {
        let schema = Schema::default();
        let v701 = V701::new(701, Schema::default());
        let map = v701.register_entities(&schema);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("WitherSkeleton"));
        assert!(map.contains_key("Stray"));
    }

    #[test]
    fn suppliers_return_type_template() {
        let schema = Schema::default();
        let v701 = V701::new(701, Schema::default());
        let map = v701.register_entities(&schema);
        assert_eq!((map.get("WitherSkeleton").unwrap())(), TypeTemplate);
        assert_eq!((map.get("Stray").unwrap())(), TypeTemplate);
    }

    #[test]
    fn negative_and_min_version_key() {
        let v_neg = V701::new(-1, Schema::default());
        assert_eq!(v_neg.schema.version_key, -1);
        let v_min = V701::new(i32::MIN, Schema::default());
        assert_eq!(v_min.schema.version_key, i32::MIN);
        let map = v_neg.register_entities(&Schema::default());
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn overflow_and_float_bits_version_key() {
        let vk_float_bits = f32::NAN.to_bits() as i32;
        let v = V701::new(vk_float_bits, Schema::default());
        assert_eq!(v.schema.version_key, vk_float_bits);

        let vk_overflow = i32::MAX.wrapping_add(1);
        let v2 = V701::new(vk_overflow, Schema::default());
        assert_eq!(v2.schema.version_key, i32::MIN);
    }
}
