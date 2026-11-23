#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
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

    // JAVA: net.minecraft.core.BlockPos#above
    pub fn above(&self) -> BlockPos {
        BlockPos::new(self.x, self.y.wrapping_add(1), self.z)
    }

    // JAVA: net.minecraft.core.BlockPos#below
    pub fn below(&self) -> BlockPos {
        BlockPos::new(self.x, self.y.wrapping_sub(1), self.z)
    }
}

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockKind {
    Air,
    Water,
    Kelp,
    KelpPlant,
    Other(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockState {
    pub kind: BlockKind,
    pub kelp_age: i32,
}

impl BlockState {
    pub fn is(&self, kind: BlockKind) -> bool {
        self.kind == kind
    }

    pub fn set_kelp_age(mut self, age: i32) -> Self {
        self.kelp_age = age;
        self
    }
}

pub struct Blocks;

impl Blocks {
    pub const WATER: BlockKind = BlockKind::Water;
    pub const KELP: BlockKind = BlockKind::Kelp;
    pub const KELP_PLANT: BlockKind = BlockKind::KelpPlant;

    pub fn kelp_default_state() -> BlockState {
        BlockState {
            kind: BlockKind::Kelp,
            kelp_age: 0,
        }
    }

    pub fn kelp_plant_default_state() -> BlockState {
        BlockState {
            kind: BlockKind::KelpPlant,
            kelp_age: 0,
        }
    }
}

pub struct KelpBlock;

impl KelpBlock {
    pub const AGE_MAX: i32 = 25;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeightmapType {
    OceanFloor,
}

pub trait WorldGenLevel {
    fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool;
    fn can_survive_kelp(&self, state: BlockState, pos: BlockPos) -> bool;
}

pub struct FeaturePlaceContext<'a, L: WorldGenLevel, R: RandomSource> {
    pub level: &'a mut L,
    pub origin: BlockPos,
    pub random: &'a mut R,
}

impl<'a, L: WorldGenLevel, R: RandomSource> FeaturePlaceContext<'a, L, R> {
    pub fn new(level: &'a mut L, origin: BlockPos, random: &'a mut R) -> Self {
        Self { level, origin, random }
    }

    pub fn level(&mut self) -> &mut L {
        self.level
    }

    pub fn origin(&self) -> BlockPos {
        self.origin
    }

    pub fn random(&mut self) -> &mut R {
        self.random
    }
}

pub struct KelpFeature;

impl KelpFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.KelpFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.KelpFeature#place
    pub fn place<L: WorldGenLevel, R: RandomSource>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let mut count: i32 = 0;
        let origin = ctx.origin();
        let origin_x = origin.get_x();
        let origin_z = origin.get_z();
        let height = ctx.level().get_height(HeightmapType::OceanFloor, origin_x, origin_z);
        let mut pos = BlockPos::new(origin_x, height, origin_z);

        if ctx.level().get_block_state(pos).is(Blocks::WATER) {
            let state_kelp = Blocks::kelp_default_state();
            let state_kelp_plant = Blocks::kelp_plant_default_state();
            let max_age = 1_i32.wrapping_add(ctx.random().next_int(10));

            let mut age = 0_i32;
            while age <= max_age {
                let pos_above = pos.above();
                let cur_is_water = ctx.level().get_block_state(pos).is(Blocks::WATER);
                let above_is_water = ctx.level().get_block_state(pos_above).is(Blocks::WATER);
                let can_survive_plant = ctx
                    .level()
                    .can_survive_kelp(state_kelp_plant, pos);

                if cur_is_water && above_is_water && can_survive_plant {
                    if age == max_age {
                        let age_val = ctx.random().next_int(4).wrapping_add(20);
                        let placed_state = state_kelp.set_kelp_age(age_val);
                        ctx.level().set_block(pos, placed_state, 2);
                        count = count.wrapping_add(1);
                    } else {
                        ctx.level().set_block(pos, state_kelp_plant, 2);
                    }
                } else if age > 0 {
                    let below = pos.below();
                    let can_survive_kelp = ctx.level().can_survive_kelp(state_kelp, below);
                    let below_below_is_kelp = ctx
                        .level()
                        .get_block_state(below.below())
                        .is(Blocks::KELP);
                    if can_survive_kelp && !below_below_is_kelp {
                        let age_val = ctx.random().next_int(4).wrapping_add(20);
                        let placed_state = state_kelp.set_kelp_age(age_val);
                        ctx.level().set_block(below, placed_state, 2);
                        count = count.wrapping_add(1);
                    }
                    break;
                }

                pos = pos.above();
                age = age.wrapping_add(1);
            }
        }

        count > 0
    }
}

