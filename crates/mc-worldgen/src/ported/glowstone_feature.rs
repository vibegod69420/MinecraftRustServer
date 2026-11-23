use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#above()
    pub fn above(&self) -> Self {
        Self::new(self.x, self.y.wrapping_add(1), self.z)
    }

    // JAVA: net.minecraft.core.BlockPos#offset(int,int,int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self::new(
            self.x.wrapping_add(dx),
            self.y.wrapping_add(dy),
            self.z.wrapping_add(dz),
        )
    }

    // JAVA: net.minecraft.core.BlockPos#relative(net.minecraft.core.Direction)
    pub fn relative(&self, dir: Direction) -> Self {
        let (dx, dy, dz) = dir.normal();
        self.offset(dx, dy, dz)
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
    pub const VALUES: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    pub const fn normal(self) -> (i32, i32, i32) {
        match self {
            Direction::Down => (0, -1, 0),
            Direction::Up => (0, 1, 0),
            Direction::North => (0, 0, -1),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Netherrack,
    Basalt,
    Blackstone,
    Glowstone,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
}

impl BlockState {
    pub const fn new(kind: BlockKind) -> Self {
        Self { kind }
    }

    pub fn is(&self, kind: BlockKind) -> bool {
        self.kind == kind
    }

    pub fn is_air(&self) -> bool {
        self.kind == BlockKind::Air
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait WorldGenLevel {
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub struct FeaturePlaceContext<'a, L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub random: &'a mut R,
}

pub struct GlowstoneFeature;

impl GlowstoneFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.GlowstoneFeature#<init>(com.mojang.serialization.Codec)
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.GlowstoneFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let origin = ctx.origin;
        if !ctx.level.is_empty_block(origin) {
            return false;
        }

        let above_state = ctx.level.get_block_state(origin.above());
        if !above_state.is(BlockKind::Netherrack)
            && !above_state.is(BlockKind::Basalt)
            && !above_state.is(BlockKind::Blackstone)
        {
            return false;
        }

        ctx.level
            .set_block(origin, BlockState::new(BlockKind::Glowstone), 2);

        for _ in 0..1500 {
            let dx = ctx
                .random
                .next_int(8)
                .wrapping_sub(ctx.random.next_int(8));
            let dy = 0i32.wrapping_sub(ctx.random.next_int(12));
            let dz = ctx
                .random
                .next_int(8)
                .wrapping_sub(ctx.random.next_int(8));
            let pos = origin.offset(dx, dy, dz);
            if ctx.level.get_block_state(pos).is_air() {
                let mut adjacent_glowstone = 0i32;
                for dir in Direction::VALUES {
                    if ctx
                        .level
                        .get_block_state(pos.relative(dir))
                        .is(BlockKind::Glowstone)
                    {
                        adjacent_glowstone = adjacent_glowstone.wrapping_add(1);
                    }
                    if adjacent_glowstone > 1 {
                        break;
                    }
                }

                if adjacent_glowstone == 1 {
                    ctx.level
                        .set_block(pos, BlockState::new(BlockKind::Glowstone), 2);
                }
            }
        }

        true
    }
}

impl Default for GlowstoneFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
        fn zeros() -> Self {
            Self::new(vec![0])
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v.rem_euclid(bound)
        }
    }

    struct MockWorld {
        blocks: HashMap<(i32, i32, i32), BlockState>,
    }

    impl MockWorld {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }

        fn set_kind(&mut self, pos: BlockPos, kind: BlockKind) {
            self.blocks.insert((pos.x, pos.y, pos.z), BlockState::new(kind));
        }

        fn kind_at(&self, pos: BlockPos) -> BlockKind {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState::new(BlockKind::Air))
                .kind
        }
    }

    impl WorldGenLevel for MockWorld {
        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.kind_at(pos) == BlockKind::Air
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            BlockState::new(self.kind_at(pos))
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, _flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            true
        }
    }

    #[test]
    fn returns_false_when_origin_not_empty() {
        let mut level = MockWorld::new();
        let origin = BlockPos::new(0, 64, 0);
        level.set_kind(origin, BlockKind::Glowstone);
        level.set_kind(origin.above(), BlockKind::Netherrack);
        let mut random = SeqRandom::zeros();
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            random: &mut random,
        };
        assert!(!GlowstoneFeature::new().place(&mut ctx));
    }

    #[test]
    fn returns_false_when_above_not_supported() {
        let mut level = MockWorld::new();
        let origin = BlockPos::new(1, 64, 1);
        level.set_kind(origin.above(), BlockKind::Other);
        let mut random = SeqRandom::zeros();
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            random: &mut random,
        };
        assert!(!GlowstoneFeature::new().place(&mut ctx));
        assert_eq!(ctx.level.kind_at(origin), BlockKind::Air);
    }

    #[test]
    fn places_origin_on_valid_support() {
        let mut level = MockWorld::new();
        let origin = BlockPos::new(2, 64, -3);
        level.set_kind(origin.above(), BlockKind::Basalt);
        let mut random = SeqRandom::zeros();
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            random: &mut random,
        };
        assert!(GlowstoneFeature::new().place(&mut ctx));
        assert_eq!(ctx.level.kind_at(origin), BlockKind::Glowstone);
    }

    #[test]
    fn spreads_to_air_block_with_single_glowstone_neighbor() {
        let mut level = MockWorld::new();
        let origin = BlockPos::new(0, 64, 0);
        level.set_kind(origin.above(), BlockKind::Netherrack);
        // First iteration offsets: dx = 0-0=0, dy = -0=0, dz = 1-0=1 => pos (0,64,1)
        // (0,64,1) is air and adjacent to origin glowstone only on North side.
        let mut seq = vec![0, 0, 0, 1, 0];
        seq.extend(std::iter::repeat(0).take(1500 * 6));
        let mut random = SeqRandom::new(seq);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            random: &mut random,
        };
        assert!(GlowstoneFeature::new().place(&mut ctx));
        assert_eq!(ctx.level.kind_at(origin), BlockKind::Glowstone);
        assert_eq!(
            ctx.level.kind_at(BlockPos::new(0, 64, 1)),
            BlockKind::Glowstone
        );
    }

    #[test]
    fn does_not_spread_when_more_than_one_neighbor() {
        let mut level = MockWorld::new();
        let origin = BlockPos::new(0, 64, 0);
        level.set_kind(origin.above(), BlockKind::Blackstone);
        // Pre-place glowstone so target (0,64,1) would have two neighbors: origin and (0,64,2)
        level.set_kind(BlockPos::new(0, 64, 2), BlockKind::Glowstone);
        let mut seq = vec![0, 0, 0, 1, 0];
        seq.extend(std::iter::repeat(0).take(1500 * 6));
        let mut random = SeqRandom::new(seq);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            random: &mut random,
        };
        assert!(GlowstoneFeature::new().place(&mut ctx));
        // Target should remain air because it would touch origin and (0,64,2)
        assert_eq!(ctx.level.kind_at(BlockPos::new(0, 64, 1)), BlockKind::Air);
    }

    #[test]
    fn negative_offset_uses_wrapping_sub_semantics() {
        let mut level = MockWorld::new();
        let origin = BlockPos::new(i32::MIN, 64, 0);
        level.set_kind(origin.above(), BlockKind::Netherrack);
        // dx = nextInt(8)-nextInt(8) = 1-0 = 1, so x wraps from MIN to MIN+1
        let mut seq = vec![1, 0, 0, 0, 0];
        seq.extend(std::iter::repeat(0).take(1500 * 6));
        let mut random = SeqRandom::new(seq);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            origin,
            random: &mut random,
        };
        assert!(GlowstoneFeature::new().place(&mut ctx));
        let expected = BlockPos::new(i32::MIN.wrapping_add(1), 64, 0);
        assert_eq!(ctx.level.kind_at(expected), BlockKind::Glowstone);
    }
}
