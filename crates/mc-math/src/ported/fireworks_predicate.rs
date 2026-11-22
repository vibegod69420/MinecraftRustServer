//! Port of `net.minecraft.core.component.predicates.FireworksPredicate`.

use std::marker::PhantomData;

/// Minimal stand-in for `net.minecraft.world.item.component.FireworkExplosion`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireworkExplosion;

/// Minimal stand-in for `net.minecraft.world.item.component.Fireworks`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fireworks {
    pub explosions: Vec<FireworkExplosion>,
    pub flight_duration: i32,
}

impl Fireworks {
    // JAVA: net.minecraft.world.item.component.Fireworks#explosions
    pub fn explosions(&self) -> &[FireworkExplosion] {
        &self.explosions
    }

    // JAVA: net.minecraft.world.item.component.Fireworks#flightDuration
    pub fn flight_duration(&self) -> i32 {
        self.flight_duration
    }
}

/// Minimal stand-in for `net.minecraft.advancements.criterion.MinMaxBounds.Ints`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ints {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl Ints {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    // JAVA: net.minecraft.advancements.criterion.MinMaxBounds.Ints#matches
    pub fn matches(&self, value: i32) -> bool {
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

/// Minimal stand-in for `net.minecraft.advancements.criterion.CollectionPredicate`.
pub trait CollectionPredicate<T> {
    fn test(&self, items: &[T]) -> bool;
}

/// Minimal stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataComponentType<T>(PhantomData<T>);

/// Minimal stand-in for `net.minecraft.core.component.DataComponents`.
pub struct DataComponents;
impl DataComponents {
    pub const FIREWORKS: DataComponentType<Fireworks> = DataComponentType(PhantomData);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireworksPredicate<E> {
    pub explosions: Option<E>,
    pub flight_duration: Ints,
}

impl<E> FireworksPredicate<E> {
    // JAVA: net.minecraft.core.component.predicates.FireworksPredicate#explosions
    pub fn explosions(&self) -> Option<&E> {
        self.explosions.as_ref()
    }

    // JAVA: net.minecraft.core.component.predicates.FireworksPredicate#flightDuration
    pub fn flight_duration(&self) -> &Ints {
        &self.flight_duration
    }

    // JAVA: net.minecraft.core.component.predicates.FireworksPredicate#componentType
    pub fn component_type(&self) -> DataComponentType<Fireworks> {
        DataComponents::FIREWORKS
    }
}

impl<E: CollectionPredicate<FireworkExplosion>> FireworksPredicate<E> {
    // JAVA: net.minecraft.core.component.predicates.FireworksPredicate#matches
    pub fn matches(&self, fireworks: &Fireworks) -> bool {
        if let Some(ref explosions) = self.explosions {
            if !explosions.test(fireworks.explosions()) {
                return false;
            }
        }
        self.flight_duration.matches(fireworks.flight_duration())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Always(bool);
    impl CollectionPredicate<FireworkExplosion> for Always {
        fn test(&self, _items: &[FireworkExplosion]) -> bool {
            self.0
        }
    }

    fn fireworks(flight_duration: i32, explosion_count: usize) -> Fireworks {
        Fireworks {
            explosions: vec![FireworkExplosion; explosion_count],
            flight_duration,
        }
    }

    #[test]
    fn no_explosions_predicate_checks_only_flight_duration() {
        let pred = FireworksPredicate::<Always> {
            explosions: None,
            flight_duration: Ints {
                min: Some(1),
                max: Some(3),
            },
        };
        assert!(pred.matches(&fireworks(2, 0)));
        assert!(!pred.matches(&fireworks(0, 0)));
        assert!(!pred.matches(&fireworks(4, 5)));
    }

    #[test]
    fn explosions_predicate_failure_returns_false() {
        let pred = FireworksPredicate {
            explosions: Some(Always(false)),
            flight_duration: Ints::ANY,
        };
        assert!(!pred.matches(&fireworks(1, 2)));
    }

    #[test]
    fn explosions_pass_then_flight_duration_is_checked() {
        let pred = FireworksPredicate {
            explosions: Some(Always(true)),
            flight_duration: Ints {
                min: Some(2),
                max: Some(2),
            },
        };
        assert!(pred.matches(&fireworks(2, 3)));
        assert!(!pred.matches(&fireworks(1, 3)));
        assert!(!pred.matches(&fireworks(3, 3)));
    }

    #[test]
    fn ints_matches_negative_and_boundary_values() {
        let bounds = Ints {
            min: Some(-5),
            max: Some(-1),
        };
        assert!(bounds.matches(-5));
        assert!(bounds.matches(-3));
        assert!(bounds.matches(-1));
        assert!(!bounds.matches(-6));
        assert!(!bounds.matches(0));
        assert!(Ints::ANY.matches(i32::MIN));
        assert!(Ints::ANY.matches(i32::MAX));
    }

    #[test]
    fn component_type_is_fireworks() {
        let pred = FireworksPredicate::<Always> {
            explosions: None,
            flight_duration: Ints::ANY,
        };
        assert_eq!(pred.component_type(), DataComponents::FIREWORKS);
    }
}
