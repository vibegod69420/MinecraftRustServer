use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
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

    pub fn get_opposite(self) -> Self {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    pub fn get_axis(self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }
}

pub struct HorizontalPlane;

impl HorizontalPlane {
    pub const VALUES: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    pub fn get_random_direction<R: RandomSource + ?Sized>(random: &mut R) -> Direction {
        let idx = random.next_i32_bound(4);
        Self::VALUES[(idx as usize) & 3]
    }
}

pub trait RandomSource {
    fn next_i32_bound(&mut self, bound: i32) -> i32;
    fn next_f32(&mut self) -> f32;
}

impl<R: RandomSource + ?Sized> RandomSource for &mut R {
    fn next_i32_bound(&mut self, bound: i32) -> i32 {
        (**self).next_i32_bound(bound)
    }

    fn next_f32(&mut self) -> f32 {
        (**self).next_f32()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UniformInt {
    min: i32,
    max: i32,
}

impl UniformInt {
    pub fn of(min: i32, max: i32) -> Self {
        Self { min, max }
    }

    pub fn min_value(self) -> i32 {
        self.min
    }

    pub fn max_value(self) -> i32 {
        self.max
    }

    pub fn sample<R: RandomSource + ?Sized>(self, random: &mut R) -> i32 {
        if self.max <= self.min {
            return self.min;
        }
        let span = self.max.wrapping_sub(self.min).wrapping_add(1);
        if span <= 0 {
            return self.min;
        }
        self.min.wrapping_add(random.next_i32_bound(span))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntProvider {
    Constant(i32),
    Uniform(UniformInt),
}

impl IntProvider {
    pub fn sample<R: RandomSource + ?Sized>(&self, random: &mut R) -> i32 {
        match self {
            IntProvider::Constant(v) => *v,
            IntProvider::Uniform(u) => u.sample(random),
        }
    }
}

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

    pub fn below(self) -> Self {
        self.above(-1)
    }

    pub fn above(self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(dy),
            z: self.z,
        }
    }

    pub fn relative(self, dir: Direction, amount: i32) -> Self {
        let (nx, ny, nz) = dir.normal();
        Self {
            x: self.x.wrapping_add(nx.wrapping_mul(amount)),
            y: self.y.wrapping_add(ny.wrapping_mul(amount)),
            z: self.z.wrapping_add(nz.wrapping_mul(amount)),
        }
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn dist_manhattan(self, other: BlockPos) -> i32 {
        let dx = self.x.wrapping_sub(other.x).wrapping_abs();
        let dy = self.y.wrapping_sub(other.y).wrapping_abs();
        let dz = self.z.wrapping_sub(other.z).wrapping_abs();
        dx.wrapping_add(dy).wrapping_add(dz)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn set(&mut self, pos: BlockPos) -> &mut Self {
        self.x = pos.x;
        self.y = pos.y;
        self.z = pos.z;
        self
    }

    pub fn move_dir(&mut self, dir: Direction) -> &mut Self {
        self.move_dir_amount(dir, 1)
    }

    pub fn move_dir_amount(&mut self, dir: Direction, amount: i32) -> &mut Self {
        let (nx, ny, nz) = dir.normal();
        self.x = self.x.wrapping_add(nx.wrapping_mul(amount));
        self.y = self.y.wrapping_add(ny.wrapping_mul(amount));
        self.z = self.z.wrapping_add(nz.wrapping_mul(amount));
        self
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn dist_manhattan(self, other: BlockPos) -> i32 {
        let dx = self.x.wrapping_sub(other.x).wrapping_abs();
        let dy = self.y.wrapping_sub(other.y).wrapping_abs();
        let dz = self.z.wrapping_sub(other.z).wrapping_abs();
        dx.wrapping_add(dy).wrapping_add(dz)
    }

    pub fn as_block_pos(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub name: &'static str,
    pub axis: Option<Axis>,
}

impl BlockState {
    pub fn log() -> Self {
        Self {
            name: "log",
            axis: None,
        }
    }

    pub fn dirt() -> Self {
        Self {
            name: "dirt",
            axis: None,
        }
    }

    pub fn try_set_value_axis(&self, axis: Axis) -> Self {
        let mut s = self.clone();
        s.axis = Some(axis);
        s
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

impl FoliageAttachment {
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeConfiguration {
    pub trunk_provider_state: BlockState,
}

impl Default for TreeConfiguration {
    fn default() -> Self {
        Self {
            trunk_provider_state: BlockState::log(),
        }
    }
}

pub trait LevelSimulatedReader {
    fn is_state_at_position(&self, pos: BlockPos, predicate: &dyn Fn(&BlockState) -> bool) -> bool;
}

pub struct AlwaysFreeLevel;

impl LevelSimulatedReader for AlwaysFreeLevel {
    fn is_state_at_position(&self, _pos: BlockPos, _predicate: &dyn Fn(&BlockState) -> bool) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrunkPlacerType {
    CherryTrunkPlacer,
}

#[derive(Clone, Debug)]
pub struct CherryTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
    pub branch_count: IntProvider,
    pub branch_horizontal_length: IntProvider,
    pub branch_start_offset_from_top: UniformInt,
    pub second_branch_start_offset_from_top: UniformInt,
    pub branch_end_offset_from_top: IntProvider,
}

impl CherryTrunkPlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.CherryTrunkPlacer#<init>
    pub fn new(
        base_height: i32,
        height_rand_a: i32,
        height_rand_b: i32,
        branch_count: IntProvider,
        branch_horizontal_length: IntProvider,
        branch_start_offset_from_top: UniformInt,
        branch_end_offset_from_top: IntProvider,
    ) -> Self {
        let second_branch_start_offset_from_top = UniformInt::of(
            branch_start_offset_from_top.min_value(),
            branch_start_offset_from_top.max_value().wrapping_sub(1),
        );
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
            branch_count,
            branch_horizontal_length,
            branch_start_offset_from_top,
            second_branch_start_offset_from_top,
            branch_end_offset_from_top,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.CherryTrunkPlacer#type
    pub fn placer_type(&self) -> TrunkPlacerType {
        TrunkPlacerType::CherryTrunkPlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#setDirtAt
    pub fn set_dirt_at<R: RandomSource + ?Sized>(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut impl FnMut(BlockPos, BlockState),
        _random: &mut R,
        pos: BlockPos,
        _config: &TreeConfiguration,
    ) {
        if level.is_state_at_position(pos, &|_| true) {
            block_setter(pos, BlockState::dirt());
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#placeLog
    pub fn place_log<R: RandomSource + ?Sized>(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut impl FnMut(BlockPos, BlockState),
        _random: &mut R,
        pos: BlockPos,
        config: &TreeConfiguration,
        property_setter: &impl Fn(&BlockState) -> BlockState,
    ) -> bool {
        if self.valid_tree_pos(level, pos) {
            block_setter(pos, property_setter(&config.trunk_provider_state));
            true
        } else {
            false
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#validTreePos
    pub fn valid_tree_pos(&self, level: &dyn LevelSimulatedReader, pos: BlockPos) -> bool {
        level.is_state_at_position(pos, &|_| true)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.CherryTrunkPlacer#placeTrunk
    pub fn place_trunk<R: RandomSource + ?Sized>(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut impl FnMut(BlockPos, BlockState),
        random: &mut R,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> Vec<FoliageAttachment> {
        self.set_dirt_at(level, block_setter, random, pos.below(), config);

        let i6 = 0i32.max(
            free_tree_height
                .wrapping_sub(1)
                .wrapping_add(self.branch_start_offset_from_top.sample(random)),
        );
        let mut i7 = 0i32.max(
            free_tree_height
                .wrapping_sub(1)
                .wrapping_add(self.second_branch_start_offset_from_top.sample(random)),
        );
        if i7 >= i6 {
            i7 = i7.wrapping_add(1);
        }

        let branch_count = self.branch_count.sample(random);
        let is_three = branch_count == 3;
        let is_two_or_more = branch_count >= 2;
        let top = if is_three {
            free_tree_height
        } else if is_two_or_more {
            i6.max(i7).wrapping_add(1)
        } else {
            i6.wrapping_add(1)
        };

        let mut i = 0i32;
        while i < top {
            self.place_log(level, block_setter, random, pos.above(i), config, &|s: &BlockState| s.clone());
            i = i.wrapping_add(1);
        }

        let mut attachments: Vec<FoliageAttachment> = Vec::new();
        if is_three {
            attachments.push(FoliageAttachment::new(pos.above(top), 0, false));
        }

        let mut mutable = MutableBlockPos::new(0, 0, 0);
        let dir = HorizontalPlane::get_random_direction(random);
        let axis = dir.get_axis();
        let axis_setter = |s: &BlockState| s.try_set_value_axis(axis);

        attachments.push(self.generate_branch(
            level,
            block_setter,
            random,
            free_tree_height,
            pos,
            config,
            &axis_setter,
            dir,
            i6,
            i6 < top.wrapping_sub(1),
            &mut mutable,
        ));
        if is_two_or_more {
            attachments.push(self.generate_branch(
                level,
                block_setter,
                random,
                free_tree_height,
                pos,
                config,
                &axis_setter,
                dir.get_opposite(),
                i7,
                i7 < top.wrapping_sub(1),
                &mut mutable,
            ));
        }

        attachments
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.CherryTrunkPlacer#generateBranch
    fn generate_branch<R: RandomSource + ?Sized>(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut impl FnMut(BlockPos, BlockState),
        random: &mut R,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
        property_setter: &impl Fn(&BlockState) -> BlockState,
        direction: Direction,
        branch_start_offset: i32,
        extra_horizontal: bool,
        mutable: &mut MutableBlockPos,
    ) -> FoliageAttachment {
        mutable.set(pos);
        mutable.move_dir_amount(Direction::Up, branch_start_offset);

        let branch_end = free_tree_height
            .wrapping_sub(1)
            .wrapping_add(self.branch_end_offset_from_top.sample(random));
        let below_start = extra_horizontal || branch_end < branch_start_offset;
        let horiz_len = self
            .branch_horizontal_length
            .sample(random)
            .wrapping_add(if below_start { 1 } else { 0 });
        let target = pos.relative(direction, horiz_len).above(branch_end);
        let steps = if below_start { 2i32 } else { 1i32 };

        let mut step = 0i32;
        while step < steps {
            mutable.move_dir(direction);
            let mp = mutable.as_block_pos();
            self.place_log(level, block_setter, random, mp, config, property_setter);
            step = step.wrapping_add(1);
        }

        let vertical_dir = if target.get_y() > mutable.get_y() {
            Direction::Up
        } else {
            Direction::Down
        };

        loop {
            let dist = mutable.dist_manhattan(target);
            if dist == 0 {
                return FoliageAttachment::new(target.above(1), 0, false);
            }

            let dy_abs = target.get_y().wrapping_sub(mutable.get_y()).wrapping_abs();
            let f = (dy_abs as f32) / (dist as f32);
            let go_vertical = random.next_f32() < f;
            if go_vertical {
                mutable.move_dir(vertical_dir);
            } else {
                mutable.move_dir(direction);
            }
            let mp = mutable.as_block_pos();
            if go_vertical {
                self.place_log(level, block_setter, random, mp, config, &|s: &BlockState| s.clone());
            } else {
                self.place_log(level, block_setter, random, mp, config, property_setter);
            }
        }
    }
}

impl fmt::Display for TrunkPlacerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrunkPlacerType::CherryTrunkPlacer => write!(f, "cherry_trunk_placer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        ints: Vec<i32>,
        floats: Vec<f32>,
        int_idx: usize,
        float_idx: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>, floats: Vec<f32>) -> Self {
            Self {
                ints,
                floats,
                int_idx: 0,
                float_idx: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_i32_bound(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.int_idx).copied().unwrap_or(0);
            self.int_idx = self.int_idx.wrapping_add(1);
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_f32(&mut self) -> f32 {
            let v = self.floats.get(self.float_idx).copied().unwrap_or(0.0);
            self.float_idx = self.float_idx.wrapping_add(1);
            v
        }
    }

    fn test_placer() -> CherryTrunkPlacer {
        CherryTrunkPlacer::new(
            7,
            1,
            0,
            IntProvider::Uniform(UniformInt::of(1, 3)),
            IntProvider::Uniform(UniformInt::of(2, 4)),
            UniformInt::of(-4, -3),
            IntProvider::Uniform(UniformInt::of(-1, 0)),
        )
    }

    #[test]
    fn constructor_second_branch_offset_and_type() {
        let p = test_placer();
        assert_eq!(p.branch_start_offset_from_top, UniformInt::of(-4, -3));
        assert_eq!(p.second_branch_start_offset_from_top, UniformInt::of(-4, -4));
        assert_eq!(p.placer_type(), TrunkPlacerType::CherryTrunkPlacer);
    }

    #[test]
    fn place_trunk_single_branch_deterministic() {
        let p = CherryTrunkPlacer::new(
            5,
            0,
            0,
            IntProvider::Constant(1),
            IntProvider::Constant(2),
            UniformInt::of(-1, -1),
            IntProvider::Constant(0),
        );
        let level = AlwaysFreeLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |pos: BlockPos, state: BlockState| placed.push((pos, state));
        // samples: branch_start(-1)->0 offset in uniform(0 span handled), second same, branch_count constant,
        // direction index 0 -> North, branch_end constant 0, horiz constant 2
        let mut rng = SeqRandom::new(vec![0], vec![1.0, 1.0, 1.0, 1.0]);
        let config = TreeConfiguration::default();
        let attachments = p.place_trunk(&level, &mut setter, &mut rng, 5, BlockPos::new(0, 0, 0), &config);

        // top = i6+1 = (max(0,4-1))+1 = 4; trunk logs at y=0..3
        assert!(placed.iter().any(|(pos, st)| *pos == BlockPos::new(0, 0, 0) && st.name == "log"));
        assert!(placed.iter().any(|(pos, st)| *pos == BlockPos::new(0, 3, 0) && st.name == "log"));
        // dirt at below
        assert!(placed.iter().any(|(pos, st)| *pos == BlockPos::new(0, -1, 0) && st.name == "dirt"));
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].radius_offset, 0);
        assert!(!attachments[0].double_trunk);
    }

    #[test]
    fn place_trunk_three_branches_adds_top_attachment() {
        let p = CherryTrunkPlacer::new(
            5,
            0,
            0,
            IntProvider::Constant(3),
            IntProvider::Constant(1),
            UniformInt::of(0, 0),
            IntProvider::Constant(0),
        );
        let level = AlwaysFreeLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |pos: BlockPos, state: BlockState| placed.push((pos, state));
        let mut rng = SeqRandom::new(vec![0], vec![0.0; 16]);
        let config = TreeConfiguration::default();
        let attachments = p.place_trunk(&level, &mut setter, &mut rng, 5, BlockPos::new(2, 10, -3), &config);

        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].pos, BlockPos::new(2, 15, -3)); // top = free_tree_height = 5, above(5)
        assert_eq!(attachments[0].radius_offset, 0);
        assert!(!attachments[0].double_trunk);
    }

    #[test]
    fn float_division_order_and_negative_dy() {
        let dy_pos: i32 = 1;
        let dist: i32 = 3;
        let f_pos = ((dy_pos.wrapping_abs()) as f32) / (dist as f32);
        assert_eq!(f_pos.to_bits(), (1.0f32 / 3.0f32).to_bits());

        let dy_neg: i32 = -1;
        let f_neg = ((dy_neg.wrapping_abs()) as f32) / (dist as f32);
        assert_eq!(f_neg.to_bits(), f_pos.to_bits());

        // Java order: (float)Math.abs(targetY - mutableY) / (float)dist
        let target_y: i32 = -2;
        let mutable_y: i32 = 1;
        let dy_abs = target_y.wrapping_sub(mutable_y).wrapping_abs();
        let f = (dy_abs as f32) / (5i32 as f32);
        assert_eq!(f.to_bits(), (3.0f32 / 5.0f32).to_bits());
    }

    #[test]
    fn dist_manhattan_wrapping_and_min_abs() {
        let a = BlockPos::new(i32::MIN, 0, 0);
        let b = BlockPos::new(i32::MAX, 0, 0);
        // Java: MIN-MAX wraps to 1, abs(1)=1
        assert_eq!(a.dist_manhattan(b), 1);

        let c = BlockPos::new(i32::MIN, 0, 0);
        let d = BlockPos::new(0, 0, 0);
        // Java: Math.abs(Integer.MIN_VALUE) == Integer.MIN_VALUE
        assert_eq!(c.dist_manhattan(d), i32::MIN);

        let m = MutableBlockPos::new(-5, 10, -3);
        assert_eq!(m.dist_manhattan(BlockPos::new(1, 7, 2)), 14);
    }

    #[test]
    fn negative_offsets_and_wrapping_position_math() {
        let pos = BlockPos::new(0, i32::MIN, 0);
        let above = pos.above(-1);
        assert_eq!(above.y, i32::MAX);

        let rel = BlockPos::new(i32::MAX, 0, 0).relative(Direction::East, 2);
        assert_eq!(rel.x, i32::MIN.wrapping_add(1));

        let u = UniformInt::of(-4, -3);
        let mut rng = SeqRandom::new(vec![0, 1], vec![]);
        assert_eq!(u.sample(&mut rng), -4);
        assert_eq!(u.sample(&mut rng), -3);
    }
}
