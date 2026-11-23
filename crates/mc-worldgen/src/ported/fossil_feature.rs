//! 1:1 faithful port of `net.minecraft.world.level.levelgen.feature.FossilFeature` (Java 1.21.11).
//!
//! External Minecraft types are modeled as local traits/structs so this module is
//! self-contained and compilable without unported dependencies.

use std::fmt;

// ---------------------------------------------------------------------------
// Minimal local data types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#offset(int,int,int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#atY(int)
    pub fn at_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y,
            z: self.z,
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vec3i {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    None,
    Clockwise90,
    Rotation180,
    Counterclockwise90,
}

impl Rotation {
    // JAVA: net.minecraft.world.level.block.Rotation#getRandom(net.minecraft.util.RandomSource)
    pub fn get_random<R: RandomSource + ?Sized>(random: &mut R) -> Self {
        match random.next_int(4) {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Rotation180,
            3 => Rotation::Counterclockwise90,
            _ => Rotation::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

impl BoundingBox {
    // JAVA: net.minecraft.world.level.levelgen.structure.BoundingBox#forAllCorners
    pub fn for_all_corners<F: FnMut(BlockPos)>(&self, mut f: F) {
        // Java BoundingBox.forAllCorners visits the 8 corners.
        let xs = [self.min_x, self.max_x];
        let ys = [self.min_y, self.max_y];
        let zs = [self.min_z, self.max_z];
        for &x in &xs {
            for &y in &ys {
                for &z in &zs {
                    f(BlockPos { x, y, z });
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    // JAVA: net.minecraft.world.level.ChunkPos#<init>(BlockPos)
    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self {
            x: pos.x >> 4,
            z: pos.z >> 4,
        }
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockX
    pub fn get_min_block_x(&self) -> i32 {
        self.x << 4
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMinBlockZ
    pub fn get_min_block_z(&self) -> i32 {
        self.z << 4
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMaxBlockX
    pub fn get_max_block_x(&self) -> i32 {
        self.get_min_block_x().wrapping_add(15)
    }

    // JAVA: net.minecraft.world.level.ChunkPos#getMaxBlockZ
    pub fn get_max_block_z(&self) -> i32 {
        self.get_min_block_z().wrapping_add(15)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    OceanFloorWg,
}

// ---------------------------------------------------------------------------
// Traits for external Minecraft dependencies
// ---------------------------------------------------------------------------

pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextInt(int)
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait BlockStateAccess {
    // JAVA: net.minecraft.world.level.block.state.BlockState#isAir()
    fn is_air(&self) -> bool;
    // JAVA: net.minecraft.world.level.block.state.BlockBehaviour.BlockStateBase#is(Block)
    fn is_lava(&self) -> bool;
    // JAVA: net.minecraft.world.level.block.state.BlockBehaviour.BlockStateBase#is(Block)
    fn is_water(&self) -> bool;
}

pub trait WorldGenLevelAccess {
    type BlockState: BlockStateAccess;

    // JAVA: net.minecraft.world.level.LevelHeightAccessor#getMinY()
    fn get_min_y(&self) -> i32;
    // JAVA: net.minecraft.world.level.LevelHeightAccessor#getMaxY()
    fn get_max_y(&self) -> i32;
    // JAVA: net.minecraft.world.level.LevelReader#getHeight
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    // JAVA: net.minecraft.world.level.BlockGetter#getBlockState
    fn get_block_state(&self, pos: &BlockPos) -> Self::BlockState;
}

#[derive(Clone, Debug, Default)]
pub struct StructurePlaceSettingsData {
    pub rotation: Rotation,
    pub bounding_box: Option<BoundingBox>,
    pub processors: Vec<u32>,
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::None
    }
}

impl StructurePlaceSettingsData {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#<init>
    pub fn new() -> Self {
        Self::default()
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#setRotation
    pub fn set_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#setBoundingBox
    pub fn set_bounding_box(mut self, bounding_box: BoundingBox) -> Self {
        self.bounding_box = Some(bounding_box);
        self
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#clearProcessors
    pub fn clear_processors(&mut self) {
        self.processors.clear();
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#addProcessor
    pub fn add_processor(&mut self, processor: u32) {
        self.processors.push(processor);
    }
}

pub trait StructureTemplateAccess<L: WorldGenLevelAccess + ?Sized, R: RandomSource + ?Sized> {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#getSize
    fn get_size(&self, rotation: Rotation) -> Vec3i;
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#getZeroPositionWithTransform
    fn get_zero_position_with_transform(
        &self,
        pos: BlockPos,
        mirror: Mirror,
        rotation: Rotation,
    ) -> BlockPos;
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#getBoundingBox(StructurePlaceSettings,BlockPos)
    fn get_bounding_box(&self, settings: &StructurePlaceSettingsData, pos: BlockPos) -> BoundingBox;
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld
    fn place_in_world(
        &self,
        level: &mut L,
        pos: BlockPos,
        pos2: BlockPos,
        settings: &StructurePlaceSettingsData,
        random: &mut R,
        flags: i32,
    );
}

pub trait StructureTemplateManager<L: WorldGenLevelAccess + ?Sized, R: RandomSource + ?Sized> {
    type Template: StructureTemplateAccess<L, R>;
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager#getOrCreate
    fn get_or_create(&self, id: &str) -> Self::Template;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct FossilFeatureConfiguration {
    pub fossil_structures: Vec<String>,
    pub overlay_structures: Vec<String>,
    pub fossil_processors: Vec<u32>,
    pub overlay_processors: Vec<u32>,
    pub max_empty_corners_allowed: i32,
}

// ---------------------------------------------------------------------------
// Feature implementation
// ---------------------------------------------------------------------------

pub struct FossilFeature;

impl FossilFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.FossilFeature#place(FeaturePlaceContext)
    pub fn place<L, R, M>(
        level: &mut L,
        origin: BlockPos,
        random: &mut R,
        config: &FossilFeatureConfiguration,
        manager: &M,
    ) -> bool
    where
        L: WorldGenLevelAccess,
        R: RandomSource,
        M: StructureTemplateManager<L, R>,
    {
        let rotation = Rotation::get_random(random);
        let fossil_count = config.fossil_structures.len() as i32;
        let idx = random.next_int(fossil_count) as usize;
        let fossil_id = &config.fossil_structures[idx];
        let overlay_id = &config.overlay_structures[idx];
        let fossil_template = manager.get_or_create(fossil_id);
        let overlay_template = manager.get_or_create(overlay_id);

        let chunk_pos = ChunkPos::from_block_pos(origin);
        let bounding_box = BoundingBox {
            min_x: chunk_pos.get_min_block_x().wrapping_sub(16),
            min_y: level.get_min_y(),
            min_z: chunk_pos.get_min_block_z().wrapping_sub(16),
            max_x: chunk_pos.get_max_block_x().wrapping_add(16),
            max_y: level.get_max_y(),
            max_z: chunk_pos.get_max_block_z().wrapping_add(16),
        };

        let mut settings = StructurePlaceSettingsData::new()
            .set_rotation(rotation)
            .set_bounding_box(bounding_box);

        let size = fossil_template.get_size(rotation);
        let anchor = origin.offset(-(size.get_x() / 2), 0, -(size.get_z() / 2));

        let mut min_height = origin.get_y();
        let mut dx = 0i32;
        while dx < size.get_x() {
            let mut dz = 0i32;
            while dz < size.get_z() {
                let h = level.get_height(
                    HeightmapType::OceanFloorWg,
                    anchor.get_x().wrapping_add(dx),
                    anchor.get_z().wrapping_add(dz),
                );
                min_height = min_height.min(h);
                dz = dz.wrapping_add(1);
            }
            dx = dx.wrapping_add(1);
        }

        let random_offset = random.next_int(10);
        let base_y = min_height.wrapping_sub(15).wrapping_sub(random_offset);
        let min_allowed_y = level.get_min_y().wrapping_add(10);
        let place_y = base_y.max(min_allowed_y);
        let zero_pos = fossil_template.get_zero_position_with_transform(
            anchor.at_y(place_y),
            Mirror::None,
            rotation,
        );

        let template_bb = fossil_template.get_bounding_box(&settings, zero_pos);
        if count_empty_corners(level, &template_bb) > config.max_empty_corners_allowed {
            return false;
        }

        settings.clear_processors();
        for &p in &config.fossil_processors {
            settings.add_processor(p);
        }
        fossil_template.place_in_world(level, zero_pos, zero_pos, &settings, random, 260);

        settings.clear_processors();
        for &p in &config.overlay_processors {
            settings.add_processor(p);
        }
        overlay_template.place_in_world(level, zero_pos, zero_pos, &settings, random, 260);

        true
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.FossilFeature#countEmptyCorners
fn count_empty_corners<L: WorldGenLevelAccess + ?Sized>(level: &L, bb: &BoundingBox) -> i32 {
    let mut count: i32 = 0;
    bb.for_all_corners(|pos| {
        let state = level.get_block_state(&pos);
        if state.is_air() || state.is_lava() || state.is_water() {
            count = count.wrapping_add(1);
        }
    });
    count
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Rotation::None => "none",
            Rotation::Clockwise90 => "clockwise_90",
            Rotation::Rotation180 => "180",
            Rotation::Counterclockwise90 => "counterclockwise_90",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    struct MockBlockState {
        air: bool,
        lava: bool,
        water: bool,
    }

    impl BlockStateAccess for MockBlockState {
        fn is_air(&self) -> bool {
            self.air
        }
        fn is_lava(&self) -> bool {
            self.lava
        }
        fn is_water(&self) -> bool {
            self.water
        }
    }

    struct MockRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl MockRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self { ints, idx: 0 }
        }
    }

    impl RandomSource for MockRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = if self.idx < self.ints.len() {
                self.ints[self.idx]
            } else {
                0
            };
            self.idx += 1;
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }
    }

    struct MockLevel {
        min_y: i32,
        max_y: i32,
        heights: HashMap<(i32, i32), i32>,
        states: HashMap<(i32, i32, i32), MockBlockState>,
    }

    impl MockLevel {
        fn new(min_y: i32, max_y: i32) -> Self {
            Self {
                min_y,
                max_y,
                heights: HashMap::new(),
                states: HashMap::new(),
            }
        }
    }

    impl WorldGenLevelAccess for MockLevel {
        type BlockState = MockBlockState;

        fn get_min_y(&self) -> i32 {
            self.min_y
        }
        fn get_max_y(&self) -> i32 {
            self.max_y
        }
        fn get_height(&self, _heightmap: HeightmapType, x: i32, z: i32) -> i32 {
            *self.heights.get(&(x, z)).unwrap_or(&64)
        }
        fn get_block_state(&self, pos: &BlockPos) -> Self::BlockState {
            self.states
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or(MockBlockState {
                    air: false,
                    lava: false,
                    water: false,
                })
        }
    }

    #[derive(Clone)]
    struct MockTemplate {
        size: Vec3i,
        bb: BoundingBox,
        zero_offset: BlockPos,
        placed: std::rc::Rc<std::cell::RefCell<Vec<(BlockPos, i32, Vec<u32>)>>>,
    }

    impl StructureTemplateAccess<MockLevel, MockRandom> for MockTemplate {
        fn get_size(&self, _rotation: Rotation) -> Vec3i {
            self.size
        }
        fn get_zero_position_with_transform(
            &self,
            pos: BlockPos,
            _mirror: Mirror,
            _rotation: Rotation,
        ) -> BlockPos {
            BlockPos {
                x: pos.x.wrapping_add(self.zero_offset.x),
                y: pos.y.wrapping_add(self.zero_offset.y),
                z: pos.z.wrapping_add(self.zero_offset.z),
            }
        }
        fn get_bounding_box(
            &self,
            _settings: &StructurePlaceSettingsData,
            _pos: BlockPos,
        ) -> BoundingBox {
            self.bb
        }
        fn place_in_world(
            &self,
            _level: &mut MockLevel,
            pos: BlockPos,
            _pos2: BlockPos,
            settings: &StructurePlaceSettingsData,
            _random: &mut MockRandom,
            flags: i32,
        ) {
            self.placed
                .borrow_mut()
                .push((pos, flags, settings.processors.clone()));
        }
    }

    struct MockManager {
        template: MockTemplate,
    }

    impl StructureTemplateManager<MockLevel, MockRandom> for MockManager {
        type Template = MockTemplate;
        fn get_or_create(&self, _id: &str) -> Self::Template {
            self.template.clone()
        }
    }

    fn make_template(bb: BoundingBox) -> MockTemplate {
        MockTemplate {
            size: Vec3i { x: 4, y: 4, z: 4 },
            bb,
            zero_offset: BlockPos { x: 0, y: 0, z: 0 },
            placed: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
        }
    }

    fn base_config() -> FossilFeatureConfiguration {
        FossilFeatureConfiguration {
            fossil_structures: vec!["fossil1".to_string()],
            overlay_structures: vec!["overlay1".to_string()],
            fossil_processors: vec![1, 2],
            overlay_processors: vec![3],
            max_empty_corners_allowed: 4,
        }
    }

    #[test]
    fn test_place_succeeds_and_places_both_templates() {
        let mut level = MockLevel::new(-64, 320);
        let mut random = MockRandom::new(vec![0, 0, 5]); // rotation=None, idx=0, offset=5
        let config = base_config();
        let template = make_template(BoundingBox {
            min_x: 0, min_y: 0, min_z: 0, max_x: 3, max_y: 3, max_z: 3,
        });
        let placed_ref = template.placed.clone();
        let manager = MockManager { template };

        let ok = FossilFeature::place(
            &mut level,
            BlockPos { x: 16, y: 64, z: 16 },
            &mut random,
            &config,
            &manager,
        );
        assert!(ok);
        let placed = placed_ref.borrow();
        assert_eq!(placed.len(), 2);
        assert_eq!(placed[0].1, 260);
        assert_eq!(placed[1].1, 260);
        assert_eq!(placed[0].2, vec![1, 2]);
        assert_eq!(placed[1].2, vec![3]);
    }

    #[test]
    fn test_place_fails_when_too_many_empty_corners() {
        let mut level = MockLevel::new(-64, 320);
        // Make all 8 corners air
        for &x in &[0, 3] {
            for &y in &[0, 3] {
                for &z in &[0, 3] {
                    level.states.insert((x, y, z), MockBlockState { air: true, lava: false, water: false });
                }
            }
        }
        let mut random = MockRandom::new(vec![0, 0, 0]);
        let mut config = base_config();
        config.max_empty_corners_allowed = 2;
        let template = make_template(BoundingBox {
            min_x: 0, min_y: 0, min_z: 0, max_x: 3, max_y: 3, max_z: 3,
        });
        let placed_ref = template.placed.clone();
        let manager = MockManager { template };

        let ok = FossilFeature::place(
            &mut level,
            BlockPos { x: 16, y: 64, z: 16 },
            &mut random,
            &config,
            &manager,
        );
        assert!(!ok);
        assert!(placed_ref.borrow().is_empty());
    }

    #[test]
    fn test_count_empty_corners_counts_air_lava_water() {
        let mut level = MockLevel::new(-64, 320);
        let bb = BoundingBox { min_x: 0, min_y: 0, min_z: 0, max_x: 1, max_y: 1, max_z: 1 };
        level.states.insert((0, 0, 0), MockBlockState { air: true, lava: false, water: false });
        level.states.insert((1, 0, 0), MockBlockState { air: false, lava: true, water: false });
        level.states.insert((0, 1, 0), MockBlockState { air: false, lava: false, water: true });
        level.states.insert((1, 1, 1), MockBlockState { air: false, lava: false, water: false });
        let count = count_empty_corners(&level, &bb);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_chunk_pos_from_block_pos_and_bounds() {
        let cp = ChunkPos::from_block_pos(BlockPos { x: 16, y: 64, z: -1 });
        assert_eq!(cp.x, 1);
        assert_eq!(cp.z, -1);
        assert_eq!(cp.get_min_block_x(), 16);
        assert_eq!(cp.get_max_block_x(), 31);
        assert_eq!(cp.get_min_block_z(), -16);
        assert_eq!(cp.get_max_block_z(), -1);
    }

    #[test]
    fn test_rotation_get_random_mapping() {
        let mut r = MockRandom::new(vec![0, 1, 2, 3, 4]);
        assert_eq!(Rotation::get_random(&mut r), Rotation::None);
        assert_eq!(Rotation::get_random(&mut r), Rotation::Clockwise90);
        assert_eq!(Rotation::get_random(&mut r), Rotation::Rotation180);
        assert_eq!(Rotation::get_random(&mut r), Rotation::Counterclockwise90);
        assert_eq!(Rotation::get_random(&mut r), Rotation::None); // 4 % 4 == 0
    }

    #[test]
    fn test_place_y_clamped_to_min_y_plus_10() {
        let mut level = MockLevel::new(0, 256);
        // heights default to 64, min_height=64, random_offset=9 -> 64-15-9=40, min_allowed=10 -> 40
        // set heights very low to force clamp
        for x in -10..10 {
            for z in -10..10 {
                level.heights.insert((x, z), 5);
            }
        }
        let mut random = MockRandom::new(vec![0, 0, 9]); // offset 9
        let config = base_config();
        let template = make_template(BoundingBox {
            min_x: 0, min_y: 0, min_z: 0, max_x: 0, max_y: 0, max_z: 0,
        });
        let placed_ref = template.placed.clone();
        let manager = MockManager { template };

        let ok = FossilFeature::place(
            &mut level,
            BlockPos { x: 0, y: 5, z: 0 },
            &mut random,
            &config,
            &manager,
        );
        assert!(ok);
        let placed = placed_ref.borrow();
        // place_y should be max(5-15-9, 0+10) = max(-19, 10) = 10
        assert_eq!(placed[0].0.y, 10);
    }
}
