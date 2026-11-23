#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl MutableBlockPos {
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

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapType {
    MotionBlocking,
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

impl<R: RandomSource + ?Sized> RandomSource for &mut R {
    fn next_int(&mut self, bound: i32) -> i32 {
        (*self).next_int(bound)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Water,
    Lava,
    Bedrock,
    Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState {
    kind: BlockKind,
}

impl BlockState {
    pub fn of(kind: BlockKind) -> Self {
        Self { kind }
    }

    pub fn is_air(&self) -> bool {
        self.kind == BlockKind::Air
    }

    pub fn is(&self, block: BlockKind) -> bool {
        self.kind == block
    }
}

pub struct Blocks;

impl Blocks {
    pub const WATER: BlockKind = BlockKind::Water;
    pub const LAVA: BlockKind = BlockKind::Lava;
    pub const BEDROCK: BlockKind = BlockKind::Bedrock;
}

pub trait PlacementContext {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn get_min_y(&self) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntProvider {
    Constant(i32),
}

impl IntProvider {
    pub fn sample(&self, _random: &mut impl RandomSource) -> i32 {
        match self {
            Self::Constant(v) => *v,
        }
    }
}

pub struct ConstantInt;

impl ConstantInt {
    pub fn of(value: i32) -> IntProvider {
        IntProvider::Constant(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacementModifierType {
    pub name: &'static str,
}

impl PlacementModifierType {
    pub const COUNT_ON_EVERY_LAYER: Self = Self {
        name: "count_on_every_layer",
    };
}

/// Port of `net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement`.
#[derive(Debug, Clone)]
pub struct CountOnEveryLayerPlacement {
    count: IntProvider,
}

impl CountOnEveryLayerPlacement {
    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#<init>
    fn new(count: IntProvider) -> Self {
        Self { count }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#of
    pub fn of_provider(count: IntProvider) -> Self {
        Self::new(count)
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#of
    pub fn of(count: i32) -> Self {
        Self::of_provider(ConstantInt::of(count))
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#getPositions
    pub fn get_positions(
        &self,
        ctx: &impl PlacementContext,
        random: &mut impl RandomSource,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let mut positions: Vec<BlockPos> = Vec::new();
        let mut layer: i32 = 0;

        loop {
            let mut found = false;
            let mut i: i32 = 0;
            while i < self.count.sample(random) {
                let x = random.next_int(16).wrapping_add(pos.get_x());
                let z = random.next_int(16).wrapping_add(pos.get_z());
                let height = ctx.get_height(HeightmapType::MotionBlocking, x, z);
                let y = Self::find_on_ground_y_position(ctx, x, height, z, layer);
                if y != i32::MAX {
                    positions.push(BlockPos::new(x, y, z));
                    found = true;
                }
                i = i.wrapping_add(1);
            }

            layer = layer.wrapping_add(1);
            if !found {
                break;
            }
        }

        positions
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#type
    pub fn placement_type(&self) -> PlacementModifierType {
        PlacementModifierType::COUNT_ON_EVERY_LAYER
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#findOnGroundYPosition
    fn find_on_ground_y_position(
        ctx: &impl PlacementContext,
        x: i32,
        height: i32,
        z: i32,
        target_layer: i32,
    ) -> i32 {
        let mut pos = MutableBlockPos::new(x, height, z);
        let mut found_layers: i32 = 0;
        let mut above_state = ctx.get_block_state(pos.as_block_pos());

        let min_y_plus_one = ctx.get_min_y().wrapping_add(1);
        let mut y = height;
        while y >= min_y_plus_one {
            pos.set_y(y.wrapping_sub(1));
            let below_state = ctx.get_block_state(pos.as_block_pos());
            if !Self::is_empty(&below_state)
                && Self::is_empty(&above_state)
                && !below_state.is(Blocks::BEDROCK)
            {
                if found_layers == target_layer {
                    return pos.get_y().wrapping_add(1);
                }
                found_layers = found_layers.wrapping_add(1);
            }
            above_state = below_state;
            y = y.wrapping_sub(1);
        }

        i32::MAX
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#isEmpty
    fn is_empty(state: &BlockState) -> bool {
        state.is_air() || state.is(Blocks::WATER) || state.is(Blocks::LAVA)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: Vec<i32>,
        index: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.index % self.values.len()];
            self.index += 1;
            v.rem_euclid(bound)
        }
    }

    struct FlatContext {
        min_y: i32,
        height: i32,
        ground_y: i32,
    }

    impl PlacementContext for FlatContext {
        fn get_height(&self, _heightmap: HeightmapType, _x: i32, _z: i32) -> i32 {
            self.height
        }

        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            if pos.get_y() <= self.ground_y {
                BlockState::of(BlockKind::Solid)
            } else {
                BlockState::of(BlockKind::Air)
            }
        }
    }

    struct TwoLayerContext;

    impl PlacementContext for TwoLayerContext {
        fn get_height(&self, _heightmap: HeightmapType, _x: i32, _z: i32) -> i32 {
            10
        }

        fn get_min_y(&self) -> i32 {
            0
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            let y = pos.get_y();
            if y == 2 || y == 5 {
                BlockState::of(BlockKind::Solid)
            } else {
                BlockState::of(BlockKind::Air)
            }
        }
    }

    #[test]
    fn finds_position_on_flat_ground() {
        let ctx = FlatContext {
            min_y: 0,
            height: 80,
            ground_y: 64,
        };
        let mut rng = SeqRandom::new(vec![0, 0]);
        let placement = CountOnEveryLayerPlacement::of(1);
        let positions = placement.get_positions(&ctx, &mut rng, BlockPos::new(10, 0, 20));
        assert_eq!(positions, vec![BlockPos::new(10, 65, 20)]);
        assert_eq!(
            placement.placement_type(),
            PlacementModifierType::COUNT_ON_EVERY_LAYER
        );
    }

    #[test]
    fn no_position_when_all_air() {
        struct AllAirContext;
        impl PlacementContext for AllAirContext {
            fn get_height(&self, _: HeightmapType, _: i32, _: i32) -> i32 {
                100
            }
            fn get_min_y(&self) -> i32 {
                -64
            }
            fn get_block_state(&self, _: BlockPos) -> BlockState {
                BlockState::of(BlockKind::Air)
            }
        }

        let ctx = AllAirContext;
        let mut rng = SeqRandom::new(vec![0, 0]);
        let placement = CountOnEveryLayerPlacement::of(1);
        let positions = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 0, 0));
        assert!(positions.is_empty());
    }

    #[test]
    fn bedrock_ground_is_excluded() {
        struct BedrockGroundContext;
        impl PlacementContext for BedrockGroundContext {
            fn get_height(&self, _: HeightmapType, _: i32, _: i32) -> i32 {
                10
            }
            fn get_min_y(&self) -> i32 {
                0
            }
            fn get_block_state(&self, pos: BlockPos) -> BlockState {
                if pos.get_y() <= 5 {
                    BlockState::of(BlockKind::Bedrock)
                } else {
                    BlockState::of(BlockKind::Air)
                }
            }
        }

        let ctx = BedrockGroundContext;
        let mut rng = SeqRandom::new(vec![0, 0]);
        let placement = CountOnEveryLayerPlacement::of(1);
        let positions = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 0, 0));
        assert!(positions.is_empty());
    }

    #[test]
    fn finds_multiple_layers() {
        let ctx = TwoLayerContext;
        let mut rng = SeqRandom::new(vec![1, 2]);
        let placement = CountOnEveryLayerPlacement::of(1);
        let positions = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 0, 0));
        assert_eq!(
            positions,
            vec![BlockPos::new(1, 6, 2), BlockPos::new(1, 3, 2)]
        );
    }

    #[test]
    fn water_counts_as_empty_for_placement() {
        struct WaterAboveContext;
        impl PlacementContext for WaterAboveContext {
            fn get_height(&self, _: HeightmapType, _: i32, _: i32) -> i32 {
                10
            }
            fn get_min_y(&self) -> i32 {
                0
            }
            fn get_block_state(&self, pos: BlockPos) -> BlockState {
                if pos.get_y() <= 3 {
                    BlockState::of(BlockKind::Solid)
                } else if pos.get_y() <= 6 {
                    BlockState::of(BlockKind::Water)
                } else {
                    BlockState::of(BlockKind::Air)
                }
            }
        }

        let ctx = WaterAboveContext;
        let mut rng = SeqRandom::new(vec![0, 0]);
        let placement = CountOnEveryLayerPlacement::of(1);
        let positions = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 0, 0));
        assert_eq!(positions, vec![BlockPos::new(0, 4, 0)]);
    }

    #[test]
    fn zero_count_produces_no_positions() {
        let ctx = TwoLayerContext;
        let mut rng = SeqRandom::new(vec![0, 0]);
        let placement = CountOnEveryLayerPlacement::of(0);
        let positions = placement.get_positions(&ctx, &mut rng, BlockPos::new(0, 0, 0));
        assert!(positions.is_empty());
    }
}
