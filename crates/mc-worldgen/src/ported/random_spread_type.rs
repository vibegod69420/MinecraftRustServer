/// Minimal random source trait for self-contained porting.
///
/// Mirrors the `nextInt(int)` method used by Java `RandomSource`.
pub trait RandomSource {
    fn next_int(&mut self, bound: i32) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomSpreadType {
    Linear,
    Triangular,
}

impl RandomSpreadType {
    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadType#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Triangular => "triangular",
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.structure.placement.RandomSpreadType#evaluate
    pub fn evaluate<R: RandomSource>(&self, random: &mut R, bound: i32) -> i32 {
        match self {
            Self::Linear => random.next_int(bound),
            Self::Triangular => {
                let a = random.next_int(bound);
                let b = random.next_int(bound);
                a.wrapping_add(b) / 2
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SeqRng {
        values: Vec<i32>,
        idx: usize,
    }

    impl SeqRng {
        fn new(values: Vec<i32>) -> Self {
            Self { values, idx: 0 }
        }
    }

    impl RandomSource for SeqRng {
        fn next_int(&mut self, _bound: i32) -> i32 {
            let v = self.values[self.idx % self.values.len()];
            self.idx += 1;
            v
        }
    }

    #[test]
    fn serialized_names() {
        assert_eq!(RandomSpreadType::Linear.get_serialized_name(), "linear");
        assert_eq!(
            RandomSpreadType::Triangular.get_serialized_name(),
            "triangular"
        );
    }

    #[test]
    fn linear_evaluate_returns_next_int() {
        let mut rng = SeqRng::new(vec![7, 3]);
        assert_eq!(RandomSpreadType::Linear.evaluate(&mut rng, 10), 7);
        assert_eq!(RandomSpreadType::Linear.evaluate(&mut rng, 10), 3);
    }

    #[test]
    fn triangular_evaluate_averages_two_samples() {
        let mut rng = SeqRng::new(vec![2, 8]);
        // (2 + 8) / 2 = 5
        assert_eq!(RandomSpreadType::Triangular.evaluate(&mut rng, 10), 5);
    }

    #[test]
    fn triangular_evaluate_handles_negative_and_odd_sum() {
        let mut rng = SeqRng::new(vec![-3, 0]);
        // Java/Rust integer division truncates toward zero: -3 / 2 == -1
        assert_eq!(RandomSpreadType::Triangular.evaluate(&mut rng, 10), -1);
    }

    #[test]
    fn triangular_evaluate_wrapping_overflow() {
        let mut rng = SeqRng::new(vec![i32::MAX, i32::MAX]);
        // Java int overflow: MAX + MAX wraps to -2, then / 2 == -1
        let expected = i32::MAX.wrapping_add(i32::MAX) / 2;
        assert_eq!(expected, -1);
        assert_eq!(
            RandomSpreadType::Triangular.evaluate(&mut rng, i32::MAX),
            -1
        );
    }

    #[test]
    fn evaluate_with_bound_one_edge_case() {
        let mut rng = SeqRng::new(vec![0, 0, 0]);
        assert_eq!(RandomSpreadType::Linear.evaluate(&mut rng, 1), 0);
        assert_eq!(RandomSpreadType::Triangular.evaluate(&mut rng, 1), 0);
    }
}
