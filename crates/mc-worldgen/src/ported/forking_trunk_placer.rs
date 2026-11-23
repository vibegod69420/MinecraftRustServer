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

pub struct ForkingTrunkPlacer {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
}

impl ForkingTrunkPlacer {
    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.ForkingTrunkPlacer#<init>
    pub fn new(base_height: i32, height_rand_a: i32, height_rand_b: i32) -> Self {
        Self {
            base_height,
            height_rand_a,
            height_rand_b,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.ForkingTrunkPlacer#type
    pub fn placer_type(&self) -> TrunkPlacerType {
        TrunkPlacerType::ForkingTrunkPlacer
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.trunkplacers.ForkingTrunkPlacer#placeTrunk
    pub fn place_trunk(
        &self,
        level: &dyn LevelSimulatedReader,
        block_setter: &mut dyn FnMut(BlockPos, BlockState),
        random: &mut dyn RandomSource,
        free_tree_height: i32,
        pos: BlockPos,
        config: &TreeConfiguration,
    ) -> Vec<FoliageAttachment> {
        set_dirt_at(level, block_setter, random, pos.below(), config);
        let mut attachments: Vec<FoliageAttachment> = Vec::new();
        let dir = Direction::get_random_horizontal(random);
        let fork_height = free_tree_height
            .wrapping_sub(random.next_int(4))
            .wrapping_sub(1);
        let mut steps_remaining = 3i32.wrapping_sub(random.next_int(3));
        let mut mutable_pos = MutableBlockPos::new(0, 0, 0);
        let mut x = pos.get_x();
        let z_start = pos.get_z();
        let mut z = z_start;
        let mut top_y: Option<i32> = None;

        let mut i: i32 = 0;
        while i < free_tree_height {
            let y = pos.get_y().wrapping_add(i);
            if i >= fork_height && steps_remaining > 0 {
                x = x.wrapping_add(dir.step_x());
                z = z.wrapping_add(dir.step_z());
                steps_remaining = steps_remaining.wrapping_sub(1);
            }

            if place_log(
                level,
                block_setter,
                random,
                mutable_pos.set(x, y, z).to_block_pos(),
                config,
            ) {
                top_y = Some(y.wrapping_add(1));
            }
            i = i.wrapping_add(1);
        }

        if let Some(top) = top_y {
            attachments.push(FoliageAttachment::new(BlockPos::new(x, top, z), 1, false));
        }

        x = pos.get_x();
        z = z_start;
        let dir2 = Direction::get_random_horizontal(random);
        if dir2 != dir {
            let fork_height2 = fork_height
                .wrapping_sub(random.next_int(2))
                .wrapping_sub(1);
            let mut steps_remaining2 = 1i32.wrapping_add(random.next_int(3));
            top_y = None;

            let mut j: i32 = fork_height2;
            while j < free_tree_height && steps_remaining2 > 0 {
                if j >= 1 {
                    let y = pos.get_y().wrapping_add(j);
                    x = x.wrapping_add(dir2.step_x());
                    z = z.wrapping_add(dir2.step_z());
                    if place_log(
                        level,
                        block_setter,
                        random,
                        mutable_pos.set(x, y, z).to_block_pos(),
                        config,
                    ) {
                        top_y = Some(y.wrapping_add(1));
                    }
                }

                j = j.wrapping_add(1);
                steps_remaining2 = steps_remaining2.wrapping_sub(1);
            }

            if let Some(top) = top_y {
                attachments.push(FoliageAttachment::new(BlockPos::new(x, top, z), 0, false));
            }
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

    struct AlwaysValidLevel;

    impl LevelSimulatedReader for AlwaysValidLevel {
        fn is_valid_tree_pos(&self, _pos: &BlockPos) -> bool {
            true
        }

        fn is_dirt(&self, _pos: &BlockPos) -> bool {
            false
        }
    }

    struct NeverValidLevel;

    impl LevelSimulatedReader for NeverValidLevel {
        fn is_valid_tree_pos(&self, _pos: &BlockPos) -> bool {
            false
        }

        fn is_dirt(&self, _pos: &BlockPos) -> bool {
            true
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
    fn forking_trunk_basic_main_branch() {
        // dir=North(0), nextInt(4)=0, nextInt(3)=0 => stepsRemaining=3
        // dir2=North(0) same as dir => no second branch
        let mut random = SeqRandom::new(vec![0, 0, 0, 0]);
        let level = AlwaysValidLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = ForkingTrunkPlacer::new(5, 0, 0);

        let attachments = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            5,
            BlockPos::new(0, 10, 0),
            &config,
        );

        assert_eq!(placed[0].0, BlockPos::new(0, 9, 0));
        assert_eq!(placed[0].1, config.dirt_state);
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].radius_offset, 1);
        assert!(!attachments[0].double_trunk);
        // top_y should be last placed log y + 1 = 14 + 1 = 15
        assert_eq!(attachments[0].pos.get_y(), 15);
    }

    #[test]
    fn forking_trunk_second_branch_when_different_direction() {
        // dir=North(0), nextInt(4)=1 => forkHeight=5-1-1=3, nextInt(3)=0 => stepsRemaining=3
        // dir2=East(1), different; nextInt(2)=0 => forkHeight2=3-0-1=2, nextInt(3)=0 => stepsRemaining2=1
        let mut random = SeqRandom::new(vec![0, 1, 0, 1, 0, 0]);
        let level = AlwaysValidLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = ForkingTrunkPlacer::new(5, 0, 0);

        let attachments = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            5,
            BlockPos::new(0, 10, 0),
            &config,
        );

        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].radius_offset, 1);
        assert_eq!(attachments[1].radius_offset, 0);
        assert!(!attachments[0].double_trunk);
        assert!(!attachments[1].double_trunk);
    }

    #[test]
    fn forking_trunk_no_attachment_when_no_logs_placed() {
        let mut random = SeqRandom::new(vec![0, 0, 0, 0]);
        let level = NeverValidLevel;
        let mut placed: Vec<(BlockPos, BlockState)> = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = ForkingTrunkPlacer::new(5, 0, 0);

        let attachments = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            5,
            BlockPos::new(0, 10, 0),
            &config,
        );

        // dirt may still be placed depending on is_dirt; NeverValidLevel says is_dirt=true so no dirt
        assert!(attachments.is_empty());
        assert!(placed.is_empty());
    }

