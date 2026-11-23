use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Minimal self-contained type stubs for the ImposterProtoChunk port
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapTypes {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FluidState(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockEntity {
    pub pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Entity {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LevelChunkSection {
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkStatus(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Heightmap {
    pub data: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Biome(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Structure(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StructureStart(pub bool);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundTag(pub i32);

#[derive(Debug, Clone, Default)]
pub struct HolderLookupProvider;

#[derive(Debug, Clone, Default)]
pub struct CarvingMask;

#[derive(Debug, Clone, Default)]
pub struct BlendingData;

#[derive(Debug, Clone, Default)]
pub struct ChunkSkyLightSources;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PackedTicks(pub i64);

#[derive(Debug, Clone, Default)]
pub struct BiomeResolver;

#[derive(Debug, Clone, Default)]
pub struct ClimateSampler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Block;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Fluid;

#[derive(Clone)]
pub struct TickContainerAccess<T> {
    pub is_empty: bool,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for TickContainerAccess<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TickContainerAccess")
            .field("is_empty", &self.is_empty)
            .finish()
    }
}

impl<T> TickContainerAccess<T> {
    pub fn empty_container() -> Self {
        Self {
            is_empty: true,
            _marker: PhantomData,
        }
    }

    pub fn non_empty() -> Self {
        Self {
            is_empty: false,
            _marker: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// Trait representing the wrapped LevelChunk API used by ImposterProtoChunk
// ---------------------------------------------------------------------------

pub trait LevelChunkLike {
    fn get_block_entity(&self, pos: &BlockPos) -> Option<BlockEntity>;
    fn get_block_state(&self, pos: &BlockPos) -> BlockState;
    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState;
    fn get_section(&self, index: i32) -> &LevelChunkSection;
    fn set_block_state(
        &mut self,
        pos: &BlockPos,
        state: BlockState,
        flags: i32,
    ) -> Option<BlockState>;
    fn set_block_entity(&mut self, be: BlockEntity);
    fn add_entity(&mut self, entity: Entity);
    fn get_sections(&self) -> &[LevelChunkSection];
    fn get_or_create_heightmap_unprimed(&self, type_: HeightmapTypes) -> Heightmap;
    fn get_height(&self, type_: HeightmapTypes, x: i32, z: i32) -> i32;
    fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> Holder<Biome>;
    fn get_pos(&self) -> ChunkPos;
    fn get_start_for_structure(&self, structure: &Structure) -> Option<StructureStart>;
    fn get_all_starts(&self) -> HashMap<Structure, StructureStart>;
    fn get_references_for_structure(&self, structure: &Structure) -> HashSet<i64>;
    fn get_all_references(&self) -> HashMap<Structure, HashSet<i64>>;
    fn mark_unsaved(&mut self);
    fn get_persisted_status(&self) -> ChunkStatus;
    fn get_block_entity_nbt(&self, pos: &BlockPos) -> Option<CompoundTag>;
    fn get_block_entity_nbt_for_saving(
        &self,
        pos: &BlockPos,
        provider: &HolderLookupProvider,
    ) -> Option<CompoundTag>;
    fn find_blocks(
        &self,
        predicate: &dyn Fn(&BlockState) -> bool,
        output: &mut dyn FnMut(BlockPos, BlockState),
    );
    fn get_block_ticks(&self) -> TickContainerAccess<Block>;
    fn get_fluid_ticks(&self) -> TickContainerAccess<Fluid>;
    fn get_ticks_for_serialization(&self, game_time: i64) -> PackedTicks;
    fn get_blending_data(&self) -> Option<BlendingData>;
    fn is_light_correct(&self) -> bool;
    fn set_light_correct(&mut self, light_correct: bool);
    fn fill_biomes_from_noise(&mut self, resolver: &BiomeResolver, sampler: &ClimateSampler);
    fn initialize_light_sources(&mut self);
    fn get_sky_light_sources(&self) -> ChunkSkyLightSources;
}

// ---------------------------------------------------------------------------
// Minimal ProtoChunk base state for super-delegated behavior
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ProtoChunkBase {
    pub persisted_status: ChunkStatus,
    pub sections: Vec<LevelChunkSection>,
    pub carving_mask: Option<CarvingMask>,
}

impl ProtoChunkBase {
    fn new(section_count: usize) -> Self {
        Self {
            persisted_status: ChunkStatus(0),
            sections: vec![LevelChunkSection::default(); section_count],
            carving_mask: None,
        }
    }
}

// ---------------------------------------------------------------------------
// ImposterProtoChunk port
// ---------------------------------------------------------------------------

pub struct ImposterProtoChunk<W: LevelChunkLike> {
    wrapped: W,
    allow_writes: bool,
    base: ProtoChunkBase,
}

impl<W: LevelChunkLike> ImposterProtoChunk<W> {
    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#<init>
    pub fn new(wrapped: W, allow_writes: bool) -> Self {
        let section_count = wrapped.get_sections().len();
        Self {
            wrapped,
            allow_writes,
            base: ProtoChunkBase::new(section_count),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getBlockEntity
    pub fn get_block_entity(&self, pos: &BlockPos) -> Option<BlockEntity> {
        self.wrapped.get_block_entity(pos)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getBlockState
    pub fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        self.wrapped.get_block_state(pos)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getFluidState
    pub fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        self.wrapped.get_fluid_state(pos)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getSection
    pub fn get_section(&self, index: i32) -> &LevelChunkSection {
        if self.allow_writes {
            self.wrapped.get_section(index)
        } else {
            &self.base.sections[index as usize]
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setBlockState
    pub fn set_block_state(
        &mut self,
        pos: &BlockPos,
        state: BlockState,
        flags: i32,
    ) -> Option<BlockState> {
        if self.allow_writes {
            self.wrapped.set_block_state(pos, state, flags)
        } else {
            None
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setBlockEntity
    pub fn set_block_entity(&mut self, be: BlockEntity) {
        if self.allow_writes {
            self.wrapped.set_block_entity(be);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#addEntity
    pub fn add_entity(&mut self, entity: Entity) {
        if self.allow_writes {
            self.wrapped.add_entity(entity);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setPersistedStatus
    pub fn set_persisted_status(&mut self, status: ChunkStatus) {
        if self.allow_writes {
            self.base.persisted_status = status;
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getSections
    pub fn get_sections(&self) -> &[LevelChunkSection] {
        self.wrapped.get_sections()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setHeightmap
    pub fn set_heightmap(&mut self, _type_: HeightmapTypes, _data: &[i64]) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#fixType
    fn fix_type(type_: HeightmapTypes) -> HeightmapTypes {
        if type_ == HeightmapTypes::WorldSurfaceWg {
            HeightmapTypes::WorldSurface
        } else if type_ == HeightmapTypes::OceanFloorWg {
            HeightmapTypes::OceanFloor
        } else {
            type_
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getOrCreateHeightmapUnprimed
    pub fn get_or_create_heightmap_unprimed(&self, type_: HeightmapTypes) -> Heightmap {
        self.wrapped.get_or_create_heightmap_unprimed(type_)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getHeight
    pub fn get_height(&self, type_: HeightmapTypes, x: i32, z: i32) -> i32 {
        self.wrapped
            .get_height(Self::fix_type(type_), x, z)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getNoiseBiome
    pub fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> Holder<Biome> {
        self.wrapped.get_noise_biome(x, y, z)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getPos
    pub fn get_pos(&self) -> ChunkPos {
        self.wrapped.get_pos()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getStartForStructure
    pub fn get_start_for_structure(&self, structure: &Structure) -> Option<StructureStart> {
        self.wrapped.get_start_for_structure(structure)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setStartForStructure
    pub fn set_start_for_structure(&mut self, _structure: Structure, _start: StructureStart) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getAllStarts
    pub fn get_all_starts(&self) -> HashMap<Structure, StructureStart> {
        self.wrapped.get_all_starts()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setAllStarts
    pub fn set_all_starts(&mut self, _starts: HashMap<Structure, StructureStart>) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getReferencesForStructure
    pub fn get_references_for_structure(&self, structure: &Structure) -> HashSet<i64> {
        self.wrapped.get_references_for_structure(structure)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#addReferenceForStructure
    pub fn add_reference_for_structure(&mut self, _structure: Structure, _reference: i64) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getAllReferences
    pub fn get_all_references(&self) -> HashMap<Structure, HashSet<i64>> {
        self.wrapped.get_all_references()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setAllReferences
    pub fn set_all_references(&mut self, _references: HashMap<Structure, HashSet<i64>>) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#markUnsaved
    pub fn mark_unsaved(&mut self) {
        self.wrapped.mark_unsaved();
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#canBeSerialized
    pub fn can_be_serialized(&self) -> bool {
        false
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#tryMarkSaved
    pub fn try_mark_saved(&mut self) -> bool {
        false
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#isUnsaved
    pub fn is_unsaved(&self) -> bool {
        false
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getPersistedStatus
    pub fn get_persisted_status(&self) -> ChunkStatus {
        self.wrapped.get_persisted_status()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#removeBlockEntity
    pub fn remove_block_entity(&mut self, _pos: &BlockPos) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#markPosForPostprocessing
    pub fn mark_pos_for_postprocessing(&mut self, _pos: BlockPos) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setBlockEntityNbt
    pub fn set_block_entity_nbt(&mut self, _tag: CompoundTag) {}

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getBlockEntityNbt
    pub fn get_block_entity_nbt(&self, pos: &BlockPos) -> Option<CompoundTag> {
        self.wrapped.get_block_entity_nbt(pos)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getBlockEntityNbtForSaving
    pub fn get_block_entity_nbt_for_saving(
        &self,
        pos: &BlockPos,
        provider: &HolderLookupProvider,
    ) -> Option<CompoundTag> {
        self.wrapped.get_block_entity_nbt_for_saving(pos, provider)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#findBlocks
    pub fn find_blocks(
        &self,
        predicate: &dyn Fn(&BlockState) -> bool,
        output: &mut dyn FnMut(BlockPos, BlockState),
    ) {
        self.wrapped.find_blocks(predicate, output);
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getBlockTicks
    pub fn get_block_ticks(&self) -> TickContainerAccess<Block> {
        if self.allow_writes {
            self.wrapped.get_block_ticks()
        } else {
            TickContainerAccess::empty_container()
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getFluidTicks
    pub fn get_fluid_ticks(&self) -> TickContainerAccess<Fluid> {
        if self.allow_writes {
            self.wrapped.get_fluid_ticks()
        } else {
            TickContainerAccess::empty_container()
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getTicksForSerialization
    pub fn get_ticks_for_serialization(&self, game_time: i64) -> PackedTicks {
        self.wrapped.get_ticks_for_serialization(game_time)
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getBlendingData
    pub fn get_blending_data(&self) -> Option<BlendingData> {
        self.wrapped.get_blending_data()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getCarvingMask
    pub fn get_carving_mask(&self) -> &CarvingMask {
        if self.allow_writes {
            self.base
                .carving_mask
                .as_ref()
                .expect("CarvingMask not created")
        } else {
            panic!("UnsupportedOperationException: Meaningless in this context");
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getOrCreateCarvingMask
    pub fn get_or_create_carving_mask(&mut self) -> &CarvingMask {
        if self.allow_writes {
            if self.base.carving_mask.is_none() {
                self.base.carving_mask = Some(CarvingMask::default());
            }
            self.base.carving_mask.as_ref().expect("CarvingMask present")
        } else {
            panic!("UnsupportedOperationException: Meaningless in this context");
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getWrapped
    pub fn get_wrapped(&self) -> &W {
        &self.wrapped
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#isLightCorrect
    pub fn is_light_correct(&self) -> bool {
        self.wrapped.is_light_correct()
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#setLightCorrect
    pub fn set_light_correct(&mut self, light_correct: bool) {
        self.wrapped.set_light_correct(light_correct);
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#fillBiomesFromNoise
    pub fn fill_biomes_from_noise(&mut self, resolver: &BiomeResolver, sampler: &ClimateSampler) {
        if self.allow_writes {
            self.wrapped.fill_biomes_from_noise(resolver, sampler);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#initializeLightSources
    pub fn initialize_light_sources(&mut self) {
        self.wrapped.initialize_light_sources();
    }

    // JAVA: net.minecraft.world.level.chunk.ImposterProtoChunk#getSkyLightSources
    pub fn get_sky_light_sources(&self) -> ChunkSkyLightSources {
        self.wrapped.get_sky_light_sources()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockChunk {
        pos: ChunkPos,
        sections: Vec<LevelChunkSection>,
        persisted_status: ChunkStatus,
        light_correct: bool,
        mark_unsaved_count: u32,
        last_set_block: Option<(BlockPos, BlockState, i32)>,
        last_block_entity: Option<BlockEntity>,
        last_entity: Option<Entity>,
        fill_biomes_called: bool,
        init_light_called: bool,
        block_states: HashMap<BlockPos, BlockState>,
        heights: HashMap<(HeightmapTypes, i32, i32), i32>,
    }

    impl MockChunk {
        fn new() -> Self {
            Self {
                pos: ChunkPos { x: 1, z: -2 },
                sections: vec![LevelChunkSection { id: 0 }, LevelChunkSection { id: 1 }],
                persisted_status: ChunkStatus(0),
                light_correct: false,
                mark_unsaved_count: 0,
                last_set_block: None,
                last_block_entity: None,
                last_entity: None,
                fill_biomes_called: false,
                init_light_called: false,
                block_states: HashMap::new(),
                heights: HashMap::new(),
            }
        }
    }

    impl LevelChunkLike for MockChunk {
        fn get_block_entity(&self, pos: &BlockPos) -> Option<BlockEntity> {
            if pos.x == 42 {
                Some(BlockEntity { pos: *pos })
            } else {
                None
            }
        }

        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.block_states.get(pos).copied().unwrap_or_default()
        }

        fn get_fluid_state(&self, _pos: &BlockPos) -> FluidState {
            FluidState(7)
        }

        fn get_section(&self, index: i32) -> &LevelChunkSection {
            &self.sections[index as usize]
        }

        fn set_block_state(
            &mut self,
            pos: &BlockPos,
            state: BlockState,
            flags: i32,
        ) -> Option<BlockState> {
            let old = self.block_states.insert(*pos, state).unwrap_or_default();
            self.last_set_block = Some((*pos, state, flags));
            Some(old)
        }

        fn set_block_entity(&mut self, be: BlockEntity) {
            self.last_block_entity = Some(be);
        }

        fn add_entity(&mut self, entity: Entity) {
            self.last_entity = Some(entity);
        }

        fn get_sections(&self) -> &[LevelChunkSection] {
            &self.sections
        }

        fn get_or_create_heightmap_unprimed(&self, type_: HeightmapTypes) -> Heightmap {
            Heightmap {
                data: vec![type_ as i64],
            }
        }

        fn get_height(&self, type_: HeightmapTypes, x: i32, z: i32) -> i32 {
            self.heights.get(&(type_, x, z)).copied().unwrap_or(0)
        }

        fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> Holder<Biome> {
            let v = x.wrapping_add(y).wrapping_add(z) as u32;
            Holder(Biome(v))
        }

        fn get_pos(&self) -> ChunkPos {
            self.pos
        }

        fn get_start_for_structure(&self, structure: &Structure) -> Option<StructureStart> {
            if structure.0 == 1 {
                Some(StructureStart(true))
            } else {
                None
            }
        }

        fn get_all_starts(&self) -> HashMap<Structure, StructureStart> {
            let mut m = HashMap::new();
            m.insert(Structure(1), StructureStart(true));
            m
        }

        fn get_references_for_structure(&self, structure: &Structure) -> HashSet<i64> {
            let mut s = HashSet::new();
            s.insert(structure.0 as i64);
            s
        }

        fn get_all_references(&self) -> HashMap<Structure, HashSet<i64>> {
            let mut m = HashMap::new();
            m.insert(Structure(1), self.get_references_for_structure(&Structure(1)));
            m
        }

        fn mark_unsaved(&mut self) {
            self.mark_unsaved_count = self.mark_unsaved_count.wrapping_add(1);
        }

        fn get_persisted_status(&self) -> ChunkStatus {
            self.persisted_status
        }

        fn get_block_entity_nbt(&self, pos: &BlockPos) -> Option<CompoundTag> {
            if pos.y < 0 {
                Some(CompoundTag(pos.y))
            } else {
                None
            }
        }

        fn get_block_entity_nbt_for_saving(
            &self,
            pos: &BlockPos,
            _provider: &HolderLookupProvider,
        ) -> Option<CompoundTag> {
            Some(CompoundTag(pos.x))
        }

        fn find_blocks(
            &self,
            predicate: &dyn Fn(&BlockState) -> bool,
            output: &mut dyn FnMut(BlockPos, BlockState),
        ) {
            for (&pos, &state) in &self.block_states {
                if predicate(&state) {
                    output(pos, state);
                }
            }
        }

        fn get_block_ticks(&self) -> TickContainerAccess<Block> {
            TickContainerAccess::non_empty()
        }

        fn get_fluid_ticks(&self) -> TickContainerAccess<Fluid> {
            TickContainerAccess::non_empty()
        }

        fn get_ticks_for_serialization(&self, game_time: i64) -> PackedTicks {
            PackedTicks(game_time)
        }

        fn get_blending_data(&self) -> Option<BlendingData> {
            Some(BlendingData)
        }

        fn is_light_correct(&self) -> bool {
            self.light_correct
        }

        fn set_light_correct(&mut self, light_correct: bool) {
            self.light_correct = light_correct;
        }

        fn fill_biomes_from_noise(&mut self, _resolver: &BiomeResolver, _sampler: &ClimateSampler) {
            self.fill_biomes_called = true;
        }

        fn initialize_light_sources(&mut self) {
            self.init_light_called = true;
        }

        fn get_sky_light_sources(&self) -> ChunkSkyLightSources {
            ChunkSkyLightSources
        }
    }

    #[test]
    fn test_fix_type_height_and_negative_coords() {
        let mut mock = MockChunk::new();
        mock.heights
            .insert((HeightmapTypes::WorldSurface, -5, 3), 42);
        mock.heights
            .insert((HeightmapTypes::OceanFloor, i32::MIN, i32::MAX), -64);
        let imp = ImposterProtoChunk::new(mock, false);

        assert_eq!(imp.get_height(HeightmapTypes::WorldSurfaceWg, -5, 3), 42);
        assert_eq!(
            imp.get_height(HeightmapTypes::OceanFloorWg, i32::MIN, i32::MAX),
            -64
        );
        assert_eq!(imp.get_height(HeightmapTypes::MotionBlocking, 0, 0), 0);
    }

    #[test]
    fn test_no_writes_blocks_mutations() {
        let mock = MockChunk::new();
        let mut imp = ImposterProtoChunk::new(mock, false);
        let pos = BlockPos { x: 1, y: -64, z: 2 };

        let old = imp.set_block_state(&pos, BlockState(5), 3);
        assert_eq!(old, None);
        assert!(imp.get_wrapped().last_set_block.is_none());

        imp.set_block_entity(BlockEntity { pos });
        assert!(imp.get_wrapped().last_block_entity.is_none());

        imp.add_entity(Entity { id: 7 });
        assert!(imp.get_wrapped().last_entity.is_none());

        imp.set_persisted_status(ChunkStatus(9));
        assert_eq!(imp.get_wrapped().persisted_status, ChunkStatus(0));
        assert!(!imp.is_unsaved());
        assert!(!imp.can_be_serialized());
        assert!(!imp.try_mark_saved());
    }

    #[test]
    fn test_allow_writes_delegates_and_updates() {
        let mock = MockChunk::new();
        let mut imp = ImposterProtoChunk::new(mock, true);
        let pos = BlockPos { x: -1, y: 320, z: -2 };

        let old = imp.set_block_state(&pos, BlockState(10), 2);
        assert_eq!(old, Some(BlockState(0)));
        assert_eq!(
            imp.get_wrapped().last_set_block,
            Some((pos, BlockState(10), 2))
        );

        imp.mark_unsaved();
        assert_eq!(imp.get_wrapped().mark_unsaved_count, 1);

        imp.set_persisted_status(ChunkStatus(5));
        // getPersistedStatus delegates to wrapped, not base
        assert_eq!(imp.get_persisted_status(), ChunkStatus(0));

        imp.set_light_correct(true);
        assert!(imp.is_light_correct());
    }

    #[test]
    fn test_carving_mask_and_ticks_behavior() {
        let mock = MockChunk::new();
        let imp_no = ImposterProtoChunk::new(mock.clone(), false);
        assert!(imp_no.get_block_ticks().is_empty);
        assert!(imp_no.get_fluid_ticks().is_empty);

        let mut imp_yes = ImposterProtoChunk::new(mock, true);
        assert!(!imp_yes.get_block_ticks().is_empty);
        assert!(!imp_yes.get_fluid_ticks().is_empty);
        let _ = imp_yes.get_or_create_carving_mask();
        let _ = imp_yes.get_carving_mask();
    }

    #[test]
    #[should_panic(expected = "Meaningless in this context")]
    fn test_carving_mask_panics_when_no_writes() {
        let mock = MockChunk::new();
        let imp = ImposterProtoChunk::new(mock, false);
        let _ = imp.get_carving_mask();
    }

    #[test]
    fn test_integer_edges_and_float_bits() {
        let mut mock = MockChunk::new();
        let pos = BlockPos {
            x: i32::MIN,
            y: -1,
            z: i32::MAX,
        };
        let nan_bits = f32::NAN.to_bits();
        mock.block_states.insert(pos, BlockState(nan_bits));
        let imp = ImposterProtoChunk::new(mock, false);

        let bs = imp.get_block_state(&pos);
        let f = f32::from_bits(bs.0);
        assert!(f.is_nan());
        assert_eq!(f.to_bits(), nan_bits);

        let f64_bits: u64 = 0xFFF0_0000_0000_0001;
        let f64_val = f64::from_bits(f64_bits);
        assert_eq!(f64_val.to_bits(), f64_bits);

        let packed_min = imp.get_ticks_for_serialization(i64::MIN);
        let packed_max = imp.get_ticks_for_serialization(i64::MAX);
        assert_eq!(packed_min.0, i64::MIN);
        assert_eq!(packed_max.0, i64::MAX);

        let biome = imp.get_noise_biome(i32::MAX, 1, i32::MIN);
        let expected = i32::MAX.wrapping_add(1).wrapping_add(i32::MIN) as u32;
        assert_eq!(biome.0 .0, expected);
    }
}
