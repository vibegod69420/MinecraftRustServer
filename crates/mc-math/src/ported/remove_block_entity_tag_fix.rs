use std::collections::HashSet;

// JAVA: net.minecraft.util.datafix.fixes.RemoveBlockEntityTagFix#RemoveBlockEntityTagFix
pub struct RemoveBlockEntityTagFix {
    block_entity_ids_to_drop: HashSet<String>,
}

impl RemoveBlockEntityTagFix {
    // JAVA: net.minecraft.util.datafix.fixes.RemoveBlockEntityTagFix#RemoveBlockEntityTagFix
    pub fn new(block_entity_ids_to_drop: HashSet<String>) -> Self {
        Self {
            block_entity_ids_to_drop,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemoveBlockEntityTagFix#removeBlockEntity
    pub fn should_drop(&self, block_entity_id: &str) -> bool {
        self.block_entity_ids_to_drop.contains(block_entity_id)
    }

    // JAVA: net.minecraft.util.datafix.fixes.RemoveBlockEntityTagFix#removeBlockEntity
    pub fn remove_block_entity_tag(&self, block_entity_id: &str) -> bool {
        self.should_drop(block_entity_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_listed_id() {
        let fix = RemoveBlockEntityTagFix::new(HashSet::from(["minecraft:chest".to_string()]));
        assert!(fix.should_drop("minecraft:chest"));
    }

    #[test]
    fn keeps_unlisted_id() {
        let fix = RemoveBlockEntityTagFix::new(HashSet::from(["minecraft:chest".to_string()]));
        assert!(!fix.should_drop("minecraft:furnace"));
    }

    #[test]
    fn empty_set_drops_nothing() {
        let fix = RemoveBlockEntityTagFix::new(HashSet::new());
        assert!(!fix.remove_block_entity_tag("minecraft:chest"));
    }

    #[test]
    fn multiple_ids() {
        let fix = RemoveBlockEntityTagFix::new(HashSet::from([
            "minecraft:chest".to_string(),
            "minecraft:barrel".to_string(),
        ]));
        assert!(fix.should_drop("minecraft:barrel"));
        assert!(!fix.should_drop("minecraft:shulker_box"));
    }
}
