use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.BlockPos#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.BlockPos#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#mutable
    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#immutable
    pub fn immutable(&self) -> Self {
        *self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct MutableBlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl MutableBlockPos {
    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#<init>
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#set
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#move
    pub fn move_dir(&mut self, direction: Direction) -> &mut Self {
        self.x = self.x.wrapping_add(direction.step_x());
        self.y = self.y.wrapping_add(direction.step_y());
        self.z = self.z.wrapping_add(direction.step_z());
        self
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#immutable
    pub fn immutable(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    // JAVA: net.minecraft.core.BlockPos$MutableBlockPos#below
    pub fn below(&self) -> BlockPos {
        BlockPos::new(self.x, self.y.wrapping_sub(1), self.z)
    }

    pub fn to_block_pos(&self) -> BlockPos {
        self.immutable()
    }
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
    // JAVA: net.minecraft.core.Direction#getStepX
    pub fn step_x(self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepY
    pub fn step_y(self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction#getStepZ
    pub fn step_z(self) -> i32 {
        match self {
            Direction::South => 1,
            Direction::North => -1,
            _ => 0,
        }
    }

    // JAVA: net.minecraft.core.Direction$Plane#getRandomDirection
    pub fn get_random_horizontal(random: &mut dyn RandomSource) -> Self {
        const HORIZONTAL: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ];
        let idx = random.next_int(4);
        let idx = if idx < 0 { 0usize } else { (idx as usize) % 4 };
        HORIZONTAL[idx]
    }
}

pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextInt
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait IntProvider {
    // JAVA: net.minecraft.util.valueproviders.IntProvider#sample
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: u32,
}

impl BlockState {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug)]
pub struct TreeConfiguration {
    pub trunk_state: BlockState,
    pub dirt_state: BlockState,
    pub force_dirt: bool,
}

impl Default for TreeConfiguration {
    fn default() -> Self {
        Self {
            trunk_state: BlockState::new(1),
            dirt_state: BlockState::new(2),
            force_dirt: false,
        }
    }
}

pub trait LevelSimulatedReader {
    fn is_valid_tree_pos(&self, pos: &BlockPos) -> bool;
    fn is_dirt(&self, pos: &BlockPos) -> bool;
}

// JAVA: net.minecraft.world.level.levelgen.feature.TreeFeature#validTreePos
pub fn valid_tree_pos(level: &dyn LevelSimulatedReader, pos: &BlockPos) -> bool {
    level.is_valid_tree_pos(pos)
}

// JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#setDirtAt
pub fn set_dirt_at(
    level: &dyn LevelSimulatedReader,
    block_setter: &mut dyn FnMut(BlockPos, BlockState),
    random: &mut dyn RandomSource,
    pos: BlockPos,
    config: &TreeConfiguration,
) {
    let _ = random;
    if config.force_dirt || !level.is_dirt(&pos) {
        block_setter(pos, config.dirt_state);
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#placeLog
pub fn place_log(
    level: &dyn LevelSimulatedReader,
    block_setter: &mut dyn FnMut(BlockPos, BlockState),
    random: &mut dyn RandomSource,
    pos: BlockPos,
    config: &TreeConfiguration,
) -> bool {
    let _ = random;
    if valid_tree_pos(level, &pos) {
        block_setter(pos, config.trunk_state);
        true
    } else {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

impl FoliageAttachment {
    // JAVA: net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacer$FoliageAttachment#<init>
    pub fn new(pos: BlockPos, radius_offset: i32, double_trunk: bool) -> Self {
        Self {
            pos,
            radius_offset,
            double_trunk,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrunkPlacerType {
    BendingTrunkPlacer,
    ForkingTrunkPlacer,
}

pub struct BendingTrunkPlacer<P: IntProvider> {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
    pub min_height_for_leaves: i32,
    pub bend_length: P,
}

impl<P: IntProvider> BendingTrunkPlacer<P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.BendingTrunkPlacer#<init>
    pub fn new(
        base_height: i32,
        height_rand_a: i32,
        height_rand_b: i32,
        min_height_for_leaves: i32,
        bend_length: P,
    ) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
            min_height_for_leaves,
            bend_length,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.BendingTrunkPlacer#type
    pub fn placer_type(&self) -> TrunkPlacerType {
        TrunkPlacerType::BendingTrunkPlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.BendingTrunkPlacer#placeTrunk
    pub fn place_trunk(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> Vec<FoliageAttachment> {
        let dir = Direction::get_random_horizontal(random);
        let height_minus_one = free_tree_height.wrapping_sub(1);
        let mut mutable_pos = pos.mutable();
        let below_pos = mutable_pos.below();
        set_dirt_at(level, block_setter, random, below_pos, config);
        let mut attachments: Vec<FoliageAttachment> = Vec::new();

        let mut i: i32 = 0;
        while i <= height_minus_one {
            if i.wrapping_add(1) >= height_minus_one.wrapping_add(random.next_int(2)) {
                mutable_pos.move_dir(dir);
            }

            let current_pos = mutable_pos.to_block_pos();
            if valid_tree_pos(level, &current_pos) {
                place_log(level, block_setter, random, current_pos, config);
            }

            if i >= self.min_height_for_leaves {
                attachments.push(FoliageAttachment::new(mutable_pos.immutable(), 0, false));
            }

            mutable_pos.move_dir(Direction::Up);
            i = i.wrapping_add(1);
        }

        let bend_len = self.bend_length.sample(random);

        let mut j: i32 = 0;
        while j <= bend_len {
            let current_pos = mutable_pos.to_block_pos();
            if valid_tree_pos(level, &current_pos) {
                place_log(level, block_setter, random, current_pos, config);
            }

            attachments.push(FoliageAttachment::new(mutable_pos.immutable(), 0, false));
            mutable_pos.move_dir(dir);
            j = j.wrapping_add(1);
        }

        attachments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self { ints, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.ints.len() {
                let val = self.ints[self.idx];
                self.idx += 1;
                val
            } else {
                0
            };
            v.rem_euclid(bound)
        }
    }

    struct ConstInt(i32);

    impl IntProvider for ConstInt {
        fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
            self.0
        }
    }

    struct AlwaysValidLevel;

    impl LevelSimulatedReader for AlwaysValidLevel {
        fn is_valid_tree_pos(&self, _pos: &BlockPos) -> bool {
            true
        }

        fn is_dirt(&self, _pos: &BlockPos) -> bool {
            false
        }
    }

    struct CountingLevel {
        checks: Cell<u32>,
    }

    impl CountingLevel {
        fn new() -> Self {
            Self {
                checks: Cell::new(0),
            }
        }
    }

    impl LevelSimulatedReader for CountingLevel {
        fn is_valid_tree_pos(&self, _pos: &BlockPos) -> bool {
            self.checks.set(self.checks.get().wrapping_add(1));
            true
        }

        fn is_dirt(&self, _pos: &BlockPos) -> bool {
            false
        }
    }

    #[test]
    fn bending_trunk_places_bend_and_attachments() {
        let mut random = SeqRandom::new(vec![0, 0, 0, 0]); // dir=North, nextInt(2)=0,0,0
        let level = AlwaysValidLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = BendingTrunkPlacer::new(3, 0, 0, 1, ConstInt(2));

        let attachments = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            3,
            BlockPos::new(0, 10, 0),
            &config,
        );

        assert_eq!(placed[0].0, BlockPos::new(0, 9, 0));
        assert_eq!(placed[0].1, config.dirt_state);

        let logs: Vec<BlockPos> = placed
            .iter()
            .filter(|(_, s)| *s == config.trunk_state)
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(
            logs,
            vec![
                BlockPos::new(0, 10, 0),
                BlockPos::new(0, 11, -1),
                BlockPos::new(0, 12, -2),
                BlockPos::new(0, 13, -2),
                BlockPos::new(0, 13, -3),
                BlockPos::new(0, 13, -4),
            ]
        );

        assert_eq!(attachments.len(), 5);
        assert_eq!(
            attachments[0],
            FoliageAttachment::new(BlockPos::new(0, 11, -1), 0, false)
        );
        assert_eq!(
            attachments[4],
            FoliageAttachment::new(BlockPos::new(0, 13, -4), 0, false)
        );
    }

    #[test]
    fn bending_trunk_negative_height_gives_only_bend_loop() {
        let mut random = SeqRandom::new(vec![0]); // dir
        let level = AlwaysValidLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = BendingTrunkPlacer::new(0, 0, 0, 1, ConstInt(0));

        let attachments = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            0,
            BlockPos::new(1, 5, 2),
            &config,
        );

        // height_minus_one = -1, first loop skipped; bend_len=0 => one bend iteration
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].pos, BlockPos::new(1, 5, 2));
        assert!(!attachments[0].double_trunk);
        assert_eq!(attachments[0].radius_offset, 0);
    }

    #[test]
    fn bending_trunk_respects_min_height_for_leaves() {
        let mut random = SeqRandom::new(vec![0, 0, 0, 0, 0]); // dir + 4 nextInt(2)
        let level = AlwaysValidLevel;
        let mut placed = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = BendingTrunkPlacer::new(0, 0, 0, 2, ConstInt(0));

        let attachments = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            4,
            BlockPos::new(0, 0, 0),
            &config,
        );

        // First loop i=0..=3: attachments when i >= 2 => 2 attachments
        // Second loop bend_len=0 => 1 attachment
        assert_eq!(attachments.len(), 3);
        let first_loop: Vec<_> = attachments.iter().filter(|a| a.pos.get_y() < 4).collect();
        assert_eq!(first_loop.len(), 2);
    }

    #[test]
    fn bending_trunk_double_checks_valid_tree_pos() {
        let mut random = SeqRandom::new(vec![0, 0]); // dir, nextInt(2)
        let level = CountingLevel::new();
        let mut placed = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = BendingTrunkPlacer::new(0, 0, 0, 5, ConstInt(-1));

        // free_tree_height=1 => one iteration; checks valid_tree_pos in if and in place_log
        // bend_len=-1 => second loop skipped
        let _ = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            1,
            BlockPos::new(0, 0, 0),
            &config,
        );
        assert_eq!(level.checks.get(), 2);
    }

    #[test]
    fn bending_trunk_wrapping_arithmetic() {
        let h: i32 = i32::MIN;
        let h_minus_one = h.wrapping_sub(1);
        assert_eq!(h_minus_one, i32::MAX);

        let i: i32 = i32::MAX;
        let left = i.wrapping_add(1);
        let right = h_minus_one.wrapping_add(1);
        assert_eq!(left, i32::MIN);
        assert_eq!(right, i32::MIN);
        assert!(left >= right);

        let pos = BlockPos::new(0, i32::MIN, 0);
        assert_eq!(pos.below().get_y(), i32::MAX);
    }
}
