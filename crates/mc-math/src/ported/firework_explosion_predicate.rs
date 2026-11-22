use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkExplosion {
    pub shape: Shape,
    pub has_twinkle: bool,
    pub has_trail: bool,
}

impl FireworkExplosion {
    pub fn new(shape: Shape, has_twinkle: bool, has_trail: bool) -> Self {
        Self {
            shape,
            has_twinkle,
            has_trail,
        }
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }

    pub fn has_twinkle(&self) -> bool {
        self.has_twinkle
    }

    pub fn has_trail(&self) -> bool {
        self.has_trail
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType<T> {
    pub name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> DataComponentType<T> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

pub struct DataComponents;

impl DataComponents {
    pub const FIREWORK_EXPLOSION: DataComponentType<FireworkExplosion> =
        DataComponentType::new("firework_explosion");
}

pub trait SingleComponentItemPredicate<T> {
    fn component_type(&self) -> DataComponentType<T>;
    fn matches(&self, value: &T) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkPredicate {
    pub shape: Option<Shape>,
    pub twinkle: Option<bool>,
    pub trail: Option<bool>,
}

impl FireworkPredicate {
    pub fn new(shape: Option<Shape>, twinkle: Option<bool>, trail: Option<bool>) -> Self {
        Self {
            shape,
            twinkle,
            trail,
        }
    }

    // JAVA: net.minecraft.core.component.predicates.FireworkExplosionPredicate$FireworkPredicate#test
    pub fn test(&self, explosion: &FireworkExplosion) -> bool {
        if self.shape.is_some() && self.shape != Some(explosion.shape()) {
            return false;
        } else if self.twinkle.is_some() && self.twinkle != Some(explosion.has_twinkle()) {
            return false;
        } else {
            !self.trail.is_some() || self.trail == Some(explosion.has_trail())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkExplosionPredicate {
    pub predicate: FireworkPredicate,
}

impl FireworkExplosionPredicate {
    pub fn new(predicate: FireworkPredicate) -> Self {
        Self { predicate }
    }

    // JAVA: net.minecraft.core.component.predicates.FireworkExplosionPredicate#componentType
    pub fn component_type(&self) -> DataComponentType<FireworkExplosion> {
        DataComponents::FIREWORK_EXPLOSION
    }

    // JAVA: net.minecraft.core.component.predicates.FireworkExplosionPredicate#matches
    pub fn matches(&self, value: &FireworkExplosion) -> bool {
        self.predicate.test(value)
    }
}

impl SingleComponentItemPredicate<FireworkExplosion> for FireworkExplosionPredicate {
    fn component_type(&self) -> DataComponentType<FireworkExplosion> {
        FireworkExplosionPredicate::component_type(self)
    }

    fn matches(&self, value: &FireworkExplosion) -> bool {
        FireworkExplosionPredicate::matches(self, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn explosion(shape: Shape, twinkle: bool, trail: bool) -> FireworkExplosion {
        FireworkExplosion::new(shape, twinkle, trail)
    }

    #[test]
    fn empty_predicate_matches_any_explosion() {
        let pred = FireworkExplosionPredicate::new(FireworkPredicate::new(None, None, None));
        assert!(pred.matches(&explosion(Shape::SmallBall, false, false)));
        assert!(pred.matches(&explosion(Shape::Creeper, true, true)));
    }

    #[test]
    fn shape_mismatch_rejects() {
        let pred = FireworkExplosionPredicate::new(FireworkPredicate::new(
            Some(Shape::Star),
            None,
            None,
        ));
        assert!(pred.matches(&explosion(Shape::Star, false, true)));
        assert!(!pred.matches(&explosion(Shape::Burst, false, true)));
    }

    #[test]
    fn twinkle_mismatch_rejects() {
        let pred = FireworkExplosionPredicate::new(FireworkPredicate::new(
            None,
            Some(true),
            None,
        ));
        assert!(pred.matches(&explosion(Shape::SmallBall, true, false)));
        assert!(!pred.matches(&explosion(Shape::SmallBall, false, false)));
    }

    #[test]
    fn trail_mismatch_rejects_and_absent_trail_accepts() {
        let pred = FireworkExplosionPredicate::new(FireworkPredicate::new(
            Some(Shape::LargeBall),
            Some(false),
            Some(true),
        ));
        assert!(pred.matches(&explosion(Shape::LargeBall, false, true)));
        assert!(!pred.matches(&explosion(Shape::LargeBall, false, false)));
        assert!(!pred.matches(&explosion(Shape::LargeBall, true, true)));
    }

    #[test]
    fn component_type_is_firework_explosion() {
        let pred = FireworkExplosionPredicate::new(FireworkPredicate::new(None, None, None));
        assert_eq!(pred.component_type(), DataComponents::FIREWORK_EXPLOSION);
    }
}
