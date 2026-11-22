use std::collections::HashSet;

/// Local stand-in for `net.minecraft.util.context.ContextKey<?>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextKey(pub u32);

/// Local stand-in for `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    Inverted,
}

/// Local stand-in for `net.minecraft.world.level.storage.loot.ValidationContext`.
#[derive(Debug, Default)]
pub struct ValidationContext {
    pub validation_calls: i32,
}

/// Minimal self-contained loot condition interface.
pub trait LootItemCondition<C: ?Sized> {
    fn test(&self, ctx: &mut C) -> bool;
    fn get_referenced_context_params(&self) -> HashSet<ContextKey>;
    fn validate(&self, _ctx: &mut ValidationContext) {}
}

/// Port of `net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition`.
pub struct InvertedLootItemCondition<C: ?Sized> {
    term: Box<dyn LootItemCondition<C>>,
}

impl<C: ?Sized> InvertedLootItemCondition<C> {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#<init>
    pub fn new(term: Box<dyn LootItemCondition<C>>) -> Self {
        Self { term }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#term
    pub fn term(&self) -> &dyn LootItemCondition<C> {
        &*self.term
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::Inverted
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#test
    pub fn test(&self, ctx: &mut C) -> bool {
        !self.term.test(ctx)
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        self.term.get_referenced_context_params()
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#validate
    pub fn validate(&self, ctx: &mut ValidationContext) {
        // LootItemCondition.super.validate(ctx); // default no-op
        self.term.validate(ctx);
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition#invert
    pub fn invert(term: Box<dyn LootItemCondition<C>>) -> Self {
        Self::new(term)
    }
}

impl<C: ?Sized> LootItemCondition<C> for InvertedLootItemCondition<C> {
    fn test(&self, ctx: &mut C) -> bool {
        InvertedLootItemCondition::test(self, ctx)
    }

    fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
        InvertedLootItemCondition::get_referenced_context_params(self)
    }

    fn validate(&self, ctx: &mut ValidationContext) {
        InvertedLootItemCondition::validate(self, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct MockCond {
        result: bool,
        params: HashSet<ContextKey>,
        validate_count: Cell<i32>,
    }

    impl MockCond {
        fn new(result: bool, params: &[u32]) -> Self {
            let mut set = HashSet::new();
            for p in params {
                set.insert(ContextKey(*p));
            }
            Self {
                result,
                params: set,
                validate_count: Cell::new(0),
            }
        }
    }

    impl<C: ?Sized> LootItemCondition<C> for MockCond {
        fn test(&self, _ctx: &mut C) -> bool {
            self.result
        }

        fn get_referenced_context_params(&self) -> HashSet<ContextKey> {
            self.params.clone()
        }

        fn validate(&self, ctx: &mut ValidationContext) {
            self.validate_count.set(self.validate_count.get().wrapping_add(1));
            ctx.validation_calls = ctx.validation_calls.wrapping_add(1);
        }
    }

    #[test]
    fn inverts_true_and_false() {
        let mut ctx = ();
        let t = InvertedLootItemCondition::new(Box::new(MockCond::new(true, &[])));
        let f = InvertedLootItemCondition::new(Box::new(MockCond::new(false, &[])));
        assert!(!t.test(&mut ctx));
        assert!(f.test(&mut ctx));
        assert_eq!(t.get_type(), LootItemConditionType::Inverted);
    }

    #[test]
    fn delegates_referenced_context_params() {
        let cond = InvertedLootItemCondition::<()>::new(Box::new(MockCond::new(true, &[1, 2, 2])));
        let params = cond.get_referenced_context_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&ContextKey(1)));
        assert!(params.contains(&ContextKey(2)));
    }

    #[test]
    fn validate_delegates_to_term() {
        let term = MockCond::new(false, &[]);
        let term_count = &term.validate_count as *const Cell<i32>;
        let cond = InvertedLootItemCondition::<()>::new(Box::new(term));
        let mut vctx = ValidationContext::default();
        cond.validate(&mut vctx);
        // SAFETY: not used; we check through ValidationContext instead.
        // Use ValidationContext counter for self-contained observable behavior.
        assert_eq!(vctx.validation_calls, 1);
        let _ = term_count;
    }

    #[test]
    fn invert_builder_and_double_inversion() {
        let mut ctx = ();
        let cond = InvertedLootItemCondition::<()>::invert(Box::new(MockCond::new(false, &[7])));
        assert!(cond.test(&mut ctx));

        let double = InvertedLootItemCondition::<()>::invert(Box::new(cond));
        assert!(!double.test(&mut ctx));
        let params = double.get_referenced_context_params();
        assert!(params.contains(&ContextKey(7)));
    }

    #[test]
    fn validate_wrapping_edge_case() {
        struct WrapCond;
        impl<C: ?Sized> LootItemCondition<C> for WrapCond {
            fn test(&self, _ctx: &mut C) -> bool { true }
            fn get_referenced_context_params(&self) -> HashSet<ContextKey> { HashSet::new() }
            fn validate(&self, ctx: &mut ValidationContext) {
                ctx.validation_calls = ctx.validation_calls.wrapping_add(1);
            }
        }
        let cond = InvertedLootItemCondition::<()>::new(Box::new(WrapCond));
        let mut vctx = ValidationContext { validation_calls: i32::MAX };
        cond.validate(&mut vctx);
        assert_eq!(vctx.validation_calls, i32::MIN);
    }
}
