use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(int,int,int)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#below()
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#mutable()
    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#setWithOffset(Vec3i,Direction)
    pub fn set_with_offset(&mut self, pos: &BlockPos, dir: Direction) -> &mut Self {
        let (nx, ny, nz) = dir.normal();
        self.x = pos.x.wrapping_add(nx);
        self.y = pos.y.wrapping_add(ny);
        self.z = pos.z.wrapping_add(nz);
        self
    }

    pub fn as_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

impl Direction {
    // JAVA: net.minecraft.core.Direction#getUnitVec3i()
    pub fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }

    // JAVA: net.minecraft.core.Direction#getOpposite()
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}

pub trait BlockStateAccess: Clone + Debug {
    fn has_property_waterlogged(&self) -> bool;
    fn get_value_waterlogged(&self) -> bool;
    fn set_value_waterlogged(&mut self, value: bool);
    fn water_default_state() -> Self
    where
        Self: Sized;
}

pub trait LevelAccess<S: BlockStateAccess> {
    fn get_block_state(&self, pos: &BlockPos) -> S;
    fn set_block(&mut self, pos: BlockPos, state: S, flags: i32) -> bool;
    fn is_face_sturdy(&self, state: &S, pos: &BlockPos, dir: Direction) -> bool;
}

pub trait RandomSource {}

#[derive(Clone, Debug, Default)]
pub struct VegetationPatchConfig;

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkGenerator;

pub trait VegetationPatchFeatureBase<S: BlockStateAccess, L: LevelAccess<S>> {
    fn place_ground_patch(
        &self,
        level: &mut L,
        config: &VegetationPatchConfig,
        random: &mut dyn RandomSource,
        pos: BlockPos,
        replaceable: &dyn Fn(&S) -> bool,
        x_radius: i32,
        y_radius: i32,
    ) -> HashSet<BlockPos>;

