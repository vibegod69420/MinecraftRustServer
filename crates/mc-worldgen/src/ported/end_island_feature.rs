#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // JAVA: net.minecraft.core.BlockPos#offset
    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        Self {
            x: self.x.wrapping_add(dx),
            y: self.y.wrapping_add(dy),
            z: self.z.wrapping_add(dz),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState(pub u32);

pub const END_STONE_DEFAULT_STATE: BlockState = BlockState(1);

pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

pub trait WorldGenLevel {
    fn set_block(&mut self, pos: BlockPos, state: BlockState) -> bool;
}

// JAVA: net.minecraft.util.Mth#floor
pub fn mth_floor(value: f32) -> i32 {
    let i = value as i32;
    if value < i as f32 {
        i.wrapping_sub(1)
    } else {
        i
    }
}

// JAVA: net.minecraft.util.Mth#ceil
pub fn mth_ceil(value: f32) -> i32 {
    let i = value as i32;
    if value > i as f32 {
        i.wrapping_add(1)
    } else {
        i
    }
}

pub struct FeaturePlaceContext<'a, L: ?Sized, R: RandomSource + ?Sized> {
    pub level: &'a mut L,
    pub random: &'a mut R,
    pub origin: BlockPos,
}

pub struct EndIslandFeature;

impl EndIslandFeature {
    // JAVA: net.minecraft.world.level.levelgen.feature.EndIslandFeature#<init>
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.Feature#setBlock
    fn set_block<L: WorldGenLevel + ?Sized>(
        &self,
        level: &mut L,
        pos: BlockPos,
        state: BlockState,
    ) -> bool {
        level.set_block(pos, state)
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.EndIslandFeature#place
    pub fn place<L: WorldGenLevel + ?Sized, R: RandomSource + ?Sized>(
        &self,
        ctx: &mut FeaturePlaceContext<'_, L, R>,
    ) -> bool {
        let origin = ctx.origin;
        let mut radius: f32 = (ctx.random.next_int(3) as f32) + 4.0_f32;

        let mut y_offset: i32 = 0;
        while radius > 0.5_f32 {
            let min = mth_floor(-radius);
            let max = mth_ceil(radius);
            let mut dx: i32 = min;
            while dx <= max {
                let mut dz: i32 = min;
                while dz <= max {
                    let dist_sq = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz));
                    if (dist_sq as f32) <= (radius + 1.0_f32) * (radius + 1.0_f32) {
                        self.set_block(
                            ctx.level,
                            origin.offset(dx, y_offset, dz),
                            END_STONE_DEFAULT_STATE,
                        );
                    }
                    dz = dz.wrapping_add(1);
                }
                dx = dx.wrapping_add(1);
            }
            radius -= (ctx.random.next_int(2) as f32) + 0.5_f32;
            y_offset = y_offset.wrapping_sub(1);
        }

        true
    }
}

impl Default for EndIslandFeature {
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

    struct MockLevel {
        set_calls: Vec<(BlockPos, BlockState)>,
    }

    impl MockLevel {
        fn new() -> Self {
            Self {
                set_calls: Vec::new(),
            }
        }
    }

    impl WorldGenLevel for MockLevel {
        fn set_block(&mut self, pos: BlockPos, state: BlockState) -> bool {
            self.set_calls.push((pos, state));
            true
        }
    }

    #[test]
    fn place_always_returns_true_and_sets_end_stone() {
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![0, 0, 0, 0, 0, 0, 0, 0]);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            random: &mut rng,
            origin: BlockPos::new(0, 64, 0),
        };
        let f = EndIslandFeature::new();
        assert!(f.place(&mut ctx));
        assert!(!level.set_calls.is_empty());
        assert!(level
            .set_calls
            .iter()
            .all(|(_, s)| *s == END_STONE_DEFAULT_STATE));
    }

    #[test]
    fn radius_and_layer_progression_match_java() {
        let mut level = MockLevel::new();
        // nextInt(3)=0 -> radius 4.0; then nextInt(2)=1,1,1 -> 2.5, 1.0, -0.5
        let mut rng = SeqRandom::new(vec![0, 1, 1, 1, 1]);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            random: &mut rng,
            origin: BlockPos::new(10, 70, -20),
        };
        assert!(EndIslandFeature::new().place(&mut ctx));
        let ys: Vec<i32> = level.set_calls.iter().map(|(p, _)| p.y).collect();
        assert!(ys.contains(&70));
        assert!(ys.contains(&69));
        assert!(ys.contains(&68));
        assert!(!ys.contains(&67));
    }

    #[test]
    fn negative_origin_offsets_correctly() {
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![0, 1, 1, 1, 1]);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            random: &mut rng,
            origin: BlockPos::new(-100, -5, -200),
        };
        assert!(EndIslandFeature::new().place(&mut ctx));
        assert!(level.set_calls.iter().any(|(p, _)| p.x == -100 && p.y == -5 && p.z == -200));
        assert!(level.set_calls.iter().any(|(p, _)| p.y == -6));
    }

    #[test]
    fn mth_floor_ceil_and_float_bits_behavior() {
        assert_eq!(mth_floor(4.0_f32), 4);
        assert_eq!(mth_floor(-4.0_f32), -4);
        assert_eq!(mth_floor(3.5_f32), 3);
        assert_eq!(mth_floor(-3.5_f32), -4);
        assert_eq!(mth_floor(-0.0_f32), 0);
        assert_eq!(mth_ceil(4.0_f32), 4);
        assert_eq!(mth_ceil(-4.0_f32), -4);
        assert_eq!(mth_ceil(3.5_f32), 4);
        assert_eq!(mth_ceil(-3.5_f32), -3);
        assert_eq!(mth_ceil(-0.0_f32), 0);

        // float operation order: (radius + 1.0) * (radius + 1.0)
        let radius: f32 = 4.0_f32;
        let threshold = (radius + 1.0_f32) * (radius + 1.0_f32);
        assert_eq!(threshold.to_bits(), 25.0_f32.to_bits());
    }

    #[test]
    fn offset_wraps_on_overflow() {
        let mut level = MockLevel::new();
        let mut rng = SeqRandom::new(vec![0, 1, 1, 1, 1]);
        let mut ctx = FeaturePlaceContext {
            level: &mut level,
            random: &mut rng,
            origin: BlockPos::new(i32::MAX, 0, 0),
        };
        assert!(EndIslandFeature::new().place(&mut ctx));
        assert!(level
            .set_calls
            .iter()
            .any(|(p, _)| p.x == i32::MIN.wrapping_add(3)));
    }
}
