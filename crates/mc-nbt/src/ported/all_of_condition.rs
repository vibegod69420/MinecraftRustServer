use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootItemConditionType(pub &'static str);

pub struct LootItemConditions;
impl LootItemConditions {
    pub const ALL_OF: LootItemConditionType = LootItemConditionType("all_of");
}

#[derive(Debug, Default, Clone)]
pub struct LootContext;

pub trait LootItemCondition: fmt::Debug {
    fn get_type(&self) -> LootItemConditionType;
    fn test(&self, ctx: &LootContext) -> bool;
}

pub trait LootItemConditionBuilder: fmt::Debug {
    fn build_condition(&self) -> Box<dyn LootItemCondition>;
}

// JAVA: net.minecraft.Util#allOf
fn util_all_of(terms: &[Box<dyn LootItemCondition>], ctx: &LootContext) -> bool {
    terms.iter().all(|t| t.test(ctx))
}

#[derive(Debug)]
pub struct AllOfCondition {
    pub terms: Vec<Box<dyn LootItemCondition>>,
}

impl AllOfCondition {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition#<init>
    fn new(terms: Vec<Box<dyn LootItemCondition>>) -> Self {
        Self { terms }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition#allOf
    pub fn all_of(terms: Vec<Box<dyn LootItemCondition>>) -> Self {
        Self::new(terms)
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditions::ALL_OF
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition#test
    pub fn test(&self, ctx: &LootContext) -> bool {
        util_all_of(&self.terms, ctx)
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition#allOf
    pub fn all_of_builders(builders: Vec<Box<dyn LootItemConditionBuilder>>) -> AllOfConditionBuilder {
        AllOfConditionBuilder::new(builders)
    }
}

impl LootItemCondition for AllOfCondition {
    fn get_type(&self) -> LootItemConditionType {
        AllOfCondition::get_type(self)
    }

    fn test(&self, ctx: &LootContext) -> bool {
        AllOfCondition::test(self, ctx)
    }
}

#[derive(Debug)]
pub struct AllOfConditionBuilder {
    terms: Vec<Box<dyn LootItemConditionBuilder>>,
}

impl AllOfConditionBuilder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition$Builder#<init>
    pub fn new(terms: Vec<Box<dyn LootItemConditionBuilder>>) -> Self {
        Self { terms }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition$Builder#and
    pub fn and(mut self, term: Box<dyn LootItemConditionBuilder>) -> Self {
        self.terms.push(term);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.AllOfCondition$Builder#create
    fn create(terms: Vec<Box<dyn LootItemCondition>>) -> Box<dyn LootItemCondition> {
        Box::new(AllOfCondition::new(terms))
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.CompositeLootItemCondition$Builder#build
    pub fn build(self) -> Box<dyn LootItemCondition> {
        let built: Vec<Box<dyn LootItemCondition>> =
            self.terms.iter().map(|b| b.build_condition()).collect();
        Self::create(built)
    }
}

impl LootItemConditionBuilder for AllOfConditionBuilder {
    fn build_condition(&self) -> Box<dyn LootItemCondition> {
        let built: Vec<Box<dyn LootItemCondition>> =
            self.terms.iter().map(|b| b.build_condition()).collect();
        Box::new(AllOfCondition::new(built))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct BoolCondition(bool);

    impl LootItemCondition for BoolCondition {
        fn get_type(&self) -> LootItemConditionType {
            LootItemConditionType("bool")
        }
        fn test(&self, _ctx: &LootContext) -> bool {
            self.0
        }
    }

    #[derive(Debug)]
    struct BoolBuilder(bool);

    impl LootItemConditionBuilder for BoolBuilder {
        fn build_condition(&self) -> Box<dyn LootItemCondition> {
            Box::new(BoolCondition(self.0))
        }
    }

    #[test]
    fn all_of_empty_is_true() {
        let cond = AllOfCondition::all_of(vec![]);
        let ctx = LootContext::default();
        assert!(cond.test(&ctx));
        assert_eq!(cond.get_type(), LootItemConditions::ALL_OF);
    }

    #[test]
    fn all_of_all_true() {
        let cond = AllOfCondition::all_of(vec![
            Box::new(BoolCondition(true)),
            Box::new(BoolCondition(true)),
        ]);
        assert!(cond.test(&LootContext::default()));
    }

    #[test]
    fn all_of_one_false() {
        let cond = AllOfCondition::all_of(vec![
            Box::new(BoolCondition(true)),
            Box::new(BoolCondition(false)),
            Box::new(BoolCondition(true)),
        ]);
        assert!(!cond.test(&LootContext::default()));
    }

    #[test]
    fn builder_and_build() {
        let b1: Box<dyn LootItemConditionBuilder> = Box::new(BoolBuilder(true));
        let b2: Box<dyn LootItemConditionBuilder> = Box::new(BoolBuilder(true));
        let b3: Box<dyn LootItemConditionBuilder> = Box::new(BoolBuilder(false));
        let built = AllOfCondition::all_of_builders(vec![b1, b2]).and(b3).build();
        assert_eq!(built.get_type(), LootItemConditions::ALL_OF);
        assert!(!built.test(&LootContext::default()));
    }

    #[test]
    fn builder_all_true() {
        let b1: Box<dyn LootItemConditionBuilder> = Box::new(BoolBuilder(true));
        let b2: Box<dyn LootItemConditionBuilder> = Box::new(BoolBuilder(true));
        let built = AllOfCondition::all_of_builders(vec![b1]).and(b2).build();
        assert!(built.test(&LootContext::default()));
    }
}
