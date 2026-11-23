use std::collections::HashSet;
use serde_json::{Map, Value};

/// Port of `net.minecraft.util.datafix.fixes.BlendingDataFix`
pub struct BlendingDataFix {
    name: String,
}

impl BlendingDataFix {
    /// JAVA: net.minecraft.util.datafix.fixes.BlendingDataFix#<init>
    pub fn new(name: String) -> Self {
        BlendingDataFix { name }
    }

    /// JAVA: net.minecraft.util.datafix.fixes.BlendingDataFix#blockToSectionCoord
    pub fn block_to_section_coord(block_coord: i32) -> i32 {
        // Java: blockCoord >> 4
        block_coord >> 4
    }

    /// JAVA: net.minecraft.util.datafix.fixes.BlendingDataFix#updateBlendingData
    pub fn update_blending_data(data: &mut Map<String, Value>, min_section: i32, size: i32) {
        let min = Self::block_to_section_coord(min_section);
        let max = Self::block_to_section_coord(min_section + size);
        let mut blending_map = Map::new();
        blending_map.insert("min_section".to_string(), Value::Number(serde_json::Number::from(min)));
        blending_map.insert("max_section".to_string(), Value::Number(serde_json::Number::from(max)));
        data.insert("blending_data".to_string(), Value::Object(blending_map));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    #[test]
    fn test_block_to_section_coord() {
        assert_eq!(BlendingDataFix::block_to_section_coord(0), 0);
        assert_eq!(BlendingDataFix::block_to_section_coord(15), 0);
        assert_eq!(BlendingDataFix::block_to_section_coord(16), 1);
        assert_eq!(BlendingDataFix::block_to_section_coord(255), 15);
    }

    #[test]
    fn test_update_blending_data() {
        let mut data = Map::new();
        BlendingDataFix::update_blending_data(&mut data, 0, 16);
        assert!(data.contains_key("blending_data"));
        let blending = data.get("blending_data").unwrap();
        assert_eq!(blending.get("min_section"), Some(&Value::Number(serde_json::Number::from(0i32))));
        assert_eq!(blending.get("max_section"), Some(&Value::Number(serde_json::Number::from(1i32))));
    }
}
