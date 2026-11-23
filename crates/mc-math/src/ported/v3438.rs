use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeTemplate {
    Unit,
}

pub type TypeTemplateSupplier = Arc<dyn Fn() -> TypeTemplate + Send + Sync>;

#[derive(Debug, Default)]
pub struct Schema {
    pub version_key: i32,
    pub simple_registrations: Vec<String>,
}

impl Schema {
    pub fn new(version_key: i32) -> Self {
        Self {
            version_key,
            simple_registrations: Vec::new(),
        }
    }

    // JAVA: com.mojang.datafixers.schemas.Schema#registerSimple
    pub fn register_simple(
        &mut self,
        _map: &mut HashMap<String, TypeTemplateSupplier>,
        name: &str,
    ) {
        self.simple_registrations.push(name.to_string());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V3438 {
    pub version_key: i32,
}

impl V3438 {
    // JAVA: net.minecraft.util.datafix.schemas.V3438#<init>
    pub fn new(version_key: i32, _parent: Option<&Schema>) -> Self {
        Self { version_key }
    }

    // JAVA: net.minecraft.util.datafix.schemas.V3438#registerBlockEntities
    pub fn register_block_entities(
        &self,
        schema: &mut Schema,
        base: HashMap<String, TypeTemplateSupplier>,
    ) -> HashMap<String, TypeTemplateSupplier> {
        let mut map = base; // super.registerBlockEntities(schema)
        if let Some(supplier) = map.remove("minecraft:suspicious_sand") {
            map.insert("minecraft:brushable_block".to_string(), supplier);
        }
        schema.register_simple(&mut map, "minecraft:calibrated_sculk_sensor");
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_supplier() -> TypeTemplateSupplier {
        Arc::new(|| TypeTemplate::Unit)
    }

    #[test]
    fn constructor_stores_version_key_including_edges() {
        let v = V3438::new(3438, None);
        assert_eq!(v.version_key, 3438);

        let neg = V3438::new(-1, None);
        assert_eq!(neg.version_key, -1);

        let max = V3438::new(i32::MAX, None);
        assert_eq!(max.version_key, i32::MAX);
    }

    #[test]
    fn register_block_entities_renames_suspicious_sand() {
        let v = V3438::new(3438, None);
        let mut schema = Schema::new(3438);
        let mut base = HashMap::new();
        base.insert("minecraft:suspicious_sand".to_string(), dummy_supplier());

        let result = v.register_block_entities(&mut schema, base);

        assert!(result.contains_key("minecraft:brushable_block"));
        assert!(!result.contains_key("minecraft:suspicious_sand"));
        assert_eq!(
            schema.simple_registrations,
            vec!["minecraft:calibrated_sculk_sensor".to_string()]
        );
    }

    #[test]
    fn register_block_entities_missing_key_still_registers_simple() {
        let v = V3438::new(3438, None);
        let mut schema = Schema::new(3438);
        let base = HashMap::new();

        let result = v.register_block_entities(&mut schema, base);

        assert!(!result.contains_key("minecraft:brushable_block"));
        assert!(!result.contains_key("minecraft:suspicious_sand"));
        assert_eq!(
            schema.simple_registrations,
            vec!["minecraft:calibrated_sculk_sensor".to_string()]
        );
    }

    #[test]
    fn register_block_entities_preserves_other_entries() {
        let v = V3438::new(3438, None);
        let mut schema = Schema::new(3438);
        let mut base = HashMap::new();
        base.insert("minecraft:suspicious_sand".to_string(), dummy_supplier());
        base.insert("minecraft:chest".to_string(), dummy_supplier());

        let result = v.register_block_entities(&mut schema, base);

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("minecraft:chest"));
        assert!(result.contains_key("minecraft:brushable_block"));
    }

    #[test]
    fn register_block_entities_overwrites_existing_brushable_block_target() {
        let v = V3438::new(3438, None);
        let mut schema = Schema::new(3438);
        let mut base = HashMap::new();
        base.insert("minecraft:suspicious_sand".to_string(), dummy_supplier());
        base.insert("minecraft:brushable_block".to_string(), dummy_supplier());

        let result = v.register_block_entities(&mut schema, base);

        // Java Map.put overwrites existing key with removed supplier.
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("minecraft:brushable_block"));
    }
}
