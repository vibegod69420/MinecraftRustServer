use std::future;

pub const AIR: BlockStateId = BlockStateId(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockStateId(pub i32);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    OceanFloorWg,
    WorldSurfaceWg,
}

pub trait HeightmapKind {
    fn is_opaque(&self, state: BlockStateId) -> bool;
}

impl HeightmapKind for HeightmapType {
    fn is_opaque(&self, state: BlockStateId) -> bool {
        state != AIR
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoiseColumn {
    pub min_y: i32,
    pub states: Vec<BlockStateId>,
}

pub trait LevelHeightAccessor {
    fn get_min_y(&self) -> i32;
    fn get_height(&self) -> i32;
    fn get_max_y(&self) -> i32;
}

pub trait ChunkAccess: LevelHeightAccessor {
    fn set_block_state(&mut self, pos: BlockPos, state: BlockStateId);
    fn update_heightmap(
        &mut self,
        kind: HeightmapType,
        x: i32,
        y: i32,
        z: i32,
        state: BlockStateId,
    );
}

#[derive(Debug, Clone, Default)]
pub struct Blender;

#[derive(Debug, Clone, Default)]
pub struct RandomState;

#[derive(Debug, Clone, Default)]
pub struct StructureManager;

#[derive(Debug, Clone, Default)]
pub struct WorldGenRegion;

#[derive(Debug, Clone, Default)]
pub struct BiomeManager;

#[derive(Debug, Clone, Default)]
pub struct BiomeSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureSetHolder(pub u64);

pub trait StructureSetLookup {
    fn list_elements(&self) -> Vec<StructureSetHolder>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkGeneratorStructureState {
    pub structure_sets: Vec<StructureSetHolder>,
}

impl ChunkGeneratorStructureState {
    pub fn create_for_flat(
        _random_state: &RandomState,
        _seed: i64,
        _biome_source: &BiomeSource,
        structure_sets: Vec<StructureSetHolder>,
    ) -> Self {
        Self { structure_sets }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MapCodecPlaceholder;

pub const CODEC: MapCodecPlaceholder = MapCodecPlaceholder;

#[derive(Debug, Clone, Default)]
pub struct FlatLevelGeneratorSettings {
    layers: Vec<Option<BlockStateId>>,
    structure_overrides: Option<Vec<StructureSetHolder>>,
}

impl FlatLevelGeneratorSettings {
    pub fn new(layers: Vec<Option<BlockStateId>>) -> Self {
        Self {
            layers,
            structure_overrides: None,
        }
    }

    pub fn with_structure_overrides(mut self, overrides: Vec<StructureSetHolder>) -> Self {
        self.structure_overrides = Some(overrides);
        self
    }

    pub fn get_layers(&self) -> &[Option<BlockStateId>] {
        &self.layers
    }

    pub fn structure_overrides(&self) -> Option<&[StructureSetHolder]> {
        self.structure_overrides.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct FlatLevelSource {
    settings: FlatLevelGeneratorSettings,
    biome_source: BiomeSource,
}

impl FlatLevelSource {
    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#<init>
    pub fn new(settings: FlatLevelGeneratorSettings) -> Self {
        // Java: super(new FixedBiomeSource(settings.getBiome()), Util.memoize(settings::adjustGenerationSettings));
        Self {
            settings,
            biome_source: BiomeSource,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#createState
    pub fn create_state(
        &self,
        structure_set_lookup: &dyn StructureSetLookup,
        random_state: &RandomState,
        seed: i64,
    ) -> ChunkGeneratorStructureState {
        let structure_sets = match self.settings.structure_overrides() {
            Some(overrides) => overrides.to_vec(),
            None => structure_set_lookup.list_elements(),
        };
        ChunkGeneratorStructureState::create_for_flat(
            random_state,
            seed,
            &self.biome_source,
            structure_sets,
        )
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#codec
    pub fn codec(&self) -> MapCodecPlaceholder {
        CODEC
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#settings
    pub fn settings(&self) -> &FlatLevelGeneratorSettings {
        &self.settings
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#buildSurface
    pub fn build_surface(
        &self,
        _region: &mut WorldGenRegion,
        _structure_manager: &StructureManager,
        _random_state: &RandomState,
        _chunk: &mut dyn ChunkAccess,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#getSpawnHeight
    pub fn get_spawn_height(&self, level: &dyn LevelHeightAccessor) -> i32 {
        let layer_count = self.settings.get_layers().len() as i32;
        let min_h = level.get_height().min(layer_count);
        level.get_min_y().wrapping_add(min_h)
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#fillFromNoise
    pub fn fill_from_noise<C: ChunkAccess>(
        &self,
        _blender: &Blender,
        _random_state: &RandomState,
        _structure_manager: &StructureManager,
        mut chunk: C,
    ) -> future::Ready<C> {
        let layers = self.settings.get_layers();
        let limit = chunk.get_height().min(layers.len() as i32);
        let min_y = chunk.get_min_y();

        let mut i = 0i32;
        while i < limit {
            let state = layers[i as usize];
            if let Some(block_state) = state {
                let y = min_y.wrapping_add(i);
                let mut x = 0i32;
                while x < 16 {
                    let mut z = 0i32;
                    while z < 16 {
                        chunk.set_block_state(BlockPos::new(x, y, z), block_state);
                        chunk.update_heightmap(HeightmapType::OceanFloorWg, x, y, z, block_state);
                        chunk.update_heightmap(HeightmapType::WorldSurfaceWg, x, y, z, block_state);
                        z = z.wrapping_add(1);
                    }
                    x = x.wrapping_add(1);
                }
            }
            i = i.wrapping_add(1);
        }

        future::ready(chunk)
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#getBaseHeight
    pub fn get_base_height<H: HeightmapKind + ?Sized>(
        &self,
        _x: i32,
        _z: i32,
        heightmap_type: &H,
        level: &dyn LevelHeightAccessor,
        _random_state: &RandomState,
    ) -> i32 {
        let layers = self.settings.get_layers();
        let top = (layers.len() as i32)
            .wrapping_sub(1)
            .min(level.get_max_y());
        let mut i = top;
        while i >= 0 {
            if let Some(state) = layers[i as usize] {
                if heightmap_type.is_opaque(state) {
                    return level.get_min_y().wrapping_add(i).wrapping_add(1);
                }
            }
            i = i.wrapping_sub(1);
        }
        level.get_min_y()
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#getBaseColumn
    pub fn get_base_column(
        &self,
        _x: i32,
        _z: i32,
        level: &dyn LevelHeightAccessor,
        _random_state: &RandomState,
    ) -> NoiseColumn {
        let height = level.get_height().max(0) as usize;
        let states = self
            .settings
            .get_layers()
            .iter()
            .take(height)
            .map(|s| s.unwrap_or(AIR))
            .collect();
        NoiseColumn {
            min_y: level.get_min_y(),
            states,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#addDebugScreenInfo
    pub fn add_debug_screen_info(
        &self,
        _info: &mut Vec<String>,
        _random_state: &RandomState,
        _pos: BlockPos,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#applyCarvers
    pub fn apply_carvers(
        &self,
        _region: &mut WorldGenRegion,
        _seed: i64,
        _random_state: &RandomState,
        _biome_manager: &BiomeManager,
        _structure_manager: &StructureManager,
        _chunk: &mut dyn ChunkAccess,
    ) {
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#spawnOriginalMobs
    pub fn spawn_original_mobs(&self, _region: &mut WorldGenRegion) {}

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#getMinY
    pub fn get_min_y(&self) -> i32 {
        0
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#getGenDepth
    pub fn get_gen_depth(&self) -> i32 {
        384
    }

    // JAVA: net.minecraft.world.level.levelgen.FlatLevelSource#getSeaLevel
    pub fn get_sea_level(&self) -> i32 {
        -63
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::task::{Context, Poll, Wake, Waker};

    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: std::sync::Arc<Self>) {}
    }

    fn block_on_ready<T>(mut fut: future::Ready<T>) -> T {
        let waker = Waker::from(std::sync::Arc::new(NoopWake));
        let mut cx = Context::from_waker(&waker);
        let mut fut = std::pin::Pin::new(&mut fut);
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => v,
            Poll::Pending => panic!("Ready future pending"),
        }
    }

    #[derive(Clone)]
    struct MockLevel {
        min_y: i32,
        height: i32,
    }

    impl LevelHeightAccessor for MockLevel {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }
        fn get_height(&self) -> i32 {
            self.height
        }
        fn get_max_y(&self) -> i32 {
            self.min_y.wrapping_add(self.height).wrapping_sub(1)
        }
    }

    struct MockChunk {
        min_y: i32,
        height: i32,
        blocks: BTreeMap<(i32, i32, i32), BlockStateId>,
        heightmap_updates: Vec<(HeightmapType, i32, i32, i32, BlockStateId)>,
    }

    impl MockChunk {
        fn new(min_y: i32, height: i32) -> Self {
            Self {
                min_y,
                height,
                blocks: BTreeMap::new(),
                heightmap_updates: Vec::new(),
            }
        }
    }

    impl LevelHeightAccessor for MockChunk {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }
        fn get_height(&self) -> i32 {
            self.height
        }
        fn get_max_y(&self) -> i32 {
            self.min_y.wrapping_add(self.height).wrapping_sub(1)
        }
    }

    impl ChunkAccess for MockChunk {
        fn set_block_state(&mut self, pos: BlockPos, state: BlockStateId) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
        fn update_heightmap(
            &mut self,
            kind: HeightmapType,
            x: i32,
            y: i32,
            z: i32,
            state: BlockStateId,
        ) {
            self.heightmap_updates.push((kind, x, y, z, state));
        }
    }

    struct OpaqueNonAir;
    impl HeightmapKind for OpaqueNonAir {
        fn is_opaque(&self, state: BlockStateId) -> bool {
            state != AIR
        }
    }

    struct Lookup(Vec<StructureSetHolder>);
    impl StructureSetLookup for Lookup {
        fn list_elements(&self) -> Vec<StructureSetHolder> {
            self.0.clone()
        }
    }

    #[test]
    fn test_constants_spawn_height_and_float_bits() {
        let src = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![
            Some(BlockStateId(1)),
            Some(BlockStateId(2)),
            None,
        ]));
        assert_eq!(src.get_min_y(), 0);
        assert_eq!(src.get_gen_depth(), 384);
        assert_eq!(src.get_sea_level(), -63);

        let level = MockLevel {
            min_y: -64,
            height: 384,
        };
        assert_eq!(src.get_spawn_height(&level), -61);
        let small = MockLevel {
            min_y: 0,
            height: 2,
        };
        assert_eq!(src.get_spawn_height(&small), 2);

        // Float bits check: exact IEEE-754 bit patterns.
        assert_eq!((-63.0f32).to_bits(), 0xC27C_0000);
        assert_eq!(384.0f64.to_bits(), 0x4078_0000_0000_0000u64);
    }

    #[test]
    fn test_fill_from_noise_layers_nulls_and_heightmaps() {
        let src = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![
            Some(BlockStateId(1)),
            None,
            Some(BlockStateId(3)),
        ]));
        let chunk = MockChunk::new(0, 5);
        let chunk = block_on_ready(src.fill_from_noise(
            &Blender,
            &RandomState,
            &StructureManager,
            chunk,
        ));

        assert_eq!(chunk.blocks.get(&(0, 0, 0)), Some(&BlockStateId(1)));
        assert_eq!(chunk.blocks.get(&(15, 0, 15)), Some(&BlockStateId(1)));
        assert!(!chunk.blocks.contains_key(&(0, 1, 0)));
        assert_eq!(chunk.blocks.get(&(7, 2, 9)), Some(&BlockStateId(3)));
        assert!(!chunk.blocks.contains_key(&(0, 3, 0)));

        // 2 placed layers * 16 * 16 * 2 heightmap updates
        assert_eq!(chunk.heightmap_updates.len(), 2 * 16 * 16 * 2);
        assert!(chunk
            .heightmap_updates
            .iter()
            .any(|u| *u == (HeightmapType::OceanFloorWg, 0, 0, 0, BlockStateId(1))));
        assert!(chunk
            .heightmap_updates
            .iter()
            .any(|u| *u == (HeightmapType::WorldSurfaceWg, 0, 2, 0, BlockStateId(3))));
    }

    #[test]
    fn test_get_base_height_opaque_negative_and_empty() {
        let src = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![
            Some(BlockStateId(1)),
            Some(AIR),
            Some(BlockStateId(2)),
        ]));
        let level = MockLevel {
            min_y: -64,
            height: 384,
        };
        let h = src.get_base_height(0, 0, &OpaqueNonAir, &level, &RandomState);
        assert_eq!(h, -61); // -64 + 2 + 1

        let empty = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![]));
        assert_eq!(
            empty.get_base_height(0, 0, &OpaqueNonAir, &level, &RandomState),
            -64
        );

        let all_air = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![Some(AIR), None]));
        assert_eq!(
            all_air.get_base_height(0, 0, &OpaqueNonAir, &level, &RandomState),
            -64
        );
    }

    #[test]
    fn test_get_base_column_limit_and_null_to_air() {
        let src = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![
            Some(BlockStateId(5)),
            None,
            Some(BlockStateId(7)),
            Some(BlockStateId(9)),
        ]));
        let level = MockLevel {
            min_y: -2,
            height: 3,
        };
        let col = src.get_base_column(10, 20, &level, &RandomState);
        assert_eq!(col.min_y, -2);
        assert_eq!(col.states, vec![BlockStateId(5), AIR, BlockStateId(7)]);

        let zero_h = MockLevel {
            min_y: 5,
            height: 0,
        };
        let col2 = src.get_base_column(0, 0, &zero_h, &RandomState);
        assert_eq!(col2.min_y, 5);
        assert!(col2.states.is_empty());
    }

    #[test]
    fn test_negative_min_y_and_wrapping_overflow() {
        let src = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![Some(BlockStateId(1))]));
        let level_min = MockLevel {
            min_y: i32::MIN,
            height: 1,
        };
        assert_eq!(
            src.get_spawn_height(&level_min),
            i32::MIN.wrapping_add(1)
        );

        let level_max = MockLevel {
            min_y: i32::MAX,
            height: 2,
        };
        assert_eq!(
            src.get_spawn_height(&level_max),
            i32::MAX.wrapping_add(1)
        );

        let src2 = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![
            Some(BlockStateId(42)),
        ]));
        let chunk = MockChunk::new(-16, 16);
        let chunk = block_on_ready(src2.fill_from_noise(
            &Blender,
            &RandomState,
            &StructureManager,
            chunk,
        ));
        assert_eq!(chunk.blocks.get(&(0, -16, 0)), Some(&BlockStateId(42)));
        assert_eq!(chunk.blocks.get(&(15, -16, 15)), Some(&BlockStateId(42)));
    }

    #[test]
    fn test_create_state_overrides_and_lookup() {
        let lookup = Lookup(vec![StructureSetHolder(1), StructureSetHolder(2)]);
        let settings = FlatLevelGeneratorSettings::new(vec![])
            .with_structure_overrides(vec![StructureSetHolder(99)]);
        let src = FlatLevelSource::new(settings);
        let state = src.create_state(&lookup, &RandomState, 123);
        assert_eq!(state.structure_sets, vec![StructureSetHolder(99)]);

        let src2 = FlatLevelSource::new(FlatLevelGeneratorSettings::new(vec![]));
        let state2 = src2.create_state(&lookup, &RandomState, 456);
        assert_eq!(
            state2.structure_sets,
            vec![StructureSetHolder(1), StructureSetHolder(2)]
        );
    }
}
