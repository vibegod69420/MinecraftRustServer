#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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

pub trait RandomSource {
    fn next_int_between_inclusive(&mut self, min: i32, max: i32) -> i32;
}

pub trait IntProvider {
    fn sample(&self, random: &mut impl RandomSource) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantInt {
    pub value: i32,
}

impl ConstantInt {
    // JAVA: net.minecraft.util.valueproviders.ConstantInt#of
    pub fn of(value: i32) -> Self {
        Self { value }
    }
}

impl IntProvider for ConstantInt {
    fn sample(&self, _random: &mut impl RandomSource) -> i32 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformInt {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl UniformInt {
    pub fn of(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }
}

impl IntProvider for UniformInt {
    fn sample(&self, random: &mut impl RandomSource) -> i32 {
        random.next_int_between_inclusive(self.min_inclusive, self.max_inclusive)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomOffsetPlacement<X, Y> {
    pub xz_spread: X,
    pub y_spread: Y,
}

impl<X: IntProvider, Y: IntProvider> RandomOffsetPlacement<X, Y> {
    // JAVA: net.minecraft.world.level.levelgen.placement.RandomOffsetPlacement#<init>
    fn new(xz_spread: X, y_spread: Y) -> Self {
        Self { xz_spread, y_spread }
    }

    // JAVA: net.minecraft.world.level.levelgen.placement.RandomOffsetPlacement#getPositions
    pub fn get_positions(
        &self,
        random: &mut impl RandomSource,
        pos: &BlockPos,
    ) -> Vec<BlockPos> {
        let x_offset = self.xz_spread.sample(random);
        let y_offset = self.y_spread.sample(random);
        let z_offset = self.xz_spread.sample(random);
        let x = pos.get_x().wrapping_add(x_offset);
        let y = pos.get_y().wrapping_add(y_offset);
        let z = pos.get_z().wrapping_add(z_offset);
        vec![BlockPos::new(x, y, z)]
    }
}

// JAVA: net.minecraft.world.level.levelgen.placement.RandomOffsetPlacement#of
pub fn of<X: IntProvider, Y: IntProvider>(
    xz_spread: X,
    y_spread: Y,
) -> RandomOffsetPlacement<X, Y> {
    RandomOffsetPlacement::new(xz_spread, y_spread)
}

// JAVA: net.minecraft.world.level.levelgen.placement.RandomOffsetPlacement#vertical
pub fn vertical<Y: IntProvider>(y_spread: Y) -> RandomOffsetPlacement<ConstantInt, Y> {
    RandomOffsetPlacement::new(ConstantInt::of(0), y_spread)
}

// JAVA: net.minecraft.world.level.levelgen.placement.RandomOffsetPlacement#horizontal
pub fn horizontal<X: IntProvider>(xz_spread: X) -> RandomOffsetPlacement<X, ConstantInt> {
    RandomOffsetPlacement::new(xz_spread, ConstantInt::of(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRandom {
        values: std::vec::IntoIter<i32>,
    }

    impl SeqRandom {
        fn new(values: &[i32]) -> Self {
            Self {
                values: values.to_vec().into_iter(),
            }
        }
    }

    impl RandomSource for SeqRandom {
        fn next_int_between_inclusive(&mut self, _min: i32, _max: i32) -> i32 {
            self.values.next().unwrap_or(0)
        }
    }

    #[test]
    fn constant_offset_applies_same_xz_sample_value() {
        let placement = of(ConstantInt::of(2), ConstantInt::of(-1));
        let mut rng = SeqRandom::new(&[]);
        let pos = BlockPos::new(10, 64, 20);
        let positions = placement.get_positions(&mut rng, &pos);
        assert_eq!(positions, vec![BlockPos::new(12, 63, 22)]);
    }

    #[test]
    fn xz_spread_is_sampled_twice() {
        let placement = of(UniformInt::of(-16, 16), ConstantInt::of(0));
        let mut rng = SeqRandom::new(&[3, -5]);
        let pos = BlockPos::new(0, 10, 0);
        let positions = placement.get_positions(&mut rng, &pos);
        assert_eq!(positions, vec![BlockPos::new(3, 10, -5)]);
    }

    #[test]
    fn vertical_and_horizontal_constructors() {
        let mut rng_v = SeqRandom::new(&[7]);
        let v = vertical(UniformInt::of(-16, 16));
        assert_eq!(
            v.get_positions(&mut rng_v, &BlockPos::new(1, 2, 3)),
            vec![BlockPos::new(1, 9, 3)]
        );

        let mut rng_h = SeqRandom::new(&[-2, 4]);
        let h = horizontal(UniformInt::of(-16, 16));
        assert_eq!(
            h.get_positions(&mut rng_h, &BlockPos::new(5, 6, 7)),
            vec![BlockPos::new(3, 6, 11)]
        );
    }

    #[test]
    fn wrapping_add_on_integer_overflow() {
        let placement = of(ConstantInt::of(1), ConstantInt::of(1));
        let mut rng = SeqRandom::new(&[]);
        let pos = BlockPos::new(i32::MAX, i32::MAX, i32::MIN);
        let positions = placement.get_positions(&mut rng, &pos);
        assert_eq!(
            positions,
            vec![BlockPos::new(i32::MIN, i32::MIN, i32::MIN.wrapping_add(1))]
        );
    }

    #[test]
    fn negative_spread_offsets() {
        let placement = of(ConstantInt::of(-16), ConstantInt::of(-16));
        let mut rng = SeqRandom::new(&[]);
        let pos = BlockPos::new(0, 0, 0);
        let positions = placement.get_positions(&mut rng, &pos);
        assert_eq!(positions, vec![BlockPos::new(-16, -16, -16)]);
    }
}