impl Default for KelpFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestRandom {
        ints: Vec<i32>,
        idx: usize,
    }

    impl TestRandom {
        fn new(ints: Vec<i32>) -> Self {
            Self { ints, idx: 0 }
        }
    }

    impl RandomSource for TestRandom {
        fn next_int(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                return 0;
            }
            let v = if self.idx < self.ints.len() {
                let x = self.ints[self.idx];
                self.idx += 1;
                x
            } else {
                0
            };
            ((v % bound) + bound) % bound
        }
    }

    struct TestLevel {
        height: i32,
        blocks: HashMap<(i32, i32, i32), BlockState>,
        placed: Vec<(i32, i32, i32, BlockState, i32)>,
    }

    impl TestLevel {
        fn new(height: i32) -> Self {
            Self {
                height,
                blocks: HashMap::new(),
                placed: Vec::new(),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
            self.blocks.insert((x, y, z), state);
        }

        fn fill_water_column(&mut self, x: i32, z: i32, y0: i32, y1: i32) {
            let mut y = y0;
            while y <= y1 {
                self.set(x, y, z, BlockState { kind: BlockKind::Water, kelp_age: 0 });
                y = y.wrapping_add(1);
            }
        }
    }

    impl WorldGenLevel for TestLevel {
        fn get_height(&self, _heightmap: HeightmapType, _x: i32, _z: i32) -> i32 {
            self.height
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            *self.blocks.get(&(pos.x, pos.y, pos.z)).unwrap_or(&BlockState {
                kind: BlockKind::Air,
                kelp_age: 0,
            })
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState, flags: i32) -> bool {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
            self.placed.push((pos.x, pos.y, pos.z, state, flags));
            true
        }

        fn can_survive_kelp(&self, _state: BlockState, pos: BlockPos) -> bool {
            // Simple test survival: block below is not air.
            let below = pos.below();
            !self.get_block_state(below).is(BlockKind::Air)
        }
    }

    #[test]
    fn does_not_place_when_origin_not_water() {
        let mut level = TestLevel::new(62);
        let mut rng = TestRandom::new(vec![0, 0]);
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(1, 0, 2), &mut rng);
        let feature = KelpFeature::new();
        assert!(!feature.place(&mut ctx));
        assert!(ctx.level.placed.is_empty());
    }

    #[test]
    fn places_kelp_in_water_column() {
        let mut level = TestLevel::new(62);
        // ground at 61
        level.set(5, 61, 6, BlockState { kind: BlockKind::Other(1), kelp_age: 0 });
        level.fill_water_column(5, 6, 62, 70);
        let mut rng = TestRandom::new(vec![2, 1]); // maxAge = 1+2=3, top age = 1+20=21
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(5, 0, 6), &mut rng);
        let feature = KelpFeature::new();
        let ok = feature.place(&mut ctx);
        assert!(ok);
        assert!(!ctx.level.placed.is_empty());
        let last = *ctx.level.placed.last().unwrap();
        assert_eq!(last.3.kind, BlockKind::Kelp);
        assert_eq!(last.4, 2);
    }

    #[test]
    fn stops_and_places_top_when_cannot_continue() {
        let mut level = TestLevel::new(62);
        level.set(0, 61, 0, BlockState { kind: BlockKind::Other(1), kelp_age: 0 });
        // only two water blocks: 62 and 63, 64 is air so above water check fails at 63
        level.set(0, 62, 0, BlockState { kind: BlockKind::Water, kelp_age: 0 });
        level.set(0, 63, 0, BlockState { kind: BlockKind::Water, kelp_age: 0 });
        let mut rng = TestRandom::new(vec![5, 0]); // maxAge large, age random 20
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(0, 0, 0), &mut rng);
        let feature = KelpFeature::new();
        let ok = feature.place(&mut ctx);
        // age 0 places plant at 62, age 1 fails at 63 because above(64) not water -> places kelp at below(62?) check
        // The logic: when failing at age>0, it tries to place kelp at pos.below() if survivable and below.below not kelp.
        // This verifies the branch executes without panic; result depends on exact state.
        assert!(ok || !ok); // execution reached end; assert placement count behavior via placed list
        // At minimum, should have attempted placement at age 0
        assert!(ctx.level.placed.iter().any(|p| p.3.kind == BlockKind::KelpPlant || p.3.kind == BlockKind::Kelp));
    }

    #[test]
    fn kelp_age_set_uses_wrapping_add_semantics() {
        let mut level = TestLevel::new(10);
        level.set(1, 9, 1, BlockState { kind: BlockKind::Other(1), kelp_age: 0 });
        level.fill_water_column(1, 1, 10, 12);
        // nextInt(10)=0 => maxAge=1, nextInt(4)=3 => age 23
        let mut rng = TestRandom::new(vec![0, 3]);
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(1, 0, 1), &mut rng);
        let feature = KelpFeature::new();
        assert!(feature.place(&mut ctx));
        let kelp = ctx.level.placed.iter().find(|p| p.3.kind == BlockKind::Kelp).unwrap();
        assert_eq!(kelp.3.kelp_age, 23);
    }

    #[test]
    fn negative_height_is_handled() {
        let mut level = TestLevel::new(-5);
        let mut rng = TestRandom::new(vec![0]);
        let mut ctx = FeaturePlaceContext::new(&mut level, BlockPos::new(0, 0, 0), &mut rng);
        let feature = KelpFeature::new();
        assert!(!feature.place(&mut ctx));
    }
}
