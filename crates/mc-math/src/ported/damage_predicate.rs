/// Minimal self-contained stand-in for `net.minecraft.advancements.criterion.MinMaxBounds.Ints`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ints {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl Ints {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub const fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub const fn at_least(min: i32) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub const fn at_most(max: i32) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    pub const fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    // JAVA: net.minecraft.advancements.criterion.MinMaxBounds$Ints#matches(int)
    pub fn matches(self, value: i32) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        true
    }
}

/// Minimal self-contained stand-in for `net.minecraft.core.component.DataComponentGetter`.
pub trait DataComponentGetter {
    fn damage(&self) -> Option<i32>;
    fn max_damage_or_default(&self, default: i32) -> i32;
}

/// Minimal self-contained stand-in for
/// `net.minecraft.core.component.predicates.DataComponentPredicate`.
pub trait DataComponentPredicate {
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool;
}

/// Port of `net.minecraft.core.component.predicates.DamagePredicate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamagePredicate {
    pub durability: Ints,
    pub damage: Ints,
}

impl DamagePredicate {
    pub fn new(durability: Ints, damage: Ints) -> Self {
        Self { durability, damage }
    }

    // JAVA: net.minecraft.core.component.predicates.DamagePredicate#durability
    pub fn durability(durability: Ints) -> Self {
        Self {
            durability,
            damage: Ints::ANY,
        }
    }
}

impl DataComponentPredicate for DamagePredicate {
    // JAVA: net.minecraft.core.component.predicates.DamagePredicate#matches
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        let Some(damage) = getter.damage() else {
            return false;
        };

        let max_damage = getter.max_damage_or_default(0);
        let remaining_durability = max_damage.wrapping_sub(damage);
        if !self.durability.matches(remaining_durability) {
            return false;
        }
        self.damage.matches(damage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGetter {
        damage: Option<i32>,
        max_damage: i32,
    }

    impl MockGetter {
        fn new(damage: Option<i32>, max_damage: i32) -> Self {
            Self { damage, max_damage }
        }
    }

    impl DataComponentGetter for MockGetter {
        fn damage(&self) -> Option<i32> {
            self.damage
        }

        fn max_damage_or_default(&self, _default: i32) -> i32 {
            self.max_damage
        }
    }

    #[test]
    fn missing_damage_component_returns_false() {
        let pred = DamagePredicate::new(Ints::ANY, Ints::ANY);
        assert!(!pred.matches(&MockGetter::new(None, 100)));
    }

    #[test]
    fn matches_durability_and_damage_bounds() {
        let pred = DamagePredicate::new(Ints::between(10, 20), Ints::between(5, 15));
        assert!(pred.matches(&MockGetter::new(Some(10), 25))); // remaining 15
        assert!(!pred.matches(&MockGetter::new(Some(20), 25))); // remaining 5, durability too low
        assert!(!pred.matches(&MockGetter::new(Some(4), 20))); // damage too low
    }

    #[test]
    fn durability_factory_uses_any_damage() {
        let pred = DamagePredicate::durability(Ints::exactly(7));
        assert!(pred.matches(&MockGetter::new(Some(3), 10)));
        assert!(pred.matches(&MockGetter::new(Some(i32::MAX), 7i32.wrapping_add(i32::MAX))));
        assert!(!pred.matches(&MockGetter::new(Some(4), 10)));
    }

    #[test]
    fn wrapping_subtraction_for_remaining_durability() {
        // max_damage - damage = i32::MIN - 1 = i32::MAX in Java int semantics
        let pred = DamagePredicate::durability(Ints::exactly(i32::MAX));
        assert!(pred.matches(&MockGetter::new(Some(1), i32::MIN)));
    }

    #[test]
    fn negative_values_and_bounds() {
        let pred = DamagePredicate::new(Ints::at_least(-5), Ints::at_most(-1));
        assert!(pred.matches(&MockGetter::new(Some(-2), -3))); // remaining -1
        assert!(!pred.matches(&MockGetter::new(Some(0), -3))); // damage above max
        assert!(!pred.matches(&MockGetter::new(Some(-2), -10))); // remaining -8
    }
}
