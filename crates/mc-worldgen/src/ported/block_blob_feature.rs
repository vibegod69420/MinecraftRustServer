#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.Vec3i#getY
    pub fn get_y(&self) -> i32 {
        self.y
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> BlockPos {
        BlockPos {
            x: self.x,
            y: self.y.wrapping_sub(1),
            z: self.z,
        }
    }

    // JAVA: net.minecraft.core.BlockPos#offset(int,int,int)
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> BlockPos {
        if dx == 0 && dy == 0 && dz == 0 {
            return *self;
        }
        BlockPos {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }

    // JAVA: net.minecraft.core.Vec3i#distSqr
    pub fn dist_sqr(&self, other: BlockPos) -> f64 {
        let dx = self.x as f64 - other.x as f64;
        let dy = self.y as f64 - other.y as f64;
        let dz = self.z as f64 - other.z as f64;
        dx * dx + dy * dy + dz * dz
    }

    // JAVA: net.minecraft.core.BlockPos#betweenClosed
    pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
        let min_x = from.x.min(to.x);
        let max_x = from.x.max(to.x);
        let min_y = from.y.min(to.y);
        let max_y = from.y.max(to.y);
        let min_z = from.z.min(to.z);
        let max_z = from.z.max(to.z);

        let mut result = Vec::new();
        let mut x = min_x;
        loop {
            let mut y = min_y;
            loop {
                let mut z = min_z;
                loop {
                    result.push(BlockPos { x, y, z });
                    if z == max_z {
                        break;
                    }
                    z = z.wrapping_add(1);
                }
                if y == max_y {
                    break;
                }
                y = y.wrapping_add(1);
            }
            if x == max_x {
                break;
            }
            x = x.wrapping_add(1);
        }
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u32,
    pub is_air: bool,
    pub is_dirt: bool,
    pub is_stone: bool,
}

impl BlockState {
    pub const AIR: Self = Self {
        id: 0,
        is_air: true,
        is_dirt: false,
        is_stone: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockStateConfiguration {
    pub state: BlockState,
}

pub trait WorldGenLevel {
    // JAVA: net.minecraft.world.level.LevelHeightAccessor#getMinY
    fn get_min_y(&self) -> i32;
    // JAVA: net.minecraft.world.level.LevelReader#isEmptyBlock
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    // JAVA: net.minecraft.world.level.LevelReader#getBlockState
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    // JAVA: net.minecraft.world.level.WorldGenLevel#setBlock
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
}

pub trait RandomSource {
    // JAVA: net.minecraft.util.RandomSource#nextInt(int)
    fn next_int(&mut self, bound: i32) -> i32;
}

pub struct FeaturePlaceContext<'a, W: ?Sized, R: ?Sized> {
    pub origin: BlockPos,
    pub level: &'a mut W,
    pub random: &'a mut R,
    pub config: &'a BlockStateConfiguration,
}

pub struct BlockBlobFeature;

impl BlockBlobFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.BlockBlobFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.Feature#isDirt
    pub fn is_dirt(state: &BlockState) -> bool {
        state.is_dirt
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.Feature#isStone
    pub fn is_stone(state: &BlockState) -> bool {
        state.is_stone
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.BlockBlobFeature#place
    pub fn place<W: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, W, R>,
    ) -> bool {
        let mut pos = ctx.origin;

        while pos.get_y() > ctx.level.get_min_y().wrapping_add(3) {
            let below = pos.below();
            if !ctx.level.is_empty_block(below) {
                let state_below = ctx.level.get_block_state(below);
                if Self::is_dirt(&state_below) || Self::is_stone(&state_below) {
                    break;
                }
            }
            pos = pos.below();
        }

        if pos.get_y() <= ctx.level.get_min_y().wrapping_add(3) {
            return false;
        }

        let mut i: i32 = 0;
        while i < 3 {
            let radius_x = ctx.random.next_int(2);
            let radius_y = ctx.random.next_int(2);
            let radius_z = ctx.random.next_int(2);
            let radius = (radius_x
                .wrapping_add(radius_y)
                .wrapping_add(radius_z)) as f32
                * 0.333_f32
                + 0.5_f32;

            let from = pos.offset(
                radius_x.wrapping_neg(),
                radius_y.wrapping_neg(),
                radius_z.wrapping_neg(),
            );
            let to = pos.offset(radius_x, radius_y, radius_z);
            for p in BlockPos::between_closed(from, to) {
                if p.dist_sqr(pos) <= (radius * radius) as f64 {
                    ctx.level.set_block(p, ctx.config.state, 3);
                }
            }

            pos = pos.offset(
                (-1i32).wrapping_add(ctx.random.next_int(2)),
                0i32.wrapping_sub(ctx.random.next_int(2)),
                (-1i32).wrapping_add(ctx.random.next_int(2)),
            );
            i = i.wrapping_add(1);
        }

        true
    }
}

impl Default for BlockBlobFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockWorld {
        min_y: i32,
        blocks: HashMap<(i32, i32, i32), BlockState>,
        placed: Vec<(BlockPos, BlockState, i32)>,
    }

    impl MockWorld {
        fn new(min_y: i32) -> Self {
            Self {
                min_y,
                blocks: HashMap::new(),
                placed: Vec::new(),
            }
        }

        fn set(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

    impl WorldGenLevel for MockWorld {
        fn get_min_y(&self) -> i32 {
            self.min_y
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            match self.blocks.get(&(pos.x, pos.y, pos.z)) {
                Some(s) => s.is_air,
                None => true,
            }
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState::AIR)
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            self.placed.push((pos, state, flags));
            true
        }
    }

    struct SeqRandom {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRandom {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            ((v % bound) + bound) % bound
        }
    }

    fn dirt() -> BlockState {
        BlockState {
            id: 1,
            is_air: false,
            is_dirt: true,
            is_stone: false,
        }
    }

    fn stone() -> BlockState {
        BlockState {
            id: 2,
            is_air: false,
            is_dirt: false,
            is_stone: true,
        }
    }

    fn solid() -> BlockState {
        BlockState {
            id: 99,
            is_air: false,
            is_dirt: false,
            is_stone: false,
        }
    }

    #[test]
    fn returns_false_when_too_low() {
        let mut world = MockWorld::new(0);
        let mut rng = SeqRandom::new(vec![0; 9]);
        let config = BlockStateConfiguration { state: solid() };
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos { x: 0, y: 3, z: 0 },
            level: &mut world,
            random: &mut rng,
            config: &config,
        };
        assert!(!BlockBlobFeature::new().place(&mut ctx));
        assert!(world.placed.is_empty());
    }

    #[test]
    fn places_blob_on_dirt_and_uses_flag_3() {
        let mut world = MockWorld::new(-64);
        world.set(BlockPos { x: 0, y: 4, z: 0 }, dirt());
        let mut rng = SeqRandom::new(vec![0; 9]);
        let config = BlockStateConfiguration { state: solid() };
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos { x: 0, y: 5, z: 0 },
            level: &mut world,
            random: &mut rng,
            config: &config,
        };
        assert!(BlockBlobFeature::new().place(&mut ctx));
        assert!(!world.placed.is_empty());
        for (_, s, flag) in &world.placed {
            assert_eq!(*s, solid());
            assert_eq!(*flag, 3);
        }
    }

    #[test]
    fn places_blob_on_stone_with_negative_coords() {
        let mut world = MockWorld::new(-64);
        world.set(BlockPos { x: -5, y: -10, z: 7 }, stone());
        let mut rng = SeqRandom::new(vec![1, 0, 1, 0, 1, 0, 1, 0, 1]);
        let config = BlockStateConfiguration { state: solid() };
        let mut ctx = FeaturePlaceContext {
            origin: BlockPos { x: -5, y: -9, z: 7 },
            level: &mut world,
            random: &mut rng,
            config: &config,
        };
        assert!(BlockBlobFeature::new().place(&mut ctx));
        assert!(!world.placed.is_empty());
    }

    #[test]
    fn radius_float_bits_match_java_order() {
        let r0 = (0_i32.wrapping_add(0).wrapping_add(0)) as f32 * 0.333_f32 + 0.5_f32;
        assert_eq!(r0.to_bits(), (0.0_f32 * 0.333_f32 + 0.5_f32).to_bits());

        let r3 = (1_i32.wrapping_add(1).wrapping_add(1)) as f32 * 0.333_f32 + 0.5_f32;
        assert_eq!(r3.to_bits(), (3.0_f32 * 0.333_f32 + 0.5_f32).to_bits());

        let r2 = (1_i32.wrapping_add(0).wrapping_add(1)) as f32 * 0.333_f32 + 0.5_f32;
        assert_eq!(r2.to_bits(), (2.0_f32 * 0.333_f32 + 0.5_f32).to_bits());
    }

    #[test]
    fn offset_and_below_wrap_on_overflow() {
        let p = BlockPos {
            x: i32::MAX,
            y: i32::MIN,
            z: -1,
        };
        let o = p.offset(1, -1, -1);
        assert_eq!(o.x, i32::MIN);
        assert_eq!(o.y, i32::MAX);
        assert_eq!(o.z, -2);
        assert_eq!(p.below().y, i32::MAX);
    }

    #[test]
    fn dist_sqr_and_between_closed_basic() {
        let a = BlockPos { x: 0, y: 0, z: 0 };
        let b = BlockPos { x: 1, y: 2, z: 3 };
        assert_eq!(a.dist_sqr(b), 14.0_f64);

        let pts = BlockPos::between_closed(
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 1, y: 0, z: 1 },
        );
        assert_eq!(pts.len(), 4);
    }
}
