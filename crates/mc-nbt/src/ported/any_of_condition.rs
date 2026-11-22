use std::fmt;
use std::sync::{Arc, OnceLock};

// --- Self-contained stubs for unported dependencies ---

#[derive(Clone)]
pub struct MapCodec<T: ?Sized>(std::marker::PhantomData<fn() -> T>);

impl<T: ?Sized> fmt::Debug for MapCodec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MapCodec(..)")
    }
}

impl<T: ?Sized> MapCodec<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T: ?Sized> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemConditionType(pub u32);

pub struct LootItemConditions;

impl LootItemConditions {
    pub const ANY_OF: LootItemConditionType = LootItemConditionType(1);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LootContext {
    pub value: i32,
    pub fbits: u32,
}

pub trait LootItemCondition: fmt::Debug + Send + Sync {
    fn get_type(&self) -> LootItemConditionType;
    fn test(&self, ctx: &LootContext) -> bool;
}

pub trait LootItemConditionBuilder: fmt::Debug + Send + Sync {
    fn build(&self) -> Arc<dyn LootItemCondition>;
}

// JAVA: net.minecraft.util.Util#anyOf
fn util_any_of(
    terms: Vec<Arc<dyn LootItemCondition>>,
) -> Arc<dyn Fn(&LootContext) -> bool + Send + Sync> {
    Arc::new(move |ctx: &LootContext| terms.iter().any(|t| t.test(ctx)))
}

// --- Self-contained base: CompositeLootItemCondition ---

pub struct CompositeLootItemConditionBase {
    terms: Vec<Arc<dyn LootItemCondition>>,
    composed: Arc<dyn Fn(&LootContext) -> bool + Send + Sync>,
}

impl fmt::Debug for CompositeLootItemConditionBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositeLootItemConditionBase")
            .field("terms_len", &self.terms.len())
            .finish()
    }
}

