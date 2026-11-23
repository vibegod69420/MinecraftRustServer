//! Port of `net.minecraft.world.level.levelgen.feature.IceSpikeFeature` (Minecraft Java 1.21.11).

use std::fmt;

/// Minimal block identifier for this self-contained port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air,
    SnowBlock,
    Ice,
    PackedIce,
    Dirt,
    Other,
}

/// Minimal block state for this self-contained port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub block: Block,
}

impl BlockState {
    pub const fn new(block: Block) -> Self {
        Self { block }
    }

    pub fn is_air(&self) -> bool {
        self.block == Block::Air
    }

    pub fn is(&self, block: Block) -> bool {
        self.block == block
    }
}

/// Minimal block position with Java `int` wrapping semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn below(&self) -> Self {
        self.below_n(1)
    }

    pub fn below_n(&self, n: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_sub(n),
            z: self.z,
        }
    }

    pub fn above(&self, n: i32) -> Self {
        Self {
            x: self.x,
            y: self.y.wrapping_add(n),
            z: self.z,
        }
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Minimal random source matching the Java methods used by `IceSpikeFeature`.
pub trait RandomSource {
    /// Returns a value in `[0, bound)`, like Java `RandomSource.nextInt(int)`.
    fn next_int(&mut self, bound: i32) -> i32;
    /// Returns a value in `[0.0, 1.0)`, like Java `RandomSource.nextFloat()`.
    fn next_float(&mut self) -> f32;
}

/// Minimal world/level access required by `IceSpikeFeature`.
pub trait WorldGenLevel {
    fn min_y(&self) -> i32;
    fn is_empty_block(&self, pos: BlockPos) -> bool;
    fn get_block_state(&self, pos: BlockPos) -> BlockState;
    fn set_block(&mut self, pos: BlockPos, state: BlockState);
}

// JAVA: net.minecraft.util.Mth#ceil(float)
fn mth_ceil(value: f32) -> i32 {
    let i = value as i32;
    if value > i as f32 {
        i.wrapping_add(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#abs(int)
fn mth_abs_int(a: i32) -> i32 {
    if a >= 0 {
        a
    } else {
        a.wrapping_neg()
    }
}

// JAVA: net.minecraft.world.level.levelgen.feature.Feature#isDirt
fn is_dirt(state: BlockState) -> bool {
    state.block == Block::Dirt
}

/// Port of `net.minecraft.world.level.levelgen.feature.IceSpikeFeature`.
pub struct IceSpikeFeature;

impl IceSpikeFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.IceSpikeFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        level: &mut L,
        random: &mut R,
        origin: BlockPos,
    ) -> bool {
        let mut pos = origin;

        while level.is_empty_block(pos) && pos.get_y() > level.min_y().wrapping_add(2) {
            pos = pos.below();
        }

        if !level.get_block_state(pos).is(Block::SnowBlock) {
            return false;
        }

        pos = pos.above(random.next_int(4));
        let height = random.next_int(4).wrapping_add(7);
        let radius = (height / 4).wrapping_add(random.next_int(2));
        if radius > 1 && random.next_int(60) == 0 {
            pos = pos.above(random.next_int(30).wrapping_add(10));
        }

        let mut y_off: i32 = 0;
        while y_off < height {
            let f = (1.0f32 - (y_off as f32) / (height as f32)) * (radius as f32);
            let r = mth_ceil(f);

            let mut dx = r.wrapping_neg();
            while dx <= r {
                let f_dx = (mth_abs_int(dx) as f32) - 0.25f32;
                let mut dz = r.wrapping_neg();
                while dz <= r {
                    let f_dz = (mth_abs_int(dz) as f32) - 0.25f32;

                    let on_axis = dx == 0 && dz == 0;
                    let in_circle = !((f_dx * f_dx + f_dz * f_dz) > f * f);
                    if on_axis || in_circle {
                        let on_edge = dx == r.wrapping_neg()
                            || dx == r
                            || dz == r.wrapping_neg()
                            || dz == r;
                        let edge_pass = !on_edge || !(random.next_float() > 0.75f32);
                        if edge_pass {
                            let p = pos.offset(dx, y_off, dz);
                            let state = level.get_block_state(p);
                            if state.is_air()
                                || is_dirt(state)
                                || state.is(Block::SnowBlock)
                                || state.is(Block::Ice)
                            {
                                level.set_block(p, BlockState::new(Block::PackedIce));
                            }

                            if y_off != 0 && r > 1 {
                                let p2 = pos.offset(dx, 0i32.wrapping_sub(y_off), dz);
                                let state2 = level.get_block_state(p2);
                                if state2.is_air()
                                    || is_dirt(state2)
                                    || state2.is(Block::SnowBlock)
                                    || state2.is(Block::Ice)
                                {
                                    level.set_block(p2, BlockState::new(Block::PackedIce));
                                }
                            }
                        }
                    }

                    dz = dz.wrapping_add(1);
                }
                dx = dx.wrapping_add(1);
            }

            y_off = y_off.wrapping_add(1);
        }

        let mut base_r = radius.wrapping_sub(1);
        if base_r < 0 {
            base_r = 0;
        } else if base_r > 1 {
            base_r = 1;
        }

        let mut bx = base_r.wrapping_neg();
        while bx <= base_r {
            let mut bz = base_r.wrapping_neg();
            while bz <= base_r {
                let mut p = pos.offset(bx, -1, bz);
                let mut fill_depth: i32 = 50;
                if mth_abs_int(bx) == 1 && mth_abs_int(bz) == 1 {
                    fill_depth = random.next_int(5);
                }

                while p.get_y() > 50 {
                    let st = level.get_block_state(p);
                    if !st.is_air()
                        && !is_dirt(st)
                        && !st.is(Block::SnowBlock)
                        && !st.is(Block::Ice)
                        && !st.is(Block::PackedIce)
                    {
                        break;
                    }

                    level.set_block(p, BlockState::new(Block::PackedIce));
                    p = p.below();
                    fill_depth = fill_depth.wrapping_sub(1);
                    if fill_depth <= 0 {
                        p = p.below_n(random.next_int(5).wrapping_add(1));
                        fill_depth = random.next_int(5);
                    }
                }

                bz = bz.wrapping_add(1);
            }
            bx = bx.wrapping_add(1);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockLevel {
        min_y: i32,
        blocks: HashMap<(i32, i32, i32), BlockState>,
    }

    impl MockLevel {
        fn new(min_y: i32) -> Self {
            Self {
                min_y,
                blocks: HashMap::new(),
            }
        }

        fn set(&mut self, x: i32, y: i32, z: i32, block: Block) {
            self.blocks.insert((x, y, z), BlockState::new(block));
        }

        fn get(&self, x: i32, y: i32, z: i32) -> Block {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockState::new(Block::Air))
                .block
        }
    }

    impl WorldGenLevel for MockLevel {
        fn min_y(&self) -> i32 {
            self.min_y
        }

        fn is_empty_block(&self, pos: BlockPos) -> bool {
            self.get_block_state(pos).is_air()
        }

        fn get_block_state(&self, pos: BlockPos) -> BlockState {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .copied()
                .unwrap_or(BlockState::new(Block::Air))
        }

        fn set_block(&mut self, pos: BlockPos, state: BlockState) {
            self.blocks.insert((pos.x, pos.y, pos.z), state);
        }
    }

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
        fn next_int(&mut self, bound: i32) -> i32 {
            let v = self.ints.get(self.int_idx).copied().unwrap_or(0);
            self.int_idx += 1;
            if bound <= 0 {
                0
            } else {
                v.rem_euclid(bound)
            }
        }

        fn next_float(&mut self) -> f32 {
            let v = self.floats.get(self.float_idx).copied().unwrap_or(0.0);
            self.float_idx += 1;
            v
        }
    }

    #[test]
    fn returns_false_when_not_on_snow() {
        let mut level = MockLevel::new(-64);
        level.set(0, 64, 0, Block::Dirt);
        let mut rng = SeqRandom::new(vec![], vec![]);
        let ok = IceSpikeFeature.place(&mut level, &mut rng, BlockPos::new(0, 70, 0));
        assert!(!ok);
    }

    #[test]
    fn places_packed_ice_spike_on_snow() {
        let mut level = MockLevel::new(-64);
        for x in -5..=5 {
            for z in -5..=5 {
                level.set(x, 64, z, Block::SnowBlock);
                for y in 55..64 {
                    level.set(x, y, z, Block::Dirt);
                }
            }
        }
        let mut rng = SeqRandom::new(vec![0, 0, 0], vec![0.0; 512]);
        let ok = IceSpikeFeature.place(&mut level, &mut rng, BlockPos::new(0, 65, 0));
        assert!(ok);
        let has_packed_ice = (-3..=3).any(|x| {
            (-3..=3).any(|z| (60..80).any(|y| level.get(x, y, z) == Block::PackedIce))
        });
        assert!(has_packed_ice);
    }

    #[test]
    fn mth_ceil_matches_java_edge_cases() {
        assert_eq!(mth_ceil(0.0), 0);
        assert_eq!(mth_ceil(1.0), 1);
        assert_eq!(mth_ceil(1.2), 2);
        assert_eq!(mth_ceil(-0.5), 0);
        assert_eq!(mth_ceil(-1.5), -1);
        assert_eq!(mth_ceil(-0.0), 0);
        assert_eq!(mth_ceil(f32::from_bits(0x3F80_0001)), 2); // just above 1.0
        assert_eq!(mth_ceil(f32::from_bits(0xBF00_0000)), 0); // -0.5
    }

    #[test]
    fn block_pos_wrapping_and_negative_inputs() {
        let p = BlockPos::new(i32::MAX, i32::MIN, -1);
        let up = p.above(1);
        assert_eq!(up.y, i32::MIN.wrapping_add(1));
        let down = p.below();
        assert_eq!(down.y, i32::MAX);
        let off = p.offset(1, -1, i32::MIN);
        assert_eq!(off.x, i32::MIN);
        assert_eq!(off.y, i32::MAX);
        assert_eq!(off.z, i32::MIN.wrapping_sub(1));

        assert_eq!(mth_abs_int(i32::MIN), i32::MIN);
        assert_eq!(mth_abs_int(-1), 1);
        assert_eq!(mth_abs_int(0), 0);
    }

    #[test]
    fn edge_random_short_circuit_preserves_rng() {
        // height=7 (next_int 0+7), radius=1 (7/4+0), no tall spike (radius not >1)
        // At y=0, r=1, f=1.0: corner (-1,-1) has dist 1.125 > 1.0, so it must NOT consume next_float.
        // Midpoint (-1,0): on edge and in circle => consumes next_float.
        let mut level = MockLevel::new(-64);
        for x in -3..=3 {
            for z in -3..=3 {
                level.set(x, 64, z, Block::SnowBlock);
            }
        }

        // Provide floats that would reject edge placement if consumed (1.0 > 0.75).
        // If corners incorrectly consumed floats, placement pattern would change drastically.
        let mut rng = SeqRandom::new(vec![0, 0, 0], vec![1.0; 512]);
        let ok = IceSpikeFeature.place(&mut level, &mut rng, BlockPos::new(0, 65, 0));
        assert!(ok);

        // Center column should still be placed because (0,0) is on_axis and not on edge.
        assert_eq!(level.get(0, 65, 0), Block::PackedIce);
        // Some floats were consumed for valid edge-midpoint cells, but not unboundedly.
        assert!(rng.float_idx > 0);
        assert!(rng.float_idx < 512);
    }
}
