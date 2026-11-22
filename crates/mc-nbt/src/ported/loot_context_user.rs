use std::collections::HashSet;

/// Type-erased stand-in for `net.minecraft.util.context.ContextKey<?>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKeyAny {
    pub id: u64,
}

impl ContextKeyAny {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

/// Minimal stand-in for `net.minecraft.world.level.storage.loot.ValidationContext`.
#[derive(Debug, Default)]
pub struct ValidationContext {
    pub validate_context_usage_calls: usize,
}

impl ValidationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_context_usage<U: LootContextUser + ?Sized>(&mut self, _user: &U) {
        self.validate_context_usage_calls = self.validate_context_usage_calls.wrapping_add(1);
    }
}

/// Port of `net.minecraft.world.level.storage.loot.LootContextUser`.
pub trait LootContextUser {
    // JAVA: net.minecraft.world.level.storage.loot.LootContextUser#getReferencedContextParams
    fn get_referenced_context_params(&self) -> HashSet<ContextKeyAny> {
        HashSet::new()
    }

    // JAVA: net.minecraft.world.level.storage.loot.LootContextUser#validate
    fn validate(&self, ctx: &mut ValidationContext) {
        ctx.validate_context_usage(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DefaultUser;
    impl LootContextUser for DefaultUser {}

    struct CustomUser;
    impl LootContextUser for CustomUser {
        fn get_referenced_context_params(&self) -> HashSet<ContextKeyAny> {
            let mut set = HashSet::new();
            set.insert(ContextKeyAny::new(1));
            set.insert(ContextKeyAny::new(u64::MAX));
            set
        }
    }

    #[test]
    fn default_referenced_context_params_is_empty() {
        let user = DefaultUser;
        assert!(user.get_referenced_context_params().is_empty());
    }

    #[test]
    fn default_validate_calls_validation_context() {
        let user = DefaultUser;
        let mut ctx = ValidationContext::new();
        user.validate(&mut ctx);
        assert_eq!(ctx.validate_context_usage_calls, 1);
        user.validate(&mut ctx);
        assert_eq!(ctx.validate_context_usage_calls, 2);
    }

    #[test]
    fn custom_override_returns_expected_params() {
        let user = CustomUser;
        let params = user.get_referenced_context_params();
        assert_eq!(params.len(), 2);
        assert!(params.contains(&ContextKeyAny::new(1)));
        assert!(params.contains(&ContextKeyAny::new(u64::MAX)));
    }

    #[test]
    fn trait_object_validate_works() {
        let user: &dyn LootContextUser = &DefaultUser;
        let mut ctx = ValidationContext::new();
        user.validate(&mut ctx);
        assert_eq!(ctx.validate_context_usage_calls, 1);
    }
}
