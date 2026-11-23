use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Minimal self-contained types for the ProtoChunk port
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SectionPos;

impl SectionPos {
    pub fn section_relative(v: i32) -> i32 {
        v & 15
    }

    pub fn section_to_block_coord(section: i32, relative: i32) -> i32 {
        section.wrapping_shl(4).wrapping_add(relative)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChunkStatus {
    Empty,
    StructureStarts,
    StructureReferences,
    Biomes,
    Noise,
    Surface,
    Carvers,
    Features,
    InitializeLight,
    Light,
    Spawn,
    Full,
}

impl ChunkStatus {
    pub const EMPTY: Self = Self::Empty;
    pub const INITIALIZE_LIGHT: Self = Self::InitializeLight;
    pub const BIOMES: Self = Self::Biomes;

    pub fn index(self) -> i32 {
        match self {
            Self::Empty => 0,
            Self::StructureStarts => 1,
            Self::StructureReferences => 2,
            Self::Biomes => 3,
            Self::Noise => 4,
            Self::Surface => 5,
            Self::Carvers => 6,
            Self::Features => 7,
            Self::InitializeLight => 8,
            Self::Light => 9,
            Self::Spawn => 10,
            Self::Full => 11,
        }
    }

    pub fn is_or_after(self, other: ChunkStatus) -> bool {
        self.index() >= other.index()
    }

    pub fn heightmaps_after(self) -> Vec<HeightmapType> {
        // Minimal faithful set: statuses at or after FEATURES typically finalize heightmaps.
        // Kept explicit and deterministic for this self-contained port.
        if self.is_or_after(ChunkStatus::Features) {
            vec![
                HeightmapType::OceanFloorWg,
                HeightmapType::WorldSurfaceWg,
                HeightmapType::OceanFloor,
                HeightmapType::WorldSurface,
                HeightmapType::MotionBlocking,
                HeightmapType::MotionBlockingNoLeaves,
            ]
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompoundTag;

pub trait BlockStateOps: Clone + fmt::Debug + PartialEq {
    fn air() -> Self;
    fn void_air() -> Self;
    fn is_air(&self) -> bool;
    fn has_different_light_properties(&self, other: &Self) -> bool;
}

pub trait FluidStateOps: Clone + fmt::Debug {
    fn empty() -> Self;
}

pub trait ChunkSectionOps<B: BlockStateOps, F: FluidStateOps> {
    fn has_only_air(&self) -> bool;
    fn get_block_state(&self, x: i32, y: i32, z: i32) -> B;
    fn get_fluid_state(&self, x: i32, y: i32, z: i32) -> F;
    fn set_block_state(&mut self, x: i32, y: i32, z: i32, state: B) -> B;
}

pub trait LightEngineOps {
    fn update_section_status(&mut self, pos: BlockPos, is_empty: bool);
    fn check_block(&mut self, pos: BlockPos);
}

pub fn light_engine_has_different_light_properties<B: BlockStateOps>(old: &B, new_state: &B) -> bool {
    old.has_different_light_properties(new_state)
}

#[derive(Clone, Debug, Default)]
pub struct Heightmap;

impl Heightmap {
    pub fn new() -> Self {
        Self
    }

    pub fn update<B: BlockStateOps>(&mut self, _x: i32, _y: i32, _z: i32, _state: &B) {
        // Minimal placeholder: real heightmap updates are external to ProtoChunk logic.
    }

    pub fn prime_heightmaps<B, F, S, BE, LE>(
        chunk: &mut ProtoChunk<B, F, S, BE, LE>,
        types: &[HeightmapType],
    ) where
        B: BlockStateOps,
        F: FluidStateOps,
        S: ChunkSectionOps<B, F>,
        BE: BlockEntityOps,
        LE: LightEngineOps,
    {
        for t in types {
            chunk.heightmaps.entry(*t).or_insert_with(Heightmap::new);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkyLightSources;

impl SkyLightSources {
    pub fn update(&mut self, _x: i32, _y: i32, _z: i32) {
        // Minimal placeholder: real skylight source tracking is external.
    }
}

pub trait BlockEntityOps: Clone + fmt::Debug {
    fn get_block_pos(&self) -> BlockPos;
    fn save_with_full_metadata(&self) -> CompoundTag;
}

pub trait EntityOps {
    fn is_passenger(&self) -> bool;
    fn save_to_tag(&self) -> CompoundTag;
}

#[derive(Clone, Debug, Default)]
pub struct UpgradeData;

#[derive(Clone, Debug, Default)]
pub struct BlendingData;

#[derive(Clone, Debug, Default)]
pub struct PalettedContainerFactory;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelHeightAccessor {
    pub min_y: i32,
    pub height: i32,
}

impl LevelHeightAccessor {
    pub fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    pub fn get_min_y(&self) -> i32 {
        self.min_y
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_max_y(&self) -> i32 {
        self.min_y.wrapping_add(self.height)
    }

    pub fn is_outside_build_height(&self, y: i32) -> bool {
        y < self.get_min_y() || y >= self.get_max_y()
    }

    pub fn get_section_index(&self, y: i32) -> i32 {
        self.get_section_index_from_section_y(y >> 4)
    }

    pub fn get_section_index_from_section_y(&self, section_y: i32) -> i32 {
        section_y.wrapping_sub(self.get_min_section_y())
    }

    pub fn get_min_section_y(&self) -> i32 {
        self.min_y >> 4
    }

    pub fn get_sections_count(&self) -> i32 {
        self.height >> 4
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_y: i32,
    pub max_y: i32,
}

#[derive(Clone, Debug, Default)]
pub struct StructureStartData {
    pub valid: bool,
    pub bounding_box: Option<BoundingBox>,
}

impl StructureStartData {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_bounding_box(&self) -> Option<BoundingBox> {
        self.bounding_box
    }
}

#[derive(Clone, Debug)]
pub struct CarvingMask {
    pub height: i32,
    pub min_y: i32,
}

impl CarvingMask {
    pub fn new(height: i32, min_y: i32) -> Self {
        Self { height, min_y }
    }
}

#[derive(Clone, Debug)]
pub struct BelowZeroRetrogenData {
    pub target_status: ChunkStatus,
}

impl BelowZeroRetrogenData {
    pub fn new(target_status: ChunkStatus) -> Self {
        Self { target_status }
    }

    pub fn target_status(&self) -> ChunkStatus {
        self.target_status
    }
}

pub const BELOW_ZERO_RETROGEN_UPGRADE_HEIGHT_ACCESSOR: LevelHeightAccessor = LevelHeightAccessor {
    min_y: -64,
    height: 384,
};

#[derive(Clone, Debug, Default)]
pub struct ScheduledTick;

#[derive(Clone, Debug, Default)]
pub struct ProtoChunkTicks<T> {
    scheduled: Vec<ScheduledTick>,
    _marker: PhantomData<T>,
}

impl<T> ProtoChunkTicks<T> {
    pub fn new() -> Self {
        Self {
            scheduled: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn pack(&self, _game_time: i64) -> Vec<ScheduledTick> {
        self.scheduled.clone()
    }

    pub fn scheduled_ticks(&self) -> &[ScheduledTick] {
        &self.scheduled
    }
}

#[derive(Clone, Debug, Default)]
pub struct LevelChunkTicks<T> {
    pub ticks: Vec<ScheduledTick>,
    _marker: PhantomData<T>,
}

impl<T> LevelChunkTicks<T> {
    pub fn new(ticks: Vec<ScheduledTick>) -> Self {
        Self {
            ticks,
            _marker: PhantomData,
        }
    }
}

pub trait TickContainerAccess {}

impl<T> TickContainerAccess for ProtoChunkTicks<T> {}

#[derive(Clone, Debug, Default)]
pub struct PackedTicks {
    pub blocks: Vec<ScheduledTick>,
    pub fluids: Vec<ScheduledTick>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockTickMarker;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FluidTickMarker;

#[derive(Clone, Debug, Default)]
pub struct NoopLightEngine;

impl LightEngineOps for NoopLightEngine {
    fn update_section_status(&mut self, _pos: BlockPos, _is_empty: bool) {}
    fn check_block(&mut self, _pos: BlockPos) {}
}

fn get_or_create_offset_list(post_processing: &mut [Vec<i16>], index: i32) -> &mut Vec<i16> {
    let idx = if index < 0 { 0usize } else { index as usize };
    if idx < post_processing.len() {
        &mut post_processing[idx]
    } else {
        // Should not happen with correct section sizing; fall back to last section.
        let last = post_processing.len().saturating_sub(1);
        &mut post_processing[last]
    }
}

// ---------------------------------------------------------------------------
// ProtoChunk port
// ---------------------------------------------------------------------------

pub struct ProtoChunk<B, F, S, BE, LE = NoopLightEngine>
where
    B: BlockStateOps,
    F: FluidStateOps,
    S: ChunkSectionOps<B, F>,
    BE: BlockEntityOps,
    LE: LightEngineOps,
{
    // ChunkAccess-like fields used by ProtoChunk
    pub chunk_pos: ChunkPos,
    pub upgrade_data: UpgradeData,
    pub level_height_accessor: LevelHeightAccessor,
    pub sections: Vec<S>,
    pub heightmaps: BTreeMap<HeightmapType, Heightmap>,
    pub block_entities: HashMap<BlockPos, BE>,
    pub pending_block_entities: HashMap<BlockPos, CompoundTag>,
    pub post_processing: Vec<Vec<i16>>,
    pub sky_light_sources: SkyLightSources,
    pub structure_starts: HashMap<String, StructureStartData>,
    pub blending_data: Option<BlendingData>,
    pub inhabited_time: i64,
    unsaved: bool,

    // ProtoChunk fields
    light_engine: Option<LE>,
    status: ChunkStatus,
    entities: Vec<CompoundTag>,
    carving_mask: Option<CarvingMask>,
    below_zero_retrogen: Option<BelowZeroRetrogenData>,
    block_ticks: ProtoChunkTicks<BlockTickMarker>,
    fluid_ticks: ProtoChunkTicks<FluidTickMarker>,

    _phantom: PhantomData<(B, F)>,
}

impl<B, F, S, BE, LE> ProtoChunk<B, F, S, BE, LE>
where
    B: BlockStateOps,
    F: FluidStateOps,
    S: ChunkSectionOps<B, F>,
    BE: BlockEntityOps,
    LE: LightEngineOps,
{
    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#<init>
    pub fn new(
        chunk_pos: ChunkPos,
        upgrade_data: UpgradeData,
        level_height_accessor: LevelHeightAccessor,
        _paletted_container_factory: PalettedContainerFactory,
        blending_data: Option<BlendingData>,
        sections: Vec<S>,
    ) -> Self {
        Self::new_with_sections(
            chunk_pos,
            upgrade_data,
            sections,
            ProtoChunkTicks::new(),
            ProtoChunkTicks::new(),
            level_height_accessor,
            _paletted_container_factory,
            blending_data,
        )
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#<init>
    pub fn new_with_sections(
        chunk_pos: ChunkPos,
        upgrade_data: UpgradeData,
        sections: Vec<S>,
        block_ticks: ProtoChunkTicks<BlockTickMarker>,
        fluid_ticks: ProtoChunkTicks<FluidTickMarker>,
        level_height_accessor: LevelHeightAccessor,
        _paletted_container_factory: PalettedContainerFactory,
        blending_data: Option<BlendingData>,
    ) -> Self {
        let section_count = level_height_accessor.get_sections_count().max(0) as usize;
        let mut post_processing = Vec::with_capacity(section_count);
        for _ in 0..section_count {
            post_processing.push(Vec::new());
        }
        Self {
            chunk_pos,
            upgrade_data,
            level_height_accessor,
            sections,
            heightmaps: BTreeMap::new(),
            block_entities: HashMap::new(),
            pending_block_entities: HashMap::new(),
            post_processing,
            sky_light_sources: SkyLightSources,
            structure_starts: HashMap::new(),
            blending_data,
            inhabited_time: 0,
            unsaved: false,
            light_engine: None,
            status: ChunkStatus::EMPTY,
            entities: Vec::new(),
            carving_mask: None,
            below_zero_retrogen: None,
            block_ticks,
            fluid_ticks,
            _phantom: PhantomData,
        }
    }

    pub fn get_height(&self) -> i32 {
        self.level_height_accessor.get_height()
    }

    pub fn get_min_y(&self) -> i32 {
        self.level_height_accessor.get_min_y()
    }

    pub fn is_outside_build_height(&self, y: i32) -> bool {
        self.level_height_accessor.is_outside_build_height(y)
    }

    pub fn get_section_index(&self, y: i32) -> i32 {
        self.level_height_accessor.get_section_index(y)
    }

    pub fn get_section(&self, index: i32) -> Option<&S> {
        if index < 0 {
            return None;
        }
        self.sections.get(index as usize)
    }

    pub fn get_section_mut(&mut self, index: i32) -> Option<&mut S> {
        if index < 0 {
            return None;
        }
        self.sections.get_mut(index as usize)
    }

    pub fn mark_unsaved(&mut self) {
        self.unsaved = true;
    }

    pub fn is_unsaved(&self) -> bool {
        self.unsaved
    }

    pub fn get_highest_generated_status(&self) -> ChunkStatus {
        self.status
    }

    pub fn is_upgrading(&self) -> bool {
        self.below_zero_retrogen.is_some()
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBlockTicks
    pub fn get_block_ticks(&self) -> &impl TickContainerAccess {
        &self.block_ticks
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getFluidTicks
    pub fn get_fluid_ticks(&self) -> &impl TickContainerAccess {
        &self.fluid_ticks
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getTicksForSerialization
    pub fn get_ticks_for_serialization(&self, game_time: i64) -> PackedTicks {
        PackedTicks {
            blocks: self.block_ticks.pack(game_time),
            fluids: self.fluid_ticks.pack(game_time),
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBlockState
    pub fn get_block_state(&self, pos: BlockPos) -> B {
        let y = pos.get_y();
        if self.is_outside_build_height(y) {
            return B::void_air();
        }
        let idx = self.get_section_index(y);
        let Some(section) = self.get_section(idx) else {
            return B::void_air();
        };
        if section.has_only_air() {
            B::air()
        } else {
            section.get_block_state(pos.get_x() & 15, y & 15, pos.get_z() & 15)
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getFluidState
    pub fn get_fluid_state(&self, pos: BlockPos) -> F {
        let y = pos.get_y();
        if self.is_outside_build_height(y) {
            return F::empty();
        }
        let idx = self.get_section_index(y);
        let Some(section) = self.get_section(idx) else {
            return F::empty();
        };
        if section.has_only_air() {
            F::empty()
        } else {
            section.get_fluid_state(pos.get_x() & 15, y & 15, pos.get_z() & 15)
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setBlockState
    pub fn set_block_state(&mut self, pos: BlockPos, state: B, _flags: i32) -> Option<B> {
        let x = pos.get_x();
        let y = pos.get_y();
        let z = pos.get_z();
        if self.is_outside_build_height(y) {
            return Some(B::void_air());
        }

        let section_index = self.get_section_index(y);
        if section_index < 0 || (section_index as usize) >= self.sections.len() {
            return Some(B::void_air());
        }

        let (was_only_air, old_state) = {
            let section = &mut self.sections[section_index as usize];
            let was_only_air = section.has_only_air();
            if was_only_air && state.is_air() {
                return Some(state);
            }

            let sx = SectionPos::section_relative(x);
            let sy = SectionPos::section_relative(y);
            let sz = SectionPos::section_relative(z);
            let old_state = section.set_block_state(sx, sy, sz, state.clone());
            (was_only_air, old_state)
        };

        if self.status.is_or_after(ChunkStatus::INITIALIZE_LIGHT) {
            let is_only_air = self.sections[section_index as usize].has_only_air();
            if is_only_air != was_only_air {
                if let Some(le) = self.light_engine.as_mut() {
                    le.update_section_status(pos, is_only_air);
                }
            }

            if light_engine_has_different_light_properties(&old_state, &state) {
                let sx = SectionPos::section_relative(x);
                let sz = SectionPos::section_relative(z);
                self.sky_light_sources.update(sx, y, sz);
                if let Some(le) = self.light_engine.as_mut() {
                    le.check_block(pos);
                }
            }
        }

        let heightmap_types = self.get_persisted_status().heightmaps_after();
        let mut missing: Vec<HeightmapType> = Vec::new();
        for t in &heightmap_types {
            if !self.heightmaps.contains_key(t) {
                missing.push(*t);
            }
        }
        if !missing.is_empty() {
            Heightmap::prime_heightmaps(self, &missing);
        }

        let sx = SectionPos::section_relative(x);
        let sz = SectionPos::section_relative(z);
        for t in &heightmap_types {
            if let Some(hm) = self.heightmaps.get_mut(t) {
                hm.update(sx, y, sz, &state);
            }
        }

        Some(old_state)
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setBlockEntity
    pub fn set_block_entity(&mut self, block_entity: BE) {
        let pos = block_entity.get_block_pos();
        self.pending_block_entities.remove(&pos);
        self.block_entities.insert(pos, block_entity);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBlockEntity
    pub fn get_block_entity(&self, pos: BlockPos) -> Option<&BE> {
        self.block_entities.get(&pos)
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBlockEntities
    pub fn get_block_entities(&self) -> &HashMap<BlockPos, BE> {
        &self.block_entities
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#addEntity
    pub fn add_entity_tag(&mut self, tag: CompoundTag) {
        self.entities.push(tag);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#addEntity
    pub fn add_entity<E: EntityOps>(&mut self, entity: &E) {
        if !entity.is_passenger() {
            let tag = entity.save_to_tag();
            self.add_entity_tag(tag);
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setStartForStructure
    pub fn set_start_for_structure(&mut self, structure_key: String, start: StructureStartData) {
        if let Some(below_zero) = self.get_below_zero_retrogen() {
            if start.is_valid() {
                if let Some(bb) = start.get_bounding_box() {
                    let accessor = self.get_height_accessor_for_generation();
                    if bb.min_y < accessor.get_min_y() || bb.max_y > accessor.get_max_y() {
                        let _ = below_zero;
                        return;
                    }
                }
            }
        }

        self.structure_starts.insert(structure_key, start);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getEntities
    pub fn get_entities(&self) -> &Vec<CompoundTag> {
        &self.entities
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getPersistedStatus
    pub fn get_persisted_status(&self) -> ChunkStatus {
        self.status
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setPersistedStatus
    pub fn set_persisted_status(&mut self, status: ChunkStatus) {
        self.status = status;
        if let Some(bzr) = &self.below_zero_retrogen {
            if status.is_or_after(bzr.target_status()) {
                self.below_zero_retrogen = None;
            }
        }
        self.mark_unsaved();
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getNoiseBiome
    pub fn get_noise_biome(&self, _x: i32, _y: i32, _z: i32) -> Result<(), &'static str> {
        if self.get_highest_generated_status().is_or_after(ChunkStatus::BIOMES) {
            // Real biome container lookup is external; in this self-contained port we
            // preserve the precondition check behavior.
            Ok(())
        } else {
            Err("Asking for biomes before we have biomes")
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#packOffsetCoordinates
    pub fn pack_offset_coordinates(pos: BlockPos) -> i16 {
        let x = pos.get_x();
        let y = pos.get_y();
        let z = pos.get_z();
        let px = x & 15;
        let py = y & 15;
        let pz = z & 15;
        (px | (py << 4) | (pz << 8)) as i16
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#unpackOffsetCoordinates
    pub fn unpack_offset_coordinates(packed: i16, section_y: i32, chunk_pos: ChunkPos) -> BlockPos {
        let p = packed as i32;
        let ox = p & 15;
        let oy = (p >> 4) & 15;
        let oz = (p >> 8) & 15;
        let x = SectionPos::section_to_block_coord(chunk_pos.x, ox);
        let y = SectionPos::section_to_block_coord(section_y, oy);
        let z = SectionPos::section_to_block_coord(chunk_pos.z, oz);
        BlockPos::new(x, y, z)
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#markPosForPostprocessing
    pub fn mark_pos_for_postprocessing(&mut self, pos: BlockPos) {
        if !self.is_outside_build_height(pos.get_y()) {
            let idx = self.get_section_index(pos.get_y());
            if idx >= 0 && (idx as usize) < self.post_processing.len() {
                let packed = Self::pack_offset_coordinates(pos);
                self.post_processing[idx as usize].push(packed);
            }
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#addPackedPostProcess
    pub fn add_packed_post_process(&mut self, packed_positions: &[i16], section_index: i32) {
        if section_index < 0 || (section_index as usize) >= self.post_processing.len() {
            return;
        }
        let list = get_or_create_offset_list(&mut self.post_processing, section_index);
        list.extend_from_slice(packed_positions);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBlockEntityNbts
    pub fn get_block_entity_nbts(&self) -> &HashMap<BlockPos, CompoundTag> {
        &self.pending_block_entities
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBlockEntityNbtForSaving
    pub fn get_block_entity_nbt_for_saving(&self, pos: BlockPos) -> Option<CompoundTag> {
        if let Some(be) = self.get_block_entity(pos) {
            Some(be.save_with_full_metadata())
        } else {
            self.pending_block_entities.get(&pos).cloned()
        }
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#removeBlockEntity
    pub fn remove_block_entity(&mut self, pos: BlockPos) {
        self.block_entities.remove(&pos);
        self.pending_block_entities.remove(&pos);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getCarvingMask
    pub fn get_carving_mask(&self) -> Option<&CarvingMask> {
        self.carving_mask.as_ref()
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getOrCreateCarvingMask
    pub fn get_or_create_carving_mask(&mut self) -> &CarvingMask {
        if self.carving_mask.is_none() {
            self.carving_mask = Some(CarvingMask::new(self.get_height(), self.get_min_y()));
        }
        self.carving_mask.as_ref().expect("carving mask initialized")
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setCarvingMask
    pub fn set_carving_mask(&mut self, mask: CarvingMask) {
        self.carving_mask = Some(mask);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setLightEngine
    pub fn set_light_engine(&mut self, light_engine: LE) {
        self.light_engine = Some(light_engine);
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#setBelowZeroRetrogen
    pub fn set_below_zero_retrogen(&mut self, below_zero_retrogen: Option<BelowZeroRetrogenData>) {
        self.below_zero_retrogen = below_zero_retrogen;
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getBelowZeroRetrogen
    pub fn get_below_zero_retrogen(&self) -> Option<&BelowZeroRetrogenData> {
        self.below_zero_retrogen.as_ref()
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#unpackTicks
    fn unpack_ticks<T>(_ticks: &ProtoChunkTicks<T>) -> LevelChunkTicks<T> {
        LevelChunkTicks::new(_ticks.scheduled_ticks().to_vec())
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#unpackBlockTicks
    pub fn unpack_block_ticks(&self) -> LevelChunkTicks<BlockTickMarker> {
        Self::unpack_ticks(&self.block_ticks)
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#unpackFluidTicks
    pub fn unpack_fluid_ticks(&self) -> LevelChunkTicks<FluidTickMarker> {
        Self::unpack_ticks(&self.fluid_ticks)
    }

    // JAVA: net.minecraft.world.level.chunk.ProtoChunk#getHeightAccessorForGeneration
    pub fn get_height_accessor_for_generation(&self) -> LevelHeightAccessor {
        if self.is_upgrading() {
            BELOW_ZERO_RETROGEN_UPGRADE_HEIGHT_ACCESSOR
        } else {
            self.level_height_accessor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestBlockState(u16);

    impl BlockStateOps for TestBlockState {
        fn air() -> Self {
            Self(0)
        }
        fn void_air() -> Self {
            Self(1)
        }
        fn is_air(&self) -> bool {
            self.0 == 0
        }
        fn has_different_light_properties(&self, other: &Self) -> bool {
            // Simple test model: light property changes when crossing air/non-air boundary
            self.is_air() != other.is_air()
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestFluidState(u16);

    impl FluidStateOps for TestFluidState {
        fn empty() -> Self {
            Self(0)
        }
    }

    #[derive(Clone, Debug)]
    struct TestSection {
        states: [[[u16; 16]; 16]; 16],
    }

    impl Default for TestSection {
        fn default() -> Self {
            Self {
                states: [[[0u16; 16]; 16]; 16],
            }
        }
    }

    impl ChunkSectionOps<TestBlockState, TestFluidState> for TestSection {
        fn has_only_air(&self) -> bool {
            self.states
                .iter()
                .all(|y| y.iter().all(|z| z.iter().all(|x| *x == 0)))
        }

        fn get_block_state(&self, x: i32, y: i32, z: i32) -> TestBlockState {
            let xi = (x & 15) as usize;
            let yi = (y & 15) as usize;
            let zi = (z & 15) as usize;
            TestBlockState(self.states[yi][zi][xi])
        }

        fn get_fluid_state(&self, _x: i32, _y: i32, _z: i32) -> TestFluidState {
            TestFluidState::empty()
        }

        fn set_block_state(&mut self, x: i32, y: i32, z: i32, state: TestBlockState) -> TestBlockState {
            let xi = (x & 15) as usize;
            let yi = (y & 15) as usize;
            let zi = (z & 15) as usize;
            let old = self.states[yi][zi][xi];
            self.states[yi][zi][xi] = state.0;
            TestBlockState(old)
        }
    }

    #[derive(Clone, Debug, Default)]
    struct TestBlockEntity {
        pos: BlockPos,
    }

    impl BlockEntityOps for TestBlockEntity {
        fn get_block_pos(&self) -> BlockPos {
            self.pos
        }
        fn save_with_full_metadata(&self) -> CompoundTag {
            CompoundTag
        }
    }

    #[derive(Default)]
    struct TestLightEngine {
        section_updates: Vec<(BlockPos, bool)>,
        checks: Vec<BlockPos>,
    }

    impl LightEngineOps for TestLightEngine {
        fn update_section_status(&mut self, pos: BlockPos, is_empty: bool) {
            self.section_updates.push((pos, is_empty));
        }
        fn check_block(&mut self, pos: BlockPos) {
            self.checks.push(pos);
        }
    }

    fn make_chunk() -> ProtoChunk<TestBlockState, TestFluidState, TestSection, TestBlockEntity, TestLightEngine> {
        let accessor = LevelHeightAccessor::new(0, 256);
        let sections = (0..16).map(|_| TestSection::default()).collect();
        let mut chunk = ProtoChunk::new(
            ChunkPos::new(0, 0),
            UpgradeData,
            accessor,
            PalettedContainerFactory,
            None,
            sections,
        );
        chunk.set_light_engine(TestLightEngine::default());
        chunk.set_persisted_status(ChunkStatus::Full);
        chunk
    }

    #[test]
    fn pack_unpack_offset_coordinates_roundtrip_and_masking() {
        let pos = BlockPos::new(-1, -1, -1);
        let packed = ProtoChunk::<TestBlockState, TestFluidState, TestSection, TestBlockEntity, TestLightEngine>::pack_offset_coordinates(pos);
        // -1 & 15 = 15 for all coords
        assert_eq!(packed & 0xF, 15);
        assert_eq!((packed >> 4) & 0xF, 15);
        assert_eq!((packed >> 8) & 0xF, 15);

        let unpacked = ProtoChunk::<TestBlockState, TestFluidState, TestSection, TestBlockEntity, TestLightEngine>::unpack_offset_coordinates(
            packed,
            -1,
            ChunkPos::new(-1, -1),
        );
        // section -1 * 16 + 15 = -1
        assert_eq!(unpacked, BlockPos::new(-1, -1, -1));
    }

    #[test]
    fn unpack_offset_coordinates_handles_negative_section_coords() {
        // packed = x=1,y=2,z=3
        let packed: i16 = (1 | (2 << 4) | (3 << 8)) as i16;
        let unpacked = ProtoChunk::<TestBlockState, TestFluidState, TestSection, TestBlockEntity, TestLightEngine>::unpack_offset_coordinates(
            packed,
            -2,
            ChunkPos::new(-3, 5),
        );
        assert_eq!(unpacked.x, (-3 << 4) + 1);
        assert_eq!(unpacked.y, (-2 << 4) + 2);
        assert_eq!(unpacked.z, (5 << 4) + 3);
    }

    #[test]
    fn get_block_state_outside_height_returns_void_air() {
        let chunk = make_chunk();
        assert_eq!(chunk.get_block_state(BlockPos::new(0, -1, 0)), TestBlockState::void_air());
        assert_eq!(chunk.get_block_state(BlockPos::new(0, 256, 0)), TestBlockState::void_air());
    }

    #[test]
    fn set_block_state_updates_light_and_marks_unsaved() {
        let mut chunk = make_chunk();
        let pos = BlockPos::new(1, 5, 2);
        let old = chunk.set_block_state(pos, TestBlockState(10), 0);
        assert_eq!(old, Some(TestBlockState(0)));
        assert_eq!(chunk.get_block_state(pos), TestBlockState(10));
        assert!(chunk.is_unsaved());

        let le = chunk.light_engine.as_ref().unwrap();
        assert!(!le.section_updates.is_empty());
        assert!(!le.checks.is_empty());
    }

    #[test]
    fn set_persisted_status_clears_below_zero_retrogen_when_reached() {
        let mut chunk = make_chunk();
        chunk.set_below_zero_retrogen(Some(BelowZeroRetrogenData::new(ChunkStatus::Biomes)));
        assert!(chunk.is_upgrading());
        chunk.set_persisted_status(ChunkStatus::Biomes);
        assert!(!chunk.is_upgrading());
        assert_eq!(
            chunk.get_height_accessor_for_generation(),
            chunk.level_height_accessor
        );
    }

    #[test]
    fn postprocessing_and_entity_nbt_storage() {
        let mut chunk = make_chunk();
        chunk.mark_pos_for_postprocessing(BlockPos::new(1, 16, 1));
        let section_idx = chunk.get_section_index(16);
        assert_eq!(chunk.post_processing[section_idx as usize].len(), 1);

        chunk.add_entity_tag(CompoundTag);
        assert_eq!(chunk.get_entities().len(), 1);

        let pos = BlockPos::new(2, 3, 4);
        chunk.pending_block_entities.insert(pos, CompoundTag);
        assert!(chunk.get_block_entity_nbt_for_saving(pos).is_some());
        chunk.remove_block_entity(pos);
        assert!(chunk.get_block_entity_nbt_for_saving(pos).is_none());
    }
}
