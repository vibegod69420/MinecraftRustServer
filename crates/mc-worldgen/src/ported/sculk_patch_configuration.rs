pub trait FeatureConfiguration {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntProvider {
    pub value: i32,
}

impl IntProvider {
    pub fn constant(value: i32) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SculkPatchConfiguration {
    pub charge_count: i32,
    pub amount_per_charge: i32,
    pub spread_attempts: i32,
    pub growth_rounds: i32,
    pub spread_rounds: i32,
    pub extra_rare_growths: IntProvider,
    pub catalyst_chance: f32,
}

impl FeatureConfiguration for SculkPatchConfiguration {}

impl SculkPatchConfiguration {
    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#<init>
    pub fn new(
        charge_count: i32,
        amount_per_charge: i32,
        spread_attempts: i32,
        growth_rounds: i32,
        spread_rounds: i32,
        extra_rare_growths: IntProvider,
        catalyst_chance: f32,
    ) -> Self {
        Self {
            charge_count,
            amount_per_charge,
            spread_attempts,
            growth_rounds,
            spread_rounds,
            extra_rare_growths,
            catalyst_chance,
        }
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#chargeCount
    pub fn charge_count(&self) -> i32 {
        self.charge_count
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#amountPerCharge
    pub fn amount_per_charge(&self) -> i32 {
        self.amount_per_charge
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#spreadAttempts
    pub fn spread_attempts(&self) -> i32 {
        self.spread_attempts
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#growthRounds
    pub fn growth_rounds(&self) -> i32 {
        self.growth_rounds
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#spreadRounds
    pub fn spread_rounds(&self) -> i32 {
        self.spread_rounds
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#extraRareGrowths
    pub fn extra_rare_growths(&self) -> &IntProvider {
        &self.extra_rare_growths
    }

    // JAVA: net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration#catalystChance
    pub fn catalyst_chance(&self) -> f32 {
        self.catalyst_chance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_and_accessors() {
        let cfg = SculkPatchConfiguration::new(
            10,
            32,
            64,
            1,
            1,
            IntProvider::constant(7),
            0.5,
        );
        assert_eq!(cfg.charge_count(), 10);
        assert_eq!(cfg.amount_per_charge(), 32);
        assert_eq!(cfg.spread_attempts(), 64);
        assert_eq!(cfg.growth_rounds(), 1);
        assert_eq!(cfg.spread_rounds(), 1);
        assert_eq!(cfg.extra_rare_growths(), &IntProvider::constant(7));
        assert_eq!(cfg.catalyst_chance().to_bits(), 0.5f32.to_bits());
    }

    #[test]
    fn negative_and_extreme_int_values() {
        let cfg = SculkPatchConfiguration::new(
            i32::MIN,
            -1,
            i32::MAX,
            -100,
            0,
            IntProvider::constant(i32::MIN),
            0.0,
        );
        assert_eq!(cfg.charge_count(), i32::MIN);
        assert_eq!(cfg.amount_per_charge(), -1);
        assert_eq!(cfg.spread_attempts(), i32::MAX);
        assert_eq!(cfg.growth_rounds(), -100);
        assert_eq!(cfg.spread_rounds(), 0);
        assert_eq!(cfg.extra_rare_growths().value, i32::MIN);
    }

    #[test]
    fn float_bits_preserved() {
        let nan = f32::NAN;
        let cfg_nan =
            SculkPatchConfiguration::new(1, 1, 1, 0, 0, IntProvider::constant(0), nan);
        assert!(cfg_nan.catalyst_chance().is_nan());

        let neg_zero = -0.0f32;
        let cfg_nz =
            SculkPatchConfiguration::new(1, 1, 1, 0, 0, IntProvider::constant(0), neg_zero);
        assert_eq!(cfg_nz.catalyst_chance().to_bits(), neg_zero.to_bits());
        assert_ne!(cfg_nz.catalyst_chance().to_bits(), 0.0f32.to_bits());

        let inf = f32::INFINITY;
        let cfg_inf =
            SculkPatchConfiguration::new(1, 1, 1, 0, 0, IntProvider::constant(0), inf);
        assert_eq!(cfg_inf.catalyst_chance().to_bits(), inf.to_bits());
    }

    #[test]
    fn clone_and_equality() {
        let cfg = SculkPatchConfiguration::new(2, 3, 4, 5, 6, IntProvider::constant(1), 0.25);
        let cfg2 = cfg.clone();
        assert_eq!(cfg, cfg2);
    }
}
