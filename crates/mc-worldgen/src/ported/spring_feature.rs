#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#<init>(III)
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.Vec3i#getX
    pub fn get_x(&self) -> i32 {
        self.x
    }

    // JAVA: net.minecraft.core.Vec3i#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.Vec3i#getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self) -> Self {
        self.offset(0, 1, 0)
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> Self {
        self.offset(0, -1, 0)
    }

    // JAVA: net.minecraft.core.BlockPos#west
    pub fn west(&self) -> Self {
        self.offset(-1, 0, 0)
    }

    // JAVA: net.minecraft.core.BlockPos#east
    pub fn east(&self) -> Self {
        self.offset(1, 0, 0)
    }

    // JAVA: net.minecraft.core.BlockPos#north
    pub fn north(&self) -> Self {
        self.offset(0, 0, -1)
    }

    // JAVA: net.minecraft.core.BlockPos#south
    pub fn south(&self) -> Self {
        self.offset(0, 0, 1)
    }
}

pub trait SpringLevel<B: Clone> {
    fn get_block_state(&self, pos: BlockPos) -> B;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: B, flags: i32) -> bool;
    fn schedule_tick(&mut self, pos: BlockPos, fluid: i32, delay: i32);
    fn is_air(&self, state: &B) -> bool;
    fn is_valid_block(&self, state: &B, valid_blocks: &[i32]) -> bool;
    fn state_block_id(&self, state: &B) -> i32;
}

#[derive(Clone, Debug)]
pub struct SpringFluidState {
    pub fluid_type: i32,
}

impl SpringFluidState {
    pub fn create_legacy_block<B: Clone>(&self, block_state: B) -> B {
        block_state
    }

    pub fn get_type(&self) -> i32 {
        self.fluid_type
    }
}

#[derive(Clone, Debug)]
pub struct SpringConfiguration<B: Clone> {
    pub state: SpringFluidState,
    pub valid_blocks: Vec<i32>,
    pub requires_block_below: bool,
    pub rock_count: i32,
    pub hole_count: i32,
    pub placed_block_state: B,
}

pub struct SpringFeature;

impl SpringFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.SpringFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.SpringFeature#place
    pub fn place<B, W>(&self, config: &SpringConfiguration<B>, level: &mut W, origin: BlockPos) -> bool
    where
        B: Clone,
        W: SpringLevel<B>,
    {
        let above_state = level.get_block_state(origin.above());
        if !level.is_valid_block(&above_state, &config.valid_blocks) {
            return false;
        }

        if config.requires_block_below {
            let below_state = level.get_block_state(origin.below());
            if !level.is_valid_block(&below_state, &config.valid_blocks) {
                return false;
            }
        }

        let origin_state = level.get_block_state(origin);
        if !level.is_air(&origin_state) && !level.is_valid_block(&origin_state, &config.valid_blocks) {
            return false;
        }

        let mut placed_count: i32 = 0;
        let mut rock_count: i32 = 0;

        if level.is_valid_block(&level.get_block_state(origin.west()), &config.valid_blocks) {
            rock_count = rock_count.wrapping_add(1);
        }
        if level.is_valid_block(&level.get_block_state(origin.east()), &config.valid_blocks) {
            rock_count = rock_count.wrapping_add(1);
        }
        if level.is_valid_block(&level.get_block_state(origin.north()), &config.valid_blocks) {
            rock_count = rock_count.wrapping_add(1);
        }
        if level.is_valid_block(&level.get_block_state(origin.south()), &config.valid_blocks) {
            rock_count = rock_count.wrapping_add(1);
        }
        if level.is_valid_block(&level.get_block_state(origin.below()), &config.valid_blocks) {
            rock_count = rock_count.wrapping_add(1);
        }

        let mut hole_count: i32 = 0;
        if level.is_empty_block(origin.west()) {
            hole_count = hole_count.wrapping_add(1);
        }
        if level.is_empty_block(origin.east()) {
            hole_count = hole_count.wrapping_add(1);
        }
        if level.is_empty_block(origin.north()) {
            hole_count = hole_count.wrapping_add(1);
        }
        if level.is_empty_block(origin.south()) {
            hole_count = hole_count.wrapping_add(1);
        }
        if level.is_empty_block(origin.below()) {
            hole_count = hole_count.wrapping_add(1);
        }

        if rock_count == config.rock_count && hole_count == config.hole_count {
            let placed_state = config.state.create_legacy_block(config.placed_block_state.clone());
            level.set_block(origin, placed_state, 2);
            level.schedule_tick(origin, config.state.get_type(), 0);
            placed_count = placed_count.wrapping_add(1);
        }

        placed_count > 0
    }
}