    fn place_vegetation(
        &self,
        level: &mut L,
        config: &VegetationPatchConfig,
        chunk_generator: &ChunkGenerator,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool;
}

pub struct WaterloggedVegetationPatchFeature<B> {
    pub base: B,
}

impl<B> WaterloggedVegetationPatchFeature<B> {
    // JAVA: net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature#<init>(Codec)
    pub fn new(base: B) -> Self {
        Self { base }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature#placeGroundPatch(WorldGenLevel,VegetationPatchConfiguration,RandomSource,BlockPos,Predicate,int,int)
    pub fn place_ground_patch<S, L>(
        &self,
        level: &mut L,
        config: &VegetationPatchConfig,
        random: &mut dyn RandomSource,
        pos: BlockPos,
        replaceable: &dyn Fn(&S) -> bool,
        x_radius: i32,
        y_radius: i32,
    ) -> HashSet<BlockPos>
    where
        S: BlockStateAccess,
        L: LevelAccess<S>,
        B: VegetationPatchFeatureBase<S, L>,
    {
        let ground = self.base.place_ground_patch(
            level,
            config,
            random,
            pos,
            replaceable,
            x_radius,
            y_radius,
        );
        let mut exposed_positions = HashSet::new();
        let mut mpos = BlockPos::new(0, 0, 0).mutable();

        for p in &ground {
            if !Self::is_exposed::<S, L>(level, &ground, p, &mut mpos) {
                exposed_positions.insert(*p);
            }
        }

        for p in &exposed_positions {
            level.set_block(*p, S::water_default_state(), 2);
        }

        exposed_positions
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature#isExposed(WorldGenLevel,Set,BlockPos,BlockPos.MutableBlockPos)
    fn is_exposed<S, L>(
        level: &L,
        _positions: &HashSet<BlockPos>,
        pos: &BlockPos,
        mpos: &mut MutableBlockPos,
    ) -> bool
    where
        S: BlockStateAccess,
        L: LevelAccess<S>,
    {
        Self::is_exposed_direction::<S, L>(level, pos, mpos, Direction::North)
            || Self::is_exposed_direction::<S, L>(level, pos, mpos, Direction::East)
            || Self::is_exposed_direction::<S, L>(level, pos, mpos, Direction::South)
            || Self::is_exposed_direction::<S, L>(level, pos, mpos, Direction::West)
            || Self::is_exposed_direction::<S, L>(level, pos, mpos, Direction::Down)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature#isExposedDirection(WorldGenLevel,BlockPos,BlockPos.MutableBlockPos,Direction)
    fn is_exposed_direction<S, L>(
        level: &L,
        pos: &BlockPos,
        mpos: &mut MutableBlockPos,
        dir: Direction,
    ) -> bool
    where
        S: BlockStateAccess,
        L: LevelAccess<S>,
    {
        mpos.set_with_offset(pos, dir);
        let p = mpos.as_pos();
        let state = level.get_block_state(&p);
        !level.is_face_sturdy(&state, &p, dir.opposite())
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature#placeVegetation(WorldGenLevel,VegetationPatchConfiguration,ChunkGenerator,RandomSource,BlockPos)
    pub fn place_vegetation<S, L>(
        &self,
        level: &mut L,
        config: &VegetationPatchConfig,
        chunk_generator: &ChunkGenerator,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> bool
    where
        S: BlockStateAccess,
        L: LevelAccess<S>,
        B: VegetationPatchFeatureBase<S, L>,
    {
        if self
            .base
            .place_vegetation(level, config, chunk_generator, random, pos.below())
        {
            let state = level.get_block_state(&pos);
            if state.has_property_waterlogged() && !state.get_value_waterlogged() {
                let mut new_state = state;
                new_state.set_value_waterlogged(true);
                level.set_block(pos, new_state, 2);
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockState {
        id: u32,
        waterlogged: Option<bool>,
        sturdy: bool,
    }

    impl MockState {
        fn new(id: u32) -> Self {
            Self {
                id,
                waterlogged: None,
                sturdy: true,
            }
        }

        fn with_waterlogged(mut self, v: bool) -> Self {
            self.waterlogged = Some(v);
            self
        }

        fn with_sturdy(mut self, v: bool) -> Self {
            self.sturdy = v;
            self
        }
    }

    impl BlockStateAccess for MockState {
        fn has_property_waterlogged(&self) -> bool {
            self.waterlogged.is_some()
        }

        fn get_value_waterlogged(&self) -> bool {
            self.waterlogged.unwrap_or(false)
        }

        fn set_value_waterlogged(&mut self, value: bool) {
            self.waterlogged = Some(value);
        }

        fn water_default_state() -> Self {
            Self::new(1)
        }
    }

    struct MockLevel {
        blocks: BTreeMap<(i32, i32, i32), MockState>,
        default: MockState,
    }

    impl MockLevel {
        fn new(default: MockState) -> Self {
            Self {
                blocks: BTreeMap::new(),
                default,
            }
        }

        fn set(&mut self, pos: BlockPos, state: MockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

    impl LevelAccess<MockState> for MockLevel {
        fn get_block_state(&self, pos: &BlockPos) -> MockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(|| self.default.clone())
        }

        fn set_block(&mut self, pos: BlockPos, state: MockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }

        fn is_face_sturdy(&self, state: &MockState, _pos: &BlockPos, _dir: Direction) -> bool {
            state.sturdy
        }
    }

    struct NullRandom;
    impl RandomSource for NullRandom {}

    struct MockBase {
        ground_patch: HashSet<BlockPos>,
        vegetation_result: bool,
    }

    impl VegetationPatchFeatureBase<MockState, MockLevel> for MockBase {
        fn place_ground_patch(
            &self,
            _level: &mut MockLevel,
            _config: &VegetationPatchConfig,
            _random: &mut dyn RandomSource,
            _pos: BlockPos,
            _replaceable: &dyn Fn(&MockState) -> bool,
            _x_radius: i32,
            _y_radius: i32,
        ) -> HashSet<BlockPos> {
            self.ground_patch.clone()
        }

        fn place_vegetation(
            &self,
            _level: &mut MockLevel,
            _config: &VegetationPatchConfig,
            _chunk_generator: &ChunkGenerator,
            _random: &mut dyn RandomSource,
            _pos: BlockPos,
        ) -> bool {
            self.vegetation_result
        }
    }

    #[test]
    fn exposed_direction_when_not_sturdy() {
        let mut level = MockLevel::new(MockState::new(0).with_sturdy(true));
        level.set(
            BlockPos::new(0, 0, -1),
            MockState::new(2).with_sturdy(false),
        );
        let mut mpos = BlockPos::new(0, 0, 0).mutable();
        let exposed = WaterloggedVegetationPatchFeature::<MockBase>::is_exposed_direction::<
            MockState,
            MockLevel,
        >(&level, &BlockPos::new(0, 0, 0), &mut mpos, Direction::North);
        assert!(exposed);
    }

    #[test]
    fn not_exposed_when_all_neighbors_sturdy() {
        let level = MockLevel::new(MockState::new(0).with_sturdy(true));
        let mut mpos = BlockPos::new(0, 0, 0).mutable();
        let positions = HashSet::new();
        let exposed = WaterloggedVegetationPatchFeature::<MockBase>::is_exposed::<MockState, MockLevel>(
            &level,
            &positions,
            &BlockPos::new(0, 0, 0),
            &mut mpos,
        );
        assert!(!exposed);
    }

    #[test]
    fn place_ground_patch_sets_water_on_non_exposed() {
        let mut ground = HashSet::new();
        ground.insert(BlockPos::new(0, 0, 0));
        ground.insert(BlockPos::new(1, 0, 0));
        let mut level = MockLevel::new(MockState::new(0).with_sturdy(true));
        level.set(
            BlockPos::new(1, 0, -1),
            MockState::new(2).with_sturdy(false),
        );
        let feature = WaterloggedVegetationPatchFeature::new(MockBase {
            ground_patch: ground,
            vegetation_result: false,
        });
        let mut random = NullRandom;
        let config = VegetationPatchConfig;
        let result = feature.place_ground_patch(
            &mut level,
            &config,
            &mut random,
            BlockPos::new(0, 0, 0),
            &|_| true,
            2,
            2,
        );
        assert!(result.contains(&BlockPos::new(0, 0, 0)));
        assert!(!result.contains(&BlockPos::new(1, 0, 0)));
        assert_eq!(
            level.get_block_state(&BlockPos::new(0, 0, 0)),
            MockState::water_default_state()
        );
        assert_ne!(
            level.get_block_state(&BlockPos::new(1, 0, 0)),
            MockState::water_default_state()
        );
    }

    #[test]
    fn place_vegetation_waterlogs_when_base_succeeds() {
        let mut level = MockLevel::new(MockState::new(0));
        let pos = BlockPos::new(0, 5, 0);
        level.set(pos, MockState::new(3).with_waterlogged(false));
        let feature = WaterloggedVegetationPatchFeature::new(MockBase {
            ground_patch: HashSet::new(),
            vegetation_result: true,
        });
        let mut random = NullRandom;
        let ok = feature.place_vegetation(
            &mut level,
            &VegetationPatchConfig,
            &ChunkGenerator,
            &mut random,
            pos,
        );
        assert!(ok);
        let state = level.get_block_state(&pos);
        assert!(state.has_property_waterlogged());
        assert!(state.get_value_waterlogged());
    }

    #[test]
    fn place_vegetation_respects_base_failure_and_existing_waterlog() {
        let mut level = MockLevel::new(MockState::new(0));
        let pos = BlockPos::new(0, 5, 0);
        level.set(pos, MockState::new(3).with_waterlogged(false));
        let feature_fail = WaterloggedVegetationPatchFeature::new(MockBase {
            ground_patch: HashSet::new(),
            vegetation_result: false,
        });
        let mut random = NullRandom;
        let ok = feature_fail.place_vegetation(
            &mut level,
            &VegetationPatchConfig,
            &ChunkGenerator,
            &mut random,
            pos,
        );
        assert!(!ok);
        assert!(!level.get_block_state(&pos).get_value_waterlogged());

        let feature_ok = WaterloggedVegetationPatchFeature::new(MockBase {
            ground_patch: HashSet::new(),
            vegetation_result: true,
        });
        level.set(pos, MockState::new(3).with_waterlogged(true));
        let ok2 = feature_ok.place_vegetation(
            &mut level,
            &VegetationPatchConfig,
            &ChunkGenerator,
            &mut random,
            pos,
        );
        assert!(ok2);
        assert!(level.get_block_state(&pos).get_value_waterlogged());
    }
}
