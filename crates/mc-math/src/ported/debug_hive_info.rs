/// Minimal self-contained placeholder for `net.minecraft.world.level.block.Block`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: String,
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    pub block: Block,
    pub honey_level: i32,
}

impl BlockState {
    pub fn get_block(&self) -> &Block {
        &self.block
    }
}

/// Minimal self-contained placeholder for `net.minecraft.world.level.block.entity.BeehiveBlockEntity`.
pub struct BeehiveBlockEntity {
    block_state: BlockState,
    occupant_count: i32,
    sedated: bool,
}

impl BeehiveBlockEntity {
    pub fn new(block_state: BlockState, occupant_count: i32, sedated: bool) -> Self {
        Self {
            block_state,
            occupant_count,
            sedated,
        }
    }

    pub fn get_block_state(&self) -> &BlockState {
        &self.block_state
    }

    pub fn get_occupant_count(&self) -> i32 {
        self.occupant_count
    }

    pub fn is_sedated(&self) -> bool {
        self.sedated
    }

    // JAVA: net.minecraft.world.level.block.entity.BeehiveBlockEntity#getHoneyLevel
    pub fn get_honey_level(state: &BlockState) -> i32 {
        state.honey_level
    }
}

/// Faithful port of `net.minecraft.util.debug.DebugHiveInfo`.
/// STREAM_CODEC is omitted because the Minecraft networking codec framework is not ported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugHiveInfo {
    pub block_type: Block,
    pub occupant_count: i32,
    pub honey_level: i32,
    pub sedated: bool,
}

impl DebugHiveInfo {
    // JAVA: net.minecraft.util.debug.DebugHiveInfo#<init>
    pub fn new(block_type: Block, occupant_count: i32, honey_level: i32, sedated: bool) -> Self {
        Self {
            block_type,
            occupant_count,
            honey_level,
            sedated,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugHiveInfo#pack
    pub fn pack(entity: &BeehiveBlockEntity) -> Self {
        Self {
            block_type: entity.get_block_state().get_block().clone(),
            occupant_count: entity.get_occupant_count(),
            honey_level: BeehiveBlockEntity::get_honey_level(entity.get_block_state()),
            sedated: entity.is_sedated(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bee_block(id: &str) -> Block {
        Block { id: id.to_string() }
    }

    fn hive_entity(id: &str, honey_level: i32, occupant_count: i32, sedated: bool) -> BeehiveBlockEntity {
        BeehiveBlockEntity::new(
            BlockState {
                block: bee_block(id),
                honey_level,
            },
            occupant_count,
            sedated,
        )
    }

    #[test]
    fn pack_extracts_all_fields() {
        let entity = hive_entity("minecraft:beehive", 5, 3, true);
        let info = DebugHiveInfo::pack(&entity);
        assert_eq!(info.block_type, bee_block("minecraft:beehive"));
        assert_eq!(info.occupant_count, 3);
        assert_eq!(info.honey_level, 5);
        assert!(info.sedated);
    }

    #[test]
    fn pack_with_zero_occupants_and_no_honey() {
        let entity = hive_entity("minecraft:bee_nest", 0, 0, false);
        let info = DebugHiveInfo::pack(&entity);
        assert_eq!(info.occupant_count, 0);
        assert_eq!(info.honey_level, 0);
        assert!(!info.sedated);
    }

    #[test]
    fn new_stores_fields_directly() {
        let info = DebugHiveInfo::new(bee_block("minecraft:beehive"), 2, 4, false);
        assert_eq!(info.block_type.id, "minecraft:beehive");
        assert_eq!(info.occupant_count, 2);
        assert_eq!(info.honey_level, 4);
        assert!(!info.sedated);
    }

    #[test]
    fn get_honey_level_reads_block_state() {
        let state = BlockState {
            block: bee_block("minecraft:beehive"),
            honey_level: 5,
        };
        assert_eq!(BeehiveBlockEntity::get_honey_level(&state), 5);
    }
}