impl CompositeLootItemConditionBase {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition#<init>
    pub fn new(
        terms: Vec<Arc<dyn LootItemCondition>>,
        composed: Arc<dyn Fn(&LootContext) -> bool + Send + Sync>,
    ) -> Self {
        Self { terms, composed }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition#test
    pub fn test(&self, ctx: &LootContext) -> bool {
        (self.composed)(ctx)
    }

    pub fn terms(&self) -> &[Arc<dyn LootItemCondition>] {
        &self.terms
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition
pub struct AnyOfCondition {
    base: CompositeLootItemConditionBase,
}

impl fmt::Debug for AnyOfCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyOfCondition")
            .field("base", &self.base)
            .finish()
    }
}

impl AnyOfCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition#CODEC
    pub fn codec() -> &'static MapCodec<AnyOfCondition> {
        static CODEC: OnceLock<MapCodec<AnyOfCondition>> = OnceLock::new();
        CODEC.get_or_init(MapCodec::new)
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition#<init>
    pub fn new(terms: Vec<Arc<dyn LootItemCondition>>) -> Self {
        let composed = util_any_of(terms.clone());
        Self {
            base: CompositeLootItemConditionBase::new(terms, composed),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition#anyOf
    pub fn any_of(builders: Vec<Arc<dyn LootItemConditionBuilder>>) -> AnyOfConditionBuilder {
        AnyOfConditionBuilder::new(builders)
    }
}

impl LootItemCondition for AnyOfCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition#getType
    fn get_type(&self) -> LootItemConditionType {
        LootItemConditions::ANY_OF
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition#test
    fn test(&self, ctx: &LootContext) -> bool {
        self.base.test(ctx)
    }
}

#[derive(Debug)]
pub struct AnyOfConditionBuilder {
    terms: Vec<Arc<dyn LootItemConditionBuilder>>,
}

impl AnyOfConditionBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition$Builder#<init>
    pub fn new(terms: Vec<Arc<dyn LootItemConditionBuilder>>) -> Self {
        Self { terms }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition$Builder#addTerm
    pub fn add_term(&mut self, term: Arc<dyn LootItemConditionBuilder>) {
        self.terms.push(term);
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition$Builder#or
    pub fn or(mut self, term: Arc<dyn LootItemConditionBuilder>) -> Self {
        self.add_term(term);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AnyOfCondition$Builder#create
    fn create(&self, terms: Vec<Arc<dyn LootItemCondition>>) -> Arc<dyn LootItemCondition> {
        Arc::new(AnyOfCondition::new(terms))
    }
}

impl LootItemConditionBuilder for AnyOfConditionBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition$Builder#build
    fn build(&self) -> Arc<dyn LootItemCondition> {
        let built: Vec<Arc<dyn LootItemCondition>> = self.terms.iter().map(|b| b.build()).collect();
        self.create(built)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug)]
    struct FixedCondition(bool);
    impl LootItemCondition for FixedCondition {
        fn get_type(&self) -> LootItemConditionType {
            LootItemConditionType(99)
        }
        fn test(&self, _ctx: &LootContext) -> bool {
            self.0
        }
    }

    #[derive(Debug)]
    struct FixedBuilder(bool);
    impl LootItemConditionBuilder for FixedBuilder {
        fn build(&self) -> Arc<dyn LootItemCondition> {
            Arc::new(FixedCondition(self.0))
        }
    }

    #[test]
    fn empty_any_of_is_false_and_has_any_of_type() {
        let cond = AnyOfCondition::new(vec![]);
        let ctx = LootContext::default();
        assert!(!cond.test(&ctx));
        assert_eq!(cond.get_type(), LootItemConditions::ANY_OF);
    }

    #[test]
    fn any_of_true_when_any_matches() {
        let cond = AnyOfCondition::new(vec![
            Arc::new(FixedCondition(false)),
            Arc::new(FixedCondition(true)),
            Arc::new(FixedCondition(false)),
        ]);
        assert!(cond.test(&LootContext::default()));
    }

    #[test]
    fn any_of_false_when_none_match() {
        let cond = AnyOfCondition::new(vec![
            Arc::new(FixedCondition(false)),
            Arc::new(FixedCondition(false)),
        ]);
        assert!(!cond.test(&LootContext::default()));
    }

    #[test]
    fn builder_or_and_any_of_factory() {
        let built = AnyOfCondition::any_of(vec![Arc::new(FixedBuilder(false))])
            .or(Arc::new(FixedBuilder(true)))
            .build();
        assert_eq!(built.get_type(), LootItemConditions::ANY_OF);
        assert!(built.test(&LootContext::default()));
        let _codec = AnyOfCondition::codec();
    }

    #[test]
    fn overflow_and_negative_context_edge_cases() {
        #[derive(Debug)]
        struct OverflowCondition;
        impl LootItemCondition for OverflowCondition {
            fn get_type(&self) -> LootItemConditionType {
                LootItemConditionType(10)
            }
            fn test(&self, ctx: &LootContext) -> bool {
                ctx.value.wrapping_add(1) == i32::MIN
            }
        }

        #[derive(Debug)]
        struct NegativeCondition;
        impl LootItemCondition for NegativeCondition {
            fn get_type(&self) -> LootItemConditionType {
                LootItemConditionType(11)
            }
            fn test(&self, ctx: &LootContext) -> bool {
                ctx.value.wrapping_sub(1) == i32::MAX
            }
        }

        let cond = AnyOfCondition::new(vec![
            Arc::new(OverflowCondition),
            Arc::new(NegativeCondition),
        ]);
        assert!(cond.test(&LootContext {
            value: i32::MAX,
            fbits: 0
        }));
        assert!(cond.test(&LootContext {
            value: i32::MIN,
            fbits: 0
        }));
        assert!(!cond.test(&LootContext { value: 0, fbits: 0 }));
        assert!(!cond.test(&LootContext { value: -1, fbits: 0 }));
    }

    #[test]
    fn float_bits_nan_and_neg_zero() {
        #[derive(Debug)]
        struct NanCondition;
        impl LootItemCondition for NanCondition {
            fn get_type(&self) -> LootItemConditionType {
                LootItemConditionType(12)
            }
            fn test(&self, ctx: &LootContext) -> bool {
                f32::from_bits(ctx.fbits).is_nan()
            }
        }

        #[derive(Debug)]
        struct NegZeroCondition;
        impl LootItemCondition for NegZeroCondition {
            fn get_type(&self) -> LootItemConditionType {
                LootItemConditionType(13)
            }
            fn test(&self, ctx: &LootContext) -> bool {
                f32::from_bits(ctx.fbits).to_bits() == 0x8000_0000
            }
        }

        let cond = AnyOfCondition::new(vec![
            Arc::new(NanCondition),
            Arc::new(NegZeroCondition),
        ]);
        assert!(cond.test(&LootContext {
            value: 0,
            fbits: f32::NAN.to_bits()
        }));
        assert!(cond.test(&LootContext {
            value: 0,
            fbits: (-0.0_f32).to_bits()
        }));
        assert!(!cond.test(&LootContext {
            value: 0,
            fbits: 0.0_f32.to_bits()
        }));
    }
}
