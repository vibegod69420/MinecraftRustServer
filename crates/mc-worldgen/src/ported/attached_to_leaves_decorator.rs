use std::collections::HashSet;
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// Minimal self-contained support types
// ---------------------------------------------------------------------------

/// Minimal block position, equivalent to `net.minecraft.core.BlockPos`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    // JAVA: net.minecraft.core.BlockPos#relative(Direction)
    pub fn relative(self, dir: Direction) -> Self {
        self.relative_dist(dir, 1)
    }

    // JAVA: net.minecraft.core.BlockPos#relative(Direction, int)
    pub fn relative_dist(self, dir: Direction, dist: i32) -> Self {
        if dist == 0 {
            return self;
        }
        Self {
            x: self.x.wrapping_add(dir.step_x().wrapping_mul(dist)),
            y: self.y.wrapping_add(dir.step_y().wrapping_mul(dist)),
            z: self.z.wrapping_add(dir.step_z().wrapping_mul(dist)),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#offset(int, int, int)
    pub fn offset(self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }

    // JAVA: net.minecraft.core.BlockPos#immutable
    pub fn immutable(self) -> Self {
        self
    }
}

// JAVA: net.minecraft.core.BlockPos#betweenClosed(BlockPos, BlockPos)
pub fn between_closed(from: BlockPos, to: BlockPos) -> Vec<BlockPos> {
    let min_x = from.x.min(to.x);
    let min_y = from.y.min(to.y);
    let min_z = from.z.min(to.z);
    let max_x = from.x.max(to.x);
    let max_y = from.y.max(to.y);
    let max_z = from.z.max(to.z);

    let mut out = Vec::new();
    let mut x = min_x;
    loop {
        let mut y = min_y;
        loop {
            let mut z = min_z;
            loop {
                out.push(BlockPos { x, y, z });
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
    out
}

/// Minimal direction enum, equivalent to `net.minecraft.core.Direction`.
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
    #[inline]
    pub fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    #[inline]
    pub fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    #[inline]
    pub fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }
}

/// Minimal random source, equivalent to `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn next_float(&mut self) -> f32;
    fn next_int(&mut self, bound: i32) -> i32;
}

// JAVA: net.minecraft.Util#shuffle
fn shuffle<T, R: RandomSource>(list: &mut [T], random: &mut R) {
    let mut i = list.len();
    while i > 1 {
        i -= 1;
        let j = random.next_int((i as i32).wrapping_add(1)) as usize;
        list.swap(i, j);
    }
}

// JAVA: net.minecraft.Util#shuffledCopy
pub fn shuffled_copy<T: Clone, R: RandomSource>(list: &[T], random: &mut R) -> Vec<T> {
    let mut out = list.to_vec();
    shuffle(&mut out, random);
    out
}

// JAVA: net.minecraft.Util#getRandom
pub fn get_random<T: Clone, R: RandomSource>(list: &[T], random: &mut R) -> T {
    let idx = random.next_int(list.len() as i32) as usize;
    list[idx].clone()
}

/// Minimal block state provider, equivalent to
/// `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider`.
pub trait BlockStateProvider<B, R: RandomSource> {
    fn get_state(&self, random: &mut R, pos: BlockPos) -> B;
}

/// Minimal tree decorator context, equivalent to
/// `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator.Context`.
pub trait TreeDecoratorContext<B, R: RandomSource> {
    fn rng(&mut self) -> &mut R;
    fn leaves(&self) -> Vec<BlockPos>;
    fn is_air(&self, pos: BlockPos) -> bool;
    fn set_block(&mut self, pos: BlockPos, state: B);
}

/// Minimal tree decorator type, equivalent to
/// `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecoratorType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeDecoratorType {
    AttachedToLeaves,
}

// ---------------------------------------------------------------------------
// Ported class
// ---------------------------------------------------------------------------

/// Port of `net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator`.
pub struct AttachedToLeavesDecorator<B, P> {
    pub probability: f32,
    pub exclusion_radius_xz: i32,
    pub exclusion_radius_y: i32,
    pub block_provider: P,
    pub required_empty_blocks: i32,
    pub directions: Vec<Direction>,
    _phantom: PhantomData<fn() -> B>,
}

