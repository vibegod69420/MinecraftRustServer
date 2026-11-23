//! Port of `net.minecraft.util.datafix.fixes.BlendingDataRemoveFromNetherEndFix`

// JAVA: net.minecraft.util.datafix.fixes.BlendingDataRemoveFromNetherEndFix#updateChunkTag
/// Removes blending_data from chunk tag if dimension is not "minecraft:overworld".
pub fn update_chunk_tag(dimension: &str, blending_data: &mut Option<String>) {
    let is_overworld = dimension == "minecraft:overworld";
    if !is_overworld {
        blending_data.take();
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlendingDataRemoveFromNetherEndFix#makeRule
/// Simplified structural representation of the fix rule.
pub struct BlendingDataRemoveFromNetherEndFix;

impl BlendingDataRemoveFromNetherEndFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlendingDataRemoveFromNetherEndFix#apply
    /// Applies the chunk tag update logic.
    pub fn apply(dimension: &str, chunk_tag: &mut Option<String>) {
        update_chunk_tag(dimension, chunk_tag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overworld_preserves_blending_data() {
        let dim = "minecraft:overworld";
        let mut data = Some("blending_data_value".to_string());
        BlendingDataRemoveFromNetherEndFix::apply(dim, &mut data);
        assert_eq!(data, Some("blending_data_value".to_string()));
    }

    #[test]
    fn test_nether_removes_blending_data() {
        let dim = "minecraft:nether";
        let mut data = Some("blending_data_value".to_string());
        BlendingDataRemoveFromNetherEndFix::apply(dim, &mut data);
        assert_eq!(data, None);
    }

    #[test]
    fn test_empty_dimension_removes_blending_data() {
        let dim = "";
        let mut data = Some("blending_data_value".to_string());
        BlendingDataRemoveFromNetherEndFix::apply(dim, &mut data);
        assert_eq!(data, None);
    }

    #[test]
    fn test_other_dimension_removes_blending_data() {
        let dim = "minecraft:the_end";
        let mut data = Some("blending_data_value".to_string());
        BlendingDataRemoveFromNetherEndFix::apply(dim, &mut data);
        assert_eq!(data, None);
    }

    #[test]
    fn test_nonexistent_data_unchanged() {
        let dim = "minecraft:overworld";
        let mut data: Option<String> = None;
        BlendingDataRemoveFromNetherEndFix::apply(dim, &mut data);
        assert_eq!(data, None);
    }
}