impl Default for SpringFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct BlockState {
        id: i32,
        air: bool,
    }

    impl BlockState {
        fn new(id: i32, air: bool) -> Self { Self { id, air } }
    }

    struct TestLevel {
        blocks: HashMap<(i32, i32, i32), BlockState>,
        set_calls: Vec<(BlockPos, BlockState, i32)>,
        ticks: Vec<(BlockPos, i32, i32)>,
    }

    impl TestLevel {
        fn new() -> Self { Self { blocks: HashMap::new(), set_calls: Vec::new(), ticks: Vec::new() } }
        fn with(mut self, pos: BlockPos, state: BlockState) -> Self {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            self
        }
    }

    impl SpringLevel<BlockState> for TestLevel {
        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks.get(&(pos.x, pos.y, pos.z)).cloned().unwrap_or(BlockState::new(0, true))
        }
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get_block_state(pos).air
        }
        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state.clone());
            self.set_calls.push((pos, state, flags));
            true
        }
        fn schedule_tick(&mut self, pos: BlockPos, fluid: i32, delay: i32) {
            self.ticks.push((pos, fluid, delay));
        }
        fn is_air(&self, state: &BlockState) -> bool { state.air }
        fn is_valid_block(&self, state: &BlockState, valid_blocks: &[i32]) -> bool {
            valid_blocks.contains(&state.id)
        }
        fn state_block_id(&self, state: &BlockState) -> i32 { state.id }
    }

    fn base_config() -> SpringConfiguration<BlockState> {
        SpringConfiguration {
            state: SpringFluidState { fluid_type: 7 },
            valid_blocks: vec![1],
            requires_block_below: false,
            rock_count: 4,
            hole_count: 1,
            placed_block_state: BlockState::new(99, false),
        }
    }

    #[test]
    fn rejects_when_above_not_valid() {
        let feature = SpringFeature::new();
        let mut level = TestLevel::new();
        let config = base_config();
        assert!(!feature.place(&config, &mut level, BlockPos::new(0, 64, 0)));
    }

    #[test]
    fn places_when_rock_and_hole_counts_match() {
        let feature = SpringFeature::new();
        let origin = BlockPos::new(0, 64, 0);
        let mut level = TestLevel::new()
            .with(origin.above(), BlockState::new(1, false))
            .with(origin.west(), BlockState::new(1, false))
            .with(origin.east(), BlockState::new(1, false))
            .with(origin.north(), BlockState::new(1, false))
            .with(origin.south(), BlockState::new(1, false))
            .with(origin.below(), BlockState::new(0, true)); // hole
        let config = base_config();
        assert!(feature.place(&config, &mut level, origin));
        assert_eq!(level.set_calls.len(), 1);
        assert_eq!(level.set_calls[0].2, 2);
        assert_eq!(level.ticks, vec![(origin, 7, 0)]);
    }

    #[test]
    fn requires_block_below_when_configured() {
        let feature = SpringFeature::new();
        let origin = BlockPos::new(0, 64, 0);
        let mut level = TestLevel::new()
            .with(origin.above(), BlockState::new(1, false))
            .with(origin.below(), BlockState::new(0, true));
        let mut config = base_config();
        config.requires_block_below = true;
        assert!(!feature.place(&config, &mut level, origin));
    }

    #[test]
    fn rejects_non_air_non_valid_origin_state() {
        let feature = SpringFeature::new();
        let origin = BlockPos::new(0, 64, 0);
        let mut level = TestLevel::new()
            .with(origin.above(), BlockState::new(1, false))
            .with(origin, BlockState::new(2, false));
        let config = base_config();
        assert!(!feature.place(&config, &mut level, origin));
    }

    #[test]
    fn negative_coordinates_work() {
        let feature = SpringFeature::new();
        let origin = BlockPos::new(-10, -5, -20);
        let mut level = TestLevel::new()
            .with(origin.above(), BlockState::new(1, false))
            .with(origin.west(), BlockState::new(1, false))
            .with(origin.east(), BlockState::new(1, false))
            .with(origin.north(), BlockState::new(1, false))
            .with(origin.south(), BlockState::new(1, false))
            .with(origin.below(), BlockState::new(0, true));
        let config = base_config();
        assert!(feature.place(&config, &mut level, origin));
        assert_eq!(level.set_calls[0].0, origin);
    }
}