impl<B, P> AttachedToLeavesDecorator<B, P> {
    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator#<init>
    pub fn new(
        probability: f32,
        exclusion_radius_xz: i32,
        exclusion_radius_y: i32,
        block_provider: P,
        required_empty_blocks: i32,
        directions: Vec<Direction>,
    ) -> Self {
        Self {
            probability,
            exclusion_radius_xz,
            exclusion_radius_y,
            block_provider,
            required_empty_blocks,
            directions,
            _phantom: PhantomData,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator#place
    pub fn place<R, C>(&self, ctx: &mut C)
    where
        R: RandomSource,
        C: TreeDecoratorContext<B, R>,
        P: BlockStateProvider<B, R>,
    {
        let mut exclusion: HashSet<BlockPos> = HashSet::new();
        let leaves = {
            let leaves_list = ctx.leaves();
            let rng = ctx.rng();
            shuffled_copy(&leaves_list, rng)
        };

        for leaf in leaves {
            let dir = {
                let rng = ctx.rng();
                get_random(&self.directions, rng)
            };
            let target = leaf.relative(dir);

            let roll = ctx.rng().next_float();
            if !exclusion.contains(&target)
                && roll < self.probability
                && self.has_required_empty_blocks::<R, C>(ctx, leaf, dir)
            {
                let from = target.offset(
                    0i32.wrapping_sub(self.exclusion_radius_xz),
                    0i32.wrapping_sub(self.exclusion_radius_y),
                    0i32.wrapping_sub(self.exclusion_radius_xz),
                );
                let to = target.offset(
                    self.exclusion_radius_xz,
                    self.exclusion_radius_y,
                    self.exclusion_radius_xz,
                );

                for pos in between_closed(from, to) {
                    exclusion.insert(pos.immutable());
                }

                let state = {
                    let rng = ctx.rng();
                    self.block_provider.get_state(rng, target)
                };
                ctx.set_block(target, state);
            }
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator#hasRequiredEmptyBlocks
    fn has_required_empty_blocks<R, C>(&self, ctx: &C, pos: BlockPos, dir: Direction) -> bool
    where
        R: RandomSource,
        C: TreeDecoratorContext<B, R>,
    {
        let mut i: i32 = 1;
        while i <= self.required_empty_blocks {
            let check_pos = pos.relative_dist(dir, i);
            if !ctx.is_air(check_pos) {
                return false;
            }
            i = i.wrapping_add(1);
        }
        true
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator#type
    pub fn decorator_type(&self) -> TreeDecoratorType {
        TreeDecoratorType::AttachedToLeaves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        floats: Vec<f32>,
        ints: Vec<i32>,
        fi: usize,
        ii: usize,
    }

    impl SeqRandom {
        fn new(floats: Vec<f32>, ints: Vec<i32>) -> Self {
            Self {
                floats,
                ints,
                fi: 0,
                ii: 0,
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.fi).copied().unwrap_or(0.0);
            self.fi += 1;
            v
        }

        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.ii).copied().unwrap_or(0);
            self.ii += 1;
            if bound <= 0 {
                0
            } else {
                ((v % bound) + bound) % bound
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestState(u32);

    struct ConstProvider(u32);

    impl BlockStateProvider<TestState, SeqRandom> for ConstProvider {
        fn get_state(&self, _random: &mut SeqRandom, _pos: BlockPos) -> TestState {
            TestState(self.0)
        }
    }

    struct TestCtx {
        rng: SeqRandom,
        leaves_list: Vec<BlockPos>,
        air: HashSet<BlockPos>,
        placed: Vec<(BlockPos, TestState)>,
    }

    impl TreeDecoratorContext<TestState, SeqRandom> for TestCtx {
        fn rng(&mut self) -> &mut SeqRandom {
            &mut self.rng
        }

        fn leaves(&self) -> Vec<BlockPos> {
            self.leaves_list.clone()
        }

        fn is_air(&self, pos: BlockPos) -> bool {
            self.air.contains(&pos)
        }

        fn set_block(&mut self, pos: BlockPos, state: TestState) {
            self.placed.push((pos, state));
        }
    }

    #[test]
    fn places_block_when_conditions_met() {
        let leaf = BlockPos { x: 0, y: 5, z: 0 };
        let target = BlockPos { x: 0, y: 6, z: 0 };
        let mut ctx = TestCtx {
            rng: SeqRandom::new(vec![0.0], vec![0]),
            leaves_list: vec![leaf],
            air: [target].into_iter().collect(),
            placed: Vec::new(),
        };
        let dec = AttachedToLeavesDecorator::new(
            0.5,
            1,
            1,
            ConstProvider(42),
            1,
            vec![Direction::Up],
        );
        dec.place(&mut ctx);
        assert_eq!(ctx.placed, vec![(target, TestState(42))]);
        assert_eq!(dec.decorator_type(), TreeDecoratorType::AttachedToLeaves);
    }

    #[test]
    fn probability_is_strict_and_exclusion_prevents_overlap() {
        let leaf = BlockPos { x: 0, y: 5, z: 0 };
        let target = BlockPos { x: 0, y: 6, z: 0 };
        // Shuffle for 2 elements consumes 1 int; each get_random consumes 1 int.
        let mut ctx = TestCtx {
            rng: SeqRandom::new(vec![0.5, 0.0], vec![1, 0, 0]),
            leaves_list: vec![leaf, leaf],
            air: [target].into_iter().collect(),
            placed: Vec::new(),
        };
        let dec = AttachedToLeavesDecorator::new(
            0.5,
            0,
            0,
            ConstProvider(1),
            1,
            vec![Direction::Up],
        );
        dec.place(&mut ctx);
        // 0.5 < 0.5 is false; 0.0 < 0.5 is true but target is excluded after first successful? No.
        // First leaf: roll 0.5, strict < fails, no placement, no exclusion added.
        // Second leaf: roll 0.0, passes, places.
        assert_eq!(ctx.placed, vec![(target, TestState(1))]);
    }

    #[test]
    fn requires_empty_blocks_in_direction() {
        let leaf = BlockPos { x: 0, y: 5, z: 0 };
        let up1 = BlockPos { x: 0, y: 6, z: 0 };
        let mut ctx = TestCtx {
            rng: SeqRandom::new(vec![0.0], vec![0]),
            leaves_list: vec![leaf],
            air: [up1].into_iter().collect(), // up2 missing
            placed: Vec::new(),
        };
        let dec = AttachedToLeavesDecorator::new(
            1.0,
            0,
            0,
            ConstProvider(7),
            2,
            vec![Direction::Up],
        );
        dec.place(&mut ctx);
        assert!(ctx.placed.is_empty());
    }

    #[test]
    fn handles_negative_coords_and_wrapping_offsets() {
        let leaf = BlockPos {
            x: -10,
            y: -5,
            z: -20,
        };
        let target = leaf.relative(Direction::Down);
        assert_eq!(target.y, -6);

        let mut ctx = TestCtx {
            rng: SeqRandom::new(vec![0.0], vec![0]),
            leaves_list: vec![leaf],
            air: [target].into_iter().collect(),
            placed: Vec::new(),
        };
        let dec = AttachedToLeavesDecorator::new(
            1.0,
            1,
            1,
            ConstProvider(9),
            1,
            vec![Direction::Down],
        );
        dec.place(&mut ctx);
        assert_eq!(ctx.placed, vec![(target, TestState(9))]);

        // Explicit wrapping checks for BlockPos helpers.
        let p = BlockPos {
            x: i32::MAX,
            y: i32::MIN,
            z: -1,
        };
        let o = p.offset(1, -1, i32::MIN);
        assert_eq!(o.x, i32::MIN);
        assert_eq!(o.y, i32::MAX);
        assert_eq!(o.z, (-1i32).wrapping_add(i32::MIN));

        let r = BlockPos {
            x: i32::MAX,
            y: 0,
            z: 0,
        }
        .relative_dist(Direction::East, 2);
        assert_eq!(r.x, i32::MIN.wrapping_add(1));
    }

    #[test]
    fn probability_comparison_uses_f32_bits() {
        let leaf = BlockPos { x: 0, y: 0, z: 0 };
        let target = BlockPos { x: 1, y: 0, z: 0 };

        let nan = f32::from_bits(0x7FC0_0001);
        let mut ctx = TestCtx {
            rng: SeqRandom::new(vec![0.0], vec![0]),
            leaves_list: vec![leaf],
            air: [target].into_iter().collect(),
            placed: Vec::new(),
        };
        let dec = AttachedToLeavesDecorator::new(nan, 0, 0, ConstProvider(1), 1, vec![Direction::East]);
        dec.place(&mut ctx);
        assert!(ctx.placed.is_empty());

        let neg_zero = f32::from_bits(0x8000_0000);
        let mut ctx2 = TestCtx {
            rng: SeqRandom::new(vec![0.0], vec![0]),
            leaves_list: vec![leaf],
            air: [target].into_iter().collect(),
            placed: Vec::new(),
        };
        let dec2 = AttachedToLeavesDecorator::new(
            neg_zero,
            0,
            0,
            ConstProvider(2),
            1,
            vec![Direction::East],
        );
        dec2.place(&mut ctx2);
        assert!(ctx2.placed.is_empty());
    }

    #[test]
    fn exclusion_radius_prevents_nearby_placement() {
        let leaf1 = BlockPos { x: 0, y: 0, z: 0 };
        let leaf2 = BlockPos { x: 1, y: 0, z: 0 };
        let t1 = BlockPos { x: 0, y: 1, z: 0 };
        let t2 = BlockPos { x: 1, y: 1, z: 0 };
        // Shuffle for 2 elements: 1 int. get_random for each leaf: 1 int.
        let mut ctx = TestCtx {
            rng: SeqRandom::new(vec![0.0, 0.0], vec![1, 0, 0]),
            leaves_list: vec![leaf1, leaf2],
            air: [t1, t2].into_iter().collect(),
            placed: Vec::new(),
        };
        let dec = AttachedToLeavesDecorator::new(
            1.0,
            1,
            0,
            ConstProvider(3),
            1,
            vec![Direction::Up],
        );
        dec.place(&mut ctx);
        assert_eq!(ctx.placed, vec![(t1, TestState(3))]);
    }
}
