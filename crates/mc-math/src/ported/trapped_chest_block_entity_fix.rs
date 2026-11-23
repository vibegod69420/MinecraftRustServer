// JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix
use std::collections::HashSet;

pub const SIZE: usize = 4096;

// JAVA: net.minecraft.util.datafix.fixes.LeavesFix#getIndex
pub fn get_index(x: i32, y: i32, z: i32) -> i32 {
    ((y >> 4) << 12) | (((y & 15) << 8) | ((z & 15) << 4) | (x & 15))
}

// JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix$TrappedChestSection
#[derive(Debug, Clone)]
pub struct TrappedChestSection {
    pub index: i32,
    pub palette: Vec<String>,
    pub blocks: Vec<i32>,
    chest_ids: HashSet<i32>,
}

impl TrappedChestSection {
    // JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix$TrappedChestSection#<init>
    pub fn new(index: i32, palette: Vec<String>, blocks: Vec<i32>) -> Self {
        let mut chest_ids = HashSet::new();
        for (id, name) in palette.iter().enumerate() {
            if name == "minecraft:trapped_chest" {
                chest_ids.insert(id as i32);
            }
        }
        Self {
            index,
            palette,
            blocks,
            chest_ids,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix$TrappedChestSection#skippable
    pub fn is_skippable(&self) -> bool {
        self.chest_ids.is_empty()
    }

    // JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix$TrappedChestSection#getBlock
    pub fn get_block(&self, index: usize) -> i32 {
        self.blocks[index]
    }

    // JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix$TrappedChestSection#isTrappedChest
    pub fn is_trapped_chest(&self, block: i32) -> bool {
        self.chest_ids.contains(&block)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix#makeRule
pub fn collect_trapped_chest_indices(sections: &[TrappedChestSection]) -> HashSet<i32> {
    let mut result = HashSet::new();
    for section in sections {
        if section.is_skippable() {
            continue;
        }
        for block_index in 0..SIZE {
            let block = section.get_block(block_index);
            if section.is_trapped_chest(block) {
                result.insert(section.index.wrapping_shl(12) | block_index as i32);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_index_packs_local_coordinates() {
        assert_eq!(get_index(1, 20, 3), (1 << 12) | (4 << 8) | (3 << 4) | 1);
    }

    #[test]
    fn get_index_handles_negative_coordinates_with_java_bitwise_rules() {
        assert_eq!(get_index(-1, -1, -1), ((-1i32 >> 4) << 12) | ((15 << 8) | (15 << 4) | 15));
    }

    #[test]
    fn section_is_skippable_when_no_trapped_chests() {
        let section = TrappedChestSection::new(2, vec!["minecraft:chest".to_string()], vec![0; SIZE]);
        assert!(section.is_skippable());
        assert!(!section.is_trapped_chest(0));
    }

    #[test]
    fn section_detects_trapped_chest_palette_entry() {
        let section = TrappedChestSection::new(
            2,
            vec!["minecraft:chest".to_string(), "minecraft:trapped_chest".to_string()],
            vec![1; SIZE],
        );
        assert!(!section.is_skippable());
        assert!(section.is_trapped_chest(1));
    }

    #[test]
    fn collect_trapped_chest_indices_uses_section_index() {
        let section = TrappedChestSection::new(
            3,
            vec!["minecraft:trapped_chest".to_string()],
            vec![0; SIZE],
        );
        let indices = collect_trapped_chest_indices(&[section]);
        assert_eq!(indices.len(), SIZE);
        assert!(indices.contains(&(3 << 12)));
        assert!(indices.contains(&(3 << 12 | 4095)));
    }

    #[test]
    fn collect_skips_sections_without_trapped_chests() {
        let section = TrappedChestSection::new(
            3,
            vec!["minecraft:chest".to_string()],
            vec![0; SIZE],
        );
        let indices = collect_trapped_chest_indices(&[section]);
        assert!(indices.is_empty());
    }
}