    #[test]
    fn forking_trunk_negative_inputs_use_wrapping() {
        let placer = ForkingTrunkPlacer::new(i32::MIN, -1, i32::MAX);
        assert_eq!(placer.base_height, i32::MIN);
        assert_eq!(placer.height_rand_a, -1);
        assert_eq!(placer.height_rand_b, i32::MAX);

        let h: i32 = i32::MIN;
        let fork_height = h.wrapping_sub(0).wrapping_sub(1);
        assert_eq!(fork_height, i32::MAX);

        let y = i32::MAX.wrapping_add(1);
        assert_eq!(y, i32::MIN);
    }

    #[test]
    fn forking_trunk_place_log_counted() {
        let mut random = SeqRandom::new(vec![0, 0, 0, 0]); // same dir -> no second branch
        let level = CountingLevel::new();
        let mut placed = Vec::new();
        let mut setter = |p: BlockPos, s: BlockState| placed.push((p, s));
        let config = TreeConfiguration::default();
        let placer = ForkingTrunkPlacer::new(3, 0, 0);

        let _ = placer.place_trunk(
            &level,
            &mut setter,
            &mut random,
            3,
            BlockPos::new(0, 0, 0),
            &config,
        );

        // place_log calls valid_tree_pos once per call; main loop runs 3 times
        assert_eq!(level.checks.get(), 3);
    }
}
