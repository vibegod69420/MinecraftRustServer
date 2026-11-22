/// Minimal stub for `net.minecraft.world.level.block.state.BlockState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockState(pub u32);

/// Minimal stub for `net.minecraft.world.level.levelgen.NoiseChunk`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoiseChunk;

/// Port of `net.minecraft.world.level.levelgen.material.WorldGenMaterialRule`.
pub trait WorldGenMaterialRule {
    // JAVA: net.minecraft.world.level.levelgen.material.WorldGenMaterialRule#apply
    fn apply(&self, chunk: &NoiseChunk, x: i32, y: i32, z: i32) -> Option<BlockState>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct ConstRule(Option<BlockState>);

    impl WorldGenMaterialRule for ConstRule {
        fn apply(&self, _chunk: &NoiseChunk, _x: i32, _y: i32, _z: i32) -> Option<BlockState> {
            self.0
        }
    }

    #[derive(Debug)]
    struct CoordCaptureRule {
        last: Cell<(i32, i32, i32)>,
    }

    impl CoordCaptureRule {
        fn new() -> Self {
            Self {
                last: Cell::new((0, 0, 0)),
            }
        }
    }

    impl WorldGenMaterialRule for CoordCaptureRule {
        fn apply(&self, _chunk: &NoiseChunk, x: i32, y: i32, z: i32) -> Option<BlockState> {
            self.last.set((x, y, z));
            let sum = x.wrapping_add(y).wrapping_add(z);
            if sum == 0 {
                Some(BlockState(1))
            } else {
                None
            }
        }
    }

    #[test]
    fn returns_none_for_empty_rule() {
        let rule = ConstRule(None);
        let chunk = NoiseChunk;
        assert_eq!(rule.apply(&chunk, 0, 0, 0), None);
    }

    #[test]
    fn returns_some_for_constant_rule() {
        let rule = ConstRule(Some(BlockState(42)));
        let chunk = NoiseChunk;
        assert_eq!(rule.apply(&chunk, 1, -2, 3), Some(BlockState(42)));
    }

    #[test]
    fn handles_negative_and_extreme_coords() {
        let rule = ConstRule(None);
        let chunk = NoiseChunk;
        assert_eq!(rule.apply(&chunk, i32::MIN, -1, i32::MAX), None);
        assert_eq!(rule.apply(&chunk, -64, -320, 64), None);
    }

    #[test]
    fn wrapping_coord_behavior() {
        let rule = CoordCaptureRule::new();
        let chunk = NoiseChunk;
        // i32::MAX.wrapping_add(1).wrapping_add(0) == i32::MIN
        assert_eq!(rule.apply(&chunk, i32::MAX, 1, 0), None);
        assert_eq!(rule.last.get(), (i32::MAX, 1, 0));
        assert_eq!(rule.apply(&chunk, i32::MIN, i32::MIN, 0), Some(BlockState(1)));
    }

    #[test]
    fn float_bits_as_coords() {
        let rule = CoordCaptureRule::new();
        let chunk = NoiseChunk;
        let x = f32::NAN.to_bits() as i32;
        let y = (-0.0f32).to_bits() as i32;
        let z = f32::INFINITY.to_bits() as i32;
        let _ = rule.apply(&chunk, x, y, z);
        assert_eq!(rule.last.get(), (x, y, z));
    }
}
