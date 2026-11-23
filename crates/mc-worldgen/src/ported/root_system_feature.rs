use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Minimal self-contained types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn get_x(self) -> i32 {
        self.x
    }

    #[inline]
    pub const fn get_y(self) -> i32 {
        self.y
    }

    #[inline]
    pub const fn get_z(self) -> i32 {
        self.z
    }

    #[inline]
    pub fn mutable(self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub fn below(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    #[inline]
    pub fn above(self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(1),
            z: self.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl MutableBlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn get_x(self) -> i32 {
        self.x
    }

    #[inline]
    pub const fn get_y(self) -> i32 {
        self.y
    }

    #[inline]
    pub const fn get_z(self) -> i32 {
        self.z
    }

    #[inline]
    pub fn as_pos(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn below(self) -> BlockPos {
        self.as_pos().below()
    }

    #[inline]
    pub fn above(self) -> BlockPos {
        self.as_pos().above()
    }

    #[inline]
    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        self.x = self.x.wrapping_add(dir.step_x());
        self.y = self.y.wrapping_add(dir.step_y());
        self.z = self.z.wrapping_add(dir.step_z());
        self
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    #[inline]
    pub fn set_with_offset(&mut self, base: BlockPos, dx: i32, dy: i32, dz: i32) -> &mut Self {
        self.x = base.x.wrapping_add(dx);
        self.y = base.y.wrapping_add(dy);
        self.z = base.z.wrapping_add(dz);
        self
    }

    #[inline]
    pub fn set_x(&mut self, x: i32) -> &mut Self {
        self.x = x;
        self
    }

    #[inline]
    pub fn set_z(&mut self, z: i32) -> &mut Self {
        self.z = z;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    #[inline]
    pub const fn step_x(self) -> i32 {
        match self {
            Self::West => -1,
            Self::East => 1,
            _ => 0,
        }
    }

    #[inline]
    pub const fn step_y(self) -> i32 {
        match self {
            Self::Down => -1,
            Self::Up => 1,
            _ => 0,
        }
    }

    #[inline]
    pub const fn step_z(self) -> i32 {
        match self {
            Self::North => -1,
            Self::South => 1,
            _ => 0,
        }
    }
}

pub trait BlockState: Clone {
    fn is_air(&self) -> bool;
    /// Corresponds to `blockState.getFluidState().is(FluidTags.WATER)`.
    fn is_water(&self) -> bool;
    fn is_solid(&self) -> bool;
}

pub trait RandomSource {
    /// Java `RandomSource#nextInt(int bound)` semantics: returns value in `[0, bound)`.
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait ChunkGenerator {}

pub trait WorldGenLevel {
    type BlockState: BlockState;

    fn get_block_state(&self, pos: BlockPos) -> Self::BlockState;
    /// Corresponds to `level.getFluidState(pos).is(FluidTags.LAVA)`.
    fn get_fluid_is_lava(&self, pos: BlockPos) -> bool;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: Self::BlockState, flags: i32) -> bool;
    /// Corresponds to `state.canSurvive(level, pos)`.
    fn can_survive(&self, state: &Self::BlockState, pos: BlockPos) -> bool;
    /// Corresponds to `level.getBlockState(block_pos).isFaceSturdy(level, support_pos, direction)`.
    fn is_face_sturdy(&self, block_pos: BlockPos, support_pos: BlockPos, direction: Direction) -> bool;
}

pub trait RootSystemConfig<L: WorldGenLevel, R: RandomSource, G: ChunkGenerator> {
    fn required_vertical_space_for_tree(&self) -> i32;
    fn root_radius(&self) -> i32;
    fn root_placement_attempts(&self) -> i32;
    fn root_column_max_height(&self) -> i32;
    fn hanging_root_radius(&self) -> i32;
    fn hanging_roots_vertical_span(&self) -> i32;
    fn hanging_root_placement_attempts(&self) -> i32;
    fn allowed_vertical_water_for_tree(&self) -> i32;

    fn allowed_tree_position(&self, level: &L, pos: BlockPos) -> bool;
    fn place_tree_feature(&self, level: &mut L, chunk: &G, rng: &mut R, pos: BlockPos) -> bool;
    fn is_root_replaceable(&self, state: &L::BlockState) -> bool;
    fn get_root_state(&self, rng: &mut R, pos: BlockPos) -> L::BlockState;
    fn get_hanging_root_state(&self, rng: &mut R, pos: BlockPos) -> L::BlockState;
}

pub struct FeaturePlaceContext<'a, L, R, G, C> {
    pub level: &'a mut L,
    pub chunk_generator: &'a G,
    pub random: &'a mut R,
    pub origin: BlockPos,
    pub config: &'a C,
}

// ---------------------------------------------------------------------------
// RootSystemFeature port
// ---------------------------------------------------------------------------

pub struct RootSystemFeature {
    _marker: PhantomData<()>,
}

impl Default for RootSystemFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl RootSystemFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#RootSystemFeature
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#place
    pub fn place<L, R, G, C>(&self, ctx: &mut FeaturePlaceContext<'_, L, R, G, C>) -> bool
    where
        L: WorldGenLevel,
        R: RandomSource,
        G: ChunkGenerator,
        C: RootSystemConfig<L, R, G>,
    {
        let origin = ctx.origin;
        if !ctx.level.get_block_state(origin).is_air() {
            return false;
        }

        let pos = ctx.origin;
        let mut mutable = pos.mutable();
        if Self::place_dirt_and_tree(
            ctx.level,
            ctx.chunk_generator,
            ctx.config,
            ctx.random,
            &mut mutable,
            pos,
        ) {
            Self::place_roots(ctx.level, ctx.config, ctx.random, pos, &mut mutable);
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#spaceForTree
    fn space_for_tree<L, R, G, C>(level: &L, config: &C, pos: BlockPos) -> bool
    where
        L: WorldGenLevel,
        R: RandomSource,
        G: ChunkGenerator,
        C: RootSystemConfig<L, R, G>,
    {
        let mut mutable = pos.mutable();
        let max = config.required_vertical_space_for_tree();
        let mut i: i32 = 1;
        while i <= max {
            mutable.move_dir(Direction::Up);
            let state = level.get_block_state(mutable.as_pos());
            if !Self::is_allowed_tree_space(
                &state,
                i,
                config.allowed_vertical_water_for_tree(),
            ) {
                return false;
            }
            i = i.wrapping_add(1);
        }

        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#isAllowedTreeSpace
    fn is_allowed_tree_space<B: BlockState>(
        state: &B,
        y_offset: i32,
        allowed_vertical_water_for_tree: i32,
    ) -> bool {
        if state.is_air() {
            return true;
        }

        let height = y_offset.wrapping_add(1);
        height <= allowed_vertical_water_for_tree && state.is_water()
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#placeDirtAndTree
    fn place_dirt_and_tree<L, R, G, C>(
        level: &mut L,
        chunk_generator: &G,
        config: &C,
        random: &mut R,
        mutable_pos: &mut MutableBlockPos,
        origin: BlockPos,
    ) -> bool
    where
        L: WorldGenLevel,
        R: RandomSource,
        G: ChunkGenerator,
        C: RootSystemConfig<L, R, G>,
    {
        let max_height = config.root_column_max_height();
        let mut i: i32 = 0;
        while i < max_height {
            mutable_pos.move_dir(Direction::Up);
            let current_pos = mutable_pos.as_pos();
            if config.allowed_tree_position(level, current_pos)
                && Self::space_for_tree(level, config, current_pos)
            {
                let below = mutable_pos.below();
                if level.get_fluid_is_lava(below) || !level.get_block_state(below).is_solid() {
                    return false;
                }

                if config.place_tree_feature(level, chunk_generator, random, current_pos) {
                    Self::place_dirt(
                        origin,
                        origin.get_y().wrapping_add(i),
                        level,
                        config,
                        random,
                    );
                    return true;
                }
            }
            i = i.wrapping_add(1);
        }

        false
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#placeDirt
    fn place_dirt<L, R, G, C>(
        origin: BlockPos,
        max_y: i32,
        level: &mut L,
        config: &C,
        random: &mut R,
    ) where
        L: WorldGenLevel,
        R: RandomSource,
        G: ChunkGenerator,
        C: RootSystemConfig<L, R, G>,
    {
        let x = origin.get_x();
        let z = origin.get_z();
        let mut mutable = origin.mutable();

        let mut y = origin.get_y();
        while y < max_y {
            mutable.set(x, y, z);
            Self::place_rooted_dirt(level, config, random, x, z, &mut mutable);
            y = y.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#placeRootedDirt
    fn place_rooted_dirt<L, R, G, C>(
        level: &mut L,
        config: &C,
        random: &mut R,
        x: i32,
        z: i32,
        mutable_pos: &mut MutableBlockPos,
    ) where
        L: WorldGenLevel,
        R: RandomSource,
        G: ChunkGenerator,
        C: RootSystemConfig<L, R, G>,
    {
        let radius = config.root_radius();
        let attempts = config.root_placement_attempts();

        let mut i: i32 = 0;
        while i < attempts {
            let dx = random
                .next_int(radius)
                .wrapping_sub(random.next_int(radius));
            let dz = random
                .next_int(radius)
                .wrapping_sub(random.next_int(radius));
            let base = mutable_pos.as_pos();
            mutable_pos.set_with_offset(base, dx, 0, dz);

            let current = mutable_pos.as_pos();
            let current_state = level.get_block_state(current);
            if config.is_root_replaceable(&current_state) {
                let root_state = config.get_root_state(random, current);
                level.set_block(current, root_state, 2);
            }

            mutable_pos.set_x(x);
            mutable_pos.set_z(z);
            i = i.wrapping_add(1);
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.RootSystemFeature#placeRoots
    fn place_roots<L, R, G, C>(
        level: &mut L,
        config: &C,
        random: &mut R,
        origin: BlockPos,
        mutable_pos: &mut MutableBlockPos,
    ) where
        L: WorldGenLevel,
        R: RandomSource,
        G: ChunkGenerator,
        C: RootSystemConfig<L, R, G>,
    {
        let radius = config.hanging_root_radius();
        let vertical_span = config.hanging_roots_vertical_span();
        let attempts = config.hanging_root_placement_attempts();

        let mut i: i32 = 0;
        while i < attempts {
            let dx = random
                .next_int(radius)
                .wrapping_sub(random.next_int(radius));
            let dy = random
                .next_int(vertical_span)
                .wrapping_sub(random.next_int(vertical_span));
            let dz = random
                .next_int(radius)
                .wrapping_sub(random.next_int(radius));
            mutable_pos.set_with_offset(origin, dx, dy, dz);

            let current = mutable_pos.as_pos();
            if level.is_empty_block(current) {
                let state = config.get_hanging_root_state(random, current);
                if level.can_survive(&state, current)
                    && level.is_face_sturdy(current.above(), current, Direction::Down)
                {
                    level.set_block(current, state, 2);
                }
            }
            i = i.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestState {
        id: u8,
        air: bool,
        water: bool,
        solid: bool,
    }

    impl TestState {
        const AIR: Self = Self {
            id: 0,
            air: true,
            water: false,
            solid: false,
        };
        const STONE: Self = Self {
            id: 1,
            air: false,
            water: false,
            solid: true,
        };
        const WATER: Self = Self {
            id: 2,
            air: false,
            water: true,
            solid: false,
        };
        const DIRT: Self = Self {
            id: 3,
            air: false,
            water: false,
            solid: true,
        };
        const ROOT: Self = Self {
            id: 4,
            air: false,
            water: false,
            solid: false,
        };
    }

    impl BlockState for TestState {
        fn is_air(&self) -> bool {
            self.air
        }
        fn is_water(&self) -> bool {
            self.water
        }
        fn is_solid(&self) -> bool {
            self.solid
        }
    }

    #[derive(Default)]
    struct TestLevel {
        blocks: HashMap<(i32, i32, i32), TestState>,
        lava: HashMap<(i32, i32, i32), bool>,
        sturdy_down: HashMap<(i32, i32, i32), bool>,
        survive: bool,
    }

    impl TestLevel {
        fn set(&mut self, pos: BlockPos, state: TestState) {
            self.blocks.insert((pos.get_x(), pos.get_y(), pos.get_z()), state);
        }
        fn get(&self, pos: BlockPos) -> TestState {
            self.blocks
                .get(&(pos.get_x(), pos.get_y(), pos.get_z()))
                .copied()
                .unwrap_or(TestState::AIR)
        }
    }

    impl WorldGenLevel for TestLevel {
        type BlockState = TestState;

        fn get_block_state(&self, pos: BlockPos) -> Self::BlockState {
            self.get(pos)
        }

        fn get_fluid_is_lava(&self, pos: BlockPos) -> bool {
            self.lava
                .get(&(pos.get_x(), pos.get_y(), pos.get_z()))
                .copied()
                .unwrap_or(false)
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get(pos).is_air()
        }

        fn set_block(&mut self, pos: BlockPos, state: Self::BlockState, _flags: i32) -> bool {
            self.set(pos, state);
            true
        }

        fn can_survive(&self, _state: &Self::BlockState, _pos: BlockPos) -> bool {
            self.survive
        }

        fn is_face_sturdy(
            &self,
            block_pos: BlockPos,
            _support_pos: BlockPos,
            _direction: Direction,
        ) -> bool {
            self.sturdy_down
                .get(&(block_pos.get_x(), block_pos.get_y(), block_pos.get_z()))
                .copied()
                .unwrap_or(self.get(block_pos).is_solid())
        }
    }

    struct SeqRng {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRng {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRng {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx = self.idx.wrapping_add(1);
            let m = v % bound;
            if m < 0 {
                m.wrapping_add(bound)
            } else {
                m
            }
        }
    }

    struct DummyChunk;
    impl ChunkGenerator for DummyChunk {}

    struct TestConfig {
        allowed_pos_y: i32,
        place_tree: bool,
    }

    impl RootSystemConfig<TestLevel, SeqRng, DummyChunk> for TestConfig {
        fn required_vertical_space_for_tree(&self) -> i32 {
            3
        }
        fn root_radius(&self) -> i32 {
            3
        }
        fn root_placement_attempts(&self) -> i32 {
            4
        }
        fn root_column_max_height(&self) -> i32 {
            5
        }
        fn hanging_root_radius(&self) -> i32 {
            2
        }
        fn hanging_roots_vertical_span(&self) -> i32 {
            2
        }
        fn hanging_root_placement_attempts(&self) -> i32 {
            3
        }
        fn allowed_vertical_water_for_tree(&self) -> i32 {
            2
        }
        fn allowed_tree_position(&self, _level: &TestLevel, pos: BlockPos) -> bool {
            pos.get_y() == self.allowed_pos_y
        }
        fn place_tree_feature(
            &self,
            _level: &mut TestLevel,
            _chunk: &DummyChunk,
            _rng: &mut SeqRng,
            _pos: BlockPos,
        ) -> bool {
            self.place_tree
        }
        fn is_root_replaceable(&self, state: &TestState) -> bool {
            state.is_air() || state.id == TestState::DIRT.id
        }
        fn get_root_state(&self, _rng: &mut SeqRng, _pos: BlockPos) -> TestState {
            TestState::DIRT
        }
        fn get_hanging_root_state(&self, _rng: &mut SeqRng, _pos: BlockPos) -> TestState {
            TestState::ROOT
        }
    }

    #[test]
    fn place_returns_false_when_origin_not_air() {
        let mut level = TestLevel::default();
        let origin = BlockPos::new(0, 10, 0);
        level.set(origin, TestState::STONE);
        let chunk = DummyChunk;
        let mut rng = SeqRng::new(vec![0]);
        let config = TestConfig {
            allowed_pos_y: 11,
            place_tree: true,
        };
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            chunk_generator: &chunk,
            random: &mut rng,
            origin,
            config: &config,
        };
        assert!(!RootSystemFeature::new().place(&mut ctx));
    }

    #[test]
    fn is_allowed_tree_space_air_water_and_overflow_boundary() {
        assert!(RootSystemFeature::is_allowed_tree_space(
            &TestState::AIR,
            1,
            0
        ));
        // height = y_offset.wrapping_add(1) = i32::MIN when y_offset == i32::MAX
        // i32::MIN <= 2 is true, and water is allowed -> true
        assert!(RootSystemFeature::is_allowed_tree_space(
            &TestState::WATER,
            i32::MAX,
            2
        ));
        // Solid non-water block is never allowed
        assert!(!RootSystemFeature::is_allowed_tree_space(
            &TestState::STONE,
            1,
            5
        ));
        // Water allowed only when height (y_offset + 1) <= allowed_vertical_water_for_tree
        assert!(RootSystemFeature::is_allowed_tree_space(
            &TestState::WATER,
            0,
            1
        ));
        assert!(!RootSystemFeature::is_allowed_tree_space(
            &TestState::WATER,
            1,
            1
        ));
    }

    #[test]
    fn place_rooted_dirt_handles_negative_coords_and_wrapping_sub() {
        let mut level = TestLevel::default();
        let mut rng = SeqRng::new(vec![i32::MIN, 1, 1, i32::MIN]);
        let chunk = DummyChunk;
        let config = TestConfig {
            allowed_pos_y: 0,
            place_tree: false,
        };
        let mut pos = MutableBlockPos::new(-5, -10, -7);
        RootSystemFeature::place_rooted_dirt(
            &mut level,
            &config,
            &mut rng,
            -5,
            -7,
            &mut pos,
        );
        // X/Z restored after attempts
        assert_eq!(pos.get_x(), -5);
        assert_eq!(pos.get_z(), -7);
        // At least one rooted dirt placed in air at negative coordinates
        assert!(level
            .blocks
            .values()
            .any(|s| s.id == TestState::DIRT.id));
    }

    #[test]
    fn place_roots_requires_empty_survive_and_sturdy_face() {
        let chunk = DummyChunk;
        let config = TestConfig {
            allowed_pos_y: 0,
            place_tree: false,
        };
        let origin = BlockPos::new(0, 0, 0);

        let mut level_no_support = TestLevel::default();
        level_no_support.survive = true;
        let mut rng = SeqRng::new(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut mpos = origin.mutable();
        RootSystemFeature::place_roots(&mut level_no_support, &config, &mut rng, origin, &mut mpos);
        assert!(!level_no_support
            .blocks
            .values()
            .any(|s| s.id == TestState::ROOT.id));

        let mut level_supported = TestLevel::default();
        level_supported.survive = true;
        // With rng producing 1,1,1 -> dx=0,dy=0,dz=0, target is origin (0,0,0), above is (0,1,0)
        level_supported.set(BlockPos::new(0, 1, 0), TestState::STONE);
        let mut rng2 = SeqRng::new(vec![1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut mpos2 = origin.mutable();
        RootSystemFeature::place_roots(&mut level_supported, &config, &mut rng2, origin, &mut mpos2);
        assert_eq!(level_supported.get(origin).id, TestState::ROOT.id);
    }

    #[test]
    fn float_bits_and_integer_wrapping_sanity() {
        assert_eq!(1.0f32.to_bits(), 0x3f800000);
        assert_eq!((-0.0f64).to_bits(), 0x8000_0000_0000_0000);
        assert_eq!(i32::MAX.wrapping_add(1), i32::MIN);
        assert_eq!(i32::MIN.wrapping_sub(1), i32::MAX);
        assert_eq!(BlockPos::new(0, i32::MIN, 0).below().get_y(), i32::MAX);
    }
}
