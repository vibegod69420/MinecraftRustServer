#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FluidState {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Biome {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Holder<T: Clone> {
    pub value: T,
}

#[derive(Debug, Clone, Default)]
pub struct Level;

#[derive(Debug, Clone, Default)]
pub struct BlockEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityCreationType {
    Immediate,
    Queued,
    Check,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullChunkStatus {
    Inaccessible,
    Full,
}

#[derive(Debug, Clone, Default)]
pub struct LevelChunk {
    pub level: Level,
    pub pos: ChunkPos,
}

pub struct EmptyLevelChunk {
    pub base: LevelChunk,
    biome: Holder<Biome>,
}

impl EmptyLevelChunk {
    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#<init>(Level, ChunkPos, Holder)
    pub fn new(level: Level, pos: ChunkPos, biome: Holder<Biome>) -> Self {
        Self {
            base: LevelChunk { level, pos },
            biome,
        }
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#getBlockState(BlockPos)
    pub fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
        BlockState { id: 0 } // VOID_AIR default
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#setBlockState(BlockPos, BlockState, int)
    pub fn set_block_state(
        &mut self,
        _pos: &BlockPos,
        _state: BlockState,
        _flags: i32,
    ) -> Option<BlockState> {
        None
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#getFluidState(BlockPos)
    pub fn get_fluid_state(&self, _pos: &BlockPos) -> FluidState {
        FluidState { id: 0 } // EMPTY default
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#getLightEmission(BlockPos)
    pub fn get_light_emission(&self, _pos: &BlockPos) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#getBlockEntity(BlockPos, LevelChunk.EntityCreationType)
    pub fn get_block_entity(
        &self,
        _pos: &BlockPos,
        _creation_type: EntityCreationType,
    ) -> Option<BlockEntity> {
        None
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#addAndRegisterBlockEntity(BlockEntity)
    pub fn add_and_register_block_entity(&mut self, _block_entity: BlockEntity) {}

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#setBlockEntity(BlockEntity)
    pub fn set_block_entity(&mut self, _block_entity: BlockEntity) {}

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#removeBlockEntity(BlockPos)
    pub fn remove_block_entity(&mut self, _pos: &BlockPos) {}

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#isEmpty()
    pub fn is_empty(&self) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#isYSpaceEmpty(int, int)
    pub fn is_y_space_empty(&self, _start_y: i32, _end_y: i32) -> bool {
        true
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#getFullStatus()
    pub fn get_full_status(&self) -> FullChunkStatus {
        FullChunkStatus::Full
    }

    // JAVA: net.minecraft.world.level.chunk.EmptyLevelChunk#getNoiseBiome(int, int, int)
    pub fn get_noise_biome(&self, _x: i32, _y: i32, _z: i32) -> &Holder<Biome> {
        &self.biome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk() -> EmptyLevelChunk {
        EmptyLevelChunk::new(
            Level::default(),
            ChunkPos::new(1, -2),
            Holder {
                value: Biome { id: 42 },
            },
        )
    }

    #[test]
    fn test_block_and_fluid_state() {
        let chunk = make_chunk();
        let pos = BlockPos::new(0, 64, 0);
        assert_eq!(chunk.get_block_state(&pos), BlockState { id: 0 });
        assert_eq!(chunk.get_fluid_state(&pos), FluidState { id: 0 });
        assert_eq!(chunk.get_light_emission(&pos), 0);
    }

    #[test]
    fn test_set_block_state_returns_none() {
        let mut chunk = make_chunk();
        let pos = BlockPos::new(-100, -64, 999);
        let result = chunk.set_block_state(&pos, BlockState { id: 5 }, 3);
        assert_eq!(result, None);
    }

    #[test]
    fn test_block_entity_operations_noop() {
        let mut chunk = make_chunk();
        let pos = BlockPos::new(i32::MIN, 0, i32::MAX);
        assert!(chunk
            .get_block_entity(&pos, EntityCreationType::Check)
            .is_none());
        assert!(chunk
            .get_block_entity(&pos, EntityCreationType::Immediate)
            .is_none());
        chunk.add_and_register_block_entity(BlockEntity::default());
        chunk.set_block_entity(BlockEntity::default());
        chunk.remove_block_entity(&pos);
    }

    #[test]
    fn test_empty_and_status() {
        let chunk = make_chunk();
        assert!(chunk.is_empty());
        assert!(chunk.is_y_space_empty(i32::MIN, i32::MAX));
        assert!(chunk.is_y_space_empty(0, 255));
        assert_eq!(chunk.get_full_status(), FullChunkStatus::Full);
    }

    #[test]
    fn test_noise_biome_negative_coords() {
        let chunk = make_chunk();
        let b1 = chunk.get_noise_biome(0, 0, 0);
        let b2 = chunk.get_noise_biome(i32::MIN, -64, i32::MAX);
        assert_eq!(b1.value.id, 42);
        assert_eq!(b2.value.id, 42);
        assert_eq!(chunk.base.pos, ChunkPos::new(1, -2));
    }
}
