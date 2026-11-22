use std::marker::PhantomData;

/// Minimal self-contained placeholder for BlockState.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockState {
    pub id: i32,
}

/// Minimal self-contained placeholder for RandomSource.
#[derive(Debug, Clone)]
pub struct RandomSource {
    seed: i64,
}

impl RandomSource {
    pub fn new(seed: i64) -> Self {
        Self { seed }
    }

    pub fn next_i32(&mut self) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005_i64)
            .wrapping_add(1_442_695_040_888_963_407_i64);
        (self.seed >> 33) as i32
    }

    pub fn next_f32(&mut self) -> f32 {
        let bits = ((self.next_i32() as u32 >> 9) | 0x3F80_0000).to_le_bytes();
        f32::from_le_bytes(bits) - 1.0
    }
}

/// Minimal self-contained placeholder for RuleTestType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTestType {
    AlwaysTrue,
    BlockMatch,
    BlockStateMatch,
    TagMatch,
    RandomBlockMatch,
    RandomBlockStateMatch,
}

/// Minimal self-contained placeholder for Mojang Codec.
#[derive(Debug, Clone, Copy)]
pub struct Codec<T>(PhantomData<T>);

pub trait RuleTest {
    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleTest#test
    fn test(&self, state: &BlockState, random: &mut RandomSource) -> bool;

    // JAVA: net.minecraft.world.level.levelgen.structure.templatesystem.RuleTest#getType
    fn get_type(&self) -> RuleTestType;
}

/// Placeholder for the Java CODEC field; full registry dispatch codec is not ported.
pub fn codec_placeholder() -> Codec<()> {
    Codec(PhantomData)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysTrue;
    impl RuleTest for AlwaysTrue {
        fn test(&self, _state: &BlockState, _random: &mut RandomSource) -> bool {
            true
        }
        fn get_type(&self) -> RuleTestType {
            RuleTestType::AlwaysTrue
        }
    }

    struct IdMatch(i32);
    impl RuleTest for IdMatch {
        fn test(&self, state: &BlockState, _random: &mut RandomSource) -> bool {
            state.id == self.0
        }
        fn get_type(&self) -> RuleTestType {
            RuleTestType::BlockMatch
        }
    }

    #[test]
    fn always_true_handles_negative_and_extremes() {
        let rt = AlwaysTrue;
        let mut rng = RandomSource::new(0);
        assert!(rt.test(&BlockState { id: 0 }, &mut rng));
        assert!(rt.test(&BlockState { id: -1 }, &mut rng));
        assert!(rt.test(&BlockState { id: i32::MIN }, &mut rng));
        assert!(rt.test(&BlockState { id: i32::MAX }, &mut rng));
    }

    #[test]
    fn id_match_negative_and_overflow_boundaries() {
        let rt = IdMatch(i32::MIN);
        let mut rng = RandomSource::new(-1);
        assert!(rt.test(&BlockState { id: i32::MIN }, &mut rng));
        assert!(!rt.test(&BlockState { id: i32::MIN.wrapping_add(1) }, &mut rng));
        assert!(!rt.test(&BlockState { id: i32::MAX }, &mut rng));
    }

    #[test]
    fn random_source_deterministic_and_wrapping_seed() {
        let mut a = RandomSource::new(i64::MIN);
        let mut b = RandomSource::new(i64::MIN);
        let va1 = a.next_i32();
        let vb1 = b.next_i32();
        let va2 = a.next_i32();
        let vb2 = b.next_i32();
        assert_eq!(va1, vb1);
        assert_eq!(va2, vb2);
    }

    #[test]
    fn random_source_float_bits_in_unit_range() {
        let mut rng = RandomSource::new(12345);
        for _ in 0..8 {
            let v = rng.next_f32();
            assert!(v.is_finite());
            assert!(v >= 0.0);
            assert!(v < 1.0);
            let bits = v.to_bits();
            assert_eq!(bits & 0x7F80_0000 != 0x7F80_0000, true); // not NaN/Inf
        }
    }

    #[test]
    fn get_type_and_codec_placeholder() {
        assert_eq!(AlwaysTrue.get_type(), RuleTestType::AlwaysTrue);
        assert_eq!(IdMatch(0).get_type(), RuleTestType::BlockMatch);
        let _ = codec_placeholder();
    }
}
