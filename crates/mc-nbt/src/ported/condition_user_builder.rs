use std::any::Any;

/// Minimal self-contained stub for
/// `net.minecraft.world.level.storage.loot.predicates.LootItemCondition$Builder`.
pub trait LootItemConditionBuilder: Any {}

impl<T: Any> LootItemConditionBuilder for T {}

// JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionUserBuilder
pub trait ConditionUserBuilder: Sized {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionUserBuilder#when
    fn when_condition<B: LootItemConditionBuilder>(self, condition: B) -> Self;

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionUserBuilder#when
    fn when_mapped<E, I, F, B>(self, iterable: I, mapper: F) -> Self
    where
        I: IntoIterator<Item = E>,
        F: Fn(E) -> B,
        B: LootItemConditionBuilder,
    {
        let mut result = self.unwrap_builder();
        for item in iterable {
            result = result.when_condition(mapper(item));
        }
        result
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.ConditionUserBuilder#unwrap
    fn unwrap_builder(self) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Cond(i32);

    #[derive(Debug, Default, PartialEq, Eq)]
    struct Bldr {
        ids: Vec<i32>,
        unwraps: i32,
    }

    impl ConditionUserBuilder for Bldr {
        fn when_condition<B: LootItemConditionBuilder>(mut self, condition: B) -> Self {
            if let Some(c) = (&condition as &dyn Any).downcast_ref::<Cond>() {
                self.ids.push(c.0);
            } else {
                self.ids.push(i32::MIN);
            }
            self
        }

        fn unwrap_builder(mut self) -> Self {
            self.unwraps = self.unwraps.wrapping_add(1);
            self
        }
    }

    #[test]
    fn when_mapped_applies_mapper_in_order() {
        let b = Bldr::default().when_mapped([1_i32, 2, 3], Cond);
        assert_eq!(b.ids, vec![1, 2, 3]);
        assert_eq!(b.unwraps, 1);
    }

    #[test]
    fn when_mapped_empty_calls_unwrap_only() {
        let b = Bldr::default().when_mapped(std::iter::empty::<i32>(), Cond);
        assert!(b.ids.is_empty());
        assert_eq!(b.unwraps, 1);
    }

    #[test]
    fn when_mapped_handles_negative_and_extreme_values() {
        let vals = [i32::MIN, -1, 0, i32::MAX];
        let b = Bldr::default().when_mapped(vals, Cond);
        assert_eq!(b.ids, vals.to_vec());
        assert_eq!(b.unwraps, 1);
    }

    #[test]
    fn unwrap_builder_returns_self_with_wrapping_increment() {
        let b = Bldr {
            ids: vec![7],
            unwraps: i32::MAX,
        };
        let b2 = b.unwrap_builder();
        assert_eq!(b2.ids, vec![7]);
        assert_eq!(b2.unwraps, i32::MIN); // wrapping_add(1) on i32::MAX
    }

    #[test]
    fn when_condition_accepts_direct_condition() {
        let b = Bldr::default().when_condition(Cond(42));
        assert_eq!(b.ids, vec![42]);
        assert_eq!(b.unwraps, 0);
    }
}
